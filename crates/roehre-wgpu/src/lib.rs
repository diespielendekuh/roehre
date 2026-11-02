//! # roehre-wgpu
//!
//! Röhrensimulation für Spiele mit wgpu.
//!
//! ```no_run
//! # fn beispiel(device: &wgpu::Device, queue: &wgpu::Queue, spielbild: &wgpu::Texture,
//! #             ziel: &wgpu::TextureView) -> Result<(), roehre_wgpu::Fehler> {
//! use roehre_wgpu::{Roehre, Eingang, Einstellungen};
//! let einstellungen = Einstellungen::voreinstellung("scart-rgb")?;
//! let mut roehre = Roehre::neu(device, queue, "shader", einstellungen, wgpu::TextureFormat::Bgra8UnormSrgb)?;
//! let eingang = Eingang { breite: 256, hoehe: 224, pixel_seitenverhaeltnis: 8.0 / 7.0 };
//! let mut encoder = device.create_command_encoder(&Default::default());
//! roehre.zeichne(&mut encoder, spielbild, &eingang, ziel, (1920, 1080))?;
//! queue.submit([encoder.finish()]);
//! # Ok(()) }
//! ```
//!
//! ## Ablauf je Bild
//!
//! 1. Die Röhrenfläche wird aus Seitenverhältnis, Füllung und ganzzahliger Skalierung
//!    berechnet ([`flaeche::berechne`]); sie liegt immer auf ganzen Pixeln.
//! 2. Die Shaderkette (librashader) zeichnet in eine Zwischentextur genau dieser Größe.
//!    Damit fällt die Maske 1:1 auf Ausgabepixel, es gibt kein Moiré durch Nachskalieren.
//! 3. Ein eigener Pass setzt die Fläche pixelgenau ins Ziel und füllt den Rest mit dem
//!    Hintergrund (später: Gehäuse).
//!
//! ## Umschalten ohne Ruckeln
//!
//! [`Roehre::uebernehme`] wirkt bei Reglern sofort (nur Uniform-Werte). Ist eine andere
//! Shaderkette nötig (z. B. RGB → Composite), wird sie auf einem eigenen Thread gebaut; bis
//! sie fertig ist, läuft die bisherige Kette mit den neuen Werten weiter.
//!
//! ## HDR
//!
//! Mit [`Ausgabe::scrgb`] (Rgba16Float, linear, 1,0 = 80 nit) oder [`Ausgabe::hdr10`]
//! (Rgb10a2Unorm, BT.2020 mit PQ) läuft die Röhre ohne Anheben; eine Helligkeitseichung
//! ([`eichung`]) misst, wie viel Licht Maske und Zeilen übrig lassen, und der
//! Zusammensetz-Pass skaliert so, dass Weiß im Mittel `helligkeit.hdr_weiss` erreicht. Die
//! Leuchtstreifen der Maske werden dabei heller als Weiß, bis `helligkeit.hdr_spitze`.
//!
//! ## Shader-Ordner
//!
//! Die Shader liegen als Dateien im Ordner `shader/` (GPL-Shader werden bewusst nicht ins
//! Programm eingebettet). Der Ordner wird mit dem Spiel ausgeliefert.

// `Fehler` trägt den Fehler von librashader unverpackt (gut 160 Byte). Fehler entstehen nur
// beim Laden und Umschalten, nicht je Bild; die Größe kostet daher nichts Messbares.
#![allow(clippy::result_large_err)]

pub mod bildschirmflaeche;
pub mod gehaeuse;
pub mod eichung;
pub mod flaeche;

use std::path::PathBuf;
use std::sync::mpsc;

use librashader::runtime::wgpu::{FilterChain, FilterChainOptions, FrameOptions, WgpuOutputView};
use librashader::runtime::{FilterChainParameters, Viewport};
pub use roehre_kette::{Kette, MaskenWahl};
pub use roehre_modell::{self as modell, Einstellungen, Wert};

pub use bildschirmflaeche::{FlaechenZiel, MaterialWerte};
pub use flaeche::{Eingang, Rechteck};
pub use gehaeuse::{Anordnung, Form};

/// Fehler der wgpu-Schnittstelle.
#[derive(Debug, thiserror::Error)]
pub enum Fehler {
    #[error(transparent)]
    Einstellungen(#[from] roehre_modell::Fehler),
    #[error(transparent)]
    Kette(#[from] roehre_kette::Fehler),
    #[error("Shaderkette konnte nicht gebaut werden: {0}")]
    Filter(#[from] librashader::runtime::wgpu::error::FilterChainError),
    #[error("Bauthread der Shaderkette ist abgebrochen")]
    Bauthread,
}

/// Farbraum der Ausgabe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ausgaberaum {
    /// SDR: Werte der Röhre wie für einen gewöhnlichen Bildschirm.
    Sdr,
    /// scRGB: linear, BT.709-Grundfarben, 1,0 = 80 nit (Windows-HDR, Rgba16Float).
    ScRgb,
    /// HDR10: BT.2020-Grundfarben mit PQ-Kodierung (Rgb10a2Unorm).
    Hdr10,
}

/// Format und Farbraum der Ziele, in die [`Roehre::zeichne`] schreibt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ausgabe {
    pub format: wgpu::TextureFormat,
    pub raum: Ausgaberaum,
}

impl Ausgabe {
    pub fn sdr(format: wgpu::TextureFormat) -> Self {
        Self { format, raum: Ausgaberaum::Sdr }
    }
    pub fn scrgb() -> Self {
        Self { format: wgpu::TextureFormat::Rgba16Float, raum: Ausgaberaum::ScRgb }
    }
    pub fn hdr10() -> Self {
        Self { format: wgpu::TextureFormat::Rgb10a2Unorm, raum: Ausgaberaum::Hdr10 }
    }
    pub fn ist_hdr(&self) -> bool {
        self.raum != Ausgaberaum::Sdr
    }
}

impl From<wgpu::TextureFormat> for Ausgabe {
    fn from(format: wgpu::TextureFormat) -> Self {
        Self::sdr(format)
    }
}

/// Fähigkeiten, die das Gerät für alle Shaderketten haben sollte. Das Spiel fordert sie beim
/// Anlegen des `wgpu::Device` an, soweit der Adapter sie bietet.
pub fn empfohlene_features() -> wgpu::Features {
    wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER
        | wgpu::Features::FLOAT32_FILTERABLE
        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::PIPELINE_CACHE
}

pub(crate) const ZWISCHENFORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Eine Zwischentextur mit Ansicht und Größe, auf Wunsch mit Mipmaps.
struct Zwischen {
    textur: wgpu::Texture,
    /// Oberste Stufe, roh (Ziel der Kette, Punkt-für-Punkt-Lesen)
    ansicht: wgpu::TextureView,
    groesse: (u32, u32),
    /// Alle Stufen mit sRGB-Dekodierung (gefiltertes Lesen), nur mit Mipmaps
    mip_ansicht: Option<wgpu::TextureView>,
    /// Einzelne Stufen mit sRGB (zum Erzeugen der Mipmaps)
    stufen: Vec<wgpu::TextureView>,
}

impl Zwischen {
    /// Legt die Textur an, wenn sie fehlt oder Größe bzw. Mipmaps nicht passen.
    fn sichern(platz: &mut Option<Zwischen>, device: &wgpu::Device, groesse: (u32, u32), name: &str, mips: bool) {
        if platz.as_ref().map(|z| (z.groesse, z.mip_ansicht.is_some())) == Some((groesse, mips)) {
            return;
        }
        let stufenzahl = if mips { 32 - groesse.0.max(groesse.1).max(1).leading_zeros() } else { 1 };
        let textur = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size: wgpu::Extent3d { width: groesse.0, height: groesse.1, depth_or_array_layers: 1 },
            mip_level_count: stufenzahl,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ZWISCHENFORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: if mips { &[wgpu::TextureFormat::Rgba8UnormSrgb] } else { &[] },
        });
        let ansicht = textur.create_view(&wgpu::TextureViewDescriptor { mip_level_count: Some(1), ..Default::default() });
        let (mip_ansicht, stufen) = if mips {
            let srgb = Some(wgpu::TextureFormat::Rgba8UnormSrgb);
            let alle = textur.create_view(&wgpu::TextureViewDescriptor { format: srgb, ..Default::default() });
            let einzeln = (0..stufenzahl)
                .map(|n| {
                    textur.create_view(&wgpu::TextureViewDescriptor {
                        format: srgb,
                        base_mip_level: n,
                        mip_level_count: Some(1),
                        ..Default::default()
                    })
                })
                .collect();
            (Some(alle), einzeln)
        } else {
            (None, Vec::new())
        };
        *platz = Some(Zwischen { textur, ansicht, groesse, mip_ansicht, stufen });
    }

    fn viewport(&self) -> Viewport<'_, WgpuOutputView<'_>> {
        let ausgabe = WgpuOutputView::new_from_raw(
            &self.ansicht,
            librashader::runtime::Size::new(self.groesse.0, self.groesse.1),
            ZWISCHENFORMAT,
        );
        Viewport::new_render_target_sized_origin(ausgabe, None).expect("Größe ist bekannt")
    }
}

struct Ausstehend {
    kette: Kette,
    empfaenger: mpsc::Receiver<Result<FilterChain, Fehler>>,
}

/// Die Röhre.
pub struct Roehre {
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader_ordner: PathBuf,
    einstellungen: Einstellungen,
    aktiv: (Kette, FilterChain),
    ausstehend: Option<Ausstehend>,
    zwischen: Option<Zwischen>,
    /// Vorstufe „virtuelle Zeilen“ (eigener Shader) und ihr Ziel.
    vorstufe: FilterChain,
    vorstufe_ziel: Option<Zwischen>,
    setzer: Zusammensetzer,
    bild: usize,
    /// Hintergrundfarbe außerhalb der Röhrenfläche (linear).
    pub hintergrund: [f32; 3],
    ausgabe: Ausgabe,
    eichung: eichung::Eichung,
    mip: bildschirmflaeche::MipErzeuger,
    /// Vorheriges Röhrenbild (Strahlsimulation)
    vorher: Option<Zwischen>,
    /// Anordnung des letzten Spielbilds und Zielgröße, für die sie gilt
    letzte_anordnung: Option<(Anordnung, (u32, u32))>,
    auffrischung: u64,
    zyklen: u64,
    letzter_plan: Option<roehre_kette::Plan>,
}

impl Roehre {
    /// Legt die Röhre an und baut die erste Shaderkette (blockierend).
    ///
    /// `ausgabe` nennt Format und Farbraum der Ziele, in die [`Roehre::zeichne`] später
    /// schreibt; ein bloßes `wgpu::TextureFormat` heißt SDR.
    pub fn neu(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_ordner: impl Into<PathBuf>,
        einstellungen: Einstellungen,
        ausgabe: impl Into<Ausgabe>,
    ) -> Result<Self, Fehler> {
        let ausgabe = ausgabe.into();
        let shader_ordner = shader_ordner.into();
        let kette = kette_fuer(&einstellungen);
        let filter = baue(device, queue, &shader_ordner, kette)?;
        let vorstufe = FilterChain::load_from_preset(
            roehre_kette::lade_vorstufe(&shader_ordner)?,
            device,
            queue,
            Some(&FilterChainOptions { force_no_mipmaps: true, enable_cache: mit_cache(device), adapter_info: None }),
        )?;
        Ok(Self {
            vorstufe,
            vorstufe_ziel: None,
            device: device.clone(),
            queue: queue.clone(),
            shader_ordner,
            einstellungen,
            aktiv: (kette, filter),
            ausstehend: None,
            zwischen: None,
            setzer: Zusammensetzer::neu(device, ausgabe.format),
            ausgabe,
            eichung: eichung::Eichung::neu(),
            mip: bildschirmflaeche::MipErzeuger::neu(device),
            vorher: None,
            letzte_anordnung: None,
            auffrischung: 0,
            zyklen: 0,
            bild: 0,
            hintergrund: [0.0; 3],
            letzter_plan: None,
        })
    }

    /// Aktuelle Einstellungen.
    pub fn einstellungen(&self) -> &Einstellungen {
        &self.einstellungen
    }

    /// Übernimmt neue Einstellungen. Regler wirken ab dem nächsten Bild; braucht es eine
    /// andere Shaderkette, wird sie im Hintergrund gebaut.
    pub fn uebernehme(&mut self, einstellungen: Einstellungen) {
        self.einstellungen = einstellungen;
        let gewuenscht = kette_fuer(&self.einstellungen);
        let schon_unterwegs = self.ausstehend.as_ref().map(|a| a.kette);
        if gewuenscht == self.aktiv.0 {
            self.ausstehend = None;
        } else if schon_unterwegs != Some(gewuenscht) {
            let (sender, empfaenger) = mpsc::channel();
            let (device, queue, ordner) = (self.device.clone(), self.queue.clone(), self.shader_ordner.clone());
            std::thread::Builder::new()
                .name("roehre-kettenbau".into())
                .spawn(move || {
                    let _ = sender.send(baue(&device, &queue, &ordner, gewuenscht));
                })
                .expect("Thread für den Kettenbau");
            self.ausstehend = Some(Ausstehend { kette: gewuenscht, empfaenger });
        }
    }

    /// Ist die Kette zu den aktuellen Einstellungen fertig gebaut?
    pub fn ist_bereit(&mut self) -> bool {
        let _ = self.tausche_wenn_fertig(false);
        self.ausstehend.is_none()
    }

    /// Wartet, bis eine im Hintergrund gebaute Kette fertig ist (für Werkzeuge und Tests).
    pub fn warte_auf_kette(&mut self) -> Result<(), Fehler> {
        self.tausche_wenn_fertig(true)
    }

    fn tausche_wenn_fertig(&mut self, warten: bool) -> Result<(), Fehler> {
        let Some(a) = &self.ausstehend else { return Ok(()) };
        let ergebnis = if warten {
            a.empfaenger.recv().map_err(|_| Fehler::Bauthread)?
        } else {
            match a.empfaenger.try_recv() {
                Ok(e) => e,
                Err(mpsc::TryRecvError::Empty) => return Ok(()),
                Err(mpsc::TryRecvError::Disconnected) => Err(Fehler::Bauthread),
            }
        };
        let kette = a.kette;
        self.ausstehend = None;
        self.aktiv = (kette, ergebnis?);
        Ok(())
    }

    /// Röhrenfläche für ein Ziel der angegebenen Größe (berücksichtigt die virtuelle
    /// Zeilenzahl, die bei ganzzahliger Skalierung zählt).
    pub fn flaeche(&self, eingang: &Eingang, ziel_groesse: (u32, u32)) -> Rechteck {
        self.anordnung(eingang, ziel_groesse).inhalt
    }

    /// Anordnung von Bild, Glas und Gehäuse für ein Ziel der angegebenen Größe.
    pub fn anordnung(&self, eingang: &Eingang, ziel_groesse: (u32, u32)) -> Anordnung {
        let seite = eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64;
        let wirksam = match roehre_kette::virtuelle_groesse(&self.einstellungen, (eingang.breite, eingang.hoehe), seite, ziel_groesse.1) {
            None => *eingang,
            Some((vb, vh)) => Eingang { breite: vb, hoehe: vh, pixel_seitenverhaeltnis: seite * vh as f64 / vb as f64 },
        };
        gehaeuse::anordnen(&self.einstellungen, &wirksam, ziel_groesse)
    }

    /// Zuletzt gemessene Transmission einer weißen Fläche (nur bei HDR-Ausgabe gemessen).
    pub fn transmission(&self) -> f64 {
        self.eichung.transmission
    }

    /// Wartet auf eine laufende Helligkeitseichung (für Werkzeuge und Tests).
    pub fn warte_auf_eichung(&mut self) {
        self.eichung.warte(&self.device);
    }

    /// Plan des zuletzt gezeichneten Bildes (Kette, Parameter, gewählte Maske).
    pub fn letzter_plan(&self) -> Option<&roehre_kette::Plan> {
        self.letzter_plan.as_ref()
    }

    /// Vorstufe „virtuelle Zeilen“: zeichnet bei Bedarf das verkleinerte Bild und liefert
    /// den Eingang, den die Röhre sieht (`true`, wenn die Vorstufe gezeichnet hat).
    fn vorstufe(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        eingang_textur: &wgpu::Texture,
        eingang: &Eingang,
        ziel_hoehe: u32,
    ) -> Result<(Eingang, bool), Fehler> {
        let seite = eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64;
        match roehre_kette::virtuelle_groesse(&self.einstellungen, (eingang.breite, eingang.hoehe), seite, ziel_hoehe) {
            None => Ok((*eingang, false)),
            Some((vb, vh)) => {
                Zwischen::sichern(&mut self.vorstufe_ziel, &self.device, (vb, vh), "roehre: virtuelle Zeilen", false);
                let z = self.vorstufe_ziel.as_ref().expect("eben gesichert");
                self.vorstufe.frame(eingang_textur, &z.viewport(), encoder, self.bild, None)?;
                Ok((Eingang { breite: vb, hoehe: vh, pixel_seitenverhaeltnis: seite * vh as f64 / vb as f64 }, true))
            }
        }
    }

    fn setze_parameter(&mut self, kontext: &roehre_kette::Kontext) -> roehre_kette::Plan {
        let plan = roehre_kette::plane(&self.einstellungen, kontext);
        let parameter = self.aktiv.1.parameters();
        for (name, wert) in &plan.parameter {
            parameter.set_parameter_value(name, *wert);
        }
        plan
    }

    /// Flächen-Modus: zeichnet das Röhrenbild **ohne Maske** in `ziel` (mit Mipmaps) und
    /// liefert die Werte für die Materialfunktion `roehre_maske` (siehe
    /// [`bildschirmflaeche`]). Für einen Bildschirm als Objekt in einer 3D-Szene.
    pub fn zeichne_flaeche(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        eingang_textur: &wgpu::Texture,
        eingang: &Eingang,
        ziel: &FlaechenZiel,
    ) -> Result<MaterialWerte, Fehler> {
        self.tausche_wenn_fertig(false)?;
        let seite = eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64;
        let (wirksam, verkleinert) = self.vorstufe(encoder, eingang_textur, eingang, ziel.hoehe)?;
        let kontext = roehre_kette::Kontext {
            eingang: (wirksam.breite, wirksam.hoehe),
            flaeche: (ziel.breite, ziel.hoehe),
            hdr: false,
            flaeche_modus: true,
        };
        let plan = self.setze_parameter(&kontext);
        self.letzter_plan = Some(plan);
        let ausgabe = WgpuOutputView::new_from_raw(
            &ziel.ansicht_roh,
            librashader::runtime::Size::new(ziel.breite, ziel.hoehe),
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let viewport = Viewport::new_render_target_sized_origin(ausgabe, None).expect("Größe bekannt");
        let optionen = FrameOptions { aspect_ratio: seite as f32, ..Default::default() };
        let quelle = if verkleinert { &self.vorstufe_ziel.as_ref().expect("Vorstufe gezeichnet").textur } else { eingang_textur };
        self.aktiv.1.frame(quelle, &viewport, encoder, self.bild, Some(&optionen))?;
        self.bild = self.bild.wrapping_add(1);
        ziel.mipmaps(&self.device, encoder);
        Ok(MaterialWerte::aus(&self.einstellungen, seite))
    }

    /// Zeichnet ein Bild. `eingang_textur` ist das Spielbild (mit `TEXTURE_BINDING`), `ziel`
    /// eine Ansicht der Zieltextur im bei [`Roehre::neu`] genannten Format.
    ///
    /// Für Bildschirme mit mehr Hertz als das Spiel und Bewegungsschärfe siehe
    /// [`Roehre::zeichne_im_takt`].
    pub fn zeichne(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        eingang_textur: &wgpu::Texture,
        eingang: &Eingang,
        ziel: &wgpu::TextureView,
        ziel_groesse: (u32, u32),
    ) -> Result<Rechteck, Fehler> {
        self.zeichne_im_takt(encoder, eingang_textur, eingang, ziel, ziel_groesse, Takt::EIN_BILD)
    }

    /// Zeichnet eine Bildschirmauffrischung. Bei einem Bildschirm mit mehr Hertz als das Spiel
    /// (z. B. 240 Hz bei 60 Bildern je Sekunde) ruft das Spiel diese Funktion **je
    /// Auffrischung** auf; `takt.neues_spielbild` ist nur bei der ersten nach einem neuen
    /// Spielbild wahr. Die Röhrenkette läuft nur dann; dazwischen rechnet allein der
    /// Zusammensetz-Pass die Teilbilder der Strahlsimulation (`bewegung.modus`).
    pub fn zeichne_im_takt(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        eingang_textur: &wgpu::Texture,
        eingang: &Eingang,
        ziel: &wgpu::TextureView,
        ziel_groesse: (u32, u32),
        takt: Takt,
    ) -> Result<Rechteck, Fehler> {
        let modus = self.einstellungen.auswahl("bewegung.modus").to_string();
        let n = takt.teilbilder_je_spielbild.max(1.0);
        let bewegung = modus != "aus" && n >= 1.5;
        let neu_noetig = takt.neues_spielbild
            || self.zwischen.is_none()
            || self.letzte_anordnung.map(|(_, g)| g) != Some(ziel_groesse);
        if neu_noetig {
            self.spielbild(encoder, eingang_textur, eingang, ziel_groesse, bewegung)?;
            self.zyklen = self.zyklen.wrapping_add(1);
            // Ganzzahliges Verhältnis: Der Strahl beginnt mit jedem Spielbild oben, auch wenn
            // das Spiel ein Bild ausgelassen hat. Sonst läuft er wie in der Vorlage durch
            // (Lage = Auffrischung mod n), damit bei 2,4 Teilbildern kurze und lange
            // Spielbilder zusammen wieder das ganze Licht abgeben.
            if n.fract() == 0.0 {
                self.auffrischung = (self.auffrischung as f64 / n as f64).ceil() as u64 * n as u64;
            }
        }
        let strahl_lage = (self.auffrischung as f64 % n as f64) as f32;
        self.auffrischung = self.auffrischung.wrapping_add(1);
        let (anordnung, _) = self.letzte_anordnung.expect("eben gezeichnet");
        let zwischen = self.zwischen.as_ref().expect("eben gezeichnet");
        let vorher = self.vorher.as_ref().filter(|v| v.groesse == zwischen.groesse);

        let e = &self.einstellungen;
        let roehre_an = e.schalter("allgemein.roehre");
        let weiss = e.zahl("helligkeit.hdr_weiss");
        let hdr_daten = if roehre_an {
            // Röhrenwert 1,0 so hell, dass eine weiße Fläche im Mittel `weiss` erreicht
            [e.zahl("helligkeit.gamma_ausgang") as f32, (weiss / self.eichung.transmission) as f32]
        } else {
            // Ohne Röhre: sRGB-ähnlich mit Gamma 2,2, Weiß = `weiss`
            [2.2, weiss as f32]
        };
        let hdr_daten = [hdr_daten[0], hdr_daten[1], e.zahl("helligkeit.hdr_spitze") as f32, weiss as f32];
        let licht = [
            e.zahl("gehaeuse.spiegelung") as f32,
            e.zahl("gehaeuse.umgebungslicht") as f32,
            e.zahl("gehaeuse.raumlicht") as f32,
            e.zahl("gehaeuse.glanz") as f32,
        ];
        // LCD-Schutz nach der Vorlage: bei geraden ganzzahligen Verhältnissen wandert die
        // Strahlphase um 0,001 Teilbilder je Spielbild (nach 1000 Bildern ein Teilbild weiter).
        let gerade = n.fract() == 0.0 && (n as u32).is_multiple_of(2);
        let phase = if bewegung && gerade && e.schalter("bewegung.lcd_schutz") {
            ((self.zyklen as f64 * 0.001) % 1.0) as f32
        } else {
            0.0
        };
        let strahl = [
            if !bewegung { 0.0 } else if modus == "schwarzbilder" { 2.0 } else { 1.0 },
            n,
            strahl_lage,
            e.zahl("bewegung.helligkeit") as f32,
            e.zahl("helligkeit.gamma_ausgang") as f32,
            phase,
            0.0,
            0.0,
        ];
        self.setzer.setze(
            &self.device,
            &self.queue,
            encoder,
            zwischen,
            vorher.unwrap_or(zwischen),
            ziel,
            &anordnung,
            self.hintergrund,
            self.ausgabe.raum,
            hdr_daten,
            licht,
            strahl,
        );
        Ok(anordnung.inhalt)
    }

    /// Ein neues Spielbild durch Vorstufe und Röhrenkette in die Röhrentextur zeichnen.
    fn spielbild(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        eingang_textur: &wgpu::Texture,
        eingang: &Eingang,
        ziel_groesse: (u32, u32),
        bewegung: bool,
    ) -> Result<(), Fehler> {
        self.tausche_wenn_fertig(false)?;

        // Vorstufe: hochaufgelöste Bilder auf die Zeilenzahl der Röhre bringen
        let seite = eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64;
        let virtuell = roehre_kette::virtuelle_groesse(&self.einstellungen, (eingang.breite, eingang.hoehe), seite, ziel_groesse.1);
        let (roehren_eingang, wirksam) = match virtuell {
            None => (eingang_textur, *eingang),
            Some((vb, vh)) => {
                Zwischen::sichern(&mut self.vorstufe_ziel, &self.device, (vb, vh), "roehre: virtuelle Zeilen", false);
                let z = self.vorstufe_ziel.as_ref().expect("eben gesichert");
                self.vorstufe.frame(eingang_textur, &z.viewport(), encoder, self.bild, None)?;
                let par = seite * vh as f64 / vb as f64;
                (&z.textur, Eingang { breite: vb, hoehe: vh, pixel_seitenverhaeltnis: par })
            }
        };

        let anordnung = gehaeuse::anordnen(&self.einstellungen, &wirksam, ziel_groesse);
        let r = anordnung.inhalt;
        // Mipmaps braucht nur ein Gehäuse mit Rahmen (Spiegelung, Licht des Bildes)
        let mips = anordnung.form.is_some_and(|f| f.art >= 2);
        // Strahlsimulation: das bisherige Bild wird zum vorherigen
        if bewegung {
            std::mem::swap(&mut self.zwischen, &mut self.vorher);
        } else {
            self.vorher = None;
        }
        Zwischen::sichern(&mut self.zwischen, &self.device, (r.breite, r.hoehe), "roehre: Röhrenfläche", mips);
        let zwischen = self.zwischen.as_ref().expect("eben gesichert");

        // Parameter setzen (Regler wirken sofort, auch während eine neue Kette gebaut wird)
        let hdr = self.ausgabe.ist_hdr();
        let kontext = roehre_kette::Kontext { eingang: (wirksam.breite, wirksam.hoehe), flaeche: (r.breite, r.hoehe), hdr, flaeche_modus: false };
        let plan = roehre_kette::plane(&self.einstellungen, &kontext);
        if hdr && self.einstellungen.schalter("allgemein.roehre") {
            self.eichung.abholen(&self.device);
            self.eichung.bei_bedarf_messen(
                &self.device,
                &self.queue,
                &self.shader_ordner,
                &plan,
                (wirksam.breite, wirksam.hoehe),
                (r.breite, r.hoehe),
                self.einstellungen.zahl("helligkeit.gamma_ausgang"),
            )?;
        }
        {
            let parameter = self.aktiv.1.parameters();
            for (name, wert) in &plan.parameter {
                parameter.set_parameter_value(name, *wert);
            }
        }
        self.letzter_plan = Some(plan);

        let optionen = FrameOptions { aspect_ratio: seite as f32, ..Default::default() };
        self.aktiv.1.frame(roehren_eingang, &zwischen.viewport(), encoder, self.bild, Some(&optionen))?;
        self.bild = self.bild.wrapping_add(1);
        if mips {
            for n in 1..zwischen.stufen.len() {
                self.mip.stufe(&self.device, encoder, &zwischen.stufen[n - 1], &zwischen.stufen[n]);
            }
        }
        self.letzte_anordnung = Some((anordnung, ziel_groesse));
        Ok(())
    }
}

/// Takt einer Bildschirmauffrischung (siehe [`Roehre::zeichne_im_takt`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Takt {
    /// Seit der letzten Auffrischung ist ein neues Spielbild da.
    pub neues_spielbild: bool,
    /// Bildschirmauffrischungen je Spielbild: Hertz des Bildschirms geteilt durch die
    /// Bildrate des Spiels (240 / 60 = 4; 144 / 60 = 2,4 geht auch, ganzzahlig ist besser).
    pub teilbilder_je_spielbild: f32,
}

impl Takt {
    /// Ein Spielbild je Auffrischung (keine Strahlsimulation).
    pub const EIN_BILD: Takt = Takt { neues_spielbild: true, teilbilder_je_spielbild: 1.0 };
}

fn kette_fuer(e: &Einstellungen) -> Kette {
    // Die Kettenwahl hängt nicht von den Größen ab.
    let k = roehre_kette::Kontext { eingang: (1, 1), flaeche: (1, 1), hdr: false, flaeche_modus: false };
    roehre_kette::plane(e, &k).kette
}

/// librashader legt einen Pipeline-Cache nur an, wenn das Gerät ihn kann. Fehlt das Feature
/// (wgpu unter D3D12 kennt es nicht), bricht wgpu sonst mit einem Prüffehler ab – genau so am
/// 2026-11-02 im ersten D3D12-Lauf auf der Grafikkarte des Anwenders.
fn mit_cache(device: &wgpu::Device) -> bool {
    device.features().contains(wgpu::Features::PIPELINE_CACHE)
}

pub(crate) fn baue(device: &wgpu::Device, queue: &wgpu::Queue, ordner: &std::path::Path, kette: Kette) -> Result<FilterChain, Fehler> {
    let preset = kette.lade(ordner)?;
    let optionen = FilterChainOptions { force_no_mipmaps: false, enable_cache: mit_cache(device), adapter_info: None };
    Ok(FilterChain::load_from_preset(preset, device, queue, Some(&optionen))?)
}

/// Eigener Pass: Röhrenfläche ins Ziel setzen, Glas und Gehäuse zeichnen, Rest füllen.
struct Zusammensetzer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    puffer: Option<wgpu::Buffer>,
    abtaster: wgpu::Sampler,
    /// Ersatz für die Mip-Ansicht, wenn die Röhrentextur keine Mipmaps hat
    leer: wgpu::TextureView,
    srgb: bool,
}

impl Zusammensetzer {
    fn neu(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let modul = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("roehre: zusammensetzen"),
            source: wgpu::ShaderSource::Wgsl(include_str!("zusammensetzen.wgsl").into()),
        });
        let textur = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("roehre: zusammensetzen"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                textur(1, false),
                textur(2, true),
                textur(4, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("roehre: zusammensetzen"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre: zusammensetzen"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &modul,
                entry_point: Some("ecke"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &modul,
                entry_point: Some("farbe"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let abtaster = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("roehre: zusammensetzen"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let leer = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("roehre: leer"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        Self { pipeline, layout, puffer: None, abtaster, leer, srgb: format.is_srgb() }
    }

    #[allow(clippy::too_many_arguments)]
    fn setze(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        quelle: &Zwischen,
        vorher: &Zwischen,
        ziel: &wgpu::TextureView,
        an: &Anordnung,
        hintergrund: [f32; 3],
        raum: Ausgaberaum,
        hdr: [f32; 4],
        licht: [f32; 4],
        strahl: [f32; 8],
    ) {
        let art = match raum {
            Ausgaberaum::Sdr if self.srgb => 1.0,
            Ausgaberaum::Sdr => 0.0,
            Ausgaberaum::ScRgb => 2.0,
            Ausgaberaum::Hdr10 => 3.0,
        };
        let r = an.inhalt;
        let f = an.form.unwrap_or(gehaeuse::Form {
            art: 0,
            radius_glas: 0.0,
            fase: 0.0,
            seite: 0.0,
            oben: 0.0,
            unten: 0.0,
            radius_aussen: 0.0,
            farbe: [0.0; 3],
            rauheit: 1.0,
        });
        let mip_max = quelle.stufen.len().saturating_sub(1) as f32;
        let daten: [f32; 40] = [
            r.x as f32, r.y as f32, r.breite as f32, r.hoehe as f32,
            an.glas[0], an.glas[1], an.glas[2], an.glas[3],
            hintergrund[0], hintergrund[1], hintergrund[2], art,
            hdr[0], hdr[1], hdr[2], hdr[3],
            f.art as f32, f.radius_glas, f.fase, f.radius_aussen,
            f.seite, f.oben, f.unten, f.rauheit,
            f.farbe[0], f.farbe[1], f.farbe[2], mip_max,
            licht[0], licht[1], licht[2], licht[3],
            strahl[0], strahl[1], strahl[2], strahl[3],
            strahl[4], strahl[5], strahl[6], strahl[7],
        ];
        let bytes: Vec<u8> = daten.iter().flat_map(|f| f.to_le_bytes()).collect();
        let puffer = self.puffer.get_or_insert_with(|| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("roehre: zusammensetzen"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        queue.write_buffer(puffer, 0, &bytes);
        let gruppe = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre: zusammensetzen"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: puffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&quelle.ansicht) },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(quelle.mip_ansicht.as_ref().unwrap_or(&self.leer)),
                },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.abtaster) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&vorher.ansicht) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("roehre: zusammensetzen"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ziel,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &gruppe, &[]);
        pass.draw(0..3, 0..1);
    }
}
