//! Vorführung des Flächen-Modus: ein Röhrenbildschirm als Viereck in einer 3D-Szene.
//!
//! Gezeichnet wird mit der Materialfunktion `roehre_maske` aus `shader/eigen/flaeche/` und
//! zum Vergleich mit einer naiven Maske (dieselben Streifen ohne Bandbegrenzung).

use roehre_wgpu::{FlaechenZiel, MaterialWerte};

const SZENE_WGSL: &str = r#"
struct Kamera { mvp: mat4x4<f32>, naiv: vec4<f32> };
@group(0) @binding(0) var<uniform> kamera: Kamera;
@group(0) @binding(1) var<uniform> maske: RoehreMaske;
@group(0) @binding(2) var bild: texture_2d<f32>;
@group(0) @binding(3) var abtaster: sampler;

struct Aus { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex fn ecke(@builtin(vertex_index) i: u32) -> Aus {
    var ecken = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0));
    let uv = ecken[i];
    let p = vec3<f32>((uv.x - 0.5) * maske.seitenverhaeltnis, 0.5 - uv.y, 0.0);
    return Aus(kamera.mvp * vec4<f32>(p, 1.0), uv);
}

// Naive Maske: dieselben Wellen, aber ohne Bandbegrenzung
fn naive_maske(uv: vec2<f32>, m: RoehreMaske) -> vec3<f32> {
    let t = uv.x * m.triaden;
    let v = uv.y * m.triaden / (m.seitenverhaeltnis * max(m.schlitz_abstand, 0.1));
    let x = ROEHRE_TAU * (vec3<f32>(t) - vec3<f32>(0.0, 1.0 / 3.0, 2.0 / 3.0));
    let streifen = max(vec3<f32>(1.0) + m.staerke * 0.8 * cos(x) + m.staerke * 0.3 * cos(2.0 * x), vec3<f32>(0.0));
    let schlitz = 1.0 + m.schlitz * 0.5 * cos(ROEHRE_TAU * v) * cos(0.5 * ROEHRE_TAU * t);
    return streifen * schlitz;
}

@fragment fn farbe(e: Aus) -> @location(0) vec4<f32> {
    let c = textureSample(bild, abtaster, e.uv).rgb;
    let mb = roehre_maske(e.uv, maske);
    let mn = naive_maske(e.uv, maske);
    let m = select(mb, mn, kamera.naiv.x > 0.5);
    return vec4<f32>(c * m, 1.0);
}
"#;

pub struct Szene {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

/// Spaltenweise 4×4-Matrix (wie WGSL sie erwartet).
type Mat = [[f32; 4]; 4];

fn mal(a: &Mat, b: &Mat) -> Mat {
    let mut r = [[0.0; 4]; 4];
    for (c, spalte) in r.iter_mut().enumerate() {
        for (z, wert) in spalte.iter_mut().enumerate() {
            *wert = (0..4).map(|k| a[k][z] * b[c][k]).sum();
        }
    }
    r
}

/// Kamera: Abstand, Drehung des Bildschirms um die senkrechte Achse (Grad), Blick auf
/// einen Punkt der Fläche (in Flächenkoordinaten, Mitte = 0).
pub fn kamera(seite: f32, abstand: f32, drehung_grad: f32, ziel: [f32; 2]) -> Mat {
    let (s, c) = drehung_grad.to_radians().sin_cos();
    let modell: Mat = [[c, 0.0, -s, 0.0], [0.0, 1.0, 0.0, 0.0], [s, 0.0, c, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let blick: Mat = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [-ziel[0] * c, -ziel[1], -abstand, 1.0]];
    let (nah, fern) = (0.01f32, 100.0f32);
    let f = 1.0 / (20f32.to_radians()).tan();
    let proj: Mat = [
        [f / seite, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, fern / (nah - fern), -1.0],
        [0.0, 0.0, nah * fern / (nah - fern), 0.0],
    ];
    mal(&proj, &mal(&blick, &modell))
}

impl Szene {
    pub fn neu(device: &wgpu::Device, shader_ordner: &std::path::Path, format: wgpu::TextureFormat) -> Self {
        let maske = std::fs::read_to_string(shader_ordner.join("eigen/flaeche/roehre_maske.wgsl")).expect("roehre_maske.wgsl");
        let modul = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Szene"),
            source: wgpu::ShaderSource::Wgsl(format!("{maske}\n{SZENE_WGSL}").into()),
        });
        let eintrag = |binding, ty, sicht| wgpu::BindGroupLayoutEntry { binding, visibility: sicht, ty, count: None };
        let uniform = wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Szene"),
            entries: &[
                eintrag(0, uniform, wgpu::ShaderStages::VERTEX_FRAGMENT),
                eintrag(1, uniform, wgpu::ShaderStages::VERTEX_FRAGMENT),
                eintrag(
                    2,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    wgpu::ShaderStages::FRAGMENT,
                ),
                eintrag(3, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Szene"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Szene"),
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
            label: Some("Szene"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        Self { pipeline, layout, sampler }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn zeichne(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ziel: &wgpu::TextureView,
        flaeche: &FlaechenZiel,
        werte: &MaterialWerte,
        mvp: &Mat,
        naiv: bool,
    ) {
        let mut k = Vec::with_capacity(80);
        for spalte in mvp {
            for w in spalte {
                k.extend_from_slice(&w.to_le_bytes());
            }
        }
        for w in [if naiv { 1.0f32 } else { 0.0 }, 0.0, 0.0, 0.0] {
            k.extend_from_slice(&w.to_le_bytes());
        }
        let puffer = |inhalt: &[u8]| {
            let b = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Szene"),
                size: inhalt.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&b, 0, inhalt);
            b
        };
        let (kb, mb) = (puffer(&k), puffer(&werte.als_bytes()));
        let gruppe = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Szene"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: kb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: mb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&flaeche.ansicht_srgb) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Szene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ziel,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.02, b: 0.025, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &gruppe, &[]);
            pass.draw(0..6, 0..1);
        }
        queue.submit([enc.finish()]);
    }
}
