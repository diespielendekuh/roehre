//! Selbst erzeugte Materialien des Raums (keine Fotos, keine fremden Texturen).
//!
//! Jedes Material ist eine Ebene zweier Texturfelder (256 × 256):
//! * **Farbe** (sRGB) und Deckung (Alpha, für Blätter),
//! * **Eigenschaften**: R = Relief (Höhe), G = Rauheit, B = Metall, A = Selbstleuchten.
//!
//! Der Stil zielt auf die späte PS2-Zeit: wenige Polygone, aber dichte, handgemachte
//! Materialien mit Alterung (Flecken, Abnutzung, Staub).

use crate::rauschen::{fbm, wert, zufall};

/// Kantenlänge der Materialtexturen. 256 wie zur PS2-Zeit; der Raum zeigt sie ohne
/// Vergrößerungsfilter, so dass aus der Nähe die Rohpixel sichtbar sind.
pub const GROESSE: u32 = 256;

/// Die Materialien; die Nummer ist die Ebene im Texturfeld.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Material {
    Furnier = 0,
    KunststoffSchwarz,
    Chrom,
    Lautsprecherstoff,
    Tapete,
    Dielen,
    Teppich,
    Tischholz,
    Terrakotta,
    Erde,
    Blatt,
    Kassette,
    Keramik,
    KunststoffGrau,
    Vorhang,
    Wandbild,
    Lack,
    Lampenschirm,
    Gummi,
    /// Nachthimmel hinter dem Fenster (Himmel, Mond, Sterne rechnet der Shader aus der Blickrichtung)
    Himmel,
    Gluehbirne,
    LedRot,
    Decke,
    RahmenHolz,
    Zeitschrift,
    Glas,
    /// Die Bildröhre (Bild kommt aus dem Flächen-Modus, nicht aus dem Texturfeld)
    Bildroehre,
    Goldrahmen,
    Passepartout,
    Fernbedienung,
    Triebhuelle,
    Rinde,
    Buecher,
    Videorekorder,
    Blende,
    Beschriftung,
    Skala,
    Knopf,
    Kanalskala,
    LedGruen,
    Wolken,
    Parkboden,
    Baumkulisse,
    Ferne,
    Laterne,
    Auto,
    Heizlack,
    Taste,
}

pub const ANZAHL: u32 = Material::Taste as u32 + 1;

/// Alle Materialien in Ebenen-Reihenfolge.
pub const ALLE: [Material; ANZAHL as usize] = {
    use Material::*;
    [
        Furnier, KunststoffSchwarz, Chrom, Lautsprecherstoff, Tapete, Dielen, Teppich, Tischholz, Terrakotta, Erde,
        Blatt, Kassette, Keramik, KunststoffGrau, Vorhang, Wandbild, Lack, Lampenschirm, Gummi, Himmel, Gluehbirne,
        LedRot, Decke, RahmenHolz, Zeitschrift, Glas, Bildroehre, Goldrahmen, Passepartout, Fernbedienung,
        Triebhuelle, Rinde, Buecher, Videorekorder, Blende, Beschriftung, Skala, Knopf, Kanalskala, LedGruen,
        Wolken, Parkboden, Baumkulisse, Ferne, Laterne, Auto, Heizlack, Taste,
    ]
};

fn mische(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn mal(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn glatt(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Holzmaserung entlang u: Jahresringe, von Rauschen verzerrt, mit Poren.
fn maserung(u: f32, v: f32, dichte: f32, saat: u32) -> (f32, f32) {
    let verzug = fbm(u * 0.5, v, 2, 4, saat) * 6.0 + fbm(u, v * 2.0, 4, 3, saat + 1) * 1.5;
    let ring = ((v * dichte + verzug) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let poren = wert(u * 300.0, v * 40.0, 300, saat + 2);
    (ring.powf(2.5), poren)
}

/// 5 × 7-Pixelschrift (Großbuchstaben, Ziffern, Umlaute, wenige Zeichen). Zeile für Zeile,
/// Bit 4 ist das linke Pixel.
fn glyphe(z: char) -> [u8; 7] {
    match z {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 17, 25, 21, 19, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        'Ä' => [10, 0, 14, 17, 31, 17, 17],
        'Ö' => [10, 0, 14, 17, 17, 17, 14],
        'Ü' => [10, 0, 17, 17, 17, 17, 14],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [31, 2, 4, 2, 1, 17, 14],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 30, 1, 1, 17, 14],
        '6' => [6, 8, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 2, 12],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '\'' => [4, 4, 8, 0, 0, 0, 0],
        '&' => [12, 18, 20, 8, 21, 18, 13],
        '!' => [4, 4, 4, 4, 4, 0, 4],
        _ => [0; 7],
    }
}

/// Deckt die Pixelschrift das Texel (tx, ty)? `ox`, `oy` linke obere Ecke, `s` Vergrößerung,
/// `wackeln` versetzt jedes Zeichen senkrecht um bis zu einen Pixel (Handschrift).
fn schrift(text: &str, ox: i32, oy: i32, s: i32, wackeln: bool, tx: i32, ty: i32) -> bool {
    let (gx, gy) = ((tx - ox).div_euclid(s), (ty - oy).div_euclid(s));
    if gx < 0 {
        return false;
    }
    let (i, spalte) = ((gx / 6) as usize, gx % 6);
    let Some(z) = text.chars().nth(i) else { return false };
    let dy = if wackeln { (zufall(i as i32, 811) * 2.0) as i32 - 1 } else { 0 };
    let zeile = gy - dy;
    spalte < 5 && (0..7).contains(&zeile) && glyphe(z)[zeile as usize] >> (4 - spalte) & 1 == 1
}

/// Rücken einer Videohülle (Zeile `zeile` des Atlas), Texel (tx 0 … 255, ty 0 … 31).
fn kassettenruecken(zeile: usize, tx: i32, ty: i32, u: f32, v: f32) -> ([f32; 3], f32) {
    // (Titel, Grund, Schrift, Akzent): erfundene Filme im Stil der Genres
    type Huelle = (&'static str, [f32; 3], [f32; 3], [f32; 3]);
    const HUELLEN: [Huelle; 8] = [
        ("NEBELNACHT", [0.010, 0.010, 0.012], [0.62, 0.02, 0.02], [0.30, 0.30, 0.30]),
        ("STERNENFEUER", [0.012, 0.025, 0.12], [0.85, 0.62, 0.06], [0.50, 0.50, 0.55]),
        ("DER STILLE SEE", [0.60, 0.56, 0.44], [0.03, 0.12, 0.06], [0.10, 0.25, 0.12]),
        ("SCHATTENSTADT", [0.045, 0.045, 0.05], [0.78, 0.26, 0.02], [0.60, 0.20, 0.02]),
        ("LETZTE FAHRT", [0.42, 0.03, 0.02], [0.85, 0.85, 0.82], [0.05, 0.05, 0.05]),
        ("WINTERREISE", [0.70, 0.72, 0.76], [0.06, 0.22, 0.48], [0.25, 0.40, 0.60]),
        ("GEISTERSTUNDE", [0.11, 0.03, 0.17], [0.22, 0.62, 0.10], [0.45, 0.45, 0.10]),
        ("SOMMER '88", [0.016, 0.016, 0.018], [0.05, 0.08, 0.40], [0.66, 0.65, 0.60]),
    ];
    let (titel, grund, farbe, akzent) = HUELLEN[zeile];
    let alter = fbm(u, v, 16, 3, 821 + zeile as u32);
    let mut c = grund;
    let mut h = 0.5;
    if zeile == 7 {
        // Selbst bespielt: weißer Aufkleber, mit Kuli beschriftet
        if (36..220).contains(&tx) && (6..26).contains(&ty) {
            c = mal(akzent, 0.9 + 0.15 * alter);
            if ty == 8 || ty == 23 {
                c = mische(c, [0.3, 0.05, 0.05], 0.4); // vorgedruckte Linien
            }
            if schrift(titel, 66, 9, 2, true, tx, ty) {
                c = farbe;
            }
            h = 0.55;
        }
    } else {
        // Verleihhülle: Verlaufsband, Studiokasten links, Freigabefleck rechts, Titel
        let verlauf = ty as f32 / 31.0;
        c = mische(grund, mal(grund, 1.6), 0.5 - (verlauf - 0.5).abs());
        if (6..34).contains(&tx) && (5..27).contains(&ty) {
            c = akzent;
            if (10..30).contains(&tx) && (13..19).contains(&ty) {
                c = mal(grund, 0.6);
            }
        }
        if (228..246).contains(&tx) && (8..24).contains(&ty) {
            c = [[0.05, 0.25, 0.6], [0.05, 0.4, 0.08], [0.7, 0.6, 0.05], [0.6, 0.05, 0.04]][zeile % 4];
        }
        if zeile == 1 && zufall(tx * 37 + ty, 831) > 0.985 {
            c = [0.8, 0.8, 0.7]; // Sterne
        }
        if schrift(titel, 44, 9, 2, false, tx, ty) {
            c = farbe;
            h = 0.6;
        }
    }
    // Rand, Abrieb, Staub
    if ty == 0 || ty == 31 {
        c = mal(c, 0.5);
    }
    let c = mal(c, 0.9 + 0.12 * alter);
    (c, h)
}

/// Front des Videorekorders (Texel tx 0 … 255, ty 0 … 53). Liefert Farbe, Höhe, Rauheit,
/// Selbstleuchten.
fn videorekorder(tx: i32, ty: i32, u: f32, v: f32) -> ([f32; 3], f32, f32, f32) {
    let gebuerstet = wert(u * 4.0, v * 256.0, 4, 841);
    let mut c = mal([0.028, 0.028, 0.031], 0.9 + 0.2 * gebuerstet);
    let (mut h, mut rauh, mut leuchten) = (0.5, 0.45, 0.0);
    let kasten = |x0: i32, y0: i32, x1: i32, y1: i32| (x0..x1).contains(&tx) && (y0..y1).contains(&ty);
    // Zierleisten oben und unten (gebürstetes Silber)
    if ty < 3 || (48..54).contains(&ty) {
        c = mal([0.20, 0.20, 0.21], 0.85 + 0.3 * gebuerstet);
        rauh = 0.3;
    }
    // Kassettenklappe mit Rand
    if kasten(18, 10, 132, 33) {
        c = [0.008, 0.008, 0.009];
        rauh = 0.18;
        h = 0.4;
        if ty == 10 || ty == 32 || tx == 18 || tx == 131 {
            c = [0.09, 0.09, 0.095];
            h = 0.6;
        }
        if ty == 21 && (30..120).contains(&tx) {
            c = [0.05, 0.05, 0.055];
        }
    }
    // Anzeige (Leuchtziffern) mit blinkender Uhrzeit, die nie jemand eingestellt hat
    if kasten(150, 10, 236, 28) {
        c = [0.004, 0.006, 0.010];
        rauh = 0.1;
        if schrift("12:00", 175, 13, 2, false, tx, ty) {
            c = [0.10, 0.80, 0.62];
            leuchten = 1.0;
        }
        if schrift("CH", 154, 15, 1, false, tx, ty) || schrift("03", 154, 21, 1, false, tx, ty) {
            c = [0.08, 0.55, 0.45];
            leuchten = 0.8;
        }
    }
    // Tastenreihe unter der Anzeige
    for i in 0..5 {
        if kasten(150 + i * 17, 35, 163 + i * 17, 42) {
            c = mal([0.16, 0.16, 0.17], 0.9 + 0.2 * gebuerstet);
            h = 0.75;
            rauh = 0.35;
        }
    }
    // Netzschalter mit roter Leuchte
    if kasten(20, 37, 40, 44) {
        c = [0.14, 0.14, 0.15];
        h = 0.75;
    }
    if kasten(46, 39, 49, 42) {
        c = [0.9, 0.05, 0.02];
        leuchten = 1.0;
    }
    (c, h, rauh, leuchten)
}

/// Massive Eiche, geölt. `a` läuft entlang der Faser (0 … 1, kachelbar), `b` quer über das
/// Brett (0 … 1); `saat` wählt das Brett. Was Eiche als Eiche lesbar macht: viele feine,
/// gerade Faserlinien, Jahresringe als lange, leicht wellige Streifen mit dunklem Spätholz,
/// Porenrillen als kurze dunkle Striche im Frühholz, selten ein kleiner Ast, dazu ein warmes,
/// mittleres Braun (nicht honiggelb). Liefert Farbe (linear), Höhe, Rauheit.
fn eiche(a: f32, b: f32, saat: u32, ringe: f32) -> ([f32; 3], f32, f32) {
    use std::f32::consts::TAU;
    let s = saat as i32;
    // Fladerung: die Ringe biegen sich über die Länge langsam (kachelbar in a)
    let bogen = 0.10 * (a * TAU + zufall(s, 401) * TAU).sin() + 0.04 * (2.0 * a * TAU + zufall(s, 402) * TAU).sin();
    let welle = fbm(a, b * 0.5, 4, 3, saat + 11) * 0.5 + fbm(a, b, 16, 2, saat + 12) * 0.06;
    let rk = b * ringe + (bogen + welle) * ringe * 0.35;
    let ring = rk.fract();
    let spaet = glatt(0.40, 0.70, ring) * (1.0 - glatt(0.86, 1.0, ring));
    let frueh = 1.0 - glatt(0.0, 0.22, ring);
    // Poren: kurze Striche entlang der Faser, gehäuft im Frühholz
    let poren = glatt(0.68, 0.86, wert(a * 48.0, b * 160.0 + rk * 3.0, 48, saat + 13)) * (0.35 + 0.65 * frueh);
    // Feine Faserlinien quer verteilt, jede mit eigener Stärke
    let fl = b * 110.0 + welle * 20.0;
    let faser = glatt(0.78, 0.98, 1.0 - (fl.fract() - 0.5).abs() * 2.0) * (0.3 + 0.7 * zufall(fl.floor() as i32 + s * 1000, 403));
    // Seltener kleiner Ast
    let mut ast = 0.0f32;
    if zufall(s, 404) > 0.72 {
        let (cx, cy) = (zufall(s, 405), 0.2 + 0.6 * zufall(s, 406));
        let (dx, dy) = ((a - cx).abs().min(1.0 - (a - cx).abs()) * 4.0, (b - cy) * 1.4);
        let d = (dx * dx + dy * dy).sqrt();
        ast = 1.0 - glatt(0.015, 0.035, d);
    }
    let ton = 0.82 + 0.28 * zufall(s, 407) + 0.12 * (fbm(a, b, 2, 2, saat + 14) - 0.5);
    let hell = [0.225, 0.13, 0.062];
    let dunkel = [0.12, 0.066, 0.03];
    let mut c = mische(hell, dunkel, 0.12 + 0.5 * spaet);
    c = mal(c, ton * (1.0 - 0.16 * faser) * (1.0 - 0.38 * poren));
    c = mische(c, [0.04, 0.022, 0.01], ast * 0.85);
    // Relief flach halten: geöltes Holz ist fast glatt, starke Poren glitzern als Einzelpixel
    let hoehe = 0.55 + 0.04 * spaet - 0.10 * poren - 0.04 * faser - 0.15 * ast;
    (c, hoehe, 0.48 + 0.2 * poren + 0.08 * spaet)
}

/// Farbe (linear, Alpha) und Eigenschaften (Höhe, Rauheit, Metall, Leuchten) bei (u, v).
fn texel(m: Material, u: f32, v: f32) -> ([f32; 4], [f32; 4]) {
    use Material::*;
    let staub = fbm(u, v, 4, 4, 999);
    match m {
        Furnier => {
            // Nussbaumfurnier mit Klarlack: dunkle Ringe, warme Zwischenräume, feine Poren
            let (ring, poren) = maserung(u, v, 22.0, 11);
            let c = mische([0.075, 0.042, 0.022], [0.035, 0.018, 0.009], 0.35 + 0.5 * ring);
            let c = mal(c, 0.85 + 0.25 * fbm(u, v, 8, 3, 12) - 0.12 * poren);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.3 * ring - 0.2 * poren, 0.32, 0.0, 0.0])
        }
        KunststoffSchwarz => {
            let fein = wert(u * 400.0, v * 400.0, 400, 21);
            let g = 0.022 + 0.006 * fein + 0.01 * staub * staub;
            ([g, g, g * 1.05, 1.0], [0.5 + 0.2 * fein, 0.38 + 0.2 * staub, 0.0, 0.0])
        }
        Chrom => {
            // gebürstetes Metall: feine Riefen in u-Richtung
            let riefen = fbm(u * 0.05, v * 256.0, 2, 3, 31);
            let g = 0.55 + 0.12 * riefen;
            ([g, g, g * 1.02, 1.0], [0.5 + 0.2 * riefen, 0.18 + 0.1 * riefen, 1.0, 0.0])
        }
        Lautsprecherstoff => {
            let t = std::f32::consts::TAU * 128.0;
            let webart = ((u * t).sin() * (v * t).sin()).abs();
            let g = 0.035 + 0.025 * webart;
            ([g, g * 0.95, g * 0.9, 1.0], [webart, 0.95, 0.0, 0.0])
        }
        Tapete => {
            // 80er-Tapete: blasses Ocker-Grün, feine Streifen, ruhiges Rankenmuster; vergilbt,
            // mit Wasserrändern von oben und Schmutz zum Boden hin
            let streifen = glatt(0.45, 0.5, ((u * 12.0).fract() - 0.5).abs());
            let (su, sv) = ((u * 6.0).fract() - 0.5, (v * 6.0).fract() - 0.5);
            let blatt = (1.0 - glatt(0.06, 0.1, (su.abs() - 0.18 * (sv * 9.0).cos().abs()).abs()))
                * glatt(0.45, 0.2, sv.abs());
            let mut c = mische([0.30, 0.27, 0.19], [0.25, 0.23, 0.16], streifen);
            c = mische(c, [0.22, 0.20, 0.13], blatt * 0.45);
            let flecken = glatt(0.5, 0.85, fbm(u, v, 3, 5, 41));
            let wasser = glatt(0.55, 0.6, fbm(u * 0.5, v * 0.5, 2, 5, 43)) - glatt(0.6, 0.63, fbm(u * 0.5, v * 0.5, 2, 5, 43));
            c = mal(c, 1.0 - 0.18 * flecken - 0.1 * staub);
            c = mische(c, [0.16, 0.12, 0.06], wasser.max(0.0) * 0.45);
            let faser = wert(u * 128.0, v * 128.0, 128, 42);
            c = mal(c, 0.94 + 0.08 * faser);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.15 * faser + 0.2 * blatt, 0.85, 0.0, 0.0])
        }
        Dielen => {
            // Wildeiche-Dielen: acht schmale Bretter je Kachel, versetzte Stöße, gefaste Fugen
            let reihe = (v * 8.0).floor();
            let lokal_v = (v * 8.0).fract();
            let versatz = zufall(reihe as i32, 51);
            let stoss_u = (u + versatz) * 1.5;
            let brett = reihe as i32 * 31 + stoss_u.floor() as i32;
            let (mut c, h, rauh) = eiche(stoss_u.fract(), lokal_v, 500 + brett as u32, 4.0);
            c = mal(c, 0.82 + 0.3 * zufall(brett, 53));
            let stoss = stoss_u.fract();
            let fuge = (1.0 - glatt(0.0, 0.04, lokal_v.min(1.0 - lokal_v))).max(1.0 - glatt(0.0, 0.008, stoss.min(1.0 - stoss)));
            c = mal(c, 1.0 - 0.8 * fuge);
            let abnutzung = glatt(0.55, 0.8, fbm(u, v, 2, 4, 54));
            c = mische(c, [0.20, 0.14, 0.09], abnutzung * 0.25);
            ([c[0], c[1], c[2], 1.0], [h * (1.0 - fuge) , rauh - 0.1 * abnutzung, 0.0, 0.0])
        }
        Teppich => {
            // Orientteppich: Rand mit Borten, Mittelfeld mit Raute, Wollflor
            let (du, dv) = ((u - 0.5).abs(), (v - 0.5).abs());
            let rand = du.max(dv);
            let mut c = [0.11, 0.03, 0.025];
            if rand > 0.40 {
                c = [0.035, 0.03, 0.05];
            }
            if rand > 0.42 && rand < 0.46 {
                c = [0.16, 0.11, 0.05];
            }
            let raute = du + dv;
            if raute < 0.22 {
                c = mische([0.16, 0.11, 0.05], [0.035, 0.03, 0.05], glatt(0.12, 0.13, raute));
            }
            let muster = ((u * 40.0).sin() * (v * 40.0).sin()).abs() * glatt(0.39, 0.2, rand) * glatt(0.22, 0.26, raute);
            c = mische(c, [0.12, 0.02, 0.02], muster * 0.6);
            let flor = wert(u * 300.0, v * 300.0, 300, 61);
            c = mal(c, 0.8 + 0.35 * flor - 0.15 * staub);
            ([c[0], c[1], c[2], 1.0], [flor, 1.0, 0.0, 0.0])
        }
        Tischholz => {
            // Massive Eichenplatte: sechs verleimte Bretter je Kachel, Faser entlang u
            // (Tischlänge), Leimfugen kaum sichtbar
            let brett = (v * 6.0).floor();
            let lb = (v * 6.0).fract();
            let (c, h, rauh) = eiche(u, lb, 700 + brett as u32, 9.0);
            let fuge = 1.0 - glatt(0.0, 0.012, lb.min(1.0 - lb));
            let c = mal(c, 1.0 - 0.35 * fuge);
            ([c[0], c[1], c[2], 1.0], [h - 0.2 * fuge, rauh, 0.0, 0.0])
        }
        Terrakotta => {
            let n = fbm(u, v, 8, 4, 81);
            let c = mal([0.34, 0.12, 0.06], 0.8 + 0.4 * n);
            let kalk = glatt(0.6, 0.8, fbm(u, v * 2.0, 4, 4, 82)) * glatt(0.6, 1.0, v);
            let c = mische(c, [0.40, 0.33, 0.27], kalk * 0.5);
            ([c[0], c[1], c[2], 1.0], [n, 0.85, 0.0, 0.0])
        }
        Erde => {
            let n = fbm(u, v, 16, 4, 91);
            let c = mal([0.035, 0.022, 0.014], 0.6 + 0.8 * n);
            ([c[0], c[1], c[2], 1.0], [n, 1.0, 0.0, 0.0])
        }
        Blatt => {
            // Gummibaum (Ficus elastica): großes, längliches Oval mit kurzer Spitze, ledrig
            // glänzend dunkelgrün, heller Mittelnerv, sehr feine parallele Seitennerven,
            // schmaler heller Rand, einzelne Flecken
            let ellipse = (1.0 - (2.0 * v - 1.0).powi(2)).max(0.0).sqrt();
            let spitze = 1.0 - glatt(0.88, 1.0, v) * 0.85;
            let breite = 0.48 * ellipse * spitze * (1.0 - 0.12 * v);
            let x = u - 0.5;
            let alpha = if x.abs() < breite { 1.0 } else { 0.0 };
            let rand_abst = (breite - x.abs()) / breite.max(1e-3);
            let nerv_breite = 0.012 * (1.0 - 0.8 * v) + 0.002;
            let rippe = 1.0 - glatt(nerv_breite * 0.5, nerv_breite, x.abs());
            let seiten = glatt(0.9, 1.0, ((x.abs() * 1.6 + v) * 28.0).fract()) * (1.0 - rippe) * glatt(0.0, 0.2, rand_abst);
            let farbe_n = fbm(u, v, 4, 4, 101);
            let mut c = mische([0.010, 0.032, 0.013], [0.022, 0.062, 0.026], farbe_n * 0.8 + 0.2 * v);
            c = mische(c, [0.016, 0.045, 0.018], seiten * 0.6);
            c = mische(c, [0.11, 0.12, 0.05], rippe * 0.85);
            c = mische(c, [0.05, 0.06, 0.025], (1.0 - glatt(0.0, 0.05, rand_abst)) * 0.7);
            let fleck = glatt(0.78, 0.82, wert(u * 12.0, v * 12.0, 12, 102)) * glatt(0.1, 0.3, rand_abst);
            c = mische(c, [0.04, 0.05, 0.02], fleck * 0.25);
            ([c[0], c[1], c[2], alpha], [0.5 + 0.3 * rippe - 0.1 * seiten, 0.2 + 0.2 * fleck, 0.0, 0.0])
        }
        Kassette => {
            // Atlas aus acht Rücken von Videohüllen, je 256 × 32 Texel (Zeile = Hülle):
            // erfundene Filmtitel im Stil von Verleihkassetten der 80er, lesbar in Pixelschrift
            let zeile = ((v * 8.0).floor() as usize).min(7);
            let tx = (u * 256.0).floor() as i32;
            let ty = ((v * 8.0).fract() * 32.0).floor() as i32;
            let (c, h) = kassettenruecken(zeile, tx, ty, u, v);
            ([c[0], c[1], c[2], 1.0], [h, 0.35, 0.0, 0.0])
        }
        Keramik => {
            let n = fbm(u, v, 6, 3, 121);
            let c = mal([0.04, 0.13, 0.12], 0.85 + 0.3 * n);
            ([c[0], c[1], c[2], 1.0], [0.5, 0.12, 0.0, 0.0])
        }
        KunststoffGrau => {
            let fein = wert(u * 300.0, v * 300.0, 300, 131);
            let g = 0.30 + 0.03 * fein - 0.05 * staub;
            ([g, g * 0.98, g * 0.94, 1.0], [0.5 + 0.2 * fein, 0.5, 0.0, 0.0])
        }
        Vorhang => {
            let faden = wert(u * 400.0, v * 60.0, 400, 141);
            let c = mal([0.15, 0.022, 0.02], 0.75 + 0.4 * faden);
            ([c[0], c[1], c[2], 1.0], [faden, 0.95, 0.0, 0.0])
        }
        Wandbild => {
            // Eigenes Ölbild: Seelandschaft in der Dämmerung, grobe Pinselstriche
            let strich = fbm(u * 3.0, v * 0.6, 8, 3, 151) * 0.12;
            let hor = 0.55 + strich * 0.5;
            let mut c = if v < hor {
                mische([0.10, 0.12, 0.22], [0.45, 0.25, 0.15], (v / hor).powf(2.0))
            } else {
                mische([0.08, 0.10, 0.12], [0.03, 0.04, 0.05], (v - hor) / (1.0 - hor))
            };
            let berg = 0.5 + 0.08 * (u * 9.0).sin() + 0.04 * (u * 23.0).sin();
            if v > berg && v < hor + 0.02 {
                c = mische([0.06, 0.06, 0.10], [0.10, 0.09, 0.12], fbm(u, v, 8, 2, 152));
            }
            c = mal(c, 0.85 + 0.3 * fbm(u * 2.0, v * 6.0, 8, 3, 153));
            ([c[0], c[1], c[2], 1.0], [0.4 + 2.0 * strich, 0.7, 0.0, 0.0])
        }
        Lack => {
            let g = 0.50 - 0.08 * staub + 0.02 * wert(u * 200.0, v * 200.0, 200, 161);
            ([g, g * 0.97, g * 0.88, 1.0], [0.5, 0.35, 0.0, 0.0])
        }
        Lampenschirm => {
            let faser = wert(u * 300.0, v * 40.0, 300, 171);
            let c = mal([0.55, 0.38, 0.20], 0.85 + 0.2 * faser);
            ([c[0], c[1], c[2], 1.0], [faser, 0.9, 0.0, 0.55])
        }
        Gummi => {
            let g = 0.018 + 0.008 * wert(u * 200.0, v * 200.0, 200, 181);
            ([g, g, g, 1.0], [0.5, 0.65, 0.0, 0.0])
        }
        Himmel => {
            // Nur Grundton: Verlauf, Mond, Sterne und Wolken entstehen im Shader aus der
            // Blickrichtung (unabhängig von der Texturauflösung)
            let c = mische([0.004, 0.006, 0.016], [0.03, 0.03, 0.04], v);
            ([c[0], c[1], c[2], 1.0], [0.5, 0.0, 0.0, 1.0])
        }
        Gluehbirne => ([1.0, 0.75, 0.45, 1.0], [0.5, 0.1, 0.0, 1.0]),
        LedRot => ([1.0, 0.08, 0.03, 1.0], [0.5, 0.2, 0.0, 1.0]),
        LedGruen => ([0.08, 1.0, 0.18, 1.0], [0.5, 0.2, 0.0, 1.0]),
        Decke => {
            let n = fbm(u, v, 8, 4, 201);
            let c = mal([0.26, 0.25, 0.23], 0.9 + 0.2 * n);
            ([c[0], c[1], c[2], 1.0], [n, 0.95, 0.0, 0.0])
        }
        RahmenHolz => {
            let (ring, _) = maserung(u, v, 30.0, 211);
            let c = mische([0.05, 0.03, 0.018], [0.02, 0.012, 0.008], ring);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.2 * ring, 0.4, 0.0, 0.0])
        }
        Zeitschrift => {
            // eigene Titelseite: Farbflächen, Balken als „Schrift“, Bildfeld
            let mut c = [0.6, 0.58, 0.52];
            if v < 0.18 {
                c = [0.55, 0.06, 0.05];
            }
            if (0.25..0.7).contains(&v) && (0.1..0.9).contains(&u) {
                c = mische([0.10, 0.2, 0.35], [0.5, 0.35, 0.15], fbm(u, v, 4, 3, 221));
            }
            if v > 0.75 && ((v * 30.0).fract() < 0.4) && wert(u * 20.0, v * 30.0, 20, 222) > 0.3 {
                c = [0.1, 0.1, 0.12];
            }
            ([c[0], c[1], c[2], 1.0], [0.5, 0.35, 0.0, 0.0])
        }
        Glas => ([0.02, 0.025, 0.025, 1.0], [0.5, 0.05, 0.0, 0.0]),
        Goldrahmen => {
            // alter vergoldeter Rahmen: Perlstab und Kehlen quer zum Profil (v), Patina in
            // den Vertiefungen, abgeriebene Kanten
            let profil = v;
            let perlen = ((u * 60.0).fract() - 0.5).abs() * 2.0;
            let perlstab = if (0.55..0.7).contains(&profil) { 1.0 - perlen } else { 0.0 };
            let kehle = ((profil * 5.0 * std::f32::consts::TAU).sin() * 0.5 + 0.5).powf(3.0);
            let patina = glatt(0.4, 0.8, fbm(u, v, 6, 4, 231)) * (1.0 - kehle);
            let mut c = mische([0.32, 0.22, 0.07], [0.20, 0.13, 0.04], fbm(u * 2.0, v, 8, 3, 232));
            c = mische(c, [0.04, 0.035, 0.02], patina * 0.7);
            c = mal(c, 0.7 + 0.5 * perlstab);
            ([c[0], c[1], c[2], 1.0], [0.3 + 0.5 * kehle + 0.3 * perlstab, 0.4 + 0.35 * patina, 0.6 - 0.45 * patina, 0.0])
        }
        Passepartout => {
            let faser = wert(u * 128.0, v * 128.0, 128, 241);
            let c = mal([0.42, 0.38, 0.30], 0.92 + 0.1 * faser - 0.12 * staub);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.1 * faser, 0.9, 0.0, 0.0])
        }
        Fernbedienung => {
            // Deckplatte der Fernbedienung: satiniertes dunkles Aluminium mit feinem Strich in
            // Längsrichtung, zwei dünne helle Trennlinien zwischen den Tastenfeldern
            let strich = fbm(u * 64.0, v * 0.5, 64, 2, 581);
            let mut c = mal([0.085, 0.085, 0.088], 0.9 + 0.18 * strich);
            for y in [0.27f32, 0.62] {
                let linie = 1.0 - glatt(0.002, 0.006, (v - y).abs());
                c = mische(c, [0.40, 0.40, 0.38], linie * glatt(0.08, 0.12, u) * glatt(0.92, 0.88, u));
            }
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.05 * strich, 0.34, 0.6, 0.0])
        }
        Triebhuelle => {
            let c = mische([0.28, 0.05, 0.04], [0.12, 0.06, 0.03], v + 0.2 * fbm(u, v, 4, 2, 261));
            ([c[0], c[1], c[2], 1.0], [0.5, 0.35, 0.0, 0.0])
        }
        Videorekorder => {
            // Front eines Videorekorders der 80er (ohne Marke), maßstäblich: 256 Texel über
            // 43 cm Breite, die Front nimmt die obersten 54 Texelzeilen ein (9 cm)
            let tx = (u * 256.0).floor() as i32;
            let ty = (v * 256.0).floor() as i32;
            let (c, h, rauh, leuchten) = videorekorder(tx, ty, u, v);
            ([c[0], c[1], c[2], 1.0], [h, rauh, 0.0, leuchten])
        }
        Blende => {
            // Zierblende des Fernsehers: dunkel eloxiertes, gebürstetes Aluminium (Anthrazit),
            // feine Riefen in u-Richtung, leicht metallisch; bewusst dunkel, damit das Gerät im
            // Halbdunkel nicht heller wirkt als sein Bild
            let strich = fbm(u * 0.08, v * 256.0, 2, 3, 851);
            let fein = wert(u * 256.0, v * 256.0, 256, 852);
            let c = mal([0.13, 0.125, 0.118], 0.82 + 0.3 * strich + 0.06 * fein - 0.12 * staub);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.12 * strich, 0.32 + 0.12 * strich + 0.15 * staub, 0.25, 0.0])
        }
        Beschriftung => {
            // Aufdrucke als Abziehbild (Alpha), acht Zeilen zu 256 × 32 Texeln, hell auf der
            // dunklen Blende: Bedienung rechts neben dem Bild und die Senderwahl
            let zeile = ((v * 8.0).floor() as usize).min(7);
            let tx = (u * 256.0).floor() as i32;
            let ty = ((v * 8.0).fract() * 32.0).floor() as i32;
            const WOERTER: [&str; 8] = ["NETZ", "LAUTSTAERKE", "BILDLAGE", "SCHAERFE", "HELLIGKEIT", "FARBTON", "FARBE", "VHF   UHF"];
            let w = WOERTER[zeile];
            let breite = w.chars().count() as i32 * 12 - 2;
            let an = schrift(w, (256 - breite) / 2, 9, 2, false, tx, ty);
            ([0.50, 0.49, 0.46, if an { 1.0 } else { 0.0 }], [0.55, 0.5, 0.0, 0.0])
        }
        Skala => {
            // Feinskala des UHF-Rads (Alpha): 60 Striche ringsum, jeder fünfte länger, hell
            let (dx, dy) = (u - 0.5, v - 0.5);
            let r = (dx * dx + dy * dy).sqrt();
            let winkel = dx.atan2(-dy).to_degrees() + 180.0;
            let k = (winkel / 6.0).round();
            let nah = (winkel - k * 6.0).abs() < 0.9;
            let lang = k as i32 % 5 == 0;
            let an = nah && r < 0.48 && r > if lang { 0.38 } else { 0.43 };
            ([0.48, 0.47, 0.44, if an { 1.0 } else { 0.0 }], [0.55, 0.5, 0.0, 0.0])
        }
        Knopf => {
            // Drehknopf aus satiniertem Kunststoff mit 36 Griffrillen ringsum (u = Umfang)
            let rille = ((u * 36.0).fract() - 0.5).abs() * 2.0;
            let c = mal([0.028, 0.028, 0.03], 0.75 + 0.35 * rille);
            ([c[0], c[1], c[2], 1.0], [0.3 + 0.5 * rille, 0.42, 0.0, 0.0])
        }
        Kanalskala => {
            // Kanalring des VHF-Wählers (Alpha): Kanäle 2 … 13 aufrecht im Kreis, dazu je ein
            // Rastpunkt innen, hell auf dunkel
            let (tx, ty) = ((u * 256.0).floor() as i32, (v * 256.0).floor() as i32);
            let mut an = false;
            for k in 0..12 {
                let w = (k as f32 * 30.0 + 15.0).to_radians();
                let (cx, cy) = (128.0 + 98.0 * w.sin(), 128.0 - 98.0 * w.cos());
                let text = (k + 2).to_string();
                let breite = text.len() as i32 * 18 - 3;
                if schrift(&text, cx as i32 - breite / 2, cy as i32 - 10, 3, false, tx, ty) {
                    an = true;
                }
                let (px, py) = (128.0 + 72.0 * w.sin(), 128.0 - 72.0 * w.cos());
                if (tx as f32 - px).powi(2) + (ty as f32 - py).powi(2) < 16.0 {
                    an = true;
                }
            }
            ([0.50, 0.49, 0.46, if an { 1.0 } else { 0.0 }], [0.55, 0.5, 0.0, 0.0])
        }
        Buecher => {
            // Buchrücken: unterschiedlich breite Bücher in gedeckten Farben, verblichen, mit
            // Querbändern und Titelfeldern (ohne Schrift); oben ungleiche Höhen (Alpha)
            let x = u * 20.0;
            let mut kante = 0.0;
            let mut nr = 0i32;
            let mut a = 0.0;
            while a <= x {
                let w = 0.5 + 1.0 * zufall(nr, 281);
                if a + w > x {
                    kante = ((x - a) / w).min((a + w - x) / w);
                    break;
                }
                a += w;
                nr += 1;
            }
            let hoehe = 0.62 + 0.38 * zufall(nr, 282);
            let farben = [[0.16, 0.05, 0.04], [0.05, 0.08, 0.05], [0.05, 0.06, 0.11], [0.2, 0.15, 0.08], [0.1, 0.09, 0.08], [0.22, 0.2, 0.17], [0.12, 0.03, 0.06]];
            let mut c = farben[(zufall(nr, 283) * 7.0) as usize % 7];
            let y = 1.0 - v;
            let band = glatt(0.0, 0.01, (y - hoehe * 0.85).abs() - 0.015) * glatt(0.0, 0.01, (y - hoehe * 0.15).abs() - 0.01);
            c = mische([0.25, 0.2, 0.08], c, band);
            let titel = (1.0 - glatt(0.3, 0.32, ((y / hoehe) - 0.6).abs())) * glatt(0.15, 0.3, kante);
            c = mische(c, [0.3, 0.28, 0.22], titel * 0.35);
            c = mal(c, (0.75 + 0.5 * glatt(0.0, 0.15, kante)) * (0.85 + 0.3 * fbm(u, v, 8, 3, 284)));
            let alpha = if y < hoehe { 1.0 } else { 0.0 };
            ([c[0], c[1], c[2], alpha], [0.4 + 0.4 * glatt(0.0, 0.2, kante), 0.8, 0.0, 0.0])
        }
        Rinde => {
            let n = fbm(u, v, 8, 3, 271);
            let lentizellen = glatt(0.8, 0.85, wert(u * 30.0, v * 60.0, 30, 272));
            let c = mische(mal([0.05, 0.05, 0.028], 0.8 + 0.4 * n), [0.12, 0.1, 0.06], lentizellen * 0.6);
            ([c[0], c[1], c[2], 1.0], [n, 0.6, 0.0, 0.0])
        }
        Wolken => {
            // Nachtwolken (kachelbar), Farbe und Deckung (Alpha); der Shader schiebt sie langsam
            // über den Himmel und mischt sie selbst ein (keine Alphaschwelle)
            let f = fbm(u, v * 2.0, 4, 5, 401);
            let dicht = glatt(0.45, 0.72, f);
            let rand = glatt(0.55, 0.45, f) * dicht;
            let c = mische([0.020, 0.022, 0.030], [0.075, 0.078, 0.090], 0.3 + 0.7 * rand + 0.2 * fbm(u * 2.0, v * 4.0, 8, 3, 402));
            ([c[0], c[1], c[2], dicht * 0.95], [0.5, 1.0, 0.0, 1.0])
        }
        Parkboden => {
            // Hang des Parks (Weltlage aus u, v wie in `szene::park`): Rasen im Mondlicht, ein
            // geschwungener Kiesweg, Lichtinseln der Laternen, dunkle Büsche
            let (x, z) = crate::szene::park_welt(u, v);
            let rasen = fbm(u * 4.0, v * 4.0, 16, 4, 411);
            let mut c = mal([0.010, 0.017, 0.012], 0.7 + 0.6 * rasen);
            let weg = (x - crate::szene::park_weg(z)).abs();
            c = mische(c, mal([0.040, 0.037, 0.034], 0.8 + 0.4 * wert(u * 128.0, v * 128.0, 128, 412)), glatt(0.5, 0.38, weg));
            let busch = glatt(0.62, 0.68, fbm(u * 2.0, v * 2.0, 8, 4, 413)) * glatt(0.9, 1.4, weg);
            c = mal(c, 1.0 - 0.6 * busch);
            // Straße hinter dem Park (auf ihr fährt ab und zu ein Auto)
            let strasse = 1.0 - glatt(0.35, 0.45, (z - crate::szene::STRASSE_Z).abs());
            c = mische(c, mal([0.016, 0.016, 0.018], 0.85 + 0.3 * rasen), strasse);
            for l in crate::szene::parklaternen() {
                let d2 = (x - l[0]).powi(2) + (z - l[1]).powi(2);
                let insel = (-d2 / 2.2).exp();
                c = [c[0] + 0.11 * insel, c[1] + 0.075 * insel, c[2] + 0.035 * insel];
            }
            ([c[0], c[1], c[2], 1.0], [0.5, 1.0, 0.0, 1.0])
        }
        Baumkulisse => {
            // Vier Bäume nebeneinander (u-Viertel): drei Laubbäume mit runder Krone, eine
            // Tanne. Dunkle Silhouetten, oben ein kühler Saum vom Mond, unten warm von den
            // Laternen. Alpha als weicher Abstand zur Kante (nicht 0/1): die Alphaschwelle zieht
            // dann eine glatte Linie zwischen den Texeln statt Treppenstufen
            let art = ((u * 4.0).floor() as i32).clamp(0, 3);
            let lu = (u * 4.0).fract();
            let saat = 420 + art as u32 * 7;
            let rand = fbm(u * 2.0, v, 16, 4, saat) - 0.5;
            let texel = 1.0 / 64.0;
            let (krone, oben) = if art == 3 {
                let breite = 0.42 * glatt(0.02, 0.85, v) + 0.08 * ((v * 9.0).fract() - 0.5);
                let d = (lu - 0.5).abs() - (breite + 0.05 * rand);
                (glatt(1.5 * texel, -1.5 * texel, d) * glatt(0.87, 0.85, v), glatt(0.0, 0.5, v))
            } else {
                let (cy, ry, rx) = (0.38 + 0.04 * art as f32, 0.34 - 0.03 * art as f32, 0.44 - 0.04 * art as f32);
                let d = (((lu - 0.5) / rx).powi(2) + ((v - cy) / ry).powi(2)).sqrt() - (1.0 + 0.45 * rand);
                (glatt(0.06, -0.06, d), glatt(cy - ry, cy + ry * 0.8, v))
            };
            let stamm = glatt(1.5 * texel, -1.5 * texel, (lu - 0.5).abs() - (0.025 + 0.04 * glatt(0.85, 1.0, v))) * glatt(0.43, 0.47, v);
            let laub = fbm(u * 6.0, v * 6.0, 24, 3, saat + 1);
            let mut c = mal([0.006, 0.009, 0.007], 0.7 + 0.6 * laub);
            c = mische(c, [0.020, 0.028, 0.038], (1.0 - oben) * 0.6 * laub);
            c = mische(c, [0.045, 0.030, 0.014], glatt(0.55, 0.9, oben) * 0.5 * laub);
            c = mische(c, [0.008, 0.006, 0.005], stamm * (1.0 - krone));
            ([c[0], c[1], c[2], krone.max(stamm)], [0.5, 1.0, 0.0, 1.0])
        }
        Ferne => {
            // Jenseits des Parks: Baumreihe, dahinter Häuser mit ein paar erleuchteten Fenstern
            // (eines bläulich: dort läuft auch ein Fernseher), ein Kirchturm
            let dach = 0.42 + 0.12 * ((u * 9.0).floor() * 1.7).sin().abs() + 0.02 * ((u * 9.0).fract() - 0.5).abs();
            let turm = (u - 0.68).abs() < 0.012 && v > 0.12 + (u - 0.68).abs() * 6.0;
            let haus = v > dach || turm;
            let baumkante = v - (0.56 + 0.08 * fbm(u * 3.0, v, 24, 4, 431));
            let mut c = [0.010, 0.011, 0.015];
            let mut a = 0.0;
            if haus {
                a = 1.0;
                let (fx, fy) = ((u * 64.0).floor() as i32, (v * 24.0).floor() as i32);
                let (lx, ly) = ((u * 64.0).fract(), (v * 24.0).fract());
                if !turm && zufall(fx * 131 + fy * 7, 432) > 0.88 && lx > 0.25 && lx < 0.75 && ly > 0.3 && ly < 0.8 && v > dach + 0.04 {
                    c = if zufall(fx + fy * 31, 433) > 0.85 { [0.10, 0.16, 0.30] } else { mal([0.50, 0.33, 0.13], 0.5 + 0.5 * zufall(fx * 3 + fy, 434)) };
                }
            }
            // weiche Kante der Baumreihe (glatte Linie statt Treppen)
            let baum_a = glatt(-0.006, 0.006, baumkante);
            if baum_a > 0.0 {
                c = mische(c, mal([0.005, 0.008, 0.006], 0.7 + 0.6 * fbm(u * 8.0, v * 4.0, 32, 3, 435)), glatt(0.0, 0.006, baumkante));
                a = f32::max(a, baum_a);
            }
            ([c[0], c[1], c[2], a], [0.5, 1.0, 0.0, 1.0])
        }
        Laterne => {
            // Links eine Parklaterne (Mast, Laternenkopf mit leuchtendem Glas), rechts oben eine
            // Parkbank, beides als Silhouette
            let (mut c, mut a) = ([0.008f32, 0.008, 0.009], 0.0f32);
            if u < 0.5 {
                let lu = u * 2.0;
                let mast = (lu - 0.5).abs() < 0.035 + 0.05 * glatt(0.9, 1.0, v) && v > 0.2;
                let glas = (lu - 0.5).abs() < 0.10 + 0.08 * glatt(0.18, 0.06, v) && v > 0.06 && v < 0.19;
                let kappe = (lu - 0.5).abs() < 0.2 * glatt(0.0, 0.06, v) && v < 0.065 && v > 0.02;
                if mast || kappe { a = 1.0; }
                if glas { a = 1.0; c = [0.85, 0.58, 0.28]; }
            } else if v < 0.5 {
                let (lu, lv) = ((u - 0.5) * 2.0, v * 2.0);
                let lehne = lv > 0.30 && lv < 0.50 && ((lv - 0.40).abs() > 0.03);
                let sitz = lv > 0.62 && lv < 0.70;
                let beine = lv > 0.62 && ((lu - 0.12).abs() < 0.03 || (lu - 0.88).abs() < 0.03);
                if (lehne || sitz) && lu > 0.05 && lu < 0.95 || beine { a = 1.0; }
                if sitz { c = [0.03, 0.022, 0.014]; }
            }
            ([c[0], c[1], c[2], a], [0.5, 1.0, 0.0, 1.0])
        }
        Auto => {
            // Ein Auto auf der Straße hinter dem Park, nur die Scheinwerfer und ein Schatten
            let mut a = 0.0;
            let mut c = [0.006f32, 0.006, 0.007];
            let karosserie = v > 0.35 && v < 0.85 && (u - 0.5).abs() < 0.45 - 0.2 * glatt(0.5, 0.35, v);
            if karosserie { a = 1.0; }
            for x in [0.2f32, 0.8] {
                let d = ((u - x).powi(2) + (v - 0.62).powi(2) * 2.0).sqrt();
                if d < 0.07 {
                    a = 1.0;
                    c = [1.0, 0.95, 0.82];
                }
            }
            ([c[0], c[1], c[2], a], [0.5, 1.0, 0.0, 1.0])
        }
        Heizlack => {
            // Heizkörperlack der 60er auf Gusseisen: warmes Altweiß, vergilbt, Läufer vom
            // Streichen, Staub in den Tiefen, an Kanten abgeplatzt bis aufs dunkle Eisen
            let laeufer = fbm(u * 0.5, v * 6.0, 4, 3, 461);
            let staub_l = glatt(0.55, 0.8, fbm(u, v, 6, 4, 462));
            let mut c = mische([0.46, 0.43, 0.36], [0.38, 0.34, 0.25], 0.4 * laeufer + 0.4 * staub_l);
            let abplatzer = glatt(0.83, 0.86, fbm(u * 2.0, v * 2.0, 16, 4, 463));
            c = mische(c, [0.05, 0.045, 0.04], abplatzer);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.2 * laeufer - 0.3 * abplatzer, 0.38 + 0.25 * staub_l + 0.3 * abplatzer, 0.0, 0.0])
        }
        Taste => {
            // Gummitasten einer Fernbedienung: u-Felder grau (Ränder), rot, grün, gelb
            let feld = (u * 8.0).floor() as i32;
            let c = match feld {
                2 | 3 => [0.30, 0.03, 0.02],
                4 => [0.03, 0.20, 0.05],
                5 => [0.40, 0.30, 0.02],
                _ => [0.055, 0.055, 0.06],
            };
            let k = 0.92 + 0.08 * wert(u * 64.0, v * 64.0, 64, 471);
            ([c[0] * k, c[1] * k, c[2] * k, 1.0], [0.5, 0.62, 0.0, 0.0])
        }
        Bildroehre => ([0.0, 0.0, 0.0, 1.0], [0.5, 0.05, 0.0, 0.0]),
    }
}

fn zu_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0 + 0.5) as u8
}

/// Erzeugt alle Ebenen: (Farbe sRGB+Alpha, Eigenschaften) je Material, parallel.
pub fn erzeugen() -> Vec<(Vec<u8>, Vec<u8>)> {
    let materialien: Vec<Material> = ALLE.to_vec();
    std::thread::scope(|s| {
        let griffe: Vec<_> = materialien
            .iter()
            .map(|&m| {
                s.spawn(move || {
                    let n = (GROESSE * GROESSE) as usize;
                    let (mut farbe, mut eig) = (Vec::with_capacity(n * 4), Vec::with_capacity(n * 4));
                    for y in 0..GROESSE {
                        for x in 0..GROESSE {
                            let (u, v) = ((x as f32 + 0.5) / GROESSE as f32, (y as f32 + 0.5) / GROESSE as f32);
                            let (f, e) = texel(m, u, v);
                            farbe.extend_from_slice(&[zu_srgb(f[0]), zu_srgb(f[1]), zu_srgb(f[2]), (f[3].clamp(0.0, 1.0) * 255.0) as u8]);
                            eig.extend(e.iter().map(|x| (x.clamp(0.0, 1.0) * 255.0) as u8));
                        }
                    }
                    (farbe, eig)
                })
            })
            .collect();
        griffe.into_iter().map(|g| g.join().expect("Texturthread")).collect()
    })
}
