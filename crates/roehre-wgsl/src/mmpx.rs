//! **MMPX** auf der CPU — die Referenz der Stufe `mmpx.wgsl` (der Prüfstand
//! verlangt Punktgleichheit) und ein Weg für Ausführer ohne GPU-Stufe.
//!
//! Morgan McGuire und Mara Gagiu, »MMPX Style-Preserving Pixel-Art
//! Magnification«, JCGT 10(2), 2021. Copyright 2020 Morgan McGuire & Mara
//! Gagiu, MIT (`wgsl/LIZENZ-MMPX`).
//!
//! Quelle der Regeln: die Rust-Abschrift `mmpx` 0.2.0 (Kyle Moore, MIT), die
//! die Regeln samt Kommentaren der JavaScript-Referenz der Autoren trägt. Ihr
//! Fehler ist nicht übernommen: sie liest `P` (zwei darüber) und `S` (zwei
//! darunter) nur für die erste Spalte einer Zeile. Gegen die Referenz der
//! Autoren selbst ist **nicht** Punkt für Punkt geprüft (ihre Seite war beim
//! Bau nicht erreichbar). Helligkeit für die Vergleiche: `r + g + b`.
//!
//! Nachbarschaft um `E`:
//!
//! ```text
//!       P
//!     A B C
//!   Q D E F R
//!     G H I
//!       S
//! ```

/// Ein Punkt als `0xRRGGBB` — Gleichheit ist die Regel, nicht Nähe.
type Punkt = u32;

fn hell(p: Punkt) -> u32 {
    (p >> 16 & 0xFF) + (p >> 8 & 0xFF) + (p & 0xFF)
}

/// Verdoppelt den Ausschnitt `x0..x1 × y0..y1` eines RGBA-Bilds der Breite
/// `breite` (Alpha wird nicht gelesen). Nachbarn außerhalb des Ausschnitts
/// sind der nächste Randpunkt **im** Ausschnitt — so mischt der beschnittene,
/// schwarze Overscan nichts in die Kante. Ergebnis: `(2·w) × (2·h)` Punkte
/// als `0xRRGGBB`, Zeile für Zeile.
pub fn verdoppeln(
    rgba: &[u8],
    breite: usize,
    ausschnitt: (usize, usize, usize, usize),
) -> Vec<Punkt> {
    let mut m = Mmpx::default();
    m.verdoppeln(rgba, breite, ausschnitt);
    m.aus
}

/// MMPX mit wiederverwendeten Puffern: ein frisch angeforderter Megabyte je
/// Bild kostete allein durch die Seitenfehler beim ersten Beschreiben einen
/// guten Teil der Zeit.
#[derive(Default)]
pub struct Mmpx {
    feld: Vec<Punkt>,
    aus: Vec<Punkt>,
}

impl Mmpx {
    /// Wie [`verdoppeln`]; das Ergebnis steht danach in [`Mmpx::ergebnis`].
    pub fn verdoppeln(
        &mut self,
        rgba: &[u8],
        breite: usize,
        (x0, x1, y0, y1): (usize, usize, usize, usize),
    ) {
        let (w, h) = (x1 - x0, y1 - y0);
        // Einmal umlegen: der Ausschnitt als `u32`, ringsum drei Punkte Polster
        // mit dem nächsten Randpunkt (die weitesten Nachbarn liegen drei weg).
        // Vorher klemmte und las jeder der rund zwanzig Nachbarzugriffe je Punkt
        // einzeln — 2,6 ms je Bild gemessen.
        const R: usize = 3;
        let pb = w + 2 * R;
        let feld = &mut self.feld;
        feld.resize(pb * (h + 2 * R), 0);
        for py in 0..h + 2 * R {
            let y = (py as i32 - R as i32).clamp(0, h as i32 - 1) as usize + y0;
            for px in 0..pb {
                let x = (px as i32 - R as i32).clamp(0, w as i32 - 1) as usize + x0;
                let i = (y * breite + x) * 4;
                feld[py * pb + px] =
                    u32::from(rgba[i]) << 16 | u32::from(rgba[i + 1]) << 8 | u32::from(rgba[i + 2]);
            }
        }
        let aus = &mut self.aus;
        aus.resize(4 * w * h, 0);
        let feld = &*feld;
        let ab = 2 * w;
        for (y, paar) in aus.chunks_mut(2 * ab).enumerate() {
            let (ausz0, ausz1) = paar.split_at_mut(ab);
            zeile_verdoppeln(feld, pb, w, y, ausz0, ausz1);
        }
    }

    /// Das letzte Ergebnis: `(2·w) × (2·h)` Punkte als `0xRRGGBB`.
    pub fn ergebnis(&self) -> &[Punkt] {
        &self.aus
    }
}

/// Eine Zeile `y` des Ausschnitts: schreibt ihre zwei Ausgabezeilen.
fn zeile_verdoppeln(
    feld: &[Punkt],
    pb: usize,
    w: usize,
    y: usize,
    ausz0: &mut [Punkt],
    ausz1: &mut [Punkt],
) {
    const R: usize = 3;
    {
        // Die sieben Zeilen um E als Scheiben (drei darüber, drei darunter):
        // so prüft der Übersetzer die Grenzen einmal je Zeile, nicht je
        // Nachbar.
        let zeile = |dy: usize| &feld[(y + dy) * pb..][..pb];
        let (z_3, z_2, z_1, z0, z1, z2, z3) = (
            zeile(0),
            zeile(1),
            zeile(2),
            zeile(3),
            zeile(4),
            zeile(5),
            zeile(6),
        );
        for x in 0..w {
            let mx = x + R;
            let (b, d, e, f, h_) = (z_1[mx], z0[mx - 1], z0[mx], z0[mx + 1], z1[mx]);
            let o = 2 * x;
            // Fläche: sind die vier direkten Nachbarn wie E, greift keine
            // Regel (jede verlangt einen Unterschied zwischen E, B, D, F, H).
            if b == e && d == e && f == e && h_ == e {
                ausz0[o] = e;
                ausz0[o + 1] = e;
                ausz1[o] = e;
                ausz1[o + 1] = e;
                continue;
            }
            let (a, c, g, i) = (z_1[mx - 1], z_1[mx + 1], z1[mx - 1], z1[mx + 1]);
            let (p, q, r, s) = (z_2[mx], z0[mx - 2], z0[mx + 2], z2[mx]);
            let (hb, hd, he, hf, hh) = (hell(b), hell(d), hell(e), hell(f), hell(h_));
            let (mut j, mut k, mut l, mut m) = (e, e, e, e);

            // 1:1 slope rules
            if d == b
                && d != h_
                && d != f
                && (he >= hd || e == a)
                && (e == a || e == c || e == g)
                && (he < hd || a != d || e != p || e != q)
            {
                j = d;
            }
            if b == f
                && b != d
                && b != h_
                && (he >= hb || e == c)
                && (e == a || e == c || e == i)
                && (he < hb || c != b || e != p || e != r)
            {
                k = b;
            }
            if h_ == d
                && h_ != f
                && h_ != b
                && (he >= hh || e == g)
                && (e == a || e == g || e == i)
                && (he < hh || g != h_ || e != s || e != q)
            {
                l = h_;
            }
            if f == h_
                && f != b
                && f != d
                && (he >= hf || e == i)
                && (e == c || e == g || e == i)
                && (he < hf || i != h_ || e != r || e != s)
            {
                m = f;
            }

            // Intersection rules
            if e != f
                && e == c
                && e == i
                && e == d
                && e == q
                && f == b
                && f == h_
                && f != z0[mx + 3]
            {
                k = f;
                m = f;
            }
            if e != d
                && e == a
                && e == g
                && e == f
                && e == r
                && d == b
                && d == h_
                && d != z0[mx - 3]
            {
                j = d;
                l = d;
            }
            if e != h_ && e == g && e == i && e == b && e == p && h_ == d && h_ == f && h_ != z3[mx]
            {
                l = h_;
                m = h_;
            }
            if e != b && e == a && e == c && e == h_ && e == s && b == d && b == f && b != z_3[mx] {
                j = b;
                k = b;
            }
            if hb < he
                && e == g
                && e == h_
                && e == i
                && e == s
                && e != a
                && e != d
                && e != c
                && e != f
            {
                j = b;
                k = b;
            }
            if hh < he
                && e == a
                && e == b
                && e == c
                && e == p
                && e != d
                && e != g
                && e != i
                && e != f
            {
                l = h_;
                m = h_;
            }
            if hf < he
                && e == a
                && e == d
                && e == g
                && e == q
                && e != b
                && e != c
                && e != i
                && e != h_
            {
                k = f;
                m = f;
            }
            if hd < he
                && e == c
                && e == f
                && e == i
                && e == r
                && e != b
                && e != a
                && e != g
                && e != h_
            {
                j = d;
                l = d;
            }

            // 2:1 slope rules
            if h_ != b {
                if h_ != a && h_ != e && h_ != c {
                    if h_ == g && h_ == f && h_ == r && h_ != d && h_ != z_1[mx + 2] {
                        l = m;
                    }
                    if h_ == i && h_ == d && h_ == q && h_ != f && h_ != z_1[mx - 2] {
                        m = l;
                    }
                }
                if b != i && b != g && b != e {
                    if b == a && b == f && b == r && b != d && b != z1[mx + 2] {
                        j = k;
                    }
                    if b == c && b == d && b == q && b != f && b != z1[mx - 2] {
                        k = j;
                    }
                }
            }
            if f != d {
                if d != i && d != e && d != c {
                    if d == a && d == h_ && d == s && d != b && d != z2[mx + 1] {
                        j = l;
                    }
                    if d == g && d == b && d == p && d != h_ && d != z_2[mx + 1] {
                        l = j;
                    }
                }
                if f != e && f != a && f != g {
                    if f == c && f == h_ && f == s && f != b && f != z2[mx - 1] {
                        k = m;
                    }
                    if f == i && f == b && f == p && f != h_ && f != z_2[mx - 1] {
                        m = k;
                    }
                }
            }

            ausz0[o] = j;
            ausz0[o + 1] = k;
            ausz1[o] = l;
            ausz1[o + 1] = m;
        }
    }
}

#[cfg(test)]
mod proben {
    use super::*;

    fn bild(w: usize, h: usize, f: impl Fn(usize, usize) -> u32) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let p = f(x, y);
                v[(y * w + x) * 4..][..4].copy_from_slice(&[
                    (p >> 16) as u8,
                    (p >> 8) as u8,
                    p as u8,
                    255,
                ]);
            }
        }
        v
    }

    /// Eine Fläche bleibt Fläche, und jeder neue Punkt ist eine Farbe der
    /// Quelle — MMPX erfindet nichts.
    #[test]
    fn flaechen_bleiben_und_keine_farbe_ist_neu() {
        let q = bild(16, 12, |_, _| 0x204080);
        assert!(verdoppeln(&q, 16, (0, 16, 0, 12))
            .iter()
            .all(|&p| p == 0x204080));
        let q = bild(16, 16, |x, y| {
            if x > y {
                0xF8F8F8
            } else if (x + y) % 5 == 0 {
                0x103010
            } else {
                0x000040
            }
        });
        let farben = [0xF8F8F8, 0x103010, 0x000040];
        assert!(verdoppeln(&q, 16, (0, 16, 0, 16))
            .iter()
            .all(|p| farben.contains(p)));
    }

    /// Eine Treppe mit Steigung 1:1 wird geglättet: auf der Diagonale füllt
    /// MMPX den halben Punkt (vorher 2 × 2-Stufen).
    #[test]
    fn eine_diagonale_wird_glatt() {
        let (hell_, dunkel) = (0xF8F8F8, 0x000040);
        let q = bild(8, 8, |x, y| if x >= y { hell_ } else { dunkel });
        let a = verdoppeln(&q, 8, (0, 8, 0, 8));
        // Punkt (4, 4) liegt hell auf der Kante, links und unter ihm dunkel:
        // sein unteres linkes Viertel (L) wird dunkel. Der dunkle Punkt
        // (3, 4) bleibt ganz dunkel — Helles übernimmt er nur, wenn auch `C`
        // seine Farbe hat (Regel »he >= hb || e == c«).
        let o = (2 * 4) * 16 + 2 * 4;
        assert_eq!(
            (a[o], a[o + 1], a[o + 16], a[o + 17]),
            (hell_, hell_, dunkel, hell_)
        );
        let o = (2 * 4) * 16 + 2 * 3;
        assert_eq!(
            (a[o], a[o + 1], a[o + 16], a[o + 17]),
            (dunkel, dunkel, dunkel, dunkel)
        );
    }

    /// Der Ausschnitt begrenzt die Nachbarschaft: eine schwarze Spalte
    /// links außerhalb wirkt nicht hinein.
    #[test]
    fn der_beschnittene_rand_wirkt_nicht_hinein() {
        let q = bild(10, 6, |x, y| {
            if x == 0 {
                0
            } else if (x + y) % 2 == 0 {
                0x808080
            } else {
                0x404040
            }
        });
        let a = verdoppeln(&q, 10, (1, 10, 0, 6));
        assert!(a.iter().all(|&p| p != 0));
    }
}
