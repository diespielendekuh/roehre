//! Prüfstein der versionsfreien Stufen (`roehre-wgsl`, Wunsch W-009): ein
//! schlichter Ausführer auf wgpu 30 — so, wie ein Spiel ihn auf seiner
//! wgpu-Version baut — und zwei Vergleiche:
//!
//! 1. `scharf-bilinear` (WGSL) gegen die Voreinstellung `aus` (librashader mit
//!    `scharf-bilinear.slang`), im selben Rechteck, Punkt für Punkt;
//! 2. `mmpx` auf der GPU gegen die CPU-Referenz `roehre_wgsl::mmpx`,
//!    Punktgleichheit.

use roehre_testbilder::Bild;
use roehre_wgpu::{Einstellungen, Roehre};
use roehre_wgsl::{Abtastung, Rahmen, Stufe};

use crate::Ergebnis;
use crate::gpu::Gpu;

/// Eine Stufe ausführen: Quelle → Ziel (ganzes Ziel oder `viewport`).
pub fn stufe_ausfuehren(
    gpu: &Gpu,
    stufe: &Stufe,
    quelle: &wgpu::Texture,
    rahmen: &Rahmen,
    ziel: &wgpu::Texture,
    viewport: Option<(u32, u32, u32, u32)>,
) {
    stufe_ausfuehren_mit(gpu, stufe, quelle, None, rahmen, ziel, viewport);
}

/// Wie [`stufe_ausfuehren`], für Stufen mit `zusatz` (Bindung 3): `original`
/// ist das Bild vor der ersten Stufe; die Zusatzstufe läuft hier darauf, mit
/// den Reglerwerten `werte`.
pub fn stufe_ausfuehren_mit(
    gpu: &Gpu,
    stufe: &Stufe,
    quelle: &wgpu::Texture,
    original: Option<(&wgpu::Texture, &[(&str, f32)])>,
    rahmen: &Rahmen,
    ziel: &wgpu::Texture,
    viewport: Option<(u32, u32, u32, u32)>,
) {
    assert_eq!(
        stufe.zusatz.is_some(),
        original.is_some(),
        "Stufe {}: Original gehört genau zu den Stufen mit `zusatz`",
        stufe.name
    );
    let zusatzbild = stufe.zusatz.zip(original).map(|(z, (o, werte))| {
        let (b, h) = (o.width(), o.height());
        let (zb, zh) = match z.groesse {
            roehre_wgsl::Groesse::QuelleMal(n) => (b * n, h * n),
            roehre_wgsl::Groesse::Ziel => (b, h),
        };
        let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(z.name),
            size: wgpu::Extent3d {
                width: zb,
                height: zh,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        stufe_ausfuehren(
            gpu,
            z,
            o,
            &Rahmen::neu((b, h), (zb, zh)).mit_reglern(z, werte),
            &t,
            None,
        );
        t
    });
    let original = zusatzbild.as_ref();
    let d = &gpu.device;
    let modul = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(stufe.name),
        source: wgpu::ShaderSource::Wgsl(stufe.wgsl().into()),
    });
    // Das Layout ausdrücklich nach der Vereinbarung (alle drei Bindungen): ein
    // abgeleitetes Layout ließe den Abtaster weg, wo eine Stufe ihn nicht
    // liest (mmpx liest mit textureLoad).
    let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("roehre-wgsl"),
        entries: &[
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
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ][..if stufe.zusatz.is_some() { 4 } else { 3 }],
    });
    let pipeline_layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(stufe.name),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(stufe.name),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &modul,
            entry_point: Some("ecke"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &modul,
            entry_point: Some("punkt"),
            targets: &[Some(ziel.format().into())],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let filter = match stufe.abtastung {
        Abtastung::Linear => wgpu::FilterMode::Linear,
        Abtastung::Naechster => wgpu::FilterMode::Nearest,
    };
    let abtaster = d.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: filter,
        min_filter: filter,
        ..Default::default()
    });
    let puffer = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Rahmen"),
        size: 128,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue.write_buffer(&puffer, 0, &rahmen.als_bytes());
    let quelle_ansicht = quelle.create_view(&Default::default());
    let original_ansicht = original.unwrap_or(quelle).create_view(&Default::default());
    let bindung = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(stufe.name),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: puffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&quelle_ansicht),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&abtaster),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&original_ansicht),
            },
        ][..if stufe.zusatz.is_some() { 4 } else { 3 }],
    });
    let ansicht = ziel.create_view(&Default::default());
    let mut enc = d.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(stufe.name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &ansicht,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        if let Some((x, y, b, h)) = viewport {
            pass.set_viewport(x as f32, y as f32, b as f32, h as f32, 0.0, 1.0);
        }
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bindung, &[]);
        pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([enc.finish()]);
}

/// Ein Ziel ohne sRGB-Umrechnung: die Stufen schreiben sRGB-kodierte Werte.
fn ziel_roh(gpu: &Gpu, b: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Ziel roh"),
        size: wgpu::Extent3d {
            width: b,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

/// `roehre-pruefstand wgsl`: beide Vergleiche; Fehler, wenn einer reißt.
pub fn pruefen(shader: &std::path::Path) -> Ergebnis {
    let gpu = Gpu::neu();
    eprintln!("Adapter: {}", gpu.adapter_name);
    let mut fehler = vec![];

    // 1. scharf-bilinear gegen `aus`.
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader,
        Einstellungen::voreinstellung("aus")?,
        crate::gpu::ZIELFORMAT,
    )?;
    for name in ["pixelszene", "schrift", "farbbalken"] {
        let bild = crate::lade_bild(name)?;
        for g in [
            (1920, 1080),
            (1280, 720),
            (1366, 768),
            (2560, 1440),
            (1000, 701),
        ] {
            let eingang = gpu.eingang(&bild);
            let ziel = gpu.ziel(g.0, g.1);
            let ansicht = ziel.create_view(&Default::default());
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            let r = roehre.zeichne(&mut enc, &eingang, &crate::eingang_von(&bild), &ansicht, g)?;
            gpu.queue.submit([enc.finish()]);
            let a = gpu.lies(&ziel);
            let roh = ziel_roh(&gpu, g.0, g.1);
            let rahmen = Rahmen::neu((bild.breite, bild.hoehe), (r.breite, r.hoehe));
            stufe_ausfuehren(
                &gpu,
                &roehre_wgsl::SCHARF_BILINEAR,
                &eingang,
                &rahmen,
                &roh,
                Some((r.x, r.y, r.breite, r.hoehe)),
            );
            let b = gpu.lies(&roh);
            let (mut groesste, mut ueber1) = (0u8, 0usize);
            for y in r.y..r.y + r.hoehe {
                for x in r.x..r.x + r.breite {
                    let (p, q) = (a.get_pixel(x, y).0, b.get_pixel(x, y).0);
                    let d = (0..3).map(|c| p[c].abs_diff(q[c])).max().unwrap_or(0);
                    groesste = groesste.max(d);
                    ueber1 += usize::from(d > 1);
                }
            }
            eprintln!(
                "scharf-bilinear {name} {}×{} (Fläche {}×{}): größte Abweichung {groesste}, über 1: {ueber1}",
                g.0, g.1, r.breite, r.hoehe
            );
            if groesste > 2 {
                fehler.push(format!(
                    "scharf-bilinear {name} {}×{}: Abweichung {groesste}",
                    g.0, g.1
                ));
            }
        }
    }

    // 1b. Regler »Weichheit«: 1 rechnet dasselbe wie lineares Abtasten, auch
    // auf einem beschnittenen Ausschnitt; 0,5 liegt dazwischen (weder gleich
    // 0 noch gleich 1).
    for name in ["pixelszene", "schrift"] {
        let bild = crate::lade_bild(name)?;
        let eingang = gpu.eingang(&bild);
        for (g, ab) in [
            ((1000u32, 701u32), [0.0f32, 0.0, 1.0, 1.0]),
            (
                (1366, 768),
                [
                    8.0 / bild.breite as f32,
                    0.0,
                    1.0 - 8.0 / bild.breite as f32,
                    1.0,
                ],
            ),
        ] {
            let lauf = |stufe: &roehre_wgsl::Stufe, w: f32| {
                let roh = ziel_roh(&gpu, g.0, g.1);
                let mut rahmen = Rahmen::neu((bild.breite, bild.hoehe), g)
                    .mit_reglern(stufe, &[("weichheit", w)]);
                rahmen.ausschnitt = ab;
                stufe_ausfuehren(&gpu, stufe, &eingang, &rahmen, &roh, None);
                gpu.lies(&roh)
            };
            let (w0, w5, w1, lin) = (
                lauf(&roehre_wgsl::SCHARF_BILINEAR, 0.0),
                lauf(&roehre_wgsl::SCHARF_BILINEAR, 0.5),
                lauf(&roehre_wgsl::SCHARF_BILINEAR, 1.0),
                lauf(&roehre_wgsl::LINEAR, 0.0),
            );
            let abstand = |a: &image::RgbaImage, b: &image::RgbaImage| {
                let mut groesste = 0u8;
                let mut anders = 0usize;
                for (p, q) in a.pixels().zip(b.pixels()) {
                    let d = (0..3).map(|c| p.0[c].abs_diff(q.0[c])).max().unwrap_or(0);
                    groesste = groesste.max(d);
                    anders += usize::from(d > 0);
                }
                (groesste, anders)
            };
            let (g1, _) = abstand(&w1, &lin);
            let (_, a50) = abstand(&w5, &w0);
            let (_, a51) = abstand(&w5, &w1);
            eprintln!(
                "scharf-bilinear Weichheit {name} {}×{}: 1 gegen linear größte Abweichung {g1}; 0,5 anders als 0: {a50}, anders als 1: {a51} Punkte",
                g.0, g.1
            );
            if g1 > 1 || a50 == 0 || a51 == 0 {
                fehler.push(format!(
                    "scharf-bilinear Weichheit {name} {}×{}: 1 gegen linear {g1}, 0,5: {a50}/{a51}",
                    g.0, g.1
                ));
            }
        }
    }

    // 2. mmpx: GPU gegen CPU, ganzes Bild und ein beschnittener Ausschnitt.
    for name in ["pixelszene", "schrift", "farbbalken"] {
        let bild = crate::lade_bild(name)?;
        for ausschnitt in [
            (0, bild.breite, 0, bild.hoehe),
            (8, bild.breite - 8, 0, bild.hoehe - 1),
        ] {
            let eingang = gpu.eingang(&bild);
            let mut rahmen =
                Rahmen::neu((bild.breite, bild.hoehe), (2 * bild.breite, 2 * bild.hoehe));
            let (bb, bh) = (bild.breite as f32, bild.hoehe as f32);
            rahmen.ausschnitt = [
                ausschnitt.0 as f32 / bb,
                ausschnitt.2 as f32 / bh,
                ausschnitt.1 as f32 / bb,
                ausschnitt.3 as f32 / bh,
            ];
            let (zb, zh) = rahmen.zielgroesse(&roehre_wgsl::MMPX);
            let roh = ziel_roh(&gpu, zb, zh);
            stufe_ausfuehren(&gpu, &roehre_wgsl::MMPX, &eingang, &rahmen, &roh, None);
            let g = gpu.lies(&roh);
            let cpu = roehre_wgsl::mmpx::verdoppeln(
                &bild.rgba,
                bild.breite as usize,
                (
                    ausschnitt.0 as usize,
                    ausschnitt.1 as usize,
                    ausschnitt.2 as usize,
                    ausschnitt.3 as usize,
                ),
            );
            let w = 2 * (ausschnitt.1 - ausschnitt.0);
            let mut anders = 0usize;
            for (i, &p) in cpu.iter().enumerate() {
                let (x, y) = (
                    2 * ausschnitt.0 + i as u32 % w,
                    2 * ausschnitt.2 + i as u32 / w,
                );
                let q = g.get_pixel(x, y).0;
                if [(p >> 16) as u8, (p >> 8) as u8, p as u8] != [q[0], q[1], q[2]] {
                    anders += 1;
                }
            }
            eprintln!(
                "mmpx {name} Ausschnitt {ausschnitt:?}: {anders} von {} Punkten anders als die CPU",
                cpu.len()
            );
            if anders > 0 {
                fehler.push(format!(
                    "mmpx {name} {ausschnitt:?}: {anders} Punkte anders"
                ));
            }
        }
    }
    // 2a. mmpx gegen die Fassung der Autoren (slang von hunterk, MIT) über
    // librashader — der eigentliche Prüfstein der Regeln.
    {
        use librashader::runtime::Viewport;
        use librashader::runtime::wgpu::{FilterChain, FilterChainOptions, WgpuOutputView};
        let preset = librashader::presets::ShaderPreset::try_parse(
            shader.join("eigen/mmpx-pruefung.slangp"),
            librashader::presets::ShaderFeatures::NONE,
        )?;
        let mut kette = FilterChain::load_from_preset(
            preset,
            &gpu.device,
            &gpu.queue,
            Some(&FilterChainOptions {
                force_no_mipmaps: true,
                enable_cache: false,
                adapter_info: None,
            }),
        )?;
        for name in ["pixelszene", "schrift", "farbbalken"] {
            let bild = crate::lade_bild(name)?;
            let eingang = gpu.eingang(&bild);
            let g = (2 * bild.breite, 2 * bild.hoehe);
            let ziel_l = ziel_roh(&gpu, g.0, g.1);
            let ansicht = ziel_l.create_view(&Default::default());
            let aus = WgpuOutputView::new_from_raw(
                &ansicht,
                librashader::runtime::Size::new(g.0, g.1),
                wgpu::TextureFormat::Rgba8Unorm,
            );
            let vp = Viewport::new_render_target_sized_origin(aus, None).expect("Größe bekannt");
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            kette.frame(&eingang, &vp, &mut enc, 0, None)?;
            gpu.queue.submit([enc.finish()]);
            let a = gpu.lies(&ziel_l);
            let roh = ziel_roh(&gpu, g.0, g.1);
            stufe_ausfuehren(
                &gpu,
                &roehre_wgsl::MMPX,
                &eingang,
                &Rahmen::neu((bild.breite, bild.hoehe), g),
                &roh,
                None,
            );
            let b = gpu.lies(&roh);
            let anders = a
                .pixels()
                .zip(b.pixels())
                .filter(|(p, q)| p.0[..3] != q.0[..3])
                .count();
            eprintln!(
                "mmpx {name}: gegen die Fassung der Autoren {anders} von {} Punkten anders",
                g.0 * g.1
            );
            if anders > 0 {
                fehler.push(format!(
                    "mmpx {name}: {anders} Punkte anders als die Autoren-Fassung"
                ));
            }
        }
    }

    // 2b. lcd-raster: Fotos (WGSL_FOTOS) und ein Maß gegen Moiré — auf einer
    // weißen Fläche muß jede Zellreihe gleich hell sein (Mittel je Zelle).
    {
        let bild = crate::lade_bild("pixelszene")?;
        let weiss = Bild {
            breite: 64,
            hoehe: 48,
            rgba: vec![255; 64 * 48 * 4],
            pixel_seitenverhaeltnis: 1.0,
        };
        for g in [(1440u32, 1080u32), (935, 701)] {
            for (name, b) in [("pixelszene", &bild), ("weiss", &weiss)] {
                let eingang = gpu.eingang(b);
                let roh = ziel_roh(&gpu, g.0, g.1);
                let rahmen = Rahmen::neu((b.breite, b.hoehe), g)
                    .mit_reglern(&roehre_wgsl::LCD_RASTER, &[("subpixel", 0.5)]);
                stufe_ausfuehren(
                    &gpu,
                    &roehre_wgsl::LCD_RASTER,
                    &eingang,
                    &rahmen,
                    &roh,
                    None,
                );
                let a = gpu.lies(&roh);
                if let Some(ordner) = std::env::var_os("WGSL_FOTOS") {
                    let _ = a.save(
                        std::path::Path::new(&ordner)
                            .join(format!("lcd-{name}-{}x{}.png", g.0, g.1)),
                    );
                }
                if name == "weiss" {
                    // Moiré ist großräumig: Spaltenhelligkeit, gleitend über vier
                    // Zellperioden gemittelt — ein gleichmäßiges Raster ergibt
                    // überall dasselbe. (Je Zelle gezählt wäre falsch: bei
                    // krummen Faktoren sind Zellen 14 oder 15 Punkte breit, und
                    // ein Randpunkt überdeckt zwei.)
                    let zb = g.0 as f64 / b.breite as f64;
                    let spalten: Vec<f64> = (0..g.0)
                        .map(|x| {
                            (0..g.1).map(|y| a.get_pixel(x, y).0[1] as f64).sum::<f64>()
                                / g.1 as f64
                        })
                        .collect();
                    // Fenster von genau vier Perioden (4 · zb, auch gebrochen):
                    // die letzte Spalte zählt mit ihrem Bruchteil.
                    let l = 4.0 * zb;
                    let ganz = l.floor() as usize;
                    let rest = l - ganz as f64;
                    let gleitend: Vec<f64> = (0..spalten.len() - ganz - 1)
                        .map(|x| {
                            (spalten[x..x + ganz].iter().sum::<f64>() + rest * spalten[x + ganz])
                                / l
                        })
                        .collect();
                    let m = gleitend.iter().sum::<f64>() / gleitend.len() as f64;
                    let abw = gleitend
                        .iter()
                        .map(|v| (v - m).abs() / m)
                        .fold(0.0, f64::max);
                    eprintln!(
                        "lcd-raster {}×{}: Helligkeit über vier Zellen weicht höchstens {:.2} % vom Mittel ab",
                        g.0,
                        g.1,
                        abw * 100.0
                    );
                    if abw > 0.01 {
                        fehler.push(format!(
                            "lcd-raster {}×{}: Moiré {:.2} %",
                            g.0,
                            g.1,
                            abw * 100.0
                        ));
                    }
                }
            }
        }
    }

    // 2c. farbe: neutral = Bild unverändert (±1 Rundung), Sättigung 0 = grau.
    {
        let bild = crate::lade_bild("farbbalken")?;
        let eingang = gpu.eingang(&bild);
        for (name, werte, grau) in [
            ("neutral", vec![], false),
            ("grau", vec![("saettigung", 0.0f32)], true),
        ] {
            let roh = ziel_roh(&gpu, bild.breite, bild.hoehe);
            let rahmen = Rahmen::neu((bild.breite, bild.hoehe), (bild.breite, bild.hoehe))
                .mit_reglern(&roehre_wgsl::FARBE, &werte);
            stufe_ausfuehren(&gpu, &roehre_wgsl::FARBE, &eingang, &rahmen, &roh, None);
            let a = gpu.lies(&roh);
            let mut groesste = 0u8;
            for (i, p) in a.pixels().enumerate() {
                let q = &bild.rgba[i * 4..i * 4 + 3];
                let d = if grau {
                    p.0[0].abs_diff(p.0[1]).max(p.0[1].abs_diff(p.0[2]))
                } else {
                    (0..3).map(|c| p.0[c].abs_diff(q[c])).max().unwrap_or(0)
                };
                groesste = groesste.max(d);
            }
            eprintln!("farbe {name}: größte Abweichung {groesste}");
            if groesste > 1 {
                fehler.push(format!("farbe {name}: Abweichung {groesste}"));
            }
        }
    }

    // 2d. leuchten: Stärke 0 = Bild unverändert; ein heller Punkt auf
    // Schwarz leuchtet symmetrisch, nach außen abnehmend, und jenseits von
    // gut 3 σ gar nicht; eine Fläche unter der Schwelle bleibt, wie sie ist;
    // die Weite ist in Spielpunkten gemessen (doppeltes Ziel = doppelter Hof).
    {
        let bild = crate::lade_bild("pixelszene")?;
        let eingang = gpu.eingang(&bild);
        let roh = ziel_roh(&gpu, bild.breite, bild.hoehe);
        let rahmen = Rahmen::neu((bild.breite, bild.hoehe), (bild.breite, bild.hoehe))
            .mit_reglern(&roehre_wgsl::LEUCHTEN, &[("staerke", 0.0)]);
        stufe_ausfuehren_mit(
            &gpu,
            &roehre_wgsl::LEUCHTEN,
            &eingang,
            Some((&eingang, &[("staerke", 0.0)])),
            &rahmen,
            &roh,
            None,
        );
        let a = gpu.lies(&roh);
        let groesste = a
            .pixels()
            .enumerate()
            .map(|(i, p)| {
                (0..3)
                    .map(|c| p.0[c].abs_diff(bild.rgba[i * 4 + c]))
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);
        eprintln!("leuchten Stärke 0: größte Abweichung {groesste}");
        if groesste > 1 {
            fehler.push(format!("leuchten Stärke 0: Abweichung {groesste}"));
        }

        // Ein weißer Punkt in der Mitte von 41 × 41 Schwarz, Ziel n-fach
        // vergrößert (nächster Punkt), Weite 1,5.
        let (b, h) = (41u32, 41u32);
        let mut rgba = vec![0u8; (b * h * 4) as usize];
        for y in 0..h {
            for x in 0..b {
                rgba[((y * b + x) * 4 + 3) as usize] = 255;
            }
        }
        let m = ((20 * b + 20) * 4) as usize;
        rgba[m..m + 3].copy_from_slice(&[255, 255, 255]);
        let punkt = Bild {
            breite: b,
            hoehe: h,
            rgba,
            pixel_seitenverhaeltnis: 1.0,
        };
        let orig = gpu.eingang(&punkt);
        let mut hof = vec![];
        for n in [3u32, 6] {
            let (zb, zh) = (b * n, h * n);
            let gross = Bild {
                breite: zb,
                hoehe: zh,
                rgba: (0..zh)
                    .flat_map(|y| {
                        (0..zb).flat_map(move |x| {
                            let i = (((y / n) * b + x / n) * 4) as usize;
                            [i, i + 1, i + 2, i + 3]
                        })
                    })
                    .map(|i| punkt.rgba[i])
                    .collect(),
                pixel_seitenverhaeltnis: 1.0,
            };
            let quelle = gpu.eingang(&gross);
            let roh = ziel_roh(&gpu, zb, zh);
            let werte = [("staerke", 1.0f32), ("weite", 1.5), ("schwelle", 0.5)];
            let rahmen =
                Rahmen::neu((zb, zh), (zb, zh)).mit_reglern(&roehre_wgsl::LEUCHTEN, &werte);
            stufe_ausfuehren_mit(
                &gpu,
                &roehre_wgsl::LEUCHTEN,
                &quelle,
                Some((&orig, &werte)),
                &rahmen,
                &roh,
                None,
            );
            let a = gpu.lies(&roh);
            let wert = |x: u32, y: u32| a.get_pixel(x, y).0[1];
            let c = zb / 2;
            // Symmetrie (links/rechts, oben/unten, quer).
            let mut asym = 0u8;
            for y in 0..zh {
                for x in 0..zb {
                    asym = asym
                        .max(wert(x, y).abs_diff(wert(zb - 1 - x, y)))
                        .max(wert(x, y).abs_diff(wert(x, zh - 1 - y)))
                        .max(wert(x, y).abs_diff(wert(y.min(zb - 1), x.min(zh - 1))));
                }
            }
            // Nach außen nicht zunehmend (auf der Mittelzeile).
            let mut steigt = 0usize;
            for x in c..zb - 1 {
                steigt += usize::from(wert(x + 1, c) > wert(x, c));
            }
            // Jenseits von 3 σ = 4,5 Spielpunkten (+ 1 für die lineare
            // Abtastung des Hofs) schwarz.
            let rand = ((20.5 + 4.5 + 1.0) * n as f32).ceil() as u32;
            let aussen = (rand..zb).map(|x| wert(x, c)).max().unwrap_or(0);
            // Halbwertsbreite des Hofs in Bildpunkten (Mittelzeile, rechts).
            let mitte = wert(c + n, c);
            let halb = (c + n..zb).find(|&x| wert(x, c) <= mitte / 2).unwrap_or(zb) - c;
            eprintln!(
                "leuchten Punkt ×{n}: Asymmetrie {asym}, steigend {steigt}, außen {aussen}, neben dem Punkt {mitte}, halb bei {halb} Bildpunkten"
            );
            if asym > 1 || steigt > 0 || aussen > 0 || mitte == 0 {
                fehler.push(format!("leuchten Punkt ×{n}: Asymmetrie {asym}, steigend {steigt}, außen {aussen}, neben {mitte}"));
            }
            hof.push(halb as f32 / n as f32);
        }
        if (hof[0] - hof[1]).abs() > 0.5 {
            fehler.push(format!(
                "leuchten: Hof in Spielpunkten hängt von der Größe ab: {hof:?}"
            ));
        }

        // Grau unter der Schwelle (linear 0,3 < 0,5): unverändert.
        let grau = Bild {
            breite: 16,
            hoehe: 16,
            rgba: [149u8, 149, 149, 255].repeat(256),
            pixel_seitenverhaeltnis: 1.0,
        };
        let gq = gpu.eingang(&grau);
        let roh = ziel_roh(&gpu, 16, 16);
        let rahmen = Rahmen::neu((16, 16), (16, 16))
            .mit_reglern(&roehre_wgsl::LEUCHTEN, &[("staerke", 1.0)]);
        stufe_ausfuehren_mit(
            &gpu,
            &roehre_wgsl::LEUCHTEN,
            &gq,
            Some((&gq, &[("staerke", 1.0)])),
            &rahmen,
            &roh,
            None,
        );
        let g = gpu
            .lies(&roh)
            .pixels()
            .map(|p| p.0[0].abs_diff(149))
            .max()
            .unwrap_or(0);
        eprintln!("leuchten Grau unter der Schwelle: größte Abweichung {g}");
        if g > 1 {
            fehler.push(format!("leuchten Grau unter der Schwelle: {g}"));
        }
    }

    // 3. Erzeugte Ketten (crt-beans) gegen librashaders eigene FilterChain.
    erzeugte_vergleichen(&gpu, shader, &mut fehler)?;
    let _ = Bild::neu;
    if fehler.is_empty() {
        eprintln!("roehre-wgsl: alle Vergleiche bestanden");
        Ok(())
    } else {
        Err(fehler.join("\n").into())
    }
}

/// Dieselbe Kette einmal über librashader, einmal über den Ausführer der
/// erzeugten WGSL-Pässe; Standardregler und die Regler, die roehre für die
/// MIT-Voreinstellungen plant.
fn erzeugte_vergleichen(gpu: &Gpu, shader: &std::path::Path, fehler: &mut Vec<String>) -> Ergebnis {
    use librashader::runtime::wgpu::{
        FilterChain, FilterChainOptions, FrameOptions, WgpuOutputView,
    };
    use librashader::runtime::{FilterChainParameters, Viewport};
    // (Kette, Voreinstellung von roehre für geplante Regler, wenn es eine gibt)
    let faelle: [(&'static roehre_wgsl::kette::Erzeugt, Option<&str>); 5] = [
        (&roehre_wgsl::erzeugt::beans::BEANS_RGB, Some("mit-rgb")),
        (
            &roehre_wgsl::erzeugt::beans::BEANS_SIGNAL,
            Some("mit-composite"),
        ),
        (&roehre_wgsl::erzeugt::beans::BEANS_SCHNELL, Some("mit-rgb")),
        (&roehre_wgsl::erzeugt::pixelkunst::SCALEFX, None),
        (&roehre_wgsl::erzeugt::pixelkunst::OMNISCALE, None),
    ];
    for (erzeugt, voreinstellung) in faelle {
        for name in ["pixelszene", "schrift"] {
            let bild = crate::lade_bild(name)?;
            for g in [(960u32, 720u32), (1440, 1080), (935, 701)] {
                for geplant in [false, voreinstellung.is_some()]
                    .into_iter()
                    .collect::<std::collections::BTreeSet<_>>()
                {
                    let mut regler: Vec<(String, f32)> = vec![];
                    if geplant {
                        let e = Einstellungen::voreinstellung(
                            voreinstellung.expect("geplant nur mit Voreinstellung"),
                        )?;
                        let k = roehre_kette::Kontext {
                            eingang: (bild.breite, bild.hoehe),
                            flaeche: g,
                            hdr: false,
                            flaeche_modus: false,
                        };
                        regler = roehre_kette::plane(&e, &k)
                            .parameter
                            .iter()
                            .map(|(n, w)| (n.to_string(), *w))
                            .collect();
                    }
                    let eingang = gpu.eingang(&bild);
                    let seite = bild.breite as f32 * bild.pixel_seitenverhaeltnis as f32
                        / bild.hoehe as f32;
                    // librashader
                    let mut kette_l = FilterChain::load_from_preset(
                        librashader::presets::ShaderPreset::try_parse(
                            shader.join(erzeugt.preset),
                            librashader::presets::ShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
                                | librashader::presets::ShaderFeatures::FRAMETIME_UNIFORMS,
                        )?,
                        &gpu.device,
                        &gpu.queue,
                        Some(&FilterChainOptions {
                            force_no_mipmaps: true,
                            enable_cache: false,
                            adapter_info: None,
                        }),
                    )?;
                    for (n, w) in &regler {
                        kette_l.parameters().set_parameter_value(n, *w);
                    }
                    let ziel_l = ziel_roh(gpu, g.0, g.1);
                    let ansicht_l = ziel_l.create_view(&Default::default());
                    let aus = WgpuOutputView::new_from_raw(
                        &ansicht_l,
                        librashader::runtime::Size::new(g.0, g.1),
                        wgpu::TextureFormat::Rgba8Unorm,
                    );
                    let vp =
                        Viewport::new_render_target_sized_origin(aus, None).expect("Größe bekannt");
                    let mut enc = gpu.device.create_command_encoder(&Default::default());
                    kette_l.frame(
                        &eingang,
                        &vp,
                        &mut enc,
                        3,
                        Some(&FrameOptions {
                            aspect_ratio: seite,
                            ..Default::default()
                        }),
                    )?;
                    gpu.queue.submit([enc.finish()]);
                    let a = gpu.lies(&ziel_l);
                    // Ausführer der erzeugten Pässe
                    let mut ausf = crate::wgsl_kette::Ausfuehrer::neu(
                        &gpu.device,
                        &gpu.queue,
                        erzeugt,
                        wgpu::TextureFormat::Rgba8Unorm,
                    );
                    let ziel_w = ziel_roh(gpu, g.0, g.1);
                    let ansicht_w = ziel_w.create_view(&Default::default());
                    let karte: std::collections::HashMap<&str, f32> =
                        regler.iter().map(|(n, w)| (n.as_str(), *w)).collect();
                    let mut enc = gpu.device.create_command_encoder(&Default::default());
                    ausf.zeichne(
                        &gpu.device,
                        &gpu.queue,
                        &mut enc,
                        &eingang,
                        &ansicht_w,
                        g,
                        3,
                        seite,
                        &karte,
                    );
                    gpu.queue.submit([enc.finish()]);
                    let b = gpu.lies(&ziel_w);
                    if let Some(ordner) = std::env::var_os("WGSL_FOTOS") {
                        let _ = b.save(std::path::Path::new(&ordner).join(format!(
                            "{}-{name}-{}x{}-{}.png",
                            erzeugt.name, g.0, g.1, geplant
                        )));
                    }
                    let hell: u64 = b
                        .pixels()
                        .map(|p| u64::from(p.0[0]) + u64::from(p.0[1]) + u64::from(p.0[2]))
                        .sum();
                    if hell == 0 {
                        fehler.push(format!("{} {name}: Bild ganz schwarz", erzeugt.name));
                    }
                    let (mut groesste, mut ueber1) = (0u8, 0usize);
                    for (p, q) in a.pixels().zip(b.pixels()) {
                        let d = (0..4).map(|c| p.0[c].abs_diff(q.0[c])).max().unwrap_or(0);
                        groesste = groesste.max(d);
                        ueber1 += usize::from(d > 1);
                    }
                    eprintln!(
                        "{} {name} {}×{} {}: größte Abweichung {groesste}, über 1: {ueber1} von {}",
                        erzeugt.name,
                        g.0,
                        g.1,
                        if geplant {
                            voreinstellung.unwrap_or("")
                        } else {
                            "Standardregler"
                        },
                        g.0 * g.1
                    );
                    if groesste > 2 {
                        fehler.push(format!(
                            "{} {name} {}×{}: Abweichung {groesste}",
                            erzeugt.name, g.0, g.1
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
