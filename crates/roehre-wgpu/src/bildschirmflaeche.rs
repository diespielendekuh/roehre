//! Flächen-Modus: ein Röhrenbildschirm als Objekt in einer 3D-Szene.
//!
//! Die Röhre zeichnet Signal, Strahl, Zeilen, Leuchtstoff und Licht **ohne Maske** in eine
//! Textur fester Größe ([`FlaechenZiel`]) und erzeugt deren Mipmaps. Das Spiel legt die
//! Textur auf das Bildschirm-Modell und rechnet die Maske im Material mit der Funktion aus
//! `shader/eigen/flaeche/roehre_maske.{wgsl,glsl,hlsl}`; die Werte dafür liefert
//! [`MaterialWerte`].
//!
//! Warum geteilt: Die Maske gehört zur Oberfläche des Glases. Ihre Streifen müssen je nach
//! Abstand und Blickwinkel unterschiedlich fein gefiltert werden; das kann nur das Material
//! (über die Ableitungen der UV-Koordinaten). In einer Textur eingebrannt würde sie aus der
//! Entfernung Moiré erzeugen. Die Zeilen dagegen sind grob genug, dass Mipmaps sie sauber
//! filtern.
//!
//! Lesen: Die Textur hat das Format `Rgba8Unorm` und erlaubt die Ansicht `Rgba8UnormSrgb`.
//! Das Material liest über [`FlaechenZiel::ansicht_srgb`] und bekommt lineare Farben.

/// Zieltextur des Flächen-Modus mit Mipmaps.
pub struct FlaechenZiel {
    pub textur: wgpu::Texture,
    /// Ansicht der obersten Stufe zum Hineinzeichnen (ohne sRGB).
    pub(crate) ansicht_roh: wgpu::TextureView,
    /// Ansicht aller Stufen mit sRGB-Dekodierung, zum Lesen im Material.
    pub ansicht_srgb: wgpu::TextureView,
    pub breite: u32,
    pub hoehe: u32,
    stufen: Vec<wgpu::TextureView>,
    mip: MipErzeuger,
}

impl FlaechenZiel {
    /// Legt die Zieltextur an. Übliche Größen: 1440×1080 für 4:3, 2048×1536 für Nahaufnahmen.
    pub fn neu(device: &wgpu::Device, breite: u32, hoehe: u32) -> Self {
        let stufenzahl = 32 - breite.max(hoehe).leading_zeros();
        let textur = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("roehre: Bildschirmfläche"),
            size: wgpu::Extent3d { width: breite, height: hoehe, depth_or_array_layers: 1 },
            mip_level_count: stufenzahl,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });
        let stufe = |n: u32, format| {
            textur.create_view(&wgpu::TextureViewDescriptor {
                format: Some(format),
                base_mip_level: n,
                mip_level_count: Some(1),
                ..Default::default()
            })
        };
        let ansicht_roh = stufe(0, wgpu::TextureFormat::Rgba8Unorm);
        let stufen = (0..stufenzahl).map(|n| stufe(n, wgpu::TextureFormat::Rgba8UnormSrgb)).collect();
        let ansicht_srgb = textur.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        });
        Self { textur, ansicht_roh, ansicht_srgb, breite, hoehe, stufen, mip: MipErzeuger::neu(device) }
    }

    /// Erzeugt die Mipmaps aus der obersten Stufe (in linearem Licht gefiltert).
    pub(crate) fn mipmaps(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        for n in 1..self.stufen.len() {
            self.mip.stufe(device, encoder, &self.stufen[n - 1], &self.stufen[n]);
        }
    }
}

/// Werte für die Materialfunktion `roehre_maske` (gleiche Reihenfolge wie das Struct dort).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialWerte {
    /// Farbtripel über die Bildbreite
    pub triaden: f32,
    /// Stärke der Streifen, 0 … 1
    pub staerke: f32,
    /// Stärke der Schlitze (0 = Streifenmaske)
    pub schlitz: f32,
    /// Breite / Höhe der Bildfläche
    pub seitenverhaeltnis: f32,
    /// Schlitzabstand senkrecht in Tripelbreiten
    pub schlitz_abstand: f32,
    /// 1 = Farbreihenfolge B, G, R
    pub bgr: f32,
}

impl MaterialWerte {
    /// Leitet die Materialwerte aus den Einstellungen ab. In der Szene gibt es keine
    /// Bildschirmpixel, auf die die Maske gelegt werden müsste: `maske.triaden` gilt hier
    /// unverändert, und die Stärke darf wie bei HDR kräftiger sein (die Funktion hält den
    /// Mittelwert bei 1).
    pub fn aus(e: &roehre_modell::Einstellungen, seitenverhaeltnis: f64) -> Self {
        let typ = e.auswahl("maske.typ");
        let staerke = if typ == "aus" || !e.schalter("allgemein.roehre") { 0.0 } else { (e.zahl("maske.staerke") * 1.55).min(1.0) };
        let schlitz = match typ {
            "schlitzmaske" => (e.zahl("maske.staerke") * 1.6).min(1.0),
            // Lochmaske: versetzte Punkte, angenähert durch dichte, kräftige Schlitze
            "lochmaske" => 1.0,
            _ => 0.0,
        };
        let schlitz_abstand = if typ == "lochmaske" { 0.6 } else { 1.5 };
        Self {
            triaden: e.zahl("maske.triaden") as f32,
            staerke: staerke as f32,
            schlitz: if staerke == 0.0 { 0.0 } else { schlitz as f32 },
            seitenverhaeltnis: seitenverhaeltnis as f32,
            schlitz_abstand,
            bgr: 0.0,
        }
    }

    /// Als Uniform-Daten (6 × f32, aufgefüllt auf 8).
    pub fn als_bytes(&self) -> [u8; 32] {
        let w = [self.triaden, self.staerke, self.schlitz, self.seitenverhaeltnis, self.schlitz_abstand, self.bgr, 0.0, 0.0];
        let mut b = [0u8; 32];
        for (i, f) in w.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&f.to_le_bytes());
        }
        b
    }
}

/// Verkleinert eine Mip-Stufe auf die nächste (bilinear, über sRGB-Ansichten also linear).
struct MipErzeuger {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

const MIP_WGSL: &str = r#"
@group(0) @binding(0) var quelle: texture_2d<f32>;
@group(0) @binding(1) var abtaster: sampler;
struct Aus { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn ecke(@builtin(vertex_index) i: u32) -> Aus {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return Aus(vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0), vec2<f32>(p.x, 1.0 - p.y));
}
@fragment fn farbe(e: Aus) -> @location(0) vec4<f32> {
    return textureSample(quelle, abtaster, e.uv);
}
"#;

impl MipErzeuger {
    fn neu(device: &wgpu::Device) -> Self {
        let modul = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("roehre: Mipmaps"),
            source: wgpu::ShaderSource::Wgsl(MIP_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("roehre: Mipmaps"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("roehre: Mipmaps"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre: Mipmaps"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &modul, entry_point: Some("ecke"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &modul,
                entry_point: Some("farbe"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8UnormSrgb.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("roehre: Mipmaps"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self { pipeline, layout, sampler }
    }

    fn stufe(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, von: &wgpu::TextureView, nach: &wgpu::TextureView) {
        let gruppe = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre: Mipmaps"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(von) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("roehre: Mipmaps"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: nach,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &gruppe, &[]);
        pass.draw(0..3, 0..1);
    }
}
