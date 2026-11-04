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
    let _ = Bild::neu;
    if fehler.is_empty() {
        eprintln!("roehre-wgsl: alle Vergleiche bestanden");
        Ok(())
    } else {
        Err(fehler.join("\n").into())
    }
}
