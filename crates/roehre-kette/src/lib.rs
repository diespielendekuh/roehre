//! # roehre-kette
//!
//! Übersetzt [`Einstellungen`] in eine librashader-Shaderkette und deren Parameterwerte.
//! Das ist der Teil von `roehre`, der unabhängig vom Grafik-Unterbau ist: Die wgpu-, Vulkan-
//! und D3D12-Wege benutzen dieselbe Übersetzung.
//!
//! Zwei Dinge werden unterschieden:
//!
//! * **Kettenwahl** ([`Kette`]): welche Shaderkette läuft. Ein Wechsel erfordert einen
//!   Neubau der Kette (im Hintergrund, siehe roehre-wgpu).
//! * **Parameter** ([`Plan::parameter`]): Uniform-Werte innerhalb der Kette. Sie lassen sich
//!   in jedem Bild ändern, ohne Neubau.
//!
//! Die Zuordnung zu den Parametern von crt-guest-advanced (guest(r), GPL-2.0-or-later) steht
//! in [`plane`]; die Namen der Parameter sind die aus den Shadern.

use std::path::{Path, PathBuf};

use librashader::presets::{ShaderFeatures, ShaderPreset};
use roehre_modell::Einstellungen;

/// Fehler beim Laden einer Kette.
#[derive(Debug, thiserror::Error)]
pub enum Fehler {
    #[error("Shader-Ordner „{0}“ fehlt oder ist unvollständig (erwartet: {1})")]
    ShaderOrdner(PathBuf, PathBuf),
    #[error("Preset nicht lesbar: {0}")]
    Preset(#[from] librashader::presets::ParsePresetError),
}

/// Die verfügbaren Shaderketten.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kette {
    /// Keine Röhre, nur scharf-bilineares Einpassen (eigener Shader).
    Sauber,
    /// crt-guest-advanced mit RGB-Eingang.
    GuestRgb,
    /// crt-guest-advanced, schnelle Fassung (Leistungsstufe „leicht“).
    GuestSchnell,
    /// crt-guest-advanced mit NTSC-Signalstufe (S-Video, Composite, Antenne).
    GuestNtsc,
    /// crt-guest-advanced mit PAL-Signalstufe.
    GuestPal,
}

impl Kette {
    /// Pfad des Presets relativ zum Shader-Ordner.
    pub fn preset_pfad(self) -> &'static str {
        match self {
            Kette::Sauber => "eigen/sauber.slangp",
            Kette::GuestRgb => "fremd/gpl/guest-advanced/crt-guest-advanced.slangp",
            Kette::GuestSchnell => "fremd/gpl/guest-advanced/crt-guest-advanced-fast.slangp",
            Kette::GuestNtsc => "fremd/gpl/guest-advanced/crt-guest-advanced-ntsc.slangp",
            Kette::GuestPal => "fremd/gpl/guest-advanced/crt-guest-advanced-pal.slangp",
        }
    }

    /// Braucht die Kette GPL-Shader?
    pub fn ist_gpl(self) -> bool {
        !matches!(self, Kette::Sauber)
    }

    /// Lädt das Preset aus dem Shader-Ordner.
    pub fn lade(self, shader_ordner: &Path) -> Result<ShaderPreset, Fehler> {
        let pfad = shader_ordner.join(self.preset_pfad());
        if !pfad.is_file() {
            return Err(Fehler::ShaderOrdner(shader_ordner.into(), pfad));
        }
        Ok(ShaderPreset::try_parse(&pfad, ShaderFeatures::ORIGINAL_ASPECT_UNIFORMS | ShaderFeatures::FRAMETIME_UNIFORMS)?)
    }
}

/// Was die Übersetzung über Eingang und Ausgabe wissen muss.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kontext {
    /// Auflösung des Spielbilds in Pixeln.
    pub eingang: (u32, u32),
    /// Größe der Röhrenfläche auf dem Bildschirm in Ausgabepixeln.
    pub flaeche: (u32, u32),
}

/// Ergebnis der Übersetzung.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub kette: Kette,
    /// Parameterwerte (Name im Shader, Wert). Nicht vorhandene Namen ignoriert die Kette.
    pub parameter: Vec<(&'static str, f32)>,
    /// Gewählte Maske: (guest-Maskennummer, Maskengröße, tatsächliche Triaden über die Breite).
    pub maske: Option<MaskenWahl>,
}

/// Wie die Maske auf die Ausgabepixel gelegt wurde.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaskenWahl {
    /// Nummer der Maske in crt-guest-advanced (`shadowMask`).
    pub muster: u8,
    /// Ganzzahliger Vergrößerungsfaktor (`masksize`).
    pub groesse: u8,
    /// Breite eines Farbtripels in Ausgabepixeln.
    pub pixel_je_triade: f64,
    /// Sich ergebende Zahl der Tripel über die Bildbreite.
    pub triaden: f64,
}

/// Streifenmuster aus crt-guest-advanced, die sich für eine saubere Abbildung auf
/// Ausgabepixel eignen: (Nummer, Breite eines Tripels in Pixeln bei `masksize` 1).
/// Nr. 6 = R|G|B, Nr. 10 = R|G|B|schwarz, Nr. 13 = weiche 6-Pixel-Maske für 4K,
/// Nr. 12 = RR|GG|BB|schwarz.
const STREIFENMUSTER: [(u8, f64); 4] = [(6, 3.0), (10, 4.0), (13, 6.0), (12, 7.0)];

/// Wählt Muster und Größe so, dass die Tripelbreite dem Wunsch am nächsten kommt.
/// Die Maske liegt dadurch immer auf ganzen Ausgabepixeln (kein Moiré durch Nachskalierung).
pub fn waehle_maske(flaechen_breite: u32, wunsch_triaden: f64) -> MaskenWahl {
    let wunsch_breite = flaechen_breite as f64 / wunsch_triaden.max(1.0);
    let mut beste: Option<(f64, MaskenWahl)> = None;
    for (muster, breite) in STREIFENMUSTER {
        for groesse in 1..=4u8 {
            let px = breite * groesse as f64;
            // Abstand auf logarithmischer Skala: 3 statt 4 ist so schlimm wie 6 statt 8.
            let abstand = (px / wunsch_breite).ln().abs();
            if beste.is_none_or(|(a, _)| abstand < a - 1e-9) {
                beste = Some((
                    abstand,
                    MaskenWahl { muster, groesse, pixel_je_triade: px, triaden: flaechen_breite as f64 / px },
                ));
            }
        }
    }
    beste.expect("Musterliste ist nicht leer").1
}

/// Übersetzt Einstellungen in Kette und Parameter.
pub fn plane(e: &Einstellungen, k: &Kontext) -> Plan {
    if !e.schalter("allgemein.roehre") {
        return Plan { kette: Kette::Sauber, parameter: Vec::new(), maske: None };
    }
    let art = e.auswahl("signal.art");
    let pal = e.auswahl("signal.norm") == "pal";
    let leicht = e.auswahl("allgemein.leistung") == "leicht";
    let kette = match (art, pal, leicht) {
        ("rgb", _, false) => Kette::GuestRgb,
        ("rgb", _, true) => Kette::GuestSchnell,
        (_, true, _) => Kette::GuestPal,
        (_, false, _) => Kette::GuestNtsc,
    };

    let z = |s: &str| e.zahl(s) as f32;
    let mut p: Vec<(&'static str, f32)> = Vec::new();

    // --- Signal
    let rauschen = z("signal.rauschen");
    p.push(("addnoised", rauschen));
    p.push(("noisetype", 1.0));
    if kette == Kette::GuestNtsc {
        // guest: „A&F Values: Svideo = 0.0 | Composite = 1.0 | RF = 2.0“
        let faktor = match art {
            "svideo" => 0.0,
            "antenne" => 2.0,
            _ => 1.0,
        };
        p.push(("cust_artifacting", z("signal.artefakte") * faktor));
        p.push(("cust_fringing", z("signal.farbsaum") * faktor));
        p.push(("ntsc_sat", z("signal.saettigung")));
        let rf = if art == "antenne" { 0.1 + rauschen } else { 0.0 };
        p.push(("RFNOISE1", rf));
        p.push(("RFNOISE2", rf));
    }
    if kette == Kette::GuestPal {
        // guest: „PAL Connection: RF | Composite | S-Video | SCART“
        let anschluss = match art {
            "antenne" => 1.0,
            "composite" => 2.0,
            _ => 3.0,
        };
        p.push(("PAL_FILT", anschluss));
        // Der Standard der PAL-Pässe ist 0,7; „1,0“ im Modell entspricht ihm.
        p.push(("ARTIFACTING", 0.7 * z("signal.artefakte")));
        p.push(("FRINGING", 0.7 * z("signal.farbsaum")));
        p.push(("SATURATION", 1.05 * z("signal.saettigung")));
    }

    // --- Strahl
    p.push(("beam_min", z("strahl.form_dunkel")));
    p.push(("beam_max", z("strahl.form_hell")));
    p.push(("beam_size", z("strahl.aufblaehen")));
    p.push(("scans", z("strahl.zeilenfarbe")));
    p.push(("h_sharp", z("strahl.schaerfe")));
    p.push((
        "interm",
        match e.auswahl("strahl.zeilensprung") {
            "aus" => 0.0,
            "geglaettet" => 4.0,
            _ => 1.0,
        },
    ));
    p.push(("deconrr", z("strahl.konvergenz_rot")));
    p.push(("deconrb", z("strahl.konvergenz_blau")));
    p.push(("deconrry", z("strahl.konvergenz_rot_y")));
    p.push(("deconrby", z("strahl.konvergenz_blau_y")));
    p.push(("dctypex", z("strahl.konvergenz_rand")));
    p.push(("dctypey", z("strahl.konvergenz_rand")));

    // --- Maske
    let typ = e.auswahl("maske.typ");
    let maske = if typ == "aus" {
        p.push(("shadowMask", -1.0));
        None
    } else {
        let wahl = waehle_maske(k.flaeche.0, e.zahl("maske.triaden"));
        p.push(("shadowMask", wahl.muster as f32));
        p.push(("masksize", wahl.groesse as f32));
        p.push(("maskstr", z("maske.staerke")));
        p.push(("mcut", z("maske.staerke_dunkel")));
        p.push(("mask_layout", if e.auswahl("maske.subpixel") == "bgr" { 1.0 } else { 0.0 }));
        let (slot_hell, slot_dunkel, versatz) = match typ {
            "schlitzmaske" => ((z("maske.staerke") * 1.5).min(1.0), (z("maske.staerke_dunkel") * 0.4).min(1.0), 0.0),
            "lochmaske" => (0.0, 0.0, 1.0),
            _ => (0.0, 0.0, 0.0),
        };
        p.push(("slotmask", slot_hell));
        p.push(("slotmask1", slot_dunkel));
        p.push(("slotwidth", 0.0));
        p.push(("double_slot", 2.0));
        p.push(("slotms", wahl.groesse as f32));
        p.push(("mshift", versatz));
        Some(wahl)
    };

    // --- Leuchtstoff
    p.push((
        "CP",
        match e.auswahl("leuchtstoff.profil") {
            "ebu" => 0.0,
            "p22" => 1.0,
            "smpte_c" => 2.0,
            "philips" => 3.0,
            "trinitron" => 4.0,
            _ => -1.0,
        },
    ));
    p.push(("WP", z("leuchtstoff.farbtemperatur")));
    p.push(("wp_saturation", z("leuchtstoff.saettigung")));
    p.push(("AS", z("leuchtstoff.nachgluehen")));

    // --- Licht
    p.push(("glow", z("licht.glow")));
    p.push(("bloom", z("licht.bloom")));
    p.push(("halation", z("licht.halation")));

    // --- Geometrie
    p.push(("warpX", z("geometrie.woelbung_x")));
    p.push(("warpY", z("geometrie.woelbung_y")));
    p.push(("csize", z("geometrie.ecken")));
    p.push(("bsize1", z("geometrie.randweich")));
    p.push(("vigstr", z("geometrie.vignette")));
    // guest misst Overscan in Pixeln des Eingangsbilds.
    let os = z("geometrie.overscan") / 100.0;
    p.push(("overscanX", (os * k.eingang.0 as f32).round()));
    p.push(("overscanY", (os * k.eingang.1 as f32).round()));

    // --- Helligkeit
    p.push(("brightboost", z("helligkeit.ausgleich_dunkel")));
    p.push(("brightboost1", z("helligkeit.ausgleich_hell")));
    p.push(("post_br", z("helligkeit.nachher")));
    p.push(("gamma_out", z("helligkeit.gamma_ausgang")));

    Plan { kette, parameter: p, maske }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maske_trifft_ganze_pixel() {
        // 1080p, 4:3-Fläche 1440 breit, 450 Tripel gewünscht → 3,2 px → Muster 6 (3 px)
        let w = waehle_maske(1440, 450.0);
        assert_eq!((w.muster, w.groesse), (6, 1));
        // 4K, Fläche 2880 breit, 450 Tripel → 6,4 px → 6 px
        let w = waehle_maske(2880, 450.0);
        assert!((w.pixel_je_triade - 6.0).abs() < 1e-9, "{w:?}");
        // 8K, Fläche 5760, 400 Tripel → 14,4 px → 14 (7×2)
        let w = waehle_maske(5760, 400.0);
        assert!((w.pixel_je_triade - 14.0).abs() < 1e-9, "{w:?}");
    }

    #[test]
    fn kettenwahl() {
        let k = Kontext { eingang: (256, 224), flaeche: (1440, 1080) };
        let mut e = Einstellungen::voreinstellung("wohnzimmer-90er-composite").unwrap();
        assert_eq!(plane(&e, &k).kette, Kette::GuestNtsc);
        e.setze("signal.norm", roehre_modell::Wert::Auswahl("pal".into()));
        assert_eq!(plane(&e, &k).kette, Kette::GuestPal);
        let e = Einstellungen::voreinstellung("aus").unwrap();
        assert_eq!(plane(&e, &k).kette, Kette::Sauber);
    }

    #[test]
    fn presets_laden() {
        let ordner = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shader");
        for kette in [Kette::Sauber, Kette::GuestRgb, Kette::GuestSchnell, Kette::GuestNtsc, Kette::GuestPal] {
            let p = kette.lade(&ordner).unwrap_or_else(|e| panic!("{kette:?}: {e}"));
            assert!(!p.passes.is_empty());
        }
    }
}
