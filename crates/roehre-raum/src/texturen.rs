//! Selbst erzeugte Materialien des Raums (keine Fotos, keine fremden Texturen).
//!
//! Jedes Material ist eine Ebene zweier Texturfelder (512 × 512):
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
    KonsoleHell,
    Lila,
    Steuergeraet,
}

pub const ANZAHL: u32 = Material::Steuergeraet as u32 + 1;

/// Alle Materialien in Ebenen-Reihenfolge.
pub const ALLE: [Material; ANZAHL as usize] = {
    use Material::*;
    [
        Furnier, KunststoffSchwarz, Chrom, Lautsprecherstoff, Tapete, Dielen, Teppich, Tischholz, Terrakotta, Erde,
        Blatt, Kassette, Keramik, KunststoffGrau, Vorhang, Wandbild, Lack, Lampenschirm, Gummi, Nacht, Gluehbirne,
        LedRot, Decke, RahmenHolz, Zeitschrift, Glas, Bildroehre, Goldrahmen, Passepartout, Fernbedienung,
        Triebhuelle, Rinde, Buecher, KonsoleHell, Lila, Steuergeraet,
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

/// Wildeiche (geölt): Maserung entlang u, Äste mit Ringen, Trockenrisse, Farbspiel zwischen
/// hellem Honig und grauem Braun. `saat` wählt das Brett. Liefert Farbe (linear), Höhe,
/// Rauheit.
fn wildeiche(u: f32, v: f32, saat: u32, dichte: f32) -> ([f32; 3], f32, f32) {
    // Äste: bis zu zwei je Brett, verzerren die Maserung ringförmig
    let mut ast_verzug = 0.0;
    let mut ast_kern: f32 = 0.0;
    let mut ast_ring = 0.0;
    for k in 0..2 {
        if zufall(saat as i32 * 7 + k, 301) < 0.55 {
            continue;
        }
        let (cx, cy) = (zufall(saat as i32 * 13 + k, 302), zufall(saat as i32 * 17 + k, 303));
        let (dx, dy) = ((u - cx) * 2.5, (v - cy) * 9.0);
        let d = (dx * dx + dy * dy).sqrt();
        let r = 0.035 + 0.03 * zufall(saat as i32 * 19 + k, 304);
        ast_verzug += 0.6 * (-d / (r * 3.0)).exp() * (dy * 20.0).signum() * 0.0 + 1.5 * (-d / (r * 2.5)).exp();
        ast_kern = ast_kern.max(1.0 - glatt(r * 0.6, r, d));
        ast_ring += ((d * 160.0).sin() * 0.5 + 0.5) * (1.0 - glatt(r, r * 3.0, d));
    }
    let verzug = fbm(u * 0.5, v, 2, 4, saat) * 3.0 + fbm(u, v * 2.0, 4, 3, saat + 1) * 0.8 + ast_verzug;
    let ring = ((v * dichte + verzug) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let fein = wert(u * 160.0, v * 60.0, 160, saat + 2);
    let poren = glatt(0.7, 0.95, wert(u * 220.0, v * 24.0, 220, saat + 3));
    // Trockenrisse: schmale dunkle Linien entlang der Maserung
    let riss = glatt(0.985, 0.998, wert(u * 2.0, (v + verzug * 0.02) * 90.0, 2, saat + 4)) * glatt(0.4, 0.8, fbm(u, v, 3, 3, saat + 5));
    let farbspiel = fbm(u * 0.7, v * 0.7, 2, 3, saat + 6);
    let hell = mische([0.40, 0.25, 0.12], [0.30, 0.22, 0.14], farbspiel);       // Honig ↔ graues Braun
    let dunkel = [0.17, 0.10, 0.05];
    let mut c = mische(hell, dunkel, 0.18 + 0.45 * ring.powf(1.6));
    c = mal(c, 0.88 + 0.18 * fein - 0.22 * poren);
    c = mische(c, [0.11, 0.06, 0.03], ast_ring * 0.55);
    c = mische(c, [0.05, 0.028, 0.014], ast_kern);
    c = mische(c, [0.03, 0.018, 0.01], riss);
    let hoehe = 0.55 + 0.15 * ring - 0.25 * poren - 0.5 * riss - 0.2 * ast_kern;
    (c, hoehe, 0.62 + 0.15 * poren)
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
            let (mut c, h, rauh) = wildeiche(stoss_u.fract(), lokal_v * 0.125 + reihe * 0.125, 500 + brett as u32, 26.0);
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
            // Tischplatte aus verleimten Wildeiche-Brettern (vier je Kachel)
            let brett = (u * 4.0).floor();
            let (c, h, rauh) = wildeiche(v, (u * 4.0).fract() * 0.25 + brett * 0.25, 700 + brett as u32, 18.0);
            let fuge = 1.0 - glatt(0.0, 0.01, (u * 4.0).fract().min(1.0 - (u * 4.0).fract()));
            let c = mal(c, (0.9 + 0.2 * zufall(brett as i32, 71)) * (1.0 - 0.5 * fuge));
            ([c[0], c[1], c[2], 1.0], [h, rauh, 0.0, 0.0])
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
            // Atlas aus vier Videokassetten-Rücken: schwarzes Gehäuse, Etikett mit
            // Farbstreifen und Kritzelschrift (kein Markenname)
            let welche = (u * 4.0).floor() as i32;
            let lu = (u * 4.0).fract();
            let mut c = [0.018, 0.018, 0.02];
            let mut h = 0.5;
            if (0.12..0.88).contains(&lu) && (0.25..0.75).contains(&v) {
                let farben = [[0.45, 0.05, 0.04], [0.04, 0.12, 0.35], [0.5, 0.4, 0.05], [0.05, 0.25, 0.08]];
                c = [0.62, 0.6, 0.55];
                if v < 0.36 {
                    c = farben[welche.rem_euclid(4) as usize];
                }
                let schrift = glatt(0.62, 0.66, wert(lu * 60.0, v * 18.0, 60, 111 + welche as u32))
                    * glatt(0.5, 0.48, (v - 0.55).abs() * 4.0);
                c = mische(c, [0.04, 0.04, 0.12], schrift);
                h = 0.6;
            }
            ([c[0], c[1], c[2], 1.0], [h, 0.45, 0.0, 0.0])
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
        KonsoleHell => {
            // hellgrauer, leicht vergilbter Kunststoff der 90er: feine Narbung, Gebrauchsspuren
            let fein = wert(u * 256.0, v * 256.0, 256, 291);
            let vergilbt = fbm(u, v, 3, 3, 292);
            let c = mische([0.50, 0.50, 0.48], [0.50, 0.47, 0.40], vergilbt * 0.6);
            let c = mal(c, 0.95 + 0.06 * fein - 0.1 * staub);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.2 * fein, 0.55, 0.0, 0.0])
        }
        Lila => {
            // die blasslila Schieber und Tasten
            let fein = wert(u * 256.0, v * 256.0, 256, 293);
            let c = mal([0.26, 0.22, 0.40], 0.95 + 0.08 * fein);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.2 * fein, 0.45, 0.0, 0.0])
        }
        Steuergeraet => {
            // Oberseite eines Controllers im Stil der 90er (Hundeknochenform), alles als Textur:
            // Steuerkreuz links, Select/Start schräg in der Mitte, vier farbige Tasten rechts in
            // einer lila Mulde. Maße in Metern (147 × 63 mm), u quer, v in die Tiefe.
            let x = (u - 0.5) * 0.147;
            let y = (v - 0.5) * 0.063;
            let kreis = |cx: f32, cy: f32, r: f32| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r;
            let koerper = kreis(-0.042, 0.0, 0.0315).min(kreis(0.042, 0.0, 0.0315)).min(
                // Steg in der Mitte mit leicht eingezogener Taille
                (x.abs() - 0.045).max(y.abs() - (0.025 + 0.004 * (x / 0.045).powi(2))),
            );
            let alpha = if koerper < 0.0 { 1.0 } else { 0.0 };
            let mut c = mal([0.50, 0.50, 0.48], 0.95 + 0.05 * wert(u * 256.0, v * 256.0, 256, 294) - 0.08 * staub);
            let mut h = 0.5;
            // Rand: leicht abgerundet (dunkler zur Kante)
            c = mal(c, 0.8 + 0.2 * glatt(0.0, 0.004, -koerper));
            // Steuerkreuz in flacher Mulde
            let mulde = 1.0 - glatt(-0.0005, 0.0005, kreis(-0.042, 0.0, 0.016));
            c = mische(c, mal(c, 0.88), mulde);
            let (dx, dy) = (x + 0.042, y);
            let kreuz = ((dx.abs() - 0.0115).max(dy.abs() - 0.0042)).min((dy.abs() - 0.0115).max(dx.abs() - 0.0042));
            let kreuz_an = 1.0 - glatt(-0.0003, 0.0003, kreuz);
            c = mische(c, [0.035, 0.035, 0.038], kreuz_an);
            h += 0.3 * kreuz_an - 0.1 * mulde;
            // Select und Start: zwei schräge Gummiknöpfe
            for sx in [-0.009f32, 0.009] {
                let (px, py) = (x - sx, y - 0.006);
                let (a, b) = ((px + py) * std::f32::consts::FRAC_1_SQRT_2, (py - px) * std::f32::consts::FRAC_1_SQRT_2);
                let pille = ((a.abs() - 0.0045).max(0.0).powi(2) + b.powi(2)).sqrt() - 0.0016;
                let an = 1.0 - glatt(-0.0002, 0.0002, pille);
                c = mische(c, [0.20, 0.20, 0.21], an);
                h += 0.2 * an;
            }
            // Tastenmulde rechts (lila) und vier farbige Tasten: oben blau, links grün, rechts
            // rot, unten gelb
            let lila = 1.0 - glatt(-0.0004, 0.0004, kreis(0.042, 0.0, 0.0215));
            c = mische(c, [0.24, 0.20, 0.36], lila);
            for (bx, by, farbe) in [
                (0.0f32, -0.0115f32, [0.06f32, 0.10, 0.45]),
                (-0.0115, 0.0, [0.06, 0.28, 0.08]),
                (0.0115, 0.0, [0.55, 0.04, 0.04]),
                (0.0, 0.0115, [0.62, 0.48, 0.03]),
            ] {
                let d = kreis(0.042 + bx, by, 0.0052);
                let an = 1.0 - glatt(-0.0003, 0.0003, d);
                let wulst = (1.0 - (d + 0.0052).max(0.0) / 0.0052).max(0.0).sqrt();
                c = mische(c, mal(farbe, 0.8 + 0.4 * wulst), an);
                h += 0.4 * an * wulst;
            }
            ([c[0], c[1], c[2], alpha], [h, 0.4, 0.0, 0.0])
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
