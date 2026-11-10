//! Der gemeinsame Kern aller C-Wege: ein wgpu-Gerät mit Röhre und (bei Bedarf) 3D-Raum.

use std::path::PathBuf;

use roehre_raum::{Ansicht, Raum};
use roehre_wgpu::{Ausgabe, Eingang, Einstellungen, FlaechenZiel, Roehre, Takt};

/// Was gezeichnet wird.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modus {
    /// Das Bild der Röhre bildschirmfüllend (mit Gehäuse, falls eingestellt).
    Flach = 0,
    /// Der 3D-Vorführraum mit dem Fernseher (Optionen der Gruppe `raum`).
    Raum = 1,
}

impl Modus {
    pub(crate) fn aus(wert: i32) -> Result<Self, String> {
        match wert {
            0 => Ok(Modus::Flach),
            1 => Ok(Modus::Raum),
            _ => Err(format!("unbekannter Modus {wert} (0 = flach, 1 = Raum)")),
        }
    }
}

/// Höhe der Bildschirmfläche im 3D-Raum (das Röhrenbild als Material); die Breite folgt dem
/// Seitenverhältnis des Bildes (4:3: 1440).
const FLAECHE_HOEHE: u32 = 1080;

/// Seitenverhältnis des Bildes auf der Röhre (Breite/Höhe), wie es `bildformat.seitenverhaeltnis`
/// festlegt: danach wird im Raum der Fernseher gebaut (8:7, 4:3, 16:9 oder dazwischen).
fn bildformat(e: &Einstellungen, eingang: &Eingang, ziel: (u32, u32)) -> f32 {
    let f = match e.auswahl("bildformat.seitenverhaeltnis") {
        "pixelgenau" => eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64,
        "16:9" => 16.0 / 9.0,
        "fenster" => ziel.0 as f64 / ziel.1.max(1) as f64,
        _ => 4.0 / 3.0,
    };
    (f as f32).clamp(1.0, 2.4)
}

pub(crate) struct Arbeitsplatz {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    roehre: Roehre,
    raum: Option<(Raum, FlaechenZiel)>,
    format: wgpu::TextureFormat,
    shader: PathBuf,
    einstellungen: Einstellungen,
}

impl Arbeitsplatz {
    pub fn neu(device: wgpu::Device, queue: wgpu::Queue, shader: PathBuf, ausgabe: Ausgabe, e: Einstellungen) -> Result<Self, String> {
        let format = ausgabe.format;
        let roehre = Roehre::neu(&device, &queue, shader.clone(), e.clone(), ausgabe).map_err(|f| f.to_string())?;
        Ok(Self { device, queue, roehre, raum: None, format, shader, einstellungen: e })
    }

    /// Neue Einstellungen. Muss die Shaderkette gewechselt werden, geschieht das im
    /// Hintergrund; bis dahin zeichnet die alte weiter (mit `warten` wird gewartet).
    pub fn einstellungen(&mut self, e: &Einstellungen, warten: bool) -> Result<(), String> {
        if *e != self.einstellungen {
            self.einstellungen = e.clone();
            self.roehre.uebernehme(e.clone());
        }
        if warten {
            self.roehre.warte_auf_kette().map_err(|f| f.to_string())?;
        }
        Ok(())
    }

    /// Zeichnet ein Bild in `ziel`.
    #[allow(clippy::too_many_arguments)]
    pub fn zeichne(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        eingang_textur: &wgpu::Texture,
        eingang: &Eingang,
        ziel: &wgpu::TextureView,
        groesse: (u32, u32),
        modus: Modus,
        zeit: f32,
        takt: Takt,
    ) -> Result<(), String> {
        match modus {
            Modus::Flach => self.roehre.zeichne_im_takt(enc, eingang_textur, eingang, ziel, groesse, takt).map(|_| ()).map_err(|f| f.to_string()),
            Modus::Raum => {
                // Der Fernseher hat das Format des Bildes. Ändert es sich (anderes Seitenverhältnis
                // im Spiel, andere Einstellung), wird der Raum neu gebaut (einige Zehntelsekunden).
                let format = bildformat(&self.einstellungen, eingang, groesse);
                let konsole = roehre_raum::Konsole::aus(self.einstellungen.auswahl("raum.konsole"));
                if self.raum.as_ref().is_none_or(|(r, _)| (r.bildformat() - format).abs() > 0.004 || r.konsole() != konsole) {
                    let raum = Raum::neu_mit(&self.device, &self.queue, &self.shader, self.format, format, konsole).map_err(|f| f.to_string())?;
                    let breite = ((FLAECHE_HOEHE as f32 * format / 2.0).round() as u32 * 2).max(2);
                    let flaeche = FlaechenZiel::neu(&self.device, breite, FLAECHE_HOEHE);
                    self.raum = Some((raum, flaeche));
                }
                let (raum, flaeche) = self.raum.as_mut().expect("angelegt");
                let werte = self.roehre.zeichne_flaeche(enc, eingang_textur, eingang, flaeche).map_err(|f| f.to_string())?;
                let a = Ansicht::aus_einstellungen_fuer(&self.einstellungen, zeit, groesse.0 as f32 / groesse.1.max(1) as f32, raum.bildformat());
                raum.zeichne(&self.device, &self.queue, enc, ziel, groesse, flaeche, &werte, &a);
                Ok(())
            }
        }
    }
}
