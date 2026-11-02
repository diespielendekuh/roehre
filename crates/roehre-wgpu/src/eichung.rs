//! Helligkeitseichung für HDR.
//!
//! Maske und Zeilen nehmen einen großen Teil des Lichts weg (gemessen 60–65 % bei
//! Schlitzmaske). In SDR gleicht roehre das durch Anheben aus, was die hellen Leuchtstreifen
//! an der Grenze 1,0 abschneidet. In HDR ist Platz nach oben: Die Röhre läuft ohne
//! Nachverstärkung, und der Zusammensetz-Pass skaliert so, dass eine weiße Fläche im Mittel
//! die gewünschte Leuchtdichte hat. Die Leuchtstreifen werden dabei heller als Weiß, so wie
//! auf einer echten Röhre.
//!
//! Dafür muss man wissen, wie viel Licht eine weiße Fläche nach Maske und Zeilen im Mittel
//! noch hat (Transmission `t`). Die Eichung misst das: Eine **eigene** Kette (damit
//! Nachglühen und Verlauf des Spielbilds unberührt bleiben) zeichnet eine weiße Fläche in
//! Eingangsgröße auf die Röhrenfläche, ein Ausschnitt aus der Mitte wird zurückgelesen und
//! gemittelt. Das geschieht nur, wenn sich Einstellungen oder Größen ändern, und läuft
//! nebenher: Bis das Ergebnis da ist, gilt der letzte Wert.

use librashader::runtime::wgpu::FilterChain;
use librashader::runtime::FilterChainParameters;

/// Größe des zurückgelesenen Ausschnitts (Breite × 4 Byte ist ein Vielfaches von 256).
const AUSSCHNITT: (u32, u32) = (192, 96);
/// Annahme, bis die erste Messung da ist.
pub const VORGABE_TRANSMISSION: f64 = 0.4;

pub struct Eichung {
    kette: Option<(roehre_kette::Kette, FilterChain)>,
    weiss: Option<(wgpu::Texture, (u32, u32))>,
    ziel: Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))>,
    puffer: Option<wgpu::Buffer>,
    laeuft: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    schluessel: Option<String>,
    gamma: f64,
    /// Breite, Höhe und Zeilenlänge in Byte des laufenden Ausschnitts.
    form: (u32, u32, u32),
    /// Zuletzt gemessene Transmission einer weißen Fläche (linear, 0…1).
    pub transmission: f64,
}

impl Eichung {
    pub fn neu() -> Self {
        Self {
            kette: None,
            weiss: None,
            ziel: None,
            puffer: None,
            laeuft: None,
            schluessel: None,
            gamma: 2.2,
            form: (0, 0, 0),
            transmission: VORGABE_TRANSMISSION,
        }
    }

    /// Liest ein fertiges Ergebnis ab, falls eins bereitliegt.
    pub fn abholen(&mut self, device: &wgpu::Device) {
        let Some(fertig) = &self.laeuft else { return };
        let _ = device.poll(wgpu::PollType::Poll);
        if !fertig.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        let puffer = self.puffer.as_ref().expect("Puffer gehört zur laufenden Messung");
        {
            let daten = puffer.slice(..).get_mapped_range().expect("abgebildet");
            let lin = |v: u8| (v as f64 / 255.0).powf(self.gamma);
            let (b, h, zeile) = self.form;
            let mut summe = 0.0;
            for y in 0..h {
                let a = (y * zeile) as usize;
                for px in daten[a..a + (4 * b) as usize].chunks_exact(4) {
                    summe += 0.2126 * lin(px[0]) + 0.7152 * lin(px[1]) + 0.0722 * lin(px[2]);
                }
            }
            self.transmission = (summe / (b * h).max(1) as f64).clamp(0.02, 1.0);
        }
        puffer.unmap();
        self.laeuft = None;
    }

    /// Stößt eine Messung an, wenn sich der Schlüssel (Kette, Parameter, Größen) geändert hat.
    #[allow(clippy::too_many_arguments)]
    pub fn bei_bedarf_messen(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_ordner: &std::path::Path,
        plan: &roehre_kette::Plan,
        eingang: (u32, u32),
        flaeche: (u32, u32),
        gamma: f64,
    ) -> Result<(), crate::Fehler> {
        if self.laeuft.is_some() {
            return Ok(());
        }
        let schluessel = format!("{:?}|{:?}|{:?}|{:?}", plan.kette, plan.parameter, eingang, flaeche);
        if self.schluessel.as_deref() == Some(schluessel.as_str()) {
            return Ok(());
        }
        // Kette passend halten (Bau nur beim Wechsel der Kettenart)
        if self.kette.as_ref().map(|k| k.0) != Some(plan.kette) {
            self.kette = Some((plan.kette, crate::baue(device, queue, shader_ordner, plan.kette)?));
        }
        let (_, kette) = self.kette.as_mut().expect("eben gebaut");
        {
            let p = kette.parameters();
            for (n, w) in &plan.parameter {
                p.set_parameter_value(n, *w);
            }
        }
        // Weiße Eingangsfläche
        if self.weiss.as_ref().map(|w| w.1) != Some(eingang) {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("roehre: Eichung weiß"),
                size: wgpu::Extent3d { width: eingang.0, height: eingang.1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let weiss = vec![255u8; (eingang.0 * eingang.1 * 4) as usize];
            queue.write_texture(
                t.as_image_copy(),
                &weiss,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * eingang.0), rows_per_image: None },
                t.size(),
            );
            self.weiss = Some((t, eingang));
        }
        if self.ziel.as_ref().map(|z| z.2) != Some(flaeche) {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("roehre: Eichung Ziel"),
                size: wgpu::Extent3d { width: flaeche.0, height: flaeche.1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: crate::ZWISCHENFORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let v = t.create_view(&Default::default());
            self.ziel = Some((t, v, flaeche));
        }
        let (aw, ah) = (AUSSCHNITT.0.min(flaeche.0 / 4 * 4).max(64), AUSSCHNITT.1.min(flaeche.1));
        let zeile = (aw * 4).div_ceil(256) * 256;
        let puffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("roehre: Eichung lesen"),
            size: (zeile * ah) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("roehre: Eichung") });
        let (ziel, ansicht, _) = self.ziel.as_ref().expect("eben angelegt");
        let ausgabe = librashader::runtime::wgpu::WgpuOutputView::new_from_raw(
            ansicht,
            librashader::runtime::Size::new(flaeche.0, flaeche.1),
            crate::ZWISCHENFORMAT,
        );
        let viewport = librashader::runtime::Viewport::new_render_target_sized_origin(ausgabe, None).expect("Größe bekannt");
        // Drei Bilder, damit Nachglühen und Verlauf eingeschwungen sind
        for bild in 0..3 {
            kette.frame(&self.weiss.as_ref().expect("angelegt").0, &viewport, &mut enc, bild, None)?;
        }
        let (x0, y0) = ((flaeche.0 - aw) / 2, (flaeche.1 - ah) / 2);
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: ziel,
                mip_level: 0,
                origin: wgpu::Origin3d { x: x0, y: y0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &puffer,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(zeile), rows_per_image: None },
            },
            wgpu::Extent3d { width: aw, height: ah, depth_or_array_layers: 1 },
        );
        queue.submit([enc.finish()]);
        let fertig = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let f2 = fertig.clone();
        puffer.slice(..).map_async(wgpu::MapMode::Read, move |e| {
            if e.is_ok() {
                f2.store(true, std::sync::atomic::Ordering::Release);
            }
        });
        self.form = (aw, ah, zeile);
        self.puffer = Some(puffer);
        self.laeuft = Some(fertig);
        self.schluessel = Some(schluessel);
        self.gamma = gamma;
        Ok(())
    }

    /// Wartet auf eine laufende Messung (für Werkzeuge und Tests).
    pub fn warte(&mut self, device: &wgpu::Device) {
        if self.laeuft.is_some() {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            self.abholen(device);
        }
    }
}
