//! Moiré-Messung.
//!
//! Gerendert wird ein gleichmäßig graues Feld. Auf einer sauberen Röhre ist dann jede
//! Zeilenperiode (und jede Maskenperiode) gleich hell. Schwebungen entstehen, wenn Zeilen
//! oder Maske nicht sauber auf die Ausgabepixel passen: Manche Perioden werden heller,
//! andere dunkler, und das Auge sieht ein grobes Streifenmuster.
//!
//! Verfahren je Richtung:
//! 1. Mittelstreifen (20 % der Fläche) zu einem Profil mitteln: je Zeile bzw. je Spalte.
//! 2. Das Profil mit einem Kasten von genau einer Periode Breite glätten (Bruchteile
//!    anteilig). Was danach noch schwankt, ist nicht das Zeilen- oder Maskenmuster selbst.
//! 3. Langsame Verläufe (Vignette, Wölbung) mit einem achtmal breiteren Kasten abziehen.
//! 4. Kennzahl: größte Abweichung des Rests, bezogen auf den Mittelwert, in Prozent.
//!
//! Faustregel aus der Wahrnehmung: unter 1 % unsichtbar, 1–3 % auf großen Flächen
//! erkennbar, darüber deutlich.

/// Kastenfilter mit gebrochener Breite über ein Profil (Summenkurve, linear interpoliert).
fn kasten(profil: &[f64], breite: f64) -> Vec<f64> {
    let n = profil.len();
    let mut summe = vec![0.0; n + 1];
    for i in 0..n {
        summe[i + 1] = summe[i] + profil[i];
    }
    let s = |x: f64| -> f64 {
        let x = x.clamp(0.0, n as f64);
        let i = (x.floor() as usize).min(n - 1);
        summe[i] + (x - i as f64) * profil[i]
    };
    (0..n)
        .map(|i| {
            let mitte = i as f64 + 0.5;
            (s(mitte + breite / 2.0) - s(mitte - breite / 2.0)) / breite
        })
        .collect()
}

/// Schwebung eines Profils in Prozent (siehe Moduldoku), ausgewertet im Inneren
/// (Ränder um 12 % plus vier Perioden gekürzt, damit Eckenrundung und Rand nicht zählen).
pub fn schwebung(profil: &[f64], periode: f64) -> f64 {
    let geglaettet = kasten(profil, periode);
    let verlauf = kasten(&geglaettet, periode * 8.0);
    let n = profil.len();
    let rand = (n as f64 * 0.12 + periode * 4.0) as usize;
    if n <= 2 * rand + 2 {
        return f64::NAN;
    }
    let mittel: f64 = geglaettet[rand..n - rand].iter().sum::<f64>() / (n - 2 * rand) as f64;
    let max = (rand..n - rand).map(|i| (geglaettet[i] - verlauf[i]).abs()).fold(0.0, f64::max);
    100.0 * max / mittel.max(1e-9)
}

/// Profile (Zeilen und Spalten) des Mittelstreifens einer Fläche, linear gemittelt.
pub fn profile(bild: &image::RgbaImage, x: u32, y: u32, b: u32, h: u32) -> (Vec<f64>, Vec<f64>) {
    let lin = |v: u8| {
        let c = v as f64 / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    let leuchte = |px: &image::Rgba<u8>| 0.2126 * lin(px[0]) + 0.7152 * lin(px[1]) + 0.0722 * lin(px[2]);
    let (sx0, sx1) = (x + b * 2 / 5, x + b * 3 / 5);
    let (sy0, sy1) = (y + h * 2 / 5, y + h * 3 / 5);
    let zeilen = (y..y + h)
        .map(|yy| (sx0..sx1).map(|xx| leuchte(bild.get_pixel(xx, yy))).sum::<f64>() / (sx1 - sx0) as f64)
        .collect();
    let spalten = (x..x + b)
        .map(|xx| (sy0..sy1).map(|yy| leuchte(bild.get_pixel(xx, yy))).sum::<f64>() / (sy1 - sy0) as f64)
        .collect();
    (zeilen, spalten)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gleichmaessiges_muster_hat_keine_schwebung() {
        // Zeilen mit genau 4 px Periode
        let p: Vec<f64> = (0..400).map(|i| if i % 4 < 2 { 1.0 } else { 0.2 }).collect();
        assert!(schwebung(&p, 4.0) < 0.01);
    }

    #[test]
    fn schwebendes_muster_wird_erkannt() {
        // Zeilen mit 2,25 px Periode, je Pixel abgetastet: deutliche Schwebung
        let p: Vec<f64> = (0..900).map(|i| (0.5 + 0.5 * (std::f64::consts::TAU * (i as f64 + 0.5) / 2.25).cos()).powi(4)).collect();
        assert!(schwebung(&p, 2.25) > 1.0, "{}", schwebung(&p, 2.25));
    }
}
