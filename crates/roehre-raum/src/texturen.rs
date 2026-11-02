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
    Nacht,
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
}

pub const ANZAHL: u32 = Material::Videorekorder as u32 + 1;

/// Alle Materialien in Ebenen-Reihenfolge.
pub const ALLE: [Material; ANZAHL as usize] = {
    use Material::*;
    [
        Furnier, KunststoffSchwarz, Chrom, Lautsprecherstoff, Tapete, Dielen, Teppich, Tischholz, Terrakotta, Erde,
        Blatt, Kassette, Keramik, KunststoffGrau, Vorhang, Wandbild, Lack, Lampenschirm, Gummi, Nacht, Gluehbirne,
        LedRot, Decke, RahmenHolz, Zeitschrift, Glas, Bildroehre, Goldrahmen, Passepartout, Fernbedienung,
        Triebhuelle, Rinde, Buecher, Videorekorder,
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
    let hell = [0.235, 0.135, 0.064];
    let dunkel = [0.085, 0.045, 0.020];
    let mut c = mische(hell, dunkel, 0.10 + 0.72 * spaet);
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
            let (c, h, rauh) = eiche(u, lb, 700 + brett as u32, 5.0);
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
        Nacht => {
            // Neblige Nacht: Dunst, der zur Laterne hin orange aufleuchtet, schemenhafte Häuser
            // und ein kahler Baum, wenige verschwommene Fenster, Schlieren auf der Scheibe
            let laterne_d = ((u - 0.68).powi(2) + (v - 0.48).powi(2) * 1.4).sqrt();
            let dunst = [0.012, 0.013, 0.015];
            let hof = (-laterne_d * 7.0).exp() * 0.12;
            let mut c = [dunst[0] + hof, dunst[1] + hof * 0.6, dunst[2] + hof * 0.25];
            // Häuser: Silhouette mit unruhiger Oberkante, nur wenig dunkler als der Nebel
            let dach = 0.52 + 0.1 * ((u * 5.0).floor() * 2.3).sin().abs() + 0.02 * fbm(u * 4.0, v, 8, 3, 193);
            if v > dach {
                c = mal(c, 0.55);
                let (fx, fy) = ((u * 24.0).floor() as i32, ((v - dach) * 24.0).floor() as i32);
                if zufall(fx * 97 + fy, 191) > 0.95 {
                    let (lx, ly) = (((u * 24.0).fract() - 0.5).abs(), (((v - dach) * 24.0).fract() - 0.5).abs());
                    let fenster = (1.0 - glatt(0.1, 0.45, lx.max(ly))) * 0.12 * (0.5 + zufall(fx * 31 + fy, 196));
                    c = [c[0] + fenster * 0.5, c[1] + fenster * 0.33, c[2] + fenster * 0.15];
                }
            }
            // Kahler Baum links: Äste aus Rauschen
            let baum = glatt(0.58, 0.62, fbm(u * 3.0, v * 2.0, 6, 5, 194)) * glatt(0.45, 0.2, u) * glatt(0.1, 0.4, v);
            c = mal(c, 1.0 - 0.6 * baum);
            // Laterne: kleiner Kopf, Lichtkegel im Nebel
            let kopf = (-laterne_d * 120.0).exp();
            let kegel = glatt(0.08, 0.0, (u - 0.68).abs() - (v - 0.48).max(0.0) * 0.25) * glatt(0.48, 0.6, v) * (1.0 - glatt(0.6, 1.0, v)) * 0.04;
            c = [c[0] + kopf * 0.9 + kegel, c[1] + kopf * 0.55 + kegel * 0.6, c[2] + kopf * 0.2 + kegel * 0.2];
            // Schlieren und Tropfen auf der Scheibe
            let schliere = glatt(0.7, 0.9, wert(u * 40.0, v * 3.0, 40, 195)) * 0.25;
            c = mal(c, 1.0 + schliere);
            ([c[0], c[1], c[2], 1.0], [0.5, 0.0, 0.0, 1.0])
        }
        Gluehbirne => ([1.0, 0.75, 0.45, 1.0], [0.5, 0.1, 0.0, 1.0]),
        LedRot => ([1.0, 0.08, 0.03, 1.0], [0.5, 0.2, 0.0, 1.0]),
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
            // Oberseite einer schlanken Fernbedienung: Netztaste, Zifferntasten, Farbtasten,
            // Programm- und Lautstärkewippen; Aufdrucke als kurze Balken (keine Schrift)
            let mut c = [0.022, 0.022, 0.024];
            let mut h = 0.4;
            let mut rauh = 0.35;
            let taste = |cx: f32, cy: f32, hx: f32, hy: f32| -> f32 {
                let d = ((u - cx).abs() - hx).max((v - cy).abs() - hy);
                1.0 - glatt(-0.004, 0.004, d)
            };
            let mut tasten = 0.0f32;
            let netz = taste(0.75, 0.08, 0.09, 0.025);
            c = mische(c, [0.35, 0.03, 0.02], netz);
            for i in 0..12 {
                let (cx, cy) = (0.25 + (i % 3) as f32 * 0.25, 0.2 + (i / 3) as f32 * 0.075);
                tasten = tasten.max(taste(cx, cy, 0.09, 0.025));
            }
            for (i, f) in [[0.35, 0.03, 0.02], [0.03, 0.25, 0.04], [0.35, 0.3, 0.02], [0.03, 0.06, 0.35]].iter().enumerate() {
                let t = taste(0.14 + i as f32 * 0.24, 0.52, 0.09, 0.018);
                c = mische(c, *f, t);
                tasten = tasten.max(t * 0.5);
            }
            for cx in [0.3f32, 0.7] {
                tasten = tasten.max(taste(cx, 0.7, 0.12, 0.08));
            }
            c = mische(c, [0.08, 0.08, 0.085], tasten);
            let aufdruck = glatt(0.6, 0.62, wert(u * 30.0, v * 50.0, 30, 251)) * (1.0 - tasten) * glatt(0.1, 0.15, v);
            c = mische(c, [0.4, 0.4, 0.4], aufdruck * 0.5);
            h += 0.4 * tasten + 0.4 * netz;
            rauh += 0.3 * tasten;
            ([c[0], c[1], c[2], 1.0], [h, rauh, 0.0, 0.0])
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
