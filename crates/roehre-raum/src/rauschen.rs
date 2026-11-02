//! Rauschfunktionen für die selbst erzeugten Texturen (Wertrauschen, fBm, Zellen).

fn hash(x: i32, y: i32, saat: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263) ^ saat.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0x00ff_ffff) as f32 / 16_777_215.0
}

fn glatt(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Wertrauschen 0 … 1, kachelbar mit Periode `periode` (in Gitterzellen).
pub fn wert(x: f32, y: f32, periode: i32, saat: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (glatt(x - x0), glatt(y - y0));
    let (ix, iy) = (x0 as i32, y0 as i32);
    let p = |a: i32| a.rem_euclid(periode.max(1));
    let h = |dx: i32, dy: i32| hash(p(ix + dx), p(iy + dy), saat);
    let a = h(0, 0) + (h(1, 0) - h(0, 0)) * fx;
    let b = h(0, 1) + (h(1, 1) - h(0, 1)) * fx;
    a + (b - a) * fy
}

/// Fraktales Rauschen über `oktaven` Stufen, kachelbar auf dem Einheitsquadrat bei
/// Grundfrequenz `frequenz` (ganzzahlig).
pub fn fbm(u: f32, v: f32, frequenz: i32, oktaven: u32, saat: u32) -> f32 {
    let (mut summe, mut amp, mut f, mut norm) = (0.0, 0.5, frequenz, 0.0);
    for o in 0..oktaven {
        summe += amp * wert(u * f as f32, v * f as f32, f, saat.wrapping_add(o * 7919));
        norm += amp;
        amp *= 0.5;
        f *= 2;
    }
    summe / norm
}

/// Zufallswert für eine ganze Zahl (z. B. je Diele, je Kassette).
pub fn zufall(n: i32, saat: u32) -> f32 {
    hash(n, 0, saat)
}
