//! Ausführer für erzeugte Ketten (`roehre_wgsl::kette`, W-009 Teil 2) auf
//! wgpu 30 — so gebaut, wie ein Spiel ihn auf seiner wgpu-Version baut — und
//! der Prüfstein: dieselbe Kette über librashaders eigene `FilterChain` auf
//! demselben Bild mit denselben Reglern, Punkt für Punkt verglichen.
//!
//! Vertrag: das Ziel ist eine eigene Textur in der Größe des Bildausschnitts
//! (wie roehre selbst zeichnet); der Viewport ist das ganze Ziel.

use std::collections::HashMap;

use roehre_wgsl::kette::{Bild, Erzeugt, Format, Ort, Pass, Rand, Wert, ECKEN_LETZTER, ECKEN_ZWISCHEN, MVP_EINHEIT, MVP_LETZTER};
use wgpu::util::DeviceExt;

fn wgpu_format(f: Format) -> wgpu::TextureFormat {
    match f {
        Format::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        Format::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        Format::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        Format::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        Format::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
    }
}

struct Stufe {
    pipeline: wgpu::RenderPipeline,
    gruppe0: wgpu::BindGroupLayout,
    gruppe1: wgpu::BindGroupLayout,
    ubo: Option<wgpu::Buffer>,
    push: Option<wgpu::Buffer>,
}

pub struct Ausfuehrer {
    kette: &'static Erzeugt,
    stufen: Vec<Stufe>,
    ecken: wgpu::Buffer,
    luts: Vec<(wgpu::Texture, wgpu::TextureView)>,
    zwischen: Vec<Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))>>,
    abtaster: HashMap<(Rand, bool), wgpu::Sampler>,
    rand_schwarz: bool,
}

impl Ausfuehrer {
    pub fn neu(device: &wgpu::Device, queue: &wgpu::Queue, kette: &'static Erzeugt, zielformat: wgpu::TextureFormat) -> Ausfuehrer {
        let letzter = kette.paesse.len() - 1;
        let stufen = kette.paesse.iter().enumerate().map(|(i, p)| Self::stufe(device, p, if i == letzter { zielformat } else { wgpu_format(p.format) })).collect();
        let ecken: Vec<f32> = ECKEN_ZWISCHEN.iter().chain(ECKEN_LETZTER.iter()).flat_map(|(l, t)| l.iter().chain(t.iter()).copied()).collect();
        let ecken = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Ecken"), contents: &bytes(&ecken), usage: wgpu::BufferUsages::VERTEX });
        let luts = kette
            .luts
            .iter()
            .map(|l| {
                let t = device.create_texture_with_data(
                    queue,
                    &wgpu::TextureDescriptor {
                        label: Some(l.name),
                        size: wgpu::Extent3d { width: l.breite, height: l.hoehe, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    },
                    wgpu::util::TextureDataOrder::LayerMajor,
                    l.rgba,
                );
                let v = t.create_view(&Default::default());
                (t, v)
            })
            .collect();
        let rand_schwarz = device.features().contains(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER);
        let mut abtaster = HashMap::new();
        for rand in [Rand::Schwarz, Rand::Kante, Rand::Wiederholen, Rand::Spiegeln] {
            for linear in [false, true] {
                let a = match rand {
                    Rand::Schwarz if rand_schwarz => wgpu::AddressMode::ClampToBorder,
                    Rand::Schwarz | Rand::Kante => wgpu::AddressMode::ClampToEdge,
                    Rand::Wiederholen => wgpu::AddressMode::Repeat,
                    Rand::Spiegeln => wgpu::AddressMode::MirrorRepeat,
                };
                let f = if linear { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest };
                let m = if linear { wgpu::MipmapFilterMode::Linear } else { wgpu::MipmapFilterMode::Nearest };
                abtaster.insert(
                    (rand, linear),
                    device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: a,
                        address_mode_v: a,
                        address_mode_w: a,
                        mag_filter: f,
                        min_filter: f,
                        mipmap_filter: m,
                        lod_min_clamp: 0.0,
                        lod_max_clamp: 1000.0,
                        border_color: Some(wgpu::SamplerBorderColor::TransparentBlack),
                        ..Default::default()
                    }),
                );
            }
        }
        Ausfuehrer { kette, stufen, ecken, luts, zwischen: (0..kette.paesse.len()).map(|_| None).collect(), abtaster, rand_schwarz }
    }

    fn stufe(device: &wgpu::Device, p: &Pass, format: wgpu::TextureFormat) -> Stufe {
        let ecke = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(p.name), source: wgpu::ShaderSource::Wgsl(p.ecke.into()) });
        let punkt = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(p.name), source: wgpu::ShaderSource::Wgsl(p.punkt.into()) });
        let sicht = |e: bool, f: bool| match (e, f) {
            (true, false) => wgpu::ShaderStages::VERTEX,
            (false, true) => wgpu::ShaderStages::FRAGMENT,
            _ => wgpu::ShaderStages::VERTEX_FRAGMENT,
        };
        let mut e0 = vec![];
        for b in [p.ubo, p.push].into_iter().flatten() {
            e0.push(wgpu::BindGroupLayoutEntry {
                binding: b.nummer,
                visibility: sicht(b.ecke, b.punkt),
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: wgpu::BufferSize::new(b.groesse as u64) },
                count: None,
            });
        }
        let mut e1 = vec![];
        for b in p.bilder {
            e0.push(wgpu::BindGroupLayoutEntry {
                binding: b.nummer,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                count: None,
            });
            e1.push(wgpu::BindGroupLayoutEntry { binding: b.nummer, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None });
        }
        let gruppe0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("Gruppe 0"), entries: &e0 });
        let gruppe1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("Gruppe 1"), entries: &e1 });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(p.name), bind_group_layouts: &[Some(&gruppe0), Some(&gruppe1)], immediate_size: 0 });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(p.name),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &ecke,
                entry_point: Some(p.ecke_einstieg),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 24,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 16, shader_location: 1 },
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState { module: &punkt, entry_point: Some(p.punkt_einstieg), compilation_options: Default::default(), targets: &[Some(format.into())] }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let puffer = |b: Option<roehre_wgsl::kette::Puffer>| {
            b.map(|b| device.create_buffer(&wgpu::BufferDescriptor { label: Some("Werte"), size: b.groesse as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }))
        };
        Stufe { pipeline, gruppe0, gruppe1, ubo: puffer(p.ubo), push: puffer(p.push) }
    }

    /// Ein Bild: `quelle` (Spielbild, Rgba8Unorm) → `ziel` (ganz, Viewport = Ziel).
    #[allow(clippy::too_many_arguments)]
    pub fn zeichne(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        enc: &mut wgpu::CommandEncoder,
        quelle: &wgpu::Texture,
        ziel: &wgpu::TextureView,
        ziel_groesse: (u32, u32),
        bildzaehler: u32,
        seitenverhaeltnis: f32,
        regler: &HashMap<&str, f32>,
    ) {
        let original = (quelle.width(), quelle.height());
        let quelle_ansicht = quelle.create_view(&Default::default());
        let kette = self.kette;
        let n = kette.paesse.len();
        // Größen: ab dem Spielbild je Pass `max(round(Bezug · Faktor), 1)`.
        let mut groessen = vec![];
        let mut vorher = original;
        for (i, p) in kette.paesse.iter().enumerate() {
            let g = if i + 1 == n { ziel_groesse } else { (p.skala.0.rechne(vorher.0, ziel_groesse.0, original.0), p.skala.1.rechne(vorher.1, ziel_groesse.1, original.1)) };
            groessen.push(g);
            vorher = g;
        }
        for i in 0..n - 1 {
            if self.zwischen[i].as_ref().is_none_or(|z| z.2 != groessen[i]) {
                let t = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(kette.paesse[i].name),
                    size: wgpu::Extent3d { width: groessen[i].0, height: groessen[i].1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu_format(kette.paesse[i].format),
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let v = t.create_view(&Default::default());
                self.zwischen[i] = Some((t, v, groessen[i]));
            }
        }
        let vec4 = |g: (u32, u32)| [g.0 as f32, g.1 as f32, 1.0 / g.0 as f32, 1.0 / g.1 as f32];
        for (i, p) in kette.paesse.iter().enumerate() {
            let letzter = i + 1 == n;
            let quelle_i = if i == 0 { (&quelle_ansicht, original) } else { let z = self.zwischen[i - 1].as_ref().unwrap(); (&z.1, z.2) };
            // Werte
            let mut ubo = vec![0u8; p.ubo.map_or(0, |b| b.groesse as usize)];
            let mut push = vec![0u8; p.push.map_or(0, |b| b.groesse as usize)];
            for f in p.felder {
                let ziel_puffer = match f.ort {
                    Ort::Ubo => &mut ubo,
                    Ort::Push => &mut push,
                };
                let o = f.lage as usize;
                let mut schreibe = |w: &[u8]| ziel_puffer[o..o + w.len()].copy_from_slice(w);
                match f.wert {
                    Wert::Mvp => schreibe(&bytes(if letzter { &MVP_LETZTER } else { &MVP_EINHEIT })),
                    Wert::Ausgabe => schreibe(&bytes(&vec4(groessen[i]))),
                    Wert::Endausgabe => schreibe(&bytes(&vec4(ziel_groesse))),
                    Wert::Bildzaehler => schreibe(&(if p.bildzaehler_modulo > 0 { bildzaehler % p.bildzaehler_modulo } else { bildzaehler }).to_le_bytes()),
                    Wert::Richtung => schreibe(&1i32.to_le_bytes()),
                    Wert::Seitenverhaeltnis => schreibe(&seitenverhaeltnis.to_le_bytes()),
                    Wert::Groesse(b) => {
                        let g = match b {
                            Bild::Original => original,
                            Bild::Quelle => quelle_i.1,
                            Bild::Pass(k) => groessen[k as usize],
                            Bild::Lut(k) => (kette.luts[k as usize].breite, kette.luts[k as usize].hoehe),
                        };
                        schreibe(&bytes(&vec4(g)));
                    }
                    Wert::Regler(name) => {
                        let w = regler.get(name).copied().unwrap_or_else(|| kette.regler.iter().find(|r| r.name == name).map_or(0.0, |r| r.standard));
                        schreibe(&w.to_le_bytes());
                    }
                }
            }
            let st = &self.stufen[i];
            if let Some(b) = &st.ubo {
                queue.write_buffer(b, 0, &ubo);
            }
            if let Some(b) = &st.push {
                queue.write_buffer(b, 0, &push);
            }
            // Bilder und Abtaster
            let mut e0 = vec![];
            if let (Some(b), Some(d)) = (&st.ubo, p.ubo) {
                e0.push(wgpu::BindGroupEntry { binding: d.nummer, resource: b.as_entire_binding() });
            }
            if let (Some(b), Some(d)) = (&st.push, p.push) {
                e0.push(wgpu::BindGroupEntry { binding: d.nummer, resource: b.as_entire_binding() });
            }
            let mut e1 = vec![];
            for b in p.bilder {
                let (ansicht, (rand, linear)) = match b.bild {
                    Bild::Original => (&quelle_ansicht, (kette.paesse[0].rand, kette.paesse[0].linear)),
                    Bild::Quelle => (quelle_i.0, (p.rand, p.linear)),
                    Bild::Pass(k) => (&self.zwischen[k as usize].as_ref().unwrap().1, (kette.paesse[k as usize].rand, kette.paesse[k as usize].linear)),
                    Bild::Lut(k) => (&self.luts[k as usize].1, (kette.luts[k as usize].rand, kette.luts[k as usize].linear)),
                };
                e0.push(wgpu::BindGroupEntry { binding: b.nummer, resource: wgpu::BindingResource::TextureView(ansicht) });
                e1.push(wgpu::BindGroupEntry { binding: b.nummer, resource: wgpu::BindingResource::Sampler(&self.abtaster[&(rand, linear)]) });
            }
            let g0 = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &st.gruppe0, entries: &e0 });
            let g1 = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &st.gruppe1, entries: &e1 });
            let aus = if letzter { ziel } else { &self.zwischen[i].as_ref().unwrap().1 };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(p.name),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: aus,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&st.pipeline);
            pass.set_bind_group(0, &g0, &[]);
            pass.set_bind_group(1, &g1, &[]);
            pass.set_vertex_buffer(0, self.ecken.slice(..));
            let ecken = if letzter { 4..8 } else { 0..4 };
            pass.draw(ecken, 0..1);
        }
        let _ = self.rand_schwarz;
    }
}

fn bytes(f: &[f32]) -> Vec<u8> {
    f.iter().flat_map(|v| v.to_le_bytes()).collect()
}
