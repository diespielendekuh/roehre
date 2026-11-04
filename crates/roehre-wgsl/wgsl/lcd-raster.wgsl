// roehre – Stufe „lcd-raster“ (Wunsch E3b des Terranigma-Ports): ein
// Handheld-/LCD-Bild — jeder Spielpunkt eine Zelle, dazwischen dunkle Lücken,
// wahlweise senkrechte RGB-Streifen.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Regler (rahmen.regler[0]):
//   x  gitter   Dunkelheit der Lücken (0 = keine, 1 = schwarz)
//   y  breite   Breite der Lücke als Anteil eines Spielpunkts
//   z  subpixel Stärke der RGB-Streifen (0 = aus)
//
// Gegen Moiré wird der Anteil der Lücken am Ausgabepunkt je Achse exakt
// integriert (Kastenfilter über die Fläche des Bildpunkts in Spielpunkten),
// nicht punktweise abgetastet — so bleibt das Raster bei jedem, auch krummem
// Faktor gleichmäßig. Die Farbe einer Zelle ist ihr Spielpunkt (nächster).

// Maß der Lücken im Intervall [0, x] bei Lücken der Breite w um jede ganze
// Zahl: um w/2 verschoben liegen sie in [k, k + w].
fn lueckenmass(x: f32, w: f32) -> f32 {
    let s = x + 0.5 * w;
    return floor(s) * w + min(fract(s), w);
}

fn lueckenanteil(a: f32, b: f32, w: f32) -> f32 {
    return (lueckenmass(b, w) - lueckenmass(a, w)) / max(b - a, 1e-6);
}

// Anteil des Streifens [lo, hi) jeder Zelle (Breite 1) im Intervall [a, b].
fn streifenanteil(a: f32, b: f32, lo: f32, hi: f32) -> f32 {
    let m = lueckenmass(b - lo - 0.5 * (hi - lo), hi - lo) - lueckenmass(a - lo - 0.5 * (hi - lo), hi - lo);
    return m / max(b - a, 1e-6);
}

@fragment
fn punkt(e: Ecke) -> @location(0) vec4<f32> {
    let gitter = rahmen.regler[0].x;
    let breite = clamp(rahmen.regler[0].y, 0.0, 0.5);
    let subpixel = rahmen.regler[0].z;
    let t = e.uv * rahmen.quelle.xy;
    let gezeigt = (rahmen.ausschnitt.zw - rahmen.ausschnitt.xy) * rahmen.quelle.xy;
    // Halbe Breite eines Ausgabepunkts in Spielpunkten.
    let h = 0.5 * gezeigt / rahmen.ziel.xy;
    let lx = lueckenanteil(t.x - h.x, t.x + h.x, breite);
    let ly = lueckenanteil(t.y - h.y, t.y + h.y, breite);
    let dunkel = 1.0 - (1.0 - lx) * (1.0 - ly);
    let zelle = clamp(vec2<i32>(floor(t)), vec2<i32>(0), vec2<i32>(rahmen.quelle.xy) - vec2<i32>(1));
    let c = textureLoad(quelle, zelle, 0).rgb;
    // RGB-Streifen: je Kanal sein Drittel der Zelle, integriert wie oben;
    // mal drei, damit eine weiße Fläche im Mittel hell bleibt.
    let r = streifenanteil(t.x - h.x, t.x + h.x, 0.0, 1.0 / 3.0);
    let g = streifenanteil(t.x - h.x, t.x + h.x, 1.0 / 3.0, 2.0 / 3.0);
    let b = streifenanteil(t.x - h.x, t.x + h.x, 2.0 / 3.0, 1.0);
    let maske = mix(vec3<f32>(1.0), 3.0 * vec3<f32>(r, g, b), subpixel);
    return vec4<f32>(clamp(c * maske * (1.0 - gitter * dunkel), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
