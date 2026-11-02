//! Gehäuse: Anordnung von Bild, Glas und Rahmen im Ziel, und die Formwerte der Gehäuse.
//!
//! Alle Gehäuse sind berechnet (Signed Distance Fields im Zusammensetz-Pass), es gibt keine
//! Bilddateien. Maße stehen in Einheiten der **halben Glashöhe**: Das Glas reicht senkrecht
//! von −1 bis 1, waagerecht von −a bis a (a = Seitenverhältnis des Glases).
//!
//! Die Bildfläche (dort zeichnet die Röhre) liegt weiterhin auf ganzen Pixeln; Glas und
//! Rahmen werden um sie herum gelegt. So bleibt die Maske moiréfrei.

use crate::flaeche::{Eingang, Rechteck};
use roehre_modell::Einstellungen;

/// Form eines Gehäuses (alle Längen in halben Glashöhen).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Form {
    /// 1 = nur Glas, 2 = Fernseher, 3 = Monitor, 4 = Automat
    pub art: u32,
    /// Eckenradius des Glases
    pub radius_glas: f32,
    /// Breite der Innenfase (schräge Kante zwischen Glas und Rahmen)
    pub fase: f32,
    /// Rahmenbreite seitlich, oben, unten
    pub seite: f32,
    pub oben: f32,
    pub unten: f32,
    /// Eckenradius außen
    pub radius_aussen: f32,
    /// Farbe des Kunststoffs (linear)
    pub farbe: [f32; 3],
    /// Rauheit 0 (glänzend) … 1 (matt)
    pub rauheit: f32,
}

impl Form {
    fn fuer(art: &str) -> Option<Self> {
        Some(match art {
            "nur_glas" => Form { art: 1, radius_glas: 0.07, fase: 0.0, seite: 0.0, oben: 0.0, unten: 0.0, radius_aussen: 0.0, farbe: [0.0; 3], rauheit: 1.0 },
            // Fernseher der 90er: dunkler Kunststoff, breiter unterer Rand mit Lautsprechergitter
            "fernseher" => Form {
                art: 2,
                radius_glas: 0.09,
                fase: 0.07,
                seite: 0.22,
                oben: 0.2,
                unten: 0.42,
                radius_aussen: 0.12,
                farbe: [0.035, 0.035, 0.038],
                rauheit: 0.55,
            },
            // PC-Monitor: heller, leicht gelblicher Kunststoff, schmaler Rand
            "monitor" => Form {
                art: 3,
                radius_glas: 0.06,
                fase: 0.06,
                seite: 0.16,
                oben: 0.15,
                unten: 0.24,
                radius_aussen: 0.07,
                farbe: [0.42, 0.40, 0.34],
                rauheit: 0.7,
            },
            // Spielautomat: schwarze, matte Blende mit Tiefe
            "automat" => Form {
                art: 4,
                radius_glas: 0.05,
                fase: 0.1,
                seite: 0.35,
                oben: 0.3,
                unten: 0.3,
                radius_aussen: 0.02,
                farbe: [0.012, 0.012, 0.013],
                rauheit: 0.9,
            },
            _ => return None,
        })
    }
}

/// Ergebnis der Anordnung.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anordnung {
    /// Bildfläche (dort zeichnet die Röhre), ganze Pixel
    pub inhalt: Rechteck,
    /// Glas: Mitte x, Mitte y, halbe Breite, halbe Höhe (Pixel)
    pub glas: [f32; 4],
    /// Gehäuseform, `None` = ohne Gehäuse (Bild wie bisher, Rest Hintergrund)
    pub form: Option<Form>,
}

/// Wie weit das Glas über die Bildfläche hinausragt (schwarzer Leuchtstoffrand), je Seite in
/// halben Glashöhen.
const UEBERSTAND: f64 = 0.03;

/// Ordnet Bild, Glas und Gehäuse im Ziel an.
pub fn anordnen(e: &Einstellungen, eingang: &Eingang, ziel: (u32, u32)) -> Anordnung {
    let form = if e.schalter("allgemein.roehre") { Form::fuer(e.auswahl("gehaeuse.typ")) } else { None };
    let Some(form) = form else {
        let inhalt = crate::flaeche::berechne(e, eingang, ziel);
        let g = [
            inhalt.x as f32 + inhalt.breite as f32 / 2.0,
            inhalt.y as f32 + inhalt.hoehe as f32 / 2.0,
            inhalt.breite as f32 / 2.0,
            inhalt.hoehe as f32 / 2.0,
        ];
        return Anordnung { inhalt, glas: g, form: None };
    };

    let (zb, zh) = (ziel.0.max(1) as f64, ziel.1.max(1) as f64);
    let fuellung = e.zahl("bildformat.fuellung") / 100.0;
    // Seitenverhältnis des Bildes (wie ohne Gehäuse)
    let inhalt_seite = match e.auswahl("bildformat.seitenverhaeltnis") {
        "pixelgenau" => eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64,
        "16:9" => 16.0 / 9.0,
        "fenster" => zb / zh,
        _ => 4.0 / 3.0,
    };
    // Glas: so breit wie das Bild oder 16:9 (Breitbild-Röhre)
    let glas_seite = if e.auswahl("bildformat.roehrenformat") == "16:9" { inhalt_seite.max(16.0 / 9.0) } else { inhalt_seite };
    let a = glas_seite;

    // Sichtbarer Ausschnitt in halben Glashöhen: zwischen ganzem Gehäuse (zoom 0) und
    // Glas allein (zoom 1)
    let zoom = if form.art == 1 { 1.0 } else { e.zahl("gehaeuse.zoom") };
    let (s, o, u) = (form.seite as f64, form.oben as f64, form.unten as f64);
    let sichtbar_h = (2.0 + o + u) * (1.0 - zoom) + 2.0 * (1.0 + UEBERSTAND) * zoom;
    let sichtbar_b = 2.0 * (a + s) * (1.0 - zoom) + 2.0 * (a + UEBERSTAND) * zoom;
    let halbe_glashoehe = (zh * fuellung / sichtbar_h).min(zb * fuellung / sichtbar_b);

    // Bildfläche auf ganze Pixel (bei ganzzahliger Skalierung auf ganze Vielfache)
    let inhalt_h = crate::flaeche::bildhoehe(2.0 * halbe_glashoehe / (1.0 + UEBERSTAND), crate::flaeche::sichtbare_zeilen(e, eingang), e.schalter("bildformat.ganzzahlig"), true);
    let inhalt_b = (inhalt_h * inhalt_seite).round();
    let hh = inhalt_h * (1.0 + UEBERSTAND) / 2.0;

    // Mitte: Das ganze Gehäuse steht mittig, beim Heranrücken wandert die Mitte zum Glas
    let cx = zb / 2.0;
    let cy = zh / 2.0 - hh * (u - o) / 2.0 * (1.0 - zoom);
    let inhalt = Rechteck {
        x: (cx - inhalt_b / 2.0).round().max(0.0) as u32,
        y: (cy - inhalt_h / 2.0).round().max(0.0) as u32,
        breite: (inhalt_b as u32).clamp(1, ziel.0.max(1)),
        hoehe: (inhalt_h as u32).clamp(1, ziel.1.max(1)),
    };
    // Glasmitte genau auf die Mitte der ganzzahligen Bildfläche legen
    let gx = inhalt.x as f64 + inhalt.breite as f64 / 2.0;
    let gy = inhalt.y as f64 + inhalt.hoehe as f64 / 2.0;
    Anordnung { inhalt, glas: [gx as f32, gy as f32, (hh * a) as f32, hh as f32], form: Some(form) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roehre_modell::Wert;

    const SNES: Eingang = Eingang { breite: 256, hoehe: 224, pixel_seitenverhaeltnis: 8.0 / 7.0 };

    #[test]
    fn ohne_gehaeuse_wie_bisher() {
        let e = Einstellungen::standard();
        let a = anordnen(&e, &SNES, (1920, 1080));
        assert_eq!(a.inhalt, crate::flaeche::berechne(&e, &SNES, (1920, 1080)));
        assert!(a.form.is_none());
    }

    #[test]
    fn fernseher_passt_ins_fenster() {
        let mut e = Einstellungen::standard();
        e.setze("gehaeuse.typ", Wert::Auswahl("fernseher".into()));
        e.setze("gehaeuse.zoom", Wert::Zahl(0.0));
        let a = anordnen(&e, &SNES, (1920, 1080));
        let f = a.form.unwrap();
        let hh = a.glas[3];
        let oben = a.glas[1] - hh * (1.0 + f.oben);
        let unten = a.glas[1] + hh * (1.0 + f.unten);
        assert!(oben >= -1.0 && unten <= 1081.0, "{oben} {unten}");
        assert!(a.inhalt.hoehe < 1080);
    }

    #[test]
    fn breitbild_glas_ist_16_zu_9() {
        let mut e = Einstellungen::standard();
        e.setze("gehaeuse.typ", Wert::Auswahl("nur_glas".into()));
        e.setze("bildformat.roehrenformat", Wert::Auswahl("16:9".into()));
        let a = anordnen(&e, &SNES, (1920, 1080));
        assert!((a.glas[2] / a.glas[3] - 16.0 / 9.0).abs() < 1e-3);
        assert!((a.inhalt.breite as f32 / a.inhalt.hoehe as f32 - 4.0 / 3.0).abs() < 0.01);
    }
}
