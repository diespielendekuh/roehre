//! # roehre-raum
//!
//! Ein 3D-Vorführraum: ein berechneter Röhrenfernseher auf einem Tisch in einem nächtlichen
//! Zimmer, auf dessen gewölbter Bildröhre das Spielbild mit Röhrensimulation läuft. Er dient
//! als Prüfstein für den Flächen-Modus von roehre und als **Vorlage** für Spiele, die einen
//! Fernseher in einer 3D-Szene zeigen wollen (z. B. das „TV-Erlebnis“ im Mission:-Impossible-
//! Port).
//!
//! Alles ist in einem einzigen Fragment-Shader berechnet (`shader/eigen/raum/raum.wgsl`,
//! Raymarching über Signed Distance Fields): keine Modelle, keine Texturen außer dem
//! Röhrenbild. Für D3D12/Vulkan lässt er sich mit naga nach HLSL bzw. SPIR-V übersetzen.
//!
//! Ablauf je Bild:
//! 1. `Roehre::zeichne_flaeche` zeichnet das Röhrenbild ohne Maske in ein `FlaechenZiel`.
//! 2. [`Raum::zeichne`] rendert die Szene; die Maske rechnet `roehre_maske` im Shader.

use roehre_wgpu::{FlaechenZiel, MaterialWerte};

/// Kamera und Stimmung des Raums.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ansicht {
    /// Lage der Kamera (Meter; der Fernseher steht im Ursprung, Bildschirm zeigt nach −z)
    pub auge: [f32; 3],
    /// Blickziel
    pub ziel: [f32; 3],
    /// Senkrechter Blickwinkel in Grad
    pub blickwinkel: f32,
    /// Zeit in Sekunden (Vorhang, Flackern)
    pub zeit: f32,
    /// Helligkeit des Bildschirms (1 = Röhrenbild unverändert)
    pub bildschirm: f32,
    /// Helligkeit der Lampe (0 = dunkel)
    pub raumlicht: f32,
    /// Flackern der Lampe 0 … 1
    pub flackern: f32,
    /// Wölbung der Bildröhre 0 (flach) … 1 (stark)
    pub woelbung: f32,
}

impl Ansicht {
    /// Sitzposition vor dem Fernseher mit sachtem Schwenk (`schwenk` 0 = ruhig).
    pub fn sitzplatz(zeit: f32, abstand: f32, schwenk: f32) -> Self {
        let winkel = schwenk * 0.12 * (zeit * 0.15).sin();
        let mitte = [0.0, 0.72 + 0.262, 0.0];
        Self {
            auge: [abstand * winkel.sin(), mitte[1] + 0.08 + 0.02 * schwenk * (zeit * 0.11).sin(), -abstand * winkel.cos()],
            ziel: [0.0, mitte[1] - 0.02, 0.0],
            blickwinkel: 40.0,
            zeit,
            bildschirm: 1.0,
            raumlicht: 0.35,
            flackern: 0.5,
            woelbung: 0.6,
        }
    }
}

/// Der Raum (Pipeline und Puffer).
pub struct Raum {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    srgb: bool,
}

/// Quelltext des Raum-Shaders samt Maskenfunktion (für eigene Backends übersetzbar).
pub fn shader_quelltext(shader_ordner: &std::path::Path) -> std::io::Result<String> {
    let maske = std::fs::read_to_string(shader_ordner.join("eigen/flaeche/roehre_maske.wgsl"))?;
    let raum = std::fs::read_to_string(shader_ordner.join("eigen/raum/raum.wgsl"))?;
    Ok(format!("{maske}\n{raum}"))
}

impl Raum {
    pub fn neu(device: &wgpu::Device, shader_ordner: &std::path::Path, format: wgpu::TextureFormat) -> std::io::Result<Self> {
        let modul = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("roehre: Raum"),
            source: wgpu::ShaderSource::Wgsl(shader_quelltext(shader_ordner)?.into()),
        });
        let uniform = wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None };
        let eintrag = |binding, ty| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty, count: None };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("roehre: Raum"),
            entries: &[
                eintrag(0, uniform),
                eintrag(1, uniform),
                eintrag(
                    2,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                eintrag(3, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("roehre: Raum"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre: Raum"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &modul, entry_point: Some("ecke"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &modul,
                entry_point: Some("farbe"),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("roehre: Raum"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        Ok(Self { pipeline, layout, sampler, srgb: format.is_srgb() })
    }

    /// Rendert den Raum in `ziel` (Größe `groesse`), mit dem Röhrenbild aus `flaeche`.
    #[allow(clippy::too_many_arguments)]
    pub fn zeichne(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        ziel: &wgpu::TextureView,
        groesse: (u32, u32),
        flaeche: &FlaechenZiel,
        material: &MaterialWerte,
        a: &Ansicht,
    ) {
        let stufen = flaeche.textur.mip_level_count() as f32 - 1.0;
        let w: [f32; 16] = [
            a.auge[0], a.auge[1], a.auge[2], a.blickwinkel,
            a.ziel[0], a.ziel[1], a.ziel[2], groesse.0 as f32 / groesse.1.max(1) as f32,
            a.zeit, a.bildschirm, a.raumlicht, a.flackern,
            material.seitenverhaeltnis, stufen, a.woelbung, if self.srgb { 1.0 } else { 0.0 },
        ];
        let bytes: Vec<u8> = w.iter().flat_map(|f| f.to_le_bytes()).collect();
        let puffer = |inhalt: &[u8]| {
            let b = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("roehre: Raum"),
                size: inhalt.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&b, 0, inhalt);
            b
        };
        let (rb, mb) = (puffer(&bytes), puffer(&material.als_bytes()));
        let gruppe = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre: Raum"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: rb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: mb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&flaeche.ansicht_srgb) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("roehre: Raum"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ziel,
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
