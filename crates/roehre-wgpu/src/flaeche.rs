//! Einpassen des Bildes in das Ziel: Seitenverhältnis, Füllung, ganzzahlige Skalierung.

use roehre_modell::Einstellungen;

/// Beschreibung des Spielbilds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eingang {
    pub breite: u32,
    pub hoehe: u32,
    /// Breite eines Pixels geteilt durch seine Höhe (SNES NTSC: 8/7, quadratisch: 1).
    pub pixel_seitenverhaeltnis: f64,
}

/// Ein Rechteck in ganzen Ausgabepixeln.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rechteck {
    pub x: u32,
    pub y: u32,
    pub breite: u32,
    pub hoehe: u32,
}

/// Berechnet die Röhrenfläche im Ziel. Die Fläche liegt immer auf ganzen Pixeln, damit die
/// Maske der Röhre 1:1 auf Ausgabepixel fällt.
pub fn berechne(e: &Einstellungen, eingang: &Eingang, ziel: (u32, u32)) -> Rechteck {
    let (zb, zh) = (ziel.0.max(1) as f64, ziel.1.max(1) as f64);
    let fuellung = e.zahl("bildformat.fuellung") / 100.0;
    let seite = match e.auswahl("bildformat.seitenverhaeltnis") {
        "pixelgenau" => eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64,
        "16:9" => 16.0 / 9.0,
        "fenster" => {
            let (b, h) = ((zb * fuellung).round(), (zh * fuellung).round());
            return mittig(ziel, b as u32, h as u32);
        }
        _ => 4.0 / 3.0,
    };
    let max_h = (zh * fuellung).min(zb * fuellung / seite);
    let h = bildhoehe(max_h, sichtbare_zeilen(e, eingang), e.schalter("bildformat.ganzzahlig"), e.schalter("allgemein.roehre"));
    let b = (h * seite).round().min(zb);
    mittig(ziel, b as u32, h as u32)
}

/// Unter so vielen Pixeln je Zeile rastet die Bildhöhe auch ohne `bildformat.ganzzahlig`
/// auf ein ganzzahliges Vielfaches der Zeilenzahl ein. Gemessen (Etappe 4): Breitbild-Röhre
/// im Gehäuse bei 720p, 2,5 px je Zeile: Zeilenschwebung 3,4–4,6 %.
pub const AUTOMATISCH_GANZZAHLIG_UNTER: f64 = 4.0;
/// … aber nur, wenn das Bild dabei höchstens um diesen Anteil kleiner wird.
pub const AUTOMATISCH_GANZZAHLIG_VERLUST: f64 = 0.3;

/// Höhe der Bildfläche in ganzen Pixeln: ganzzahlig, wenn gewünscht oder wenn die Zeilen sonst
/// zu dicht für ein sauberes Strahlprofil liegen (`automatik`, nur bei eingeschalteter Röhre).
/// `zeilen` sind die sichtbaren Zeilen (nach Overscan, siehe [`sichtbare_zeilen`]).
pub fn bildhoehe(max_h: f64, zeilen: f64, ganzzahlig: bool, automatik: bool) -> f64 {
    if zeilen < 1.0 {
        return max_h.floor();
    }
    let faktor = (max_h / zeilen).floor();
    let einrasten = faktor >= 1.0
        && (ganzzahlig
            || automatik
                && (max_h / zeilen) < AUTOMATISCH_GANZZAHLIG_UNTER
                && faktor * zeilen >= (1.0 - AUTOMATISCH_GANZZAHLIG_VERLUST) * max_h);
    if einrasten { (faktor * zeilen).round() } else { max_h.floor() }
}

/// Zeilen, die nach dem Overscan sichtbar bleiben (bei Overscan kein ganzer Wert).
pub fn sichtbare_zeilen(e: &Einstellungen, eingang: &Eingang) -> f64 {
    let (_, osy) = roehre_kette::overscan_pixel(e, (eingang.breite, eingang.hoehe));
    eingang.hoehe as f64 - osy as f64
}

fn mittig(ziel: (u32, u32), b: u32, h: u32) -> Rechteck {
    let (b, h) = (b.clamp(1, ziel.0.max(1)), h.clamp(1, ziel.1.max(1)));
    Rechteck { x: (ziel.0 - b) / 2, y: (ziel.1 - h) / 2, breite: b, hoehe: h }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roehre_modell::Wert;

    const SNES: Eingang = Eingang { breite: 256, hoehe: 224, pixel_seitenverhaeltnis: 8.0 / 7.0 };

    #[test]
    fn vier_zu_drei_in_1080p() {
        let e = Einstellungen::standard();
        assert_eq!(berechne(&e, &SNES, (1920, 1080)), Rechteck { x: 240, y: 0, breite: 1440, hoehe: 1080 });
    }

    #[test]
    fn pixelgenau_und_ganzzahlig() {
        let mut e = Einstellungen::standard();
        e.setze("bildformat.seitenverhaeltnis", Wert::Auswahl("pixelgenau".into()));
        e.setze("bildformat.ganzzahlig", Wert::Schalter(true));
        let r = berechne(&e, &SNES, (1920, 1080));
        assert_eq!(r.hoehe, 896); // 4 × 224
        assert_eq!(r.breite, 1170); // 256 × 8/7 × 4 = 1170,3
    }

    #[test]
    fn hochformat_fenster() {
        let e = Einstellungen::standard();
        let r = berechne(&e, &SNES, (600, 1000));
        // 450 px Höhe wären 2,0 px je Zeile: rastet auf 2 × 224 ein
        assert_eq!((r.breite, r.hoehe), (597, 448));
    }
}
