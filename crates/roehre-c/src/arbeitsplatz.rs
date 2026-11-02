//! Der gemeinsame Kern aller C-Wege: ein wgpu-Gerät mit Röhre und (bei Bedarf) 3D-Raum.

use std::path::PathBuf;

use roehre_raum::{Ansicht, Raum};
use roehre_wgpu::{Ausgabe, Eingang, Einstellungen, FlaechenZiel, Roehre};

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

/// Größe der Bildschirmfläche im 3D-Raum (das Röhrenbild als Material).
const FLAECHE: (u32, u32) = (1440, 1080);

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
    ) -> Result<(), String> {
        match modus {
            Modus::Flach => self.roehre.zeichne(enc, eingang_textur, eingang, ziel, groesse).map(|_| ()).map_err(|f| f.to_string()),
            Modus::Raum => {
                if self.raum.is_none() {
                    let raum = Raum::neu(&self.device, &self.queue, &self.shader, self.format).map_err(|f| f.to_string())?;
                    let flaeche = FlaechenZiel::neu(&self.device, FLAECHE.0, FLAECHE.1);
                    self.raum = Some((raum, flaeche));
                }
                let (raum, flaeche) = self.raum.as_mut().expect("angelegt");
                let werte = self.roehre.zeichne_flaeche(enc, eingang_textur, eingang, flaeche).map_err(|f| f.to_string())?;
                let a = Ansicht::aus_einstellungen(&self.einstellungen, zeit, groesse.0 as f32 / groesse.1.max(1) as f32);
                raum.zeichne(&self.device, &self.queue, enc, ziel, groesse, flaeche, &werte, &a);
                Ok(())
            }
        }
    }
}
