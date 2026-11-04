//! Zeit je Bild der versionsfreien Ketten (`roehre-wgsl`, W-009) — für Spiele,
//! die sie selbst ausführen (Terranigma-Port, Auftrag E3b: »jeder Filter:
//! Bildzeit gemessen«).
//!
//! Gemessen wird wie bei `leistung` ([`crate::leistung::Messer`]): Zeitstempel
//! der Grafikkarte, sonst die Wanduhr bis zum Ende der Arbeit; jedes Bild wird
//! einzeln abgewartet. Eingang ist ein Spielbild von 256 × 224, das Ziel das
//! 4:3-Bildrechteck in 1080p (1440 × 1080) und 4K (2880 × 2160) — so groß, wie
//! der Port es zeichnet. Die Pipelines werden vorher gebaut und bleiben; jedes
//! Bild schreibt nur Rahmen und zeichnet.
//!
//! Die Stufen hinter einem Filter (`leuchten` samt `leuchten-hof`, `farbe`)
//! stehen in eigenen Zeilen: ihre Zeit kommt zu der des Filters dazu.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

use roehre_wgsl::{Abtastung, Groesse, Rahmen, Stufe};

use crate::Ergebnis;
use crate::gpu::Gpu;
use crate::leistung::Messer;

/// Eine gebaute Stufe der Vereinbarung.
struct Gebaut {
    def: &'static Stufe,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    abtaster: wgpu::Sampler,
    puffer: wgpu::Buffer,
}

fn bauen(gpu: &Gpu, s: &'static Stufe) -> Gebaut {
    let d = &gpu.device;
    let textur = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let eintraege = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(128),
            },
            count: None,
        },
        textur(1),
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        textur(3),
    ];
    let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(s.name),
        entries: &eintraege[..if s.zusatz.is_some() { 4 } else { 3 }],
    });
    let modul = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(s.name),
        source: wgpu::ShaderSource::Wgsl(s.wgsl().into()),
    });
    let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(s.name),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(s.name),
        layout: Some(&pl),
        vertex: wgpu::VertexState {
            module: &modul,
            entry_point: Some("ecke"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &modul,
            entry_point: Some("punkt"),
            targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let filter = match s.abtastung {
        Abtastung::Linear => wgpu::FilterMode::Linear,
        Abtastung::Naechster => wgpu::FilterMode::Nearest,
    };
    let abtaster = d.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: filter,
        min_filter: filter,
        ..Default::default()
    });
    let puffer = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some(s.name),
        size: 128,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    Gebaut { def: s, pipeline, layout, abtaster, puffer }
}

fn textur(gpu: &Gpu, g: (u32, u32)) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("wgsl-leistung"),
        size: wgpu::Extent3d { width: g.0, height: g.1, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

/// Eine Stufe von `quelle` (dazu an Bindung 3 `zusatz`) nach `aus`.
fn aufnehmen(
    gpu: &Gpu,
    enc: &mut wgpu::CommandEncoder,
    st: &Gebaut,
    rahmen: &Rahmen,
    quelle: &wgpu::Texture,
    zusatz: Option<&wgpu::Texture>,
    aus: &wgpu::Texture,
) {
    gpu.queue.write_buffer(&st.puffer, 0, &rahmen.als_bytes());
    let qv = quelle.create_view(&Default::default());
    let zv = zusatz.unwrap_or(quelle).create_view(&Default::default());
    let av = aus.create_view(&Default::default());
    let eintraege = [
        wgpu::BindGroupEntry { binding: 0, resource: st.puffer.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&qv) },
        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&st.abtaster) },
        wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&zv) },
    ];
    let bindung = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(st.def.name),
        layout: &st.layout,
        entries: &eintraege[..if st.def.zusatz.is_some() { 4 } else { 3 }],
    });
    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(st.def.name),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &av,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
        })],
        ..Default::default()
    });
    pass.set_pipeline(&st.pipeline);
    pass.set_bind_group(0, &bindung, &[]);
    pass.draw(0..3, 0..1);
}

const ZIELE: [(&str, (u32, u32)); 2] = [("1080p", (1440, 1080)), ("4K", (2880, 2160))];

pub fn messen(pfad: &Path, bilder: usize) -> Ergebnis {
    let gpu = Gpu::neu();
    let messer = Messer::neu(&gpu);
    let bild = crate::lade_bild("pixelszene")?;
    let eingang = gpu.eingang(&bild);
    let quelle = (bild.breite, bild.hoehe);
    let mut zeilen: Vec<(String, [f64; 2], [f64; 2])> = vec![];
    let mut eintragen = |name: String, i: usize, k: crate::leistung::Kennzahl| {
        eprintln!("{name} {}: Median {:.3} ms, p95 {:.3} ms", ZIELE[i].0, k.median, k.p95);
        match zeilen.iter_mut().find(|z| z.0 == name) {
            Some(z) => {
                z.1[i] = k.median;
                z.2[i] = k.p95;
            }
            None => {
                let mut z = (name, [0.0; 2], [0.0; 2]);
                z.1[i] = k.median;
                z.2[i] = k.p95;
                zeilen.push(z);
            }
        }
    };

    // Die einfachen Ketten (KETTEN): Stufe für Stufe, Zwischenbilder nach `groesse`.
    for kette in roehre_wgsl::KETTEN {
        let stufen: Vec<Gebaut> = kette.stufen.iter().map(|s| bauen(&gpu, s)).collect();
        for (i, (_, ziel)) in ZIELE.iter().enumerate() {
            let mut groessen = vec![];
            let mut vorher = quelle;
            for (j, s) in stufen.iter().enumerate() {
                let g = if j + 1 == stufen.len() {
                    *ziel
                } else {
                    match s.def.groesse {
                        Groesse::Ziel => *ziel,
                        Groesse::QuelleMal(n) => (vorher.0 * n, vorher.1 * n),
                    }
                };
                groessen.push((vorher, g));
                vorher = g;
            }
            let bilder_aus: Vec<wgpu::Texture> = groessen.iter().map(|(_, g)| textur(&gpu, *g)).collect();
            let k = messer.miss(&gpu, bilder, |enc| {
                for (j, s) in stufen.iter().enumerate() {
                    let q = if j == 0 { &eingang } else { &bilder_aus[j - 1] };
                    let rahmen = Rahmen::neu(groessen[j].0, groessen[j].1);
                    aufnehmen(&gpu, enc, s, &rahmen, q, None, &bilder_aus[j]);
                }
            });
            eintragen(format!("{} (`{}`)", kette.titel, kette.name), i, k);
        }
    }

    // Die erzeugten Ketten (crt-beans, ScaleFX, OmniScale) über den Ausführer.
    let erzeugte: [&'static roehre_wgsl::kette::Erzeugt; 5] = [
        &roehre_wgsl::erzeugt::beans::BEANS_RGB,
        &roehre_wgsl::erzeugt::beans::BEANS_SIGNAL,
        &roehre_wgsl::erzeugt::beans::BEANS_SCHNELL,
        &roehre_wgsl::erzeugt::pixelkunst::SCALEFX,
        &roehre_wgsl::erzeugt::pixelkunst::OMNISCALE,
    ];
    for e in erzeugte {
        let mut ausf = crate::wgsl_kette::Ausfuehrer::neu(&gpu.device, &gpu.queue, e, wgpu::TextureFormat::Rgba8Unorm);
        let leer: HashMap<&str, f32> = HashMap::new();
        for (i, (_, ziel)) in ZIELE.iter().enumerate() {
            let aus = textur(&gpu, *ziel);
            let av = aus.create_view(&Default::default());
            let mut zaehler = 0u32;
            let k = messer.miss(&gpu, bilder, |enc| {
                zaehler += 1;
                ausf.zeichne(&gpu.device, &gpu.queue, enc, &eingang, &av, *ziel, zaehler, ziel.0 as f32 / ziel.1 as f32, &leer);
            });
            eintragen(format!("`{}` (erzeugt)", e.name), i, k);
        }
    }

    // Hinter dem Filter: Leuchten (Hof in Spielauflösung + Stufe), Farbe.
    let hof = bauen(&gpu, &roehre_wgsl::LEUCHTEN_HOF);
    let leuchten = bauen(&gpu, &roehre_wgsl::LEUCHTEN);
    let farbe = bauen(&gpu, &roehre_wgsl::FARBE);
    let hofbild = textur(&gpu, quelle);
    for (i, (_, ziel)) in ZIELE.iter().enumerate() {
        let vorlage = textur(&gpu, *ziel);
        let aus = textur(&gpu, *ziel);
        for weite in [1.5f32, 4.0] {
            let werte = [("staerke", 0.5f32), ("weite", weite), ("schwelle", 0.5)];
            let k = messer.miss(&gpu, bilder, |enc| {
                aufnehmen(&gpu, enc, &hof, &Rahmen::neu(quelle, quelle).mit_reglern(hof.def, &werte), &eingang, None, &hofbild);
                aufnehmen(&gpu, enc, &leuchten, &Rahmen::neu(*ziel, *ziel).mit_reglern(leuchten.def, &werte), &vorlage, Some(&hofbild), &aus);
            });
            eintragen(format!("+ Leuchten, Weite {weite}"), i, k);
        }
        let werte = [("saettigung", 0.5f32)];
        let k = messer.miss(&gpu, bilder, |enc| {
            aufnehmen(&gpu, enc, &farbe, &Rahmen::neu(*ziel, *ziel).mit_reglern(farbe.def, &werte), &vorlage, None, &aus);
        });
        eintragen("+ Farbe".into(), i, k);
    }

    let mut text = String::new();
    let _ = writeln!(text, "# Zeit je Bild der versionsfreien Ketten (`roehre-wgsl`)\n");
    let _ = writeln!(
        text,
        "Adapter: {}. Methode: {}. {bilder} Bilder je Zeile, Eingang {}×{}, Ziel 4:3 (1080p: 1440×1080, 4K: 2880×2160). Median und 95. Perzentil in ms.\n",
        gpu.adapter_name,
        messer.methode(),
        quelle.0,
        quelle.1
    );
    let _ = writeln!(text, "| Kette | 1080p Median | 1080p p95 | 4K Median | 4K p95 |\n|---|---:|---:|---:|---:|");
    for (name, m, p) in &zeilen {
        let _ = writeln!(text, "| {name} | {:.3} | {:.3} | {:.3} | {:.3} |", m[0], p[0], m[1], p[1]);
    }
    std::fs::write(pfad, &text)?;
    eprintln!("{text}");
    Ok(())
}
