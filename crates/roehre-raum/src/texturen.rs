//! Selbst erzeugte Materialien des Raums (keine Fotos, keine fremden Texturen).
//!
//! Jedes Material ist eine Ebene zweier Texturfelder (512 × 512):
//! * **Farbe** (sRGB) und Deckung (Alpha, für Blätter),
//! * **Eigenschaften**: R = Relief (Höhe), G = Rauheit, B = Metall, A = Selbstleuchten.
//!
//! Der Stil zielt auf die späte PS2-Zeit: wenige Polygone, aber dichte, handgemachte
//! Materialien mit Alterung (Flecken, Abnutzung, Staub).

use crate::rauschen::{fbm, wert, zufall};

pub const GROESSE: u32 = 512;

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
}

pub const ANZAHL: u32 = Material::Bildroehre as u32 + 1;

/// Alle Materialien in Ebenen-Reihenfolge.
pub const ALLE: [Material; ANZAHL as usize] = {
    use Material::*;
    [
        Furnier, KunststoffSchwarz, Chrom, Lautsprecherstoff, Tapete, Dielen, Teppich, Tischholz, Terrakotta, Erde,
        Blatt, Kassette, Keramik, KunststoffGrau, Vorhang, Wandbild, Lack, Lampenschirm, Gummi, Nacht, Gluehbirne,
        LedRot, Decke, RahmenHolz, Zeitschrift, Glas, Bildroehre,
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
            // 80er-Tapete: blasses Ocker-Grün, senkrechte Streifen, ein ruhiges Rankenmuster,
            // vergilbt und fleckig
            let streifen = glatt(0.45, 0.5, ((u * 12.0).fract() - 0.5).abs());
            let (su, sv) = ((u * 6.0).fract() - 0.5, (v * 6.0).fract() - 0.5);
            let blatt = (1.0 - glatt(0.06, 0.1, (su.abs() - 0.18 * (sv * 9.0).cos().abs()).abs()))
                * glatt(0.45, 0.2, sv.abs());
            let mut c = mische([0.34, 0.31, 0.22], [0.28, 0.27, 0.19], streifen);
            c = mische(c, [0.27, 0.25, 0.17], blatt * 0.45);
            let flecken = glatt(0.55, 0.8, fbm(u, v, 3, 5, 41));
            c = mal(c, 1.0 - 0.25 * flecken - 0.08 * staub);
            let faser = wert(u * 256.0, v * 256.0, 256, 42);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.15 * faser + 0.2 * blatt, 0.85, 0.0, 0.0])
        }
        Dielen => {
            // Vier Dielen je Kachel, jede mit eigenem Ton, Fugen, Maserung, Abnutzung
            let reihe = (v * 8.0).floor();
            let lokal_v = (v * 8.0).fract();
            let versatz = zufall(reihe as i32, 51);
            let stoss = ((u + versatz) * 2.0).fract();
            let (ring, poren) = maserung(u * 2.0 + versatz, lokal_v * 0.125 + reihe * 0.125, 40.0, 52 + reihe as u32);
            let ton = 0.75 + 0.4 * zufall(reihe as i32 * 13 + (u + versatz).floor() as i32, 53);
            let mut c = mal(mische([0.14, 0.085, 0.045], [0.10, 0.06, 0.03], 0.3 + 0.4 * ring), ton);
            let fuge = (1.0 - glatt(0.0, 0.025, lokal_v.min(1.0 - lokal_v))).max(1.0 - glatt(0.0, 0.006, stoss.min(1.0 - stoss)));
            c = mal(c, 1.0 - 0.75 * fuge);
            let abnutzung = glatt(0.5, 0.75, fbm(u, v, 2, 4, 54));
            c = mische(c, [0.20, 0.13, 0.08], abnutzung * 0.3);
            ([c[0], c[1], c[2], 1.0], [0.6 - 0.5 * fuge + 0.1 * ring - 0.1 * poren, 0.55 + 0.25 * abnutzung, 0.0, 0.0])
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
            let (ring, poren) = maserung(u, v, 26.0, 71);
            let c = mal(mische([0.22, 0.14, 0.075], [0.15, 0.09, 0.045], 0.3 + 0.5 * ring), 0.9 + 0.2 * fbm(u, v, 4, 3, 72) - 0.1 * poren);
            let kratzer = glatt(0.985, 1.0, wert(u * 3.0, v * 120.0, 3, 73));
            let c = mische(c, [0.16, 0.10, 0.06], kratzer * 0.5);
            ([c[0], c[1], c[2], 1.0], [0.5 + 0.2 * ring - 0.2 * poren - 0.3 * kratzer, 0.35 + 0.2 * kratzer, 0.0, 0.0])
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
            // Gummibaumblatt: länglich, spitz, dunkelgrün glänzend, helle Mittelrippe
            let breite = (std::f32::consts::PI * v).sin().powf(0.7) * 0.47 * (1.0 - 0.3 * v);
            let x = u - 0.5;
            let alpha = if x.abs() < breite { 1.0 } else { 0.0 };
            let rippe = 1.0 - glatt(0.004, 0.012, x.abs());
            let seiten = ((x.abs() * 30.0 - v * 18.0) * std::f32::consts::TAU / 6.0).cos().abs().powf(12.0) * (1.0 - rippe);
            let rand = glatt(0.85, 1.0, x.abs() / breite.max(1e-3));
            let mut c = mische([0.012, 0.045, 0.018], [0.03, 0.08, 0.035], fbm(u, v, 4, 3, 101));
            c = mische(c, [0.10, 0.13, 0.06], rippe * 0.8 + seiten * 0.15);
            c = mische(c, [0.04, 0.04, 0.012], rand * 0.6);
            ([c[0], c[1], c[2], alpha], [0.5 + 0.3 * rippe, 0.22, 0.0, 0.0])
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
            // Blick aus dem Fenster: Nachthimmel, Häuser mit einzelnen Lichtern, Laterne, Dunst
            let mut c = mische([0.003, 0.005, 0.012], [0.02, 0.018, 0.022], glatt(0.0, 0.7, v));
            let dach = 0.55 + 0.08 * ((u * 6.0).floor() * 1.7).sin().abs();
            if v > dach {
                c = [0.004, 0.004, 0.006];
                let (fx, fy) = ((u * 40.0).floor() as i32, ((v - dach) * 40.0).floor() as i32);
                if zufall(fx * 97 + fy, 191) > 0.88 && ((u * 40.0).fract() - 0.5).abs() < 0.3 && (((v - dach) * 40.0).fract() - 0.5).abs() < 0.3 {
                    c = mal([1.0, 0.7, 0.35], 0.4 + 0.6 * zufall(fx + fy * 31, 192));
                }
            }
            let laterne = (-((u - 0.72).powi(2) + (v - 0.62).powi(2)) * 900.0).exp();
            let hof = (-((u - 0.72).powi(2) + (v - 0.62).powi(2)) * 40.0).exp() * 0.08;
            c = [c[0] + laterne * 1.0 + hof, c[1] + laterne * 0.6 + hof * 0.6, c[2] + laterne * 0.25 + hof * 0.3];
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
