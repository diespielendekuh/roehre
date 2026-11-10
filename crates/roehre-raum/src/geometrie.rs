//! Geometrie-Bausteine: Quader mit gerundeten Kanten, Zylinder, Drehkörper, Röhren, Blätter
//! und die gewölbte Bildröhre. Alles wird beim Start erzeugt, es gibt keine Modelldateien.

use crate::texturen::Material;

/// Eine Ecke: Lage, Normale, Texturkoordinate, Material, Windgewicht (Vorhang).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Ecke {
    pub lage: [f32; 3],
    pub normale: [f32; 3],
    pub uv: [f32; 2],
    pub material: u32,
    pub wind: f32,
}

pub type V3 = [f32; 3];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn mul(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn norm(a: V3) -> V3 {
    let l = dot(a, a).sqrt().max(1e-9);
    mul(a, 1.0 / l)
}

/// Affine Abbildung: Drehung/Skalierung (Spalten) und Verschiebung.
#[derive(Clone, Copy, Debug)]
pub struct Trafo {
    pub m: [V3; 3],
    pub t: V3,
}

impl Trafo {
    pub const EINS: Trafo = Trafo { m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], t: [0.0; 3] };

    pub fn bei(t: V3) -> Self {
        Trafo { t, ..Self::EINS }
    }
    pub fn dreh_y(w: f32) -> Self {
        let (s, c) = w.sin_cos();
        Trafo { m: [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]], t: [0.0; 3] }
    }
    pub fn dreh_x(w: f32) -> Self {
        let (s, c) = w.sin_cos();
        Trafo { m: [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]], t: [0.0; 3] }
    }
    pub fn dreh_z(w: f32) -> Self {
        let (s, c) = w.sin_cos();
        Trafo { m: [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]], t: [0.0; 3] }
    }
    /// Erst `b`, dann `self`.
    pub fn nach(&self, b: &Trafo) -> Trafo {
        let m = [self.richtung(b.m[0]), self.richtung(b.m[1]), self.richtung(b.m[2])];
        Trafo { m, t: self.punkt(b.t) }
    }
    pub fn punkt(&self, p: V3) -> V3 {
        add(self.richtung(p), self.t)
    }
    pub fn richtung(&self, d: V3) -> V3 {
        add(add(mul(self.m[0], d[0]), mul(self.m[1], d[1])), mul(self.m[2], d[2]))
    }
    pub fn normale(&self, n: V3) -> V3 {
        // für Drehungen und gleichmäßige Skalierung genügt die Richtung
        norm(self.richtung(n))
    }
}

/// Ein Netz im Aufbau.
#[derive(Default)]
pub struct Netz {
    pub ecken: Vec<Ecke>,
    pub indizes: Vec<u32>,
}

impl Netz {
    fn ecke(&mut self, t: &Trafo, p: V3, n: V3, uv: [f32; 2], m: Material, wind: f32) -> u32 {
        self.ecken.push(Ecke { lage: t.punkt(p), normale: t.normale(n), uv, material: m as u32, wind });
        (self.ecken.len() - 1) as u32
    }

    fn dreieck(&mut self, a: u32, b: u32, c: u32) {
        self.indizes.extend_from_slice(&[a, b, c]);
    }

    /// Gitter aus (n+1)² Ecken, `f(s, t)` liefert Lage, Normale, uv für s, t ∈ [0, 1].
    pub fn gitter(&mut self, t: &Trafo, n: (u32, u32), m: Material, wind: f32, f: impl Fn(f32, f32) -> (V3, V3, [f32; 2])) {
        let start = self.ecken.len() as u32;
        for j in 0..=n.1 {
            for i in 0..=n.0 {
                let (s, r) = (i as f32 / n.0 as f32, j as f32 / n.1 as f32);
                let (p, nn, uv) = f(s, r);
                let w = wind * r;
                self.ecke(t, p, nn, uv, m, w);
            }
        }
        let zeile = n.0 + 1;
        for j in 0..n.1 {
            for i in 0..n.0 {
                let a = start + j * zeile + i;
                self.dreieck(a, a + 1, a + zeile + 1);
                self.dreieck(a, a + zeile + 1, a + zeile);
            }
        }
    }

    /// Rechteck in der xy-Ebene (Normale −z, zur Kamera), Mitte im Ursprung.
    pub fn flaeche(&mut self, t: &Trafo, halb: [f32; 2], m: Material, uv: [f32; 4]) {
        self.gitter(t, (1, 1), m, 0.0, |s, r| {
            ([(s * 2.0 - 1.0) * halb[0], (1.0 - r * 2.0) * halb[1], 0.0], [0.0, 0.0, -1.0], [uv[0] + s * uv[2], uv[1] + r * uv[3]])
        });
    }

    /// Quader mit gerundeten Kanten (Radius `r`), Texturmaßstab `uvm` (Kacheln je Meter),
    /// `n` Unterteilungen je Fläche (mehr = rundere Kanten).
    pub fn quader(&mut self, t: &Trafo, halb: V3, r: f32, m: Material, uvm: f32) {
        let n = if r > 0.0 { 6 } else { 1 };
        let innen = [halb[0] - r, halb[1] - r, halb[2] - r];
        // sechs Flächen: Normale, u-Achse, v-Achse
        let flaechen: [(V3, V3, V3); 6] = [
            ([0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
            ([0.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
            ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, -1.0, 0.0]),
            ([-1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, -1.0, 0.0]),
            ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
        ];
        for (nrm, ua, va) in flaechen {
            let hu = dot(halb, ua.map(f32::abs));
            let hv = dot(halb, va.map(f32::abs));
            let hn = dot(halb, nrm.map(f32::abs));
            self.gitter(t, (n, n), m, 0.0, |s, q| {
                // Punkt auf der Fläche eines kantigen Quaders …
                let roh = add(add(mul(nrm, hn), mul(ua, (s * 2.0 - 1.0) * hu)), mul(va, (q * 2.0 - 1.0) * hv));
                // … auf den gerundeten Quader projiziert
                let k = [roh[0].clamp(-innen[0], innen[0]), roh[1].clamp(-innen[1], innen[1]), roh[2].clamp(-innen[2], innen[2])];
                let d = sub(roh, k);
                let (p, nn) = if r > 0.0 && dot(d, d) > 1e-12 { (add(k, mul(norm(d), r)), norm(d)) } else { (roh, nrm) };
                let uv = [dot(p, ua) * uvm, dot(p, va) * uvm];
                (p, nn, uv)
            });
        }
    }

    /// Dreieck mit der Vorderseite in Richtung der Normale seiner ersten Ecke (dreht die
    /// Reihenfolge, wenn nötig) – für Körper, deren Umlaufsinn nicht von vornherein feststeht.
    fn dreieck_nach_normale(&mut self, a: u32, b: u32, c: u32) {
        let (pa, pb, pc) = (self.ecken[a as usize].lage, self.ecken[b as usize].lage, self.ecken[c as usize].lage);
        let n = add(add(self.ecken[a as usize].normale, self.ecken[b as usize].normale), self.ecken[c as usize].normale);
        if dot(cross(sub(pb, pa), sub(pc, pa)), n) >= 0.0 {
            self.dreieck(a, b, c);
        } else {
            self.dreieck(a, c, b);
        }
    }

    /// Körper mit frei geformtem, konvexem Umriss: der Umriss (Punkte x, z in der Grundfläche,
    /// beliebiger Umlaufsinn) wird um `hoehe` nach oben (+y) gezogen, die Oberkante mit Radius
    /// `r` gerundet, die Deckfläche zur Mitte hin um `woelbung` aufgewölbt. Ohne Boden.
    /// Damit entstehen Gehäuse, die kein Quader und kein Drehkörper sind (Konsolen, Griffschalen,
    /// Module mit Bogenkante); gedreht mit `Trafo` auch stehende Scheiben mit geformtem Umriss.
    pub fn platte(&mut self, t: &Trafo, umriss: &[[f32; 2]], hoehe: f32, r: f32, woelbung: f32, m: Material, uvm: f32) {
        let n = umriss.len();
        if n < 3 {
            return;
        }
        let mitte = {
            let mut c = [0.0f32; 2];
            for p in umriss {
                c[0] += p[0];
                c[1] += p[1];
            }
            [c[0] / n as f32, c[1] / n as f32]
        };
        // nach außen zeigende Normale je Punkt (aus den Nachbarn, von der Mitte weg)
        let aussen: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let (a, b) = (umriss[(i + n - 1) % n], umriss[(i + 1) % n]);
                let (tx, tz) = (b[0] - a[0], b[1] - a[1]);
                let l = (tx * tx + tz * tz).sqrt().max(1e-9);
                let mut nn = [tz / l, -tx / l];
                if nn[0] * (umriss[i][0] - mitte[0]) + nn[1] * (umriss[i][1] - mitte[1]) < 0.0 {
                    nn = [-nn[0], -nn[1]];
                }
                nn
            })
            .collect();
        let r = r.clamp(0.0, hoehe);
        const STUFEN: usize = 5;
        // Ringe: Fuß, Beginn der Rundung, Rundung in STUFEN Schritten bis zur Deckfläche
        let mut ringe: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0), (0.0, hoehe - r, 0.0)]; // (Einzug, Höhe, Winkel)
        if r > 0.0 {
            for k in 1..=STUFEN {
                let w = k as f32 / STUFEN as f32 * std::f32::consts::FRAC_PI_2;
                ringe.push((r * (1.0 - w.cos()), hoehe - r + r * w.sin(), w));
            }
        }
        // zwei Ringe der Deckfläche zur Mitte hin (für die Wölbung), dann die Mitte
        let start = self.ecken.len() as u32;
        let mut weg = 0.0;
        let mut laengen = vec![0.0f32; n + 1];
        for i in 0..n {
            let (a, b) = (umriss[i], umriss[(i + 1) % n]);
            weg += ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
            laengen[i + 1] = weg;
        }
        for &(einzug, y, w) in &ringe {
            for i in 0..=n {
                let j = i % n;
                let p = [umriss[j][0] - aussen[j][0] * einzug, y, umriss[j][1] - aussen[j][1] * einzug];
                let nn = norm([aussen[j][0] * w.cos(), w.sin(), aussen[j][1] * w.cos()]);
                self.ecke(t, p, nn, [laengen[i] * uvm, y * uvm], m, 0.0);
            }
        }
        let zeile = (n + 1) as u32;
        for k in 0..ringe.len() as u32 - 1 {
            for i in 0..n as u32 {
                let a = start + k * zeile + i;
                self.dreieck_nach_normale(a, a + 1, a + zeile + 1);
                self.dreieck_nach_normale(a, a + zeile + 1, a + zeile);
            }
        }
        // Deckfläche: vom innersten Ring über einen Zwischenring zur Mitte, Normale nach oben
        let innen = ringe.last().expect("Ringe").0;
        let oben = [0.0, 1.0, 0.0];
        let deck = self.ecken.len() as u32;
        for anteil in [0.0f32, 0.55] {
            for j in 0..n {
                let q = [umriss[j][0] - aussen[j][0] * innen, umriss[j][1] - aussen[j][1] * innen];
                let x = q[0] + (mitte[0] - q[0]) * anteil;
                let z = q[1] + (mitte[1] - q[1]) * anteil;
                // Kuppel: am Rand 0, in der Mitte `woelbung`
                let h = hoehe + woelbung * (1.0 - (1.0 - anteil) * (1.0 - anteil));
                self.ecke(t, [x, h, z], oben, [x * uvm, z * uvm], m, 0.0);
            }
        }
        let zentrum = self.ecke(t, [mitte[0], hoehe + woelbung, mitte[1]], oben, [mitte[0] * uvm, mitte[1] * uvm], m, 0.0);
        let nn = n as u32;
        for i in 0..nn {
            let (a, b) = (deck + i, deck + (i + 1) % nn);
            let (c, d) = (deck + nn + i, deck + nn + (i + 1) % nn);
            self.dreieck_nach_normale(a, b, d);
            self.dreieck_nach_normale(a, d, c);
            self.dreieck_nach_normale(c, d, zentrum);
        }
    }

    /// Drehkörper um die y-Achse aus einem Profil (Radius, Höhe), `segmente` rundherum.
    pub fn drehkoerper(&mut self, t: &Trafo, profil: &[(f32, f32)], segmente: u32, m: Material, uv_u: f32) {
        let start = self.ecken.len() as u32;
        let laenge: f32 = profil.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).sum();
        let mut weg = 0.0;
        for (k, &(r, y)) in profil.iter().enumerate() {
            if k > 0 {
                weg += ((r - profil[k - 1].0).powi(2) + (y - profil[k - 1].1).powi(2)).sqrt();
            }
            // Normale des Profils aus den Nachbarpunkten
            let (a, b) = (profil[k.saturating_sub(1)], profil[(k + 1).min(profil.len() - 1)]);
            let (dr, dy) = (b.0 - a.0, b.1 - a.1);
            let (nr, ny) = (dy, -dr);
            let l = (nr * nr + ny * ny).sqrt().max(1e-9);
            for i in 0..=segmente {
                let w = i as f32 / segmente as f32 * std::f32::consts::TAU;
                let (s, c) = w.sin_cos();
                let p = [r * c, y, r * s];
                let n = [nr / l * c, ny / l, nr / l * s];
                self.ecke(t, p, n, [i as f32 / segmente as f32 * uv_u, weg / laenge.max(1e-9)], m, 0.0);
            }
        }
        let zeile = segmente + 1;
        for k in 0..profil.len() as u32 - 1 {
            for i in 0..segmente {
                let a = start + k * zeile + i;
                self.dreieck(a, a + zeile, a + zeile + 1);
                self.dreieck(a, a + zeile + 1, a + 1);
            }
        }
    }

    /// Zylinder (Achse y, unten bei 0) mit Deckeln.
    pub fn zylinder(&mut self, t: &Trafo, r: f32, h: f32, segmente: u32, m: Material) {
        self.drehkoerper(t, &[(0.0, 0.0), (r, 0.0), (r, 0.0), (r, h), (r, h), (0.0, h)], segmente, m, 1.0);
    }

    /// Röhre entlang einer Punktfolge (Kabel, Stiele, Henkel).
    pub fn rohr(&mut self, punkte: &[V3], r: f32, segmente: u32, m: Material) {
        let start = self.ecken.len() as u32;
        let mut vorher_seite = [1.0, 0.0, 0.0];
        for (k, &p) in punkte.iter().enumerate() {
            let a = punkte[k.saturating_sub(1)];
            let b = punkte[(k + 1).min(punkte.len() - 1)];
            let tang = norm(sub(b, a));
            let mut seite = norm(sub(vorher_seite, mul(tang, dot(vorher_seite, tang))));
            if dot(seite, seite) < 0.5 {
                seite = norm(cross(tang, [0.0, 1.0, 0.0]));
            }
            vorher_seite = seite;
            let oben = cross(tang, seite);
            for i in 0..=segmente {
                let w = i as f32 / segmente as f32 * std::f32::consts::TAU;
                let n = add(mul(seite, w.cos()), mul(oben, w.sin()));
                self.ecke(&Trafo::EINS, add(p, mul(n, r)), n, [i as f32 / segmente as f32, k as f32 * 0.1], m, 0.0);
            }
        }
        let zeile = segmente + 1;
        for k in 0..punkte.len() as u32 - 1 {
            for i in 0..segmente {
                let a = start + k * zeile + i;
                self.dreieck(a, a + 1, a + zeile + 1);
                self.dreieck(a, a + zeile + 1, a + zeile);
            }
        }
    }

    /// Ein gewölbtes Blatt: Länge entlang lokal +y, Breite x, Biegung nach −z.
    pub fn blatt(&mut self, t: &Trafo, laenge: f32, breite: f32, biegung: f32, m: Material) {
        for seite in [1.0f32, -1.0] {
            self.gitter(t, (4, 6), m, 0.0, |s, q| {
                let x = (s - 0.5) * breite;
                let y = q * laenge;
                let z = biegung * (q * q) + 0.15 * breite * (s - 0.5).powi(2) * 4.0 * seite.signum() * 0.0;
                let dz_dy = 2.0 * biegung * q / laenge.max(1e-6);
                let n = norm([0.0, -dz_dy, -1.0]);
                ([x, y, z], mul(n, seite), [if seite > 0.0 { s } else { 1.0 - s }, 1.0 - q])
            });
        }
    }

    /// Bildröhre: Kugelkappe der Breite × Höhe, Scheitel `woelbung` vor der Ebene z = 0
    /// (zur Kamera, −z). uv (0,0) oben links.
    pub fn bildroehre(&mut self, t: &Trafo, halb: [f32; 2], woelbung: f32) {
        let r = (halb[0] * halb[0] + halb[1] * halb[1]) / (2.0 * woelbung.max(1e-4)) + woelbung / 2.0;
        self.gitter(t, (24, 18), Material::Bildroehre, 0.0, |s, q| {
            let (x, y) = ((s * 2.0 - 1.0) * halb[0], (1.0 - q * 2.0) * halb[1]);
            let z = -((r * r - x * x - y * y).max(0.0).sqrt() - (r - woelbung));
            let mitte = [0.0, 0.0, r - woelbung];
            let n = norm(sub([x, y, z], mitte));
            ([x, y, z], n, [s, q])
        });
    }
}

#[cfg(test)]
mod pruefe_platte {
    use super::*;
    use crate::texturen::Material;

    #[test]
    fn deckflaeche_ist_da() {
        // Umriss wie das Gehäuse des N64: Bogen vorn, gerade Seiten, runde Ecken hinten
        let mut u: Vec<[f32; 2]> = Vec::new();
        for i in 0..=28 {
            let x = 0.130 * (i as f32 / 28.0 * std::f32::consts::PI).cos();
            let a = (x / 0.130f32).clamp(-1.0, 1.0).acos();
            u.push([x, -0.030 - 0.065 * a.sin().max(0.0).powf(0.9)]);
        }
        u.push([-0.13, 0.077]);
        u.push([-0.112, 0.095]);
        u.push([0.112, 0.095]);
        u.push([0.13, 0.077]);
        let mut n = Netz::default();
        n.platte(&Trafo::EINS, &u, 0.046, 0.013, 0.003, Material::KunststoffGrau, 1.0);
        let mut oben = 0;
        let mut kaputt = 0;
        for d in n.indizes.chunks(3) {
            let (a, b, c) = (n.ecken[d[0] as usize].lage, n.ecken[d[1] as usize].lage, n.ecken[d[2] as usize].lage);
            if !(a[0].is_finite() && b[0].is_finite() && c[0].is_finite() && a[1].is_finite() && a[2].is_finite()) {
                kaputt += 1;
            }
            let nn = cross(sub(b, a), sub(c, a));
            if nn[1] > 1e-7 && a[1] > 0.045 {
                oben += 1;
            }
        }
        eprintln!("Dreiecke {}, nach oben {}, kaputt {}", n.indizes.len() / 3, oben, kaputt);
        assert_eq!(kaputt, 0);
        assert!(oben > 60);
    }
}
