//! Vorschaubild für Programme, die selbst keine Shader ausführen (z. B. ein Launcher, der in
//! Software zeichnet; Wunsch W-007): roehre rechnet auf einem eigenen Gerät ohne Fenster und
//! liefert das Ergebnis als RGBA-Puffer in den Arbeitsspeicher.

use std::ffi::c_char;
use std::path::PathBuf;
use std::ptr;
use std::time::Instant;

use roehre_wgpu::{Eingang, Einstellungen};

use crate::arbeitsplatz::{Arbeitsplatz, Modus};
use crate::einstellungen::{innen, RoehreEinstellungen};
use crate::fehler::{geschuetzt, text};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Undurchsichtiger Vorschau-Rechner.
pub struct RoehreVorschau {
    platz: Arbeitsplatz,
    eingang: Option<(wgpu::Texture, u32, u32)>,
    ziel: Option<wgpu::Texture>,
    start: Instant,
}

pub(crate) fn geraet() -> Result<(wgpu::Device, wgpu::Queue), String> {
    pollster::block_on(async {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })
            .await
            .map_err(|e| format!("kein Grafikadapter: {e}"))?;
        let features = adapter.features() & roehre_wgpu::empfohlene_features();
        adapter
            .request_device(&wgpu::DeviceDescriptor { label: Some("roehre-vorschau"), required_features: features, ..Default::default() })
            .await
            .map_err(|e| format!("Gerät nicht angelegt: {e}"))
    })
}

/// Legt einen Vorschau-Rechner an. `shader_ordner`: der mitgelieferte Ordner `shader/`.
/// Dauert einige hundert Millisekunden (Gerät, Shader); einmal anlegen, oft zeichnen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_vorschau_erstellen(shader_ordner: *const c_char) -> *mut RoehreVorschau {
    geschuetzt(ptr::null_mut(), || {
        let ordner = PathBuf::from(unsafe { text(shader_ordner, "Shader-Ordner") }?);
        let (device, queue) = geraet()?;
        let platz = Arbeitsplatz::neu(device, queue, ordner, FORMAT.into(), Einstellungen::standard())?;
        Ok(Box::into_raw(Box::new(RoehreVorschau { platz, eingang: None, ziel: None, start: Instant::now() })))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_vorschau_freigeben(v: *mut RoehreVorschau) {
    if !v.is_null() {
        drop(unsafe { Box::from_raw(v) });
    }
}

/// Zeichnet ein Vorschaubild. `bild_rgba` (sRGB, 4 Byte je Pixel, Zeile für Zeile ohne
/// Füllung) ist das Spielbild; NULL nimmt das eingebaute Testbild. `modus` 0 = flach, 1 = 3D-Raum.
/// `zeit` in Sekunden (Schwenk, Flackern; für ein Standbild 0). Ergebnis nach `ziel_rgba`
/// (`ziel_breite` × `ziel_hoehe` × 4 Byte, sRGB). Rückgabe 0 oder < 0 bei Fehler.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_vorschau_zeichnen(
    v: *mut RoehreVorschau,
    e: *const RoehreEinstellungen,
    modus: i32,
    zeit: f32,
    bild_rgba: *const u8,
    bild_breite: u32,
    bild_hoehe: u32,
    pixel_seitenverhaeltnis: f32,
    ziel_rgba: *mut u8,
    ziel_breite: u32,
    ziel_hoehe: u32,
) -> i32 {
    geschuetzt(-1, || {
        let v = unsafe { v.as_mut() }.ok_or("Vorschau fehlt (NULL)")?;
        let e = unsafe { innen(e) }?;
        let modus = Modus::aus(modus)?;
        if ziel_rgba.is_null() || ziel_breite == 0 || ziel_hoehe == 0 || ziel_breite > 8192 || ziel_hoehe > 8192 {
            return Err("Zielpuffer fehlt oder Größe ungültig (1 … 8192)".into());
        }
        // Spielbild: übergeben oder Testbild
        let testbild;
        let (rgba, b, h, psv): (&[u8], u32, u32, f64) = if bild_rgba.is_null() {
            testbild = roehre_testbilder::pixelszene();
            (&testbild.rgba, testbild.breite, testbild.hoehe, testbild.pixel_seitenverhaeltnis)
        } else {
            if bild_breite == 0 || bild_hoehe == 0 || bild_breite > 8192 || bild_hoehe > 8192 {
                return Err("Bildgröße ungültig (1 … 8192)".into());
            }
            let n = (bild_breite * bild_hoehe * 4) as usize;
            (unsafe { std::slice::from_raw_parts(bild_rgba, n) }, bild_breite, bild_hoehe, if pixel_seitenverhaeltnis > 0.0 { pixel_seitenverhaeltnis as f64 } else { 1.0 })
        };
        v.platz.einstellungen(e, true)?;
        let (device, queue) = (v.platz.device.clone(), v.platz.queue.clone());
        if v.eingang.as_ref().map(|(_, eb, eh)| (*eb, *eh)) != Some((b, h)) {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("roehre-vorschau: Spielbild"),
                size: wgpu::Extent3d { width: b, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            v.eingang = Some((t, b, h));
        }
        let eingang_t = &v.eingang.as_ref().expect("angelegt").0;
        queue.write_texture(
            eingang_t.as_image_copy(),
            rgba,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * b), rows_per_image: None },
            eingang_t.size(),
        );
        if v.ziel.as_ref().map(|z| (z.width(), z.height())) != Some((ziel_breite, ziel_hoehe)) {
            v.ziel = Some(device.create_texture(&wgpu::TextureDescriptor {
                label: Some("roehre-vorschau: Ziel"),
                size: wgpu::Extent3d { width: ziel_breite, height: ziel_hoehe, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            }));
        }
        let ziel = v.ziel.as_ref().expect("angelegt").clone();
        let eingang = Eingang { breite: b, hoehe: h, pixel_seitenverhaeltnis: psv };
        // Nachglühen und Strahlverlauf brauchen Vorgängerbilder: drei Durchgänge
        let zeit = if zeit >= 0.0 { zeit } else { v.start.elapsed().as_secs_f32() };
        for _ in 0..3 {
            let mut enc = device.create_command_encoder(&Default::default());
            v.platz.zeichne(&mut enc, eingang_t, &eingang, &ziel.create_view(&Default::default()), (ziel_breite, ziel_hoehe), modus, zeit, roehre_wgpu::Takt::EIN_BILD)?;
            queue.submit([enc.finish()]);
        }
        let px = lies(&device, &queue, &ziel)?;
        unsafe { std::ptr::copy_nonoverlapping(px.as_ptr(), ziel_rgba, px.len()) };
        Ok(0)
    })
}

fn lies(device: &wgpu::Device, queue: &wgpu::Queue, t: &wgpu::Texture) -> Result<Vec<u8>, String> {
    let (b, h) = (t.width(), t.height());
    let zeile = (4 * b).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let puffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("roehre-vorschau: Rücklesen"),
        size: (zeile * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo { buffer: &puffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(zeile), rows_per_image: None } },
        t.size(),
    );
    queue.submit([enc.finish()]);
    puffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("GPU-Warten: {e}"))?;
    let daten = puffer.slice(..).get_mapped_range().map_err(|e| format!("Puffer: {e:?}"))?;
    let mut px = Vec::with_capacity((4 * b * h) as usize);
    for y in 0..h {
        let a = (y * zeile) as usize;
        px.extend_from_slice(&daten[a..a + (4 * b) as usize]);
    }
    drop(daten);
    puffer.unmap();
    Ok(px)
}
