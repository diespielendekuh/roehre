//! # roehre-testbilder
//!
//! Prozedural erzeugte Testbilder für die Abstimmung und die Prüfläufe von `roehre`.
//! Alles hier ist selbst erzeugt: keine Spielgrafik, keine fremden Bilder, keine fremde
//! Schrift. Die Bilder decken ab, woran man eine Röhrensimulation beurteilt:
//!
//! | Bild | prüft |
//! |---|---|
//! | [`farbbalken`] | Farben, Farbraum, Farbsäume an harten Farbwechseln |
//! | [`graukeil`] | Gamma, Helligkeitsausgleich, Schwarzwert |
//! | [`zonenplatte`] | Moiré und Abtastung bei allen Ortsfrequenzen |
//! | [`kreuzgitter`] | Konvergenz, Geometrie, Wölbung |
//! | [`schriftbild`] | Lesbarkeit kleiner Schrift |
//! | [`pixelszene`] | Gesamteindruck einer 2D-Spielszene (SNES-artig, 256×224, Pixel 8:7) |
//! | [`szene_3d`] | Gesamteindruck einer hochaufgelösten 3D-Szene (virtuelle Zeilenzahl) |

mod schrift;

pub use schrift::{breite as textbreite, text};

/// Ein RGBA-Bild mit Pixel-Seitenverhältnis.
#[derive(Clone, Debug)]
pub struct Bild {
    pub breite: u32,
    pub hoehe: u32,
    /// RGBA, 8 Bit je Kanal, sRGB-kodiert, zeilenweise von oben.
    pub rgba: Vec<u8>,
    /// Breite eines Pixels geteilt durch seine Höhe (SNES: 8/7).
    pub pixel_seitenverhaeltnis: f64,
}

impl Bild {
    pub fn neu(breite: u32, hoehe: u32, farbe: [u8; 3]) -> Self {
        let mut rgba = Vec::with_capacity((breite * hoehe * 4) as usize);
        for _ in 0..breite * hoehe {
            rgba.extend_from_slice(&[farbe[0], farbe[1], farbe[2], 255]);
        }
        Self { breite, hoehe, rgba, pixel_seitenverhaeltnis: 1.0 }
    }

    pub fn setze(&mut self, x: i32, y: i32, farbe: [u8; 3]) {
        if x >= 0 && y >= 0 && (x as u32) < self.breite && (y as u32) < self.hoehe {
            let i = ((y as u32 * self.breite + x as u32) * 4) as usize;
            self.rgba[i..i + 3].copy_from_slice(&farbe);
        }
    }

    pub fn hole(&self, x: u32, y: u32) -> [u8; 3] {
        let i = ((y * self.breite + x) * 4) as usize;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]]
    }

    pub fn rechteck(&mut self, x: i32, y: i32, b: i32, h: i32, farbe: [u8; 3]) {
        for yy in y..y + h {
            for xx in x..x + b {
                self.setze(xx, yy, farbe);
            }
        }
    }

    /// Speichert als PNG.
    pub fn speichere(&self, pfad: impl AsRef<std::path::Path>) -> image::ImageResult<()> {
        image::save_buffer(pfad, &self.rgba, self.breite, self.hoehe, image::ExtendedColorType::Rgba8)
    }

    /// Lädt ein PNG (Pixel-Seitenverhältnis 1).
    pub fn lade(pfad: impl AsRef<std::path::Path>) -> image::ImageResult<Self> {
        let b = image::open(pfad)?.to_rgba8();
        Ok(Self { breite: b.width(), hoehe: b.height(), rgba: b.into_raw(), pixel_seitenverhaeltnis: 1.0 })
    }
}

/// Alle Testbilder mit Namen, in den für die Prüfläufe üblichen Größen.
pub fn alle() -> Vec<(&'static str, Bild)> {
    vec![
        ("pixelszene", pixelszene()),
        ("farbbalken", farbbalken(320, 240)),
        ("graukeil", graukeil(320, 240)),
        ("zonenplatte", zonenplatte(320, 240)),
        ("kreuzgitter", kreuzgitter(320, 240)),
        ("schrift", schriftbild()),
        ("szene-3d", szene_3d(1920, 1080)),
    ]
}

fn mische(a: [u8; 3], b: [u8; 3], t: f64) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2].map(|i| (a[i] as f64 + (b[i] as f64 - a[i] as f64) * t).round() as u8)
}

/// Farbbalken: oben 75-%-Balken, darunter umgekehrte Balken, unten Graustufen und ein
/// Schwarzfeld mit „unter Schwarz“-Stufen (PLUGE-artig).
pub fn farbbalken(breite: u32, hoehe: u32) -> Bild {
    let mut bild = Bild::neu(breite, hoehe, [0, 0, 0]);
    let s = 191u8; // 75 %
    let balken = [[s, s, s], [s, s, 0], [0, s, s], [0, s, 0], [s, 0, s], [s, 0, 0], [0, 0, s]];
    let w = breite as f64 / 7.0;
    let h1 = (hoehe as f64 * 0.62) as i32;
    let h2 = (hoehe as f64 * 0.72) as i32;
    for (i, f) in balken.iter().enumerate() {
        let x0 = (i as f64 * w).round() as i32;
        let x1 = ((i + 1) as f64 * w).round() as i32;
        bild.rechteck(x0, 0, x1 - x0, h1, *f);
        let umgekehrt = if i % 2 == 0 { balken[6 - i] } else { [0, 0, 0] };
        bild.rechteck(x0, h1, x1 - x0, h2 - h1, umgekehrt);
    }
    // Unten: 100-%-Weiß, Grautreppe, PLUGE (knapp über Schwarz)
    let unten = hoehe as i32 - h2;
    let feld = breite as i32 / 8;
    bild.rechteck(0, h2, feld, unten, [255, 255, 255]);
    for i in 0..5 {
        let g = (255.0 * (4 - i) as f64 / 5.0) as u8;
        bild.rechteck(feld * (1 + i), h2, feld, unten, [g, g, g]);
    }
    bild.rechteck(feld * 6, h2, feld / 3, unten, [4, 4, 4]);
    bild.rechteck(feld * 6 + feld / 3, h2, feld / 3, unten, [10, 10, 10]);
    bild.rechteck(feld * 6 + 2 * feld / 3, h2, breite as i32, unten, [16, 16, 16]);
    bild
}

/// Graukeil: oben 16 Stufen, Mitte stufenloser Verlauf, unten Verläufe in Rot, Grün, Blau.
pub fn graukeil(breite: u32, hoehe: u32) -> Bild {
    let mut bild = Bild::neu(breite, hoehe, [0, 0, 0]);
    let streifen = hoehe as i32 / 5;
    for x in 0..breite as i32 {
        let t = x as f64 / (breite - 1) as f64;
        let stufe = ((x as f64 / breite as f64 * 16.0).floor() * 17.0) as u8;
        let glatt = (t * 255.0).round() as u8;
        for y in 0..streifen * 2 {
            bild.setze(x, y, [stufe; 3]);
        }
        for y in streifen * 2..streifen * 3 {
            bild.setze(x, y, [glatt; 3]);
        }
        for y in streifen * 3..streifen * 4 {
            let f = if y < streifen * 3 + streifen / 2 { [glatt, 0, 0] } else { [0, glatt, 0] };
            bild.setze(x, y, f);
        }
        for y in streifen * 4..hoehe as i32 {
            bild.setze(x, y, [0, 0, glatt]);
        }
    }
    bild
}

/// Zonenplatte: konzentrische Ringe, deren Ortsfrequenz zum Rand bis zur Nyquist-Grenze
/// des Eingangsbildes steigt. Jede Schwebung, die außerhalb der Mitte neu entsteht, ist
/// Moiré der Simulation.
pub fn zonenplatte(breite: u32, hoehe: u32) -> Bild {
    let mut bild = Bild::neu(breite, hoehe, [0, 0, 0]);
    let (cx, cy) = (breite as f64 / 2.0, hoehe as f64 / 2.0);
    let k = std::f64::consts::PI / breite as f64;
    for y in 0..hoehe {
        for x in 0..breite {
            let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
            let v = 0.5 + 0.5 * (k * (dx * dx + dy * dy)).cos();
            let g = (v * 255.0).round() as u8;
            bild.setze(x as i32, y as i32, [g, g, g]);
        }
    }
    bild
}

/// Kreuzgitter: weiße Linien alle 16 Pixel, Punkte in den Feldmitten, Kreis in der Mitte.
pub fn kreuzgitter(breite: u32, hoehe: u32) -> Bild {
    let mut bild = Bild::neu(breite, hoehe, [0, 0, 0]);
    let w = [235, 235, 235];
    for y in 0..hoehe as i32 {
        for x in 0..breite as i32 {
            if x % 16 == 0 || y % 16 == 0 || x == breite as i32 - 1 || y == hoehe as i32 - 1 {
                bild.setze(x, y, w);
            } else if x % 16 == 8 && y % 16 == 8 {
                bild.setze(x, y, w);
            }
        }
    }
    let (cx, cy, r) = (breite as f64 / 2.0, hoehe as f64 / 2.0, hoehe as f64 * 0.4);
    for i in 0..2000 {
        let a = i as f64 / 2000.0 * std::f64::consts::TAU;
        bild.setze((cx + r * a.cos()) as i32, (cy + r * a.sin()) as i32, w);
    }
    bild
}

/// Schrift in drei Größen auf dunkelblauem Grund (320×240).
pub fn schriftbild() -> Bild {
    let mut bild = Bild::neu(320, 240, [16, 24, 88]);
    let weiss = [240, 240, 240];
    text(&mut bild, 8, 8, 1, weiss, "ROEHRE SCHRIFTPROBE 5×7 PIXEL");
    text(&mut bild, 8, 20, 1, [255, 220, 64], "DER SCHNELLE BRAUNE FUCHS SPRINGT");
    text(&mut bild, 8, 30, 1, [120, 220, 255], "ÜBER DEN FAULEN HUND. 0123456789");
    text(&mut bild, 8, 48, 2, weiss, "GRÖSSE 2: ÄÖÜ 42");
    text(&mut bild, 8, 72, 3, [255, 96, 96], "GRÖSSE 3");
    // Rote und blaue Schrift auf Schwarz: zeigt Farbsäume besonders deutlich.
    bild.rechteck(0, 110, 320, 60, [0, 0, 0]);
    text(&mut bild, 8, 116, 1, [255, 0, 0], "ROT AUF SCHWARZ: FARBSAUM?");
    text(&mut bild, 8, 128, 1, [0, 0, 255], "BLAU AUF SCHWARZ: LESBAR?");
    text(&mut bild, 8, 140, 1, [255, 0, 255], "MAGENTA AUF SCHWARZ");
    text(&mut bild, 8, 152, 1, [255, 255, 0], "GELB AUF SCHWARZ");
    // Weiße Schrift auf Weißgrau: Kontrast und Überstrahlen
    bild.rechteck(0, 176, 320, 64, [200, 200, 200]);
    text(&mut bild, 8, 184, 1, [0, 0, 0], "SCHWARZ AUF HELLGRAU");
    text(&mut bild, 8, 200, 2, [40, 40, 160], "BLAU AUF GRAU");
    bild
}

/// Eine eigene Pixelgrafik-Szene im Stil eines 16-Bit-Rollenspiels, 256×224, Pixel 8:7.
///
/// Enthält bewusst: Farbverläufe in Stufen, Schachbrett-Dithering (verschmilzt bei Composite),
/// einpixelige senkrechte Linien (Regenbogen bei Composite), eine Figur, ein Textfenster.
pub fn pixelszene() -> Bild {
    let (b, h) = (256i32, 224i32);
    let mut bild = Bild::neu(b as u32, h as u32, [0, 0, 0]);
    bild.pixel_seitenverhaeltnis = 8.0 / 7.0;

    // Himmel: 12 Farbstufen wie mit einer begrenzten Palette
    let himmel_oben = [24, 40, 120];
    let himmel_unten = [248, 168, 120];
    for y in 0..120 {
        let stufe = (y / 10) as f64 / 11.0;
        let f = mische(himmel_oben, himmel_unten, stufe);
        // Übergänge zwischen zwei Stufen mit Schachbrett-Dithering
        let naechste = mische(himmel_oben, himmel_unten, ((y / 10) + 1) as f64 / 11.0);
        for x in 0..b {
            let dither = y % 10 >= 8 && (x + y) % 2 == 0;
            bild.setze(x, y, if dither { naechste } else { f });
        }
    }
    // Sonne mit Hof
    for y in 20..90 {
        for x in 150..230 {
            let d = (((x - 190) * (x - 190) + (y - 55) * (y - 55)) as f64).sqrt();
            if d < 18.0 {
                bild.setze(x, y, [255, 244, 200]);
            } else if d < 22.0 && (x + y) % 2 == 0 {
                bild.setze(x, y, [255, 220, 150]);
            }
        }
    }
    // Ferne Berge
    for x in 0..b {
        let xf = x as f64;
        let hoehe = 78.0 + 14.0 * (xf * 0.045).sin() + 8.0 * (xf * 0.11 + 1.3).sin() + 3.0 * (xf * 0.37).sin();
        for y in hoehe as i32..130 {
            let schnee = y < hoehe as i32 + 3 && hoehe < 74.0;
            bild.setze(x, y, if schnee { [232, 232, 248] } else { [96, 88, 152] });
        }
    }
    // Hügel
    for x in 0..b {
        let xf = x as f64;
        let hoehe = 112.0 + 8.0 * (xf * 0.03 + 2.0).sin() + 4.0 * (xf * 0.09).sin();
        for y in hoehe as i32..140 {
            bild.setze(x, y, if y < hoehe as i32 + 2 { [104, 176, 72] } else { [56, 120, 64] });
        }
    }
    // Boden: Gras-Kacheln 16×16 mit Halmen, ein Weg
    for y in 140..h {
        for x in 0..b {
            let (tx, ty) = (x % 16, (y - 140) % 16);
            let mut f = [72, 152, 56];
            if (tx * 7 + ty * 3 + (x / 16) * 5) % 23 == 0 {
                f = [120, 200, 88];
            }
            if (tx == 3 && ty < 3) || (tx == 11 && (6..9).contains(&ty)) {
                f = [40, 104, 40];
            }
            let weg_mitte = 128.0 + (y as f64 - 140.0) * 0.6;
            if ((x as f64) - weg_mitte).abs() < 10.0 + (y - 140) as f64 * 0.25 {
                f = if (x + y) % 5 == 0 { [176, 144, 96] } else { [200, 168, 112] };
            }
            bild.setze(x, y, f);
        }
    }
    // Baum
    bild.rechteck(36, 150, 6, 22, [104, 64, 32]);
    bild.rechteck(37, 150, 1, 22, [136, 88, 48]);
    for y in 118..156 {
        for x in 20..58 {
            let d = (((x - 39) * (x - 39) + (y - 136) * (y - 136) * 2) as f64).sqrt();
            if d < 18.0 {
                let licht = x < 39 && y < 134;
                let f = if licht { [64, 168, 72] } else { [32, 112, 48] };
                bild.setze(x, y, if (x * 3 + y) % 7 == 0 { [24, 80, 40] } else { f });
            }
        }
    }
    // Zaun aus einpixeligen Latten (Composite-Prüfung)
    for x in (176..248).step_by(2) {
        bild.rechteck(x, 146, 1, 16, [240, 232, 216]);
    }
    bild.rechteck(176, 150, 72, 1, [200, 184, 160]);
    bild.rechteck(176, 157, 72, 1, [200, 184, 160]);
    // Figur (16×24), eigener Entwurf
    let figur = [
        "....RRRRRR......",
        "...RRRRRRRR.....",
        "..RRRrrrrRRR....",
        "..HHHHHHHHHH....",
        "..HSSSSSSSSH....",
        "..HSKSSSSKSH....",
        "...SSSSSSSS.....",
        "...SSSmmSSS.....",
        "....SSSSSS......",
        "...BBBBBBBB.....",
        "..BBBGGGGBBB....",
        ".SBBBBGGBBBBS...",
        ".SBBBBBBBBBBS...",
        ".SSBBBBBBBBSS...",
        "...BBBBBBBB.....",
        "...YYYYYYYY.....",
        "...BBBBBBBB.....",
        "...BBB..BBB.....",
        "...BBB..BBB.....",
        "...BBB..BBB.....",
        "..DDDD..DDDD....",
        "..DDDD..DDDD....",
        "................",
        "................",
    ];
    for (y, zeile) in figur.iter().enumerate() {
        for (x, c) in zeile.chars().enumerate() {
            let f = match c {
                'R' => [200, 40, 40],
                'r' => [240, 96, 80],
                'H' => [88, 48, 24],
                'S' => [248, 200, 160],
                'K' => [24, 24, 32],
                'm' => [200, 96, 96],
                'B' => [40, 72, 184],
                'G' => [232, 200, 64],
                'Y' => [184, 136, 40],
                'D' => [72, 48, 32],
                _ => continue,
            };
            bild.setze(120 + x as i32, 150 + y as i32, f);
        }
    }
    // Textfenster unten mit Verlauf und Rahmen
    let (fx, fy, fb, fh) = (8, 178, 240, 40);
    for y in fy..fy + fh {
        let f = mische([32, 48, 160], [8, 16, 72], (y - fy) as f64 / fh as f64);
        bild.rechteck(fx, y, fb, 1, f);
    }
    for (i, f) in [[248, 248, 248], [168, 168, 200]].iter().enumerate() {
        let i = i as i32;
        bild.rechteck(fx + i, fy + i, fb - 2 * i, 1, *f);
        bild.rechteck(fx + i, fy + fh - 1 - i, fb - 2 * i, 1, *f);
        bild.rechteck(fx + i, fy + i, 1, fh - 2 * i, *f);
        bild.rechteck(fx + fb - 1 - i, fy + i, 1, fh - 2 * i, *f);
    }
    text(&mut bild, fx + 8, fy + 8, 1, [248, 248, 248], "ROEHRE: DIES IST EIN TESTBILD.");
    text(&mut bild, fx + 8, fy + 20, 1, [248, 224, 96], "HP 120/120   MP 32/40   ÄÖÜ!");
    bild
}

/// Eine eigene 3D-Szene, per Strahlverfolgung auf der CPU berechnet (vierfache
/// Überabtastung): Schachbrettboden bis zum Horizont, spiegelnde Kugeln, Himmel.
/// Prüft die virtuelle Zeilenzahl bei hochaufgelöstem Eingangsbild.
pub fn szene_3d(breite: u32, hoehe: u32) -> Bild {
    type V = [f64; 3];
    fn add(a: V, b: V) -> V { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
    fn sub(a: V, b: V) -> V { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
    fn mul(a: V, s: f64) -> V { [a[0] * s, a[1] * s, a[2] * s] }
    fn dot(a: V, b: V) -> f64 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
    fn norm(a: V) -> V { mul(a, 1.0 / dot(a, a).sqrt()) }

    let kugeln: [(V, f64, V, f64); 4] = [
        ([0.0, 1.0, 6.0], 1.0, [0.9, 0.2, 0.15], 0.25),
        ([-2.3, 0.7, 7.5], 0.7, [0.2, 0.5, 0.95], 0.4),
        ([2.2, 0.8, 5.0], 0.8, [0.95, 0.85, 0.3], 0.6),
        ([0.9, 0.35, 3.6], 0.35, [0.95, 0.95, 0.95], 0.1),
    ];
    let licht = norm([-0.5, 0.8, -0.4]);

    fn himmel(d: V) -> V {
        let t = (d[1] * 1.5).clamp(0.0, 1.0);
        let basis = [0.85 - 0.5 * t, 0.88 - 0.35 * t, 0.95 - 0.1 * t];
        let sonne = dot(d, norm([-0.5, 0.8, -0.4])).max(0.0).powf(200.0) * 4.0;
        add(basis, [sonne, sonne * 0.9, sonne * 0.7])
    }

    fn treffer(o: V, d: V, kugeln: &[(V, f64, V, f64)]) -> Option<(f64, V, V, f64)> {
        let mut best: Option<(f64, V, V, f64)> = None;
        for &(c, r, farbe, spiegel) in kugeln {
            let oc = sub(o, c);
            let b = dot(oc, d);
            let disk = b * b - (dot(oc, oc) - r * r);
            if disk > 0.0 {
                let t = -b - disk.sqrt();
                if t > 1e-4 && best.is_none_or(|x| t < x.0) {
                    let p = add(o, mul(d, t));
                    best = Some((t, norm(sub(p, c)), farbe, spiegel));
                }
            }
        }
        if d[1] < 0.0 {
            let t = -o[1] / d[1];
            if t > 1e-4 && best.is_none_or(|x| t < x.0) {
                let p = add(o, mul(d, t));
                let feld = ((p[0].floor() + p[2].floor()) as i64).rem_euclid(2) == 0;
                let farbe = if feld { [0.85, 0.85, 0.82] } else { [0.12, 0.14, 0.18] };
                best = Some((t, [0.0, 1.0, 0.0], farbe, 0.15));
            }
        }
        best
    }

    fn strahl(o: V, d: V, tiefe: u32, kugeln: &[(V, f64, V, f64)], licht: V) -> V {
        match treffer(o, d, kugeln) {
            None => himmel(d),
            Some((t, n, farbe, spiegel)) => {
                let p = add(o, mul(d, t));
                let schatten = treffer(add(p, mul(n, 1e-3)), licht, kugeln).is_some();
                let diffus = if schatten { 0.0 } else { dot(n, licht).max(0.0) };
                let mut c = mul(farbe, 0.15 + 0.85 * diffus);
                if !schatten {
                    let h = norm(sub(licht, d));
                    let glanz = dot(n, h).max(0.0).powf(60.0);
                    c = add(c, [glanz; 3]);
                }
                if tiefe < 3 && spiegel > 0.0 {
                    let r = sub(d, mul(n, 2.0 * dot(d, n)));
                    let s = strahl(add(p, mul(n, 1e-3)), r, tiefe + 1, kugeln, licht);
                    c = add(mul(c, 1.0 - spiegel), mul(s, spiegel));
                }
                // Dunst in der Ferne
                let dunst = 1.0 - (-t * 0.03).exp();
                add(mul(c, 1.0 - dunst), mul(himmel([d[0], 0.05, d[2]]), dunst))
            }
        }
    }

    let mut bild = Bild::neu(breite, hoehe, [0, 0, 0]);
    let seite = breite as f64 / hoehe as f64;
    let auge = [0.0, 1.6, -1.0];
    let unterabtastung = [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)];
    for y in 0..hoehe {
        for x in 0..breite {
            let mut summe = [0.0; 3];
            for (ox, oy) in unterabtastung {
                let u = ((x as f64 + ox) / breite as f64 * 2.0 - 1.0) * seite * 0.6;
                let v = (1.0 - (y as f64 + oy) / hoehe as f64 * 2.0) * 0.6;
                let d = norm([u, v - 0.18, 1.0]);
                summe = add(summe, strahl(auge, d, 0, &kugeln, licht));
            }
            let c = mul(summe, 0.25);
            // linear → sRGB (Näherung mit Gamma 2.2) mit weicher Begrenzung
            let f = c.map(|k| ((k / (1.0 + 0.15 * k)).max(0.0).powf(1.0 / 2.2) * 255.0).round().min(255.0) as u8);
            bild.setze(x as i32, y as i32, f);
        }
    }
    bild
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groessen_stimmen() {
        let p = pixelszene();
        assert_eq!((p.breite, p.hoehe), (256, 224));
        assert!((p.pixel_seitenverhaeltnis - 8.0 / 7.0).abs() < 1e-9);
        let z = szene_3d(64, 36);
        assert_eq!(z.rgba.len(), 64 * 36 * 4);
    }
}
