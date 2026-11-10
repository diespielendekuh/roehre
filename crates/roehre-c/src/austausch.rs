//! Austausch mit einem Spiel, das selbst Vulkan oder D3D12 benutzt.
//!
//! roehre rechnet auf einem **eigenen** Gerät derselben Grafikkarte. Eingangs- und Zielbild
//! legt roehre in geteiltem Grafikspeicher an; das Spiel öffnet sie über Handles, kopiert sein
//! Bild hinein und das Ergebnis heraus. Zwei geteilte Semaphoren bzw. ein geteilter Zaun ordnen
//! die Arbeit beider Geräte.
//!
//! Warum nicht direkt auf dem Gerät des Spiels: wgpu kann ein fremdes D3D12-Gerät gar nicht
//! übernehmen, ein fremdes Vulkan-Gerät nur, wenn es mit genau den Funktionen angelegt wurde,
//! die wgpu erwartet (z. B. Timeline-Semaphoren) – das hieße, am Gerät des Spiels (bei
//! Mission: Impossible: RT64/plume) etwas zu ändern. Der geteilte Speicher kostet je Bild zwei
//! Kopien auf der Grafikkarte (bei 4K etwa 0,1 ms), braucht unter D3D12 aber keinerlei Eingriff
//! und unter Vulkan nur zwei Geräteerweiterungen.

use std::ffi::c_char;
use std::path::PathBuf;
use std::time::Instant;

use roehre_wgpu::{Ausgabe, Eingang, Einstellungen};

use crate::arbeitsplatz::{Arbeitsplatz, Modus};
use crate::einstellungen::{innen, RoehreEinstellungen};
use crate::fehler::{geschuetzt, text};

/// Parameter zum Anlegen (siehe roehre.h).
#[repr(C)]
pub struct RoehreAustauschParameter {
    pub struktur_groesse: u32,
    pub shader_ordner: *const c_char,
    pub geraet_kennung: [u8; 16],
    pub eingang_breite: u32,
    pub eingang_hoehe: u32,
    pub pixel_seitenverhaeltnis: f32,
    pub ziel_breite: u32,
    pub ziel_hoehe: u32,
    pub ausgabe: i32,
    pub einstellungen: *const RoehreEinstellungen,
    /// Kanalfolge der geteilten Bilder (seit 0.1.1, am Ende ergänzt): 0 = RGBA, 1 = BGRA.
    /// Wird nur gelesen, wenn `struktur_groesse` das Feld einschließt; sonst gilt RGBA.
    pub kanalfolge: u32,
}

/// Größe der Struktur vor der Ergänzung um `kanalfolge` (ältere Aufrufer).
const GROESSE_OHNE_KANALFOLGE: usize = std::mem::offset_of!(RoehreAustauschParameter, kanalfolge);

pub(crate) struct Gelesen {
    pub shader: PathBuf,
    pub kennung: [u8; 16],
    pub eingang: Eingang,
    pub ziel: (u32, u32),
    pub ausgabe: Ausgabe,
    pub einstellungen: Einstellungen,
    /// Format von EINGANG (RGBA8 oder BGRA8, je nach `kanalfolge`)
    pub eingang_format: wgpu::TextureFormat,
}

pub(crate) unsafe fn lies_parameter(p: *const RoehreAustauschParameter) -> Result<Gelesen, String> {
    let p = unsafe { p.as_ref() }.ok_or("Parameter fehlen (NULL)")?;
    if (p.struktur_groesse as usize) < GROESSE_OHNE_KANALFOLGE {
        return Err("struktur_groesse zu klein (sizeof(RoehreAustauschParameter) eintragen)".into());
    }
    // Das Feld liegt nur im Speicher des Aufrufers, wenn seine Struktur es schon hat.
    let bgra = if (p.struktur_groesse as usize) >= std::mem::size_of::<RoehreAustauschParameter>() {
        match p.kanalfolge {
            0 => false,
            1 => true,
            k => return Err(format!("unbekannte Kanalfolge {k} (0 RGBA, 1 BGRA)")),
        }
    } else {
        false
    };
    let ok = |b: u32, h: u32| (1..=16384).contains(&b) && (1..=16384).contains(&h);
    if !ok(p.eingang_breite, p.eingang_hoehe) || !ok(p.ziel_breite, p.ziel_hoehe) {
        return Err("Bildgrößen ungültig (1 … 16384)".into());
    }
    let ausgabe = match p.ausgabe {
        0 => Ausgabe::sdr(if bgra { wgpu::TextureFormat::Bgra8UnormSrgb } else { wgpu::TextureFormat::Rgba8UnormSrgb }),
        1 => Ausgabe::scrgb(),
        2 => Ausgabe::hdr10(),
        a => return Err(format!("unbekannte Ausgabe {a} (0 SDR, 1 scRGB, 2 HDR10)")),
    };
    if bgra && p.ausgabe != 0 {
        // RGBA16F und RGB10A2 haben keine BGRA-Fassung; EINGANG allein in BGRA wäre möglich,
        // aber bisher braucht es niemand (Swapchains in HDR sind nicht BGRA8).
        return Err("Kanalfolge BGRA gibt es nur mit der Ausgabe SDR".into());
    }
    let einstellungen = if p.einstellungen.is_null() { Einstellungen::standard() } else { unsafe { innen(p.einstellungen) }?.clone() };
    Ok(Gelesen {
        shader: PathBuf::from(unsafe { text(p.shader_ordner, "Shader-Ordner") }?),
        kennung: p.geraet_kennung,
        eingang: Eingang {
            breite: p.eingang_breite,
            hoehe: p.eingang_hoehe,
            pixel_seitenverhaeltnis: if p.pixel_seitenverhaeltnis > 0.0 { p.pixel_seitenverhaeltnis as f64 } else { 1.0 },
        },
        ziel: (p.ziel_breite, p.ziel_hoehe),
        ausgabe,
        einstellungen,
        eingang_format: if bgra { wgpu::TextureFormat::Bgra8Unorm } else { EINGANG_FORMAT },
    })
}

/// Der Teil, der von der Grafik-API abhängt.
pub(crate) trait Weg: Send {
    /// Vor dem Übergeben: auf das Spiel warten und danach melden lassen.
    fn vor_submit(&mut self, queue: &wgpu::Queue) -> Result<(), String>;
    /// Nach dem Übergeben (z. B. Zaun setzen).
    fn nach_submit(&mut self, queue: &wgpu::Queue) -> Result<(), String>;
}

/// Undurchsichtiges Austausch-Objekt.
pub struct RoehreAustausch {
    pub(crate) platz: Arbeitsplatz,
    pub(crate) eingang: wgpu::Texture,
    pub(crate) ziel: wgpu::Texture,
    pub(crate) eingang_info: Eingang,
    pub(crate) weg: Box<dyn Weg>,
    pub(crate) start: Instant,
    /// Plattformteil (Handles der Bilder und Semaphoren)
    pub(crate) teile: Box<dyn std::any::Any + Send>,
}

/// Format des Eingangsbilds: 8 Bit je Kanal, Werte wie im Spiel (sRGB-kodiert). Mit
/// `kanalfolge` = 1 stattdessen BGRA8 (`Gelesen::eingang_format`), damit ein Spiel mit
/// BGRA-Swapchain (RT64/plume unter D3D12) ohne eigenen Zeichen-Pass kopieren kann.
pub(crate) const EINGANG_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) const BILD_NUTZUNG: wgpu::TextureUsages = wgpu::TextureUsages::TEXTURE_BINDING
    .union(wgpu::TextureUsages::RENDER_ATTACHMENT)
    .union(wgpu::TextureUsages::COPY_SRC)
    .union(wgpu::TextureUsages::COPY_DST);

/// Setzt neue Einstellungen. Blockiert, bis eine neu nötige Shaderkette gebaut ist (beim
/// Wechsel der Kette bis etwa eine Sekunde, sonst sofort), damit keine Arbeit im Hintergrund
/// zwischen die Übergaben an die Grafikkarte gerät.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_austausch_einstellungen(a: *mut RoehreAustausch, e: *const RoehreEinstellungen) -> i32 {
    geschuetzt(-1, || {
        let a = unsafe { a.as_mut() }.ok_or("Austausch fehlt (NULL)")?;
        let e = unsafe { innen(e) }?;
        a.platz.einstellungen(e, true)?;
        Ok(0)
    })
}

/// Zeichnet ein Bild: wartet (auf der Grafikkarte) auf „bereit“, rechnet das Eingangsbild ins
/// Zielbild, meldet „fertig“. Kehrt sofort zurück, ohne auf die Grafikkarte zu warten.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_austausch_zeichnen(a: *mut RoehreAustausch, modus: i32, zeit: f32) -> i32 {
    unsafe { roehre_austausch_zeichnen_im_takt(a, modus, zeit, 1, 1.0) }
}

/// Wie `roehre_austausch_zeichnen`, aber je Bildschirmauffrischung bei Bildschirmen mit mehr
/// Hertz als das Spiel (Strahlsimulation und Schwarzbilder, Optionen `bewegung.*`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_austausch_zeichnen_im_takt(a: *mut RoehreAustausch, modus: i32, zeit: f32, neues_spielbild: i32, teilbilder_je_spielbild: f32) -> i32 {
    geschuetzt(-1, || {
        let takt = roehre_wgpu::Takt { neues_spielbild: neues_spielbild != 0, teilbilder_je_spielbild: teilbilder_je_spielbild.max(1.0) };
        let a = unsafe { a.as_mut() }.ok_or("Austausch fehlt (NULL)")?;
        let modus = Modus::aus(modus)?;
        let zeit = if zeit >= 0.0 { zeit } else { a.start.elapsed().as_secs_f32() };
        let device = a.platz.device.clone();
        let queue = a.platz.queue.clone();
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("roehre-austausch") });
        let ansicht = a.ziel.create_view(&Default::default());
        let groesse = (a.ziel.width(), a.ziel.height());
        a.platz.zeichne(&mut enc, &a.eingang, &a.eingang_info, &ansicht, groesse, modus, zeit, takt)?;
        // Beide Bilder verlassen roehre im Zustand „zum Lesen im Shader“ (Vulkan:
        // SHADER_READ_ONLY_OPTIMAL, D3D12: PIXEL_SHADER_RESOURCE | NON_PIXEL_SHADER_RESOURCE).
        enc.transition_resources(
            std::iter::empty(),
            [&a.eingang, &a.ziel].into_iter().map(|t| wgpu::TextureTransition { texture: t, selector: None, state: wgpu::TextureUses::RESOURCE }),
        );
        a.weg.vor_submit(&queue)?;
        queue.submit([enc.finish()]);
        a.weg.nach_submit(&queue)?;
        let _ = device.poll(wgpu::PollType::Poll);
        Ok(0)
    })
}

/// Wartet, bis alle Arbeit von roehre auf der Grafikkarte erledigt ist (zum Aufräumen, für
/// Tests; im Spiel nicht nötig).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_austausch_warten(a: *mut RoehreAustausch) -> i32 {
    geschuetzt(-1, || {
        let a = unsafe { a.as_mut() }.ok_or("Austausch fehlt (NULL)")?;
        a.platz.device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| e.to_string())?;
        Ok(0)
    })
}

/// Gibt den Austausch frei (wartet vorher auf die Grafikkarte). Handles, die das Spiel bereits
/// geöffnet hat, bleiben dort gültig, bis es sie selbst schließt.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_austausch_freigeben(a: *mut RoehreAustausch) {
    if a.is_null() {
        return;
    }
    let a = unsafe { Box::from_raw(a) };
    let _ = a.platz.device.poll(wgpu::PollType::wait_indefinitely());
    drop(a);
}
