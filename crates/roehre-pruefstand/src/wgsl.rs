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

use crate::gpu::Gpu;
use crate::Ergebnis;

/// Eine Stufe ausführen: Quelle → Ziel (ganzes Ziel oder `viewport`).
pub fn stufe_ausfuehren(gpu: &Gpu, stufe: &Stufe, quelle: &wgpu::Texture, rahmen: &Rahmen, ziel: &wgpu::Texture, viewport: Option<(u32, u32, u32, u32)>) {
    let d = &gpu.device;
    let modul = d.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(stufe.name), source: wgpu::ShaderSource::Wgsl(stufe.wgsl().into()) });
    // Das Layout ausdrücklich nach der Vereinbarung (alle drei Bindungen): ein
    // abgeleitetes Layout ließe den Abtaster weg, wo eine Stufe ihn nicht
    // liest (mmpx liest mit textureLoad).
    let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("roehre-wgsl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: wgpu::BufferSize::new(128) },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                count: None,
            },
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
        ],
    });
    let pipeline_layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(stufe.name), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
    let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(stufe.name),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState { module: &modul, entry_point: Some("ecke"), buffers: &[], compilation_options: Default::default() },
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
    let abtaster = d.create_sampler(&wgpu::SamplerDescriptor { mag_filter: filter, min_filter: filter, ..Default::default() });
    let puffer = d.create_buffer(&wgpu::BufferDescriptor { label: Some("Rahmen"), size: 128, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    gpu.queue.write_buffer(&puffer, 0, &rahmen.als_bytes());
    let quelle_ansicht = quelle.create_view(&Default::default());
    let bindung = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(stufe.name),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: puffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&quelle_ansicht) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&abtaster) },
        ],
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
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
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
        size: wgpu::Extent3d { width: b, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

/// `roehre-pruefstand wgsl`: beide Vergleiche; Fehler, wenn einer reißt.
pub fn pruefen(shader: &std::path::Path) -> Ergebnis {
    let gpu = Gpu::neu();
    eprintln!("Adapter: {}", gpu.adapter_name);
    let mut fehler = vec![];

    // 1. scharf-bilinear gegen `aus`.
    let mut roehre = Roehre::neu(&gpu.device, &gpu.queue, shader, Einstellungen::voreinstellung("aus")?, crate::gpu::ZIELFORMAT)?;
    for name in ["pixelszene", "schrift", "farbbalken"] {
        let bild = crate::lade_bild(name)?;
        for g in [(1920, 1080), (1280, 720), (1366, 768), (2560, 1440), (1000, 701)] {
            let eingang = gpu.eingang(&bild);
            let ziel = gpu.ziel(g.0, g.1);
            let ansicht = ziel.create_view(&Default::default());
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            let r = roehre.zeichne(&mut enc, &eingang, &crate::eingang_von(&bild), &ansicht, g)?;
            gpu.queue.submit([enc.finish()]);
            let a = gpu.lies(&ziel);
            let roh = ziel_roh(&gpu, g.0, g.1);
            let rahmen = Rahmen::neu((bild.breite, bild.hoehe), (r.breite, r.hoehe));
            stufe_ausfuehren(&gpu, &roehre_wgsl::SCHARF_BILINEAR, &eingang, &rahmen, &roh, Some((r.x, r.y, r.breite, r.hoehe)));
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
            eprintln!("scharf-bilinear {name} {}×{} (Fläche {}×{}): größte Abweichung {groesste}, über 1: {ueber1}", g.0, g.1, r.breite, r.hoehe);
            if groesste > 2 {
                fehler.push(format!("scharf-bilinear {name} {}×{}: Abweichung {groesste}", g.0, g.1));
            }
        }
    }

    // 2. mmpx: GPU gegen CPU, ganzes Bild und ein beschnittener Ausschnitt.
    for name in ["pixelszene", "schrift", "farbbalken"] {
        let bild = crate::lade_bild(name)?;
        for ausschnitt in [(0, bild.breite, 0, bild.hoehe), (8, bild.breite - 8, 0, bild.hoehe - 1)] {
            let eingang = gpu.eingang(&bild);
            let mut rahmen = Rahmen::neu((bild.breite, bild.hoehe), (2 * bild.breite, 2 * bild.hoehe));
            let (bb, bh) = (bild.breite as f32, bild.hoehe as f32);
            rahmen.ausschnitt = [ausschnitt.0 as f32 / bb, ausschnitt.2 as f32 / bh, ausschnitt.1 as f32 / bb, ausschnitt.3 as f32 / bh];
            let (zb, zh) = rahmen.zielgroesse(&roehre_wgsl::MMPX);
            let roh = ziel_roh(&gpu, zb, zh);
            stufe_ausfuehren(&gpu, &roehre_wgsl::MMPX, &eingang, &rahmen, &roh, None);
            let g = gpu.lies(&roh);
            let cpu = roehre_wgsl::mmpx::verdoppeln(&bild.rgba, bild.breite as usize, (ausschnitt.0 as usize, ausschnitt.1 as usize, ausschnitt.2 as usize, ausschnitt.3 as usize));
            let w = 2 * (ausschnitt.1 - ausschnitt.0);
            let mut anders = 0usize;
            for (i, &p) in cpu.iter().enumerate() {
                let (x, y) = (2 * ausschnitt.0 + i as u32 % w, 2 * ausschnitt.2 + i as u32 / w);
                let q = g.get_pixel(x, y).0;
                if [(p >> 16) as u8, (p >> 8) as u8, p as u8] != [q[0], q[1], q[2]] {
                    anders += 1;
                }
            }
            eprintln!("mmpx {name} Ausschnitt {ausschnitt:?}: {anders} von {} Punkten anders als die CPU", cpu.len());
            if anders > 0 {
                fehler.push(format!("mmpx {name} {ausschnitt:?}: {anders} Punkte anders"));
            }
        }
    }
    // 2b. lcd-raster: Fotos (WGSL_FOTOS) und ein Maß gegen Moiré — auf einer
    // weißen Fläche muß jede Zellreihe gleich hell sein (Mittel je Zelle).
    {
        let bild = crate::lade_bild("pixelszene")?;
        let weiss = Bild { breite: 64, hoehe: 48, rgba: vec![255; 64 * 48 * 4], pixel_seitenverhaeltnis: 1.0 };
        for g in [(1440u32, 1080u32), (935, 701)] {
            for (name, b) in [("pixelszene", &bild), ("weiss", &weiss)] {
                let eingang = gpu.eingang(b);
                let roh = ziel_roh(&gpu, g.0, g.1);
                let rahmen = Rahmen::neu((b.breite, b.hoehe), g).mit_reglern(&roehre_wgsl::LCD_RASTER, &[("subpixel", 0.5)]);
                stufe_ausfuehren(&gpu, &roehre_wgsl::LCD_RASTER, &eingang, &rahmen, &roh, None);
                let a = gpu.lies(&roh);
                if let Some(ordner) = std::env::var_os("WGSL_FOTOS") {
                    let _ = a.save(std::path::Path::new(&ordner).join(format!("lcd-{name}-{}x{}.png", g.0, g.1)));
                }
                if name == "weiss" {
                    // Moiré ist großräumig: Spaltenhelligkeit, gleitend über vier
                    // Zellperioden gemittelt — ein gleichmäßiges Raster ergibt
                    // überall dasselbe. (Je Zelle gezählt wäre falsch: bei
                    // krummen Faktoren sind Zellen 14 oder 15 Punkte breit, und
                    // ein Randpunkt überdeckt zwei.)
                    let zb = g.0 as f64 / b.breite as f64;
                    let spalten: Vec<f64> = (0..g.0).map(|x| (0..g.1).map(|y| a.get_pixel(x, y).0[1] as f64).sum::<f64>() / g.1 as f64).collect();
                    // Fenster von genau vier Perioden (4 · zb, auch gebrochen):
                    // die letzte Spalte zählt mit ihrem Bruchteil.
                    let l = 4.0 * zb;
                    let ganz = l.floor() as usize;
                    let rest = l - ganz as f64;
                    let gleitend: Vec<f64> = (0..spalten.len() - ganz - 1).map(|x| (spalten[x..x + ganz].iter().sum::<f64>() + rest * spalten[x + ganz]) / l).collect();
                    let m = gleitend.iter().sum::<f64>() / gleitend.len() as f64;
                    let abw = gleitend.iter().map(|v| (v - m).abs() / m).fold(0.0, f64::max);
                    eprintln!("lcd-raster {}×{}: Helligkeit über vier Zellen weicht höchstens {:.2} % vom Mittel ab", g.0, g.1, abw * 100.0);
                    if abw > 0.01 {
                        fehler.push(format!("lcd-raster {}×{}: Moiré {:.2} %", g.0, g.1, abw * 100.0));
                    }
                }
            }
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
    use librashader::runtime::wgpu::{FilterChain, FilterChainOptions, FrameOptions, WgpuOutputView};
    use librashader::runtime::{FilterChainParameters, Viewport};
    use roehre_kette::Kette;
    let faelle: [(&'static roehre_wgsl::kette::Erzeugt, Kette, &str); 3] = [
        (&roehre_wgsl::erzeugt::beans::BEANS_RGB, Kette::BeansRgb, "mit-rgb"),
        (&roehre_wgsl::erzeugt::beans::BEANS_SIGNAL, Kette::BeansSignal, "mit-composite"),
        (&roehre_wgsl::erzeugt::beans::BEANS_SCHNELL, Kette::BeansSchnell, "mit-rgb"),
    ];
    for (erzeugt, kette, voreinstellung) in faelle {
        for name in ["pixelszene", "schrift"] {
            let bild = crate::lade_bild(name)?;
            for g in [(960u32, 720u32), (1440, 1080), (935, 701)] {
                for geplant in [false, true] {
                    let mut regler: Vec<(String, f32)> = vec![];
                    if geplant {
                        let e = Einstellungen::voreinstellung(voreinstellung)?;
                        let k = roehre_kette::Kontext { eingang: (bild.breite, bild.hoehe), flaeche: g, hdr: false, flaeche_modus: false };
                        regler = roehre_kette::plane(&e, &k).parameter.iter().map(|(n, w)| (n.to_string(), *w)).collect();
                    }
                    let eingang = gpu.eingang(&bild);
                    let seite = bild.breite as f32 * bild.pixel_seitenverhaeltnis as f32 / bild.hoehe as f32;
                    // librashader
                    let mut kette_l = FilterChain::load_from_preset(
                        kette.lade(shader)?,
                        &gpu.device,
                        &gpu.queue,
                        Some(&FilterChainOptions { force_no_mipmaps: true, enable_cache: false, adapter_info: None }),
                    )?;
                    for (n, w) in &regler {
                        kette_l.parameters().set_parameter_value(n, *w);
                    }
                    let ziel_l = ziel_roh(gpu, g.0, g.1);
                    let ansicht_l = ziel_l.create_view(&Default::default());
                    let aus = WgpuOutputView::new_from_raw(&ansicht_l, librashader::runtime::Size::new(g.0, g.1), wgpu::TextureFormat::Rgba8Unorm);
                    let vp = Viewport::new_render_target_sized_origin(aus, None).expect("Größe bekannt");
                    let mut enc = gpu.device.create_command_encoder(&Default::default());
                    kette_l.frame(&eingang, &vp, &mut enc, 3, Some(&FrameOptions { aspect_ratio: seite, ..Default::default() }))?;
                    gpu.queue.submit([enc.finish()]);
                    let a = gpu.lies(&ziel_l);
                    // Ausführer der erzeugten Pässe
                    let mut ausf = crate::wgsl_kette::Ausfuehrer::neu(&gpu.device, &gpu.queue, erzeugt, wgpu::TextureFormat::Rgba8Unorm);
                    let ziel_w = ziel_roh(gpu, g.0, g.1);
                    let ansicht_w = ziel_w.create_view(&Default::default());
                    let karte: std::collections::HashMap<&str, f32> = regler.iter().map(|(n, w)| (n.as_str(), *w)).collect();
                    let mut enc = gpu.device.create_command_encoder(&Default::default());
                    ausf.zeichne(&gpu.device, &gpu.queue, &mut enc, &eingang, &ansicht_w, g, 3, seite, &karte);
                    gpu.queue.submit([enc.finish()]);
                    let b = gpu.lies(&ziel_w);
                    if let Some(ordner) = std::env::var_os("WGSL_FOTOS") {
                        let _ = b.save(std::path::Path::new(&ordner).join(format!("{}-{name}-{}x{}-{}.png", erzeugt.name, g.0, g.1, geplant)));
                    }
                    let hell: u64 = b.pixels().map(|p| u64::from(p.0[0]) + u64::from(p.0[1]) + u64::from(p.0[2])).sum();
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
                        if geplant { voreinstellung } else { "Standardregler" },
                        g.0 * g.1
                    );
                    if groesste > 2 {
                        fehler.push(format!("{} {name} {}×{}: Abweichung {groesste}", erzeugt.name, g.0, g.1));
                    }
                }
            }
        }
    }
    Ok(())
}
