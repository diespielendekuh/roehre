//! roehre muss auf einem Gerät ohne jede Zusatzfähigkeit laufen (so sieht ein D3D12-Gerät unter
//! wgpu aus: kein PIPELINE_CACHE). Am 2026-11-02 brach der erste D3D12-Lauf genau daran ab.

use roehre_wgpu::{Eingang, Einstellungen, Roehre};

#[test]
fn alle_voreinstellungen_ohne_zusatzfaehigkeiten() {
    let Some((device, queue)) = pollster::block_on(async {
        let instance = wgpu::Instance::default();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.ok()?;
        adapter
            .request_device(&wgpu::DeviceDescriptor { required_features: wgpu::Features::empty(), ..Default::default() })
            .await
            .ok()
    }) else {
        panic!("kein Grafikadapter (unter Linux ohne GPU: mesa-vulkan-drivers für lavapipe)");
    };
    let shader = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shader");
    let eingang = Eingang { breite: 64, hoehe: 48, pixel_seitenverhaeltnis: 1.0 };
    let textur = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: 64, height: 48, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let ziel = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: 320, height: 240, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let ansicht = ziel.create_view(&Default::default());
    for v in roehre_modell::voreinstellungen() {
        let e = Einstellungen::voreinstellung(v.schluessel).expect("Voreinstellung");
        let mut r = Roehre::neu(&device, &queue, &shader, e, wgpu::TextureFormat::Rgba8UnormSrgb)
            .unwrap_or_else(|f| panic!("{}: {f}", v.schluessel));
        let mut enc = device.create_command_encoder(&Default::default());
        r.zeichne(&mut enc, &textur, &eingang, &ansicht, (320, 240)).unwrap_or_else(|f| panic!("{}: {f}", v.schluessel));
        queue.submit([enc.finish()]);
    }
    device.poll(wgpu::PollType::wait_indefinitely()).expect("warten");
}
