// roehre – Stufe „leuchten“ (Wunsch E3b des Terranigma-Ports): leichtes
// Leuchten (Bloom) hinter einem Filter, Punkt für Punkt (Ziel = Quelle).
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Diese Stufe liest außer der Quelle (dem gefilterten Bild) an
// `@binding(3)` den HOF: das Ergebnis der Zusatzstufe „leuchten-hof“ auf dem
// Original (dem Spielbild in seiner eigenen Auflösung, derselbe Ausschnitt).
// Der Hof entsteht so in Spielpunkten — die Weite hängt nicht von der
// Fenstergröße ab, und ein Kantenglätter vor dieser Stufe (MMPX, ScaleFX)
// sieht das Leuchten nie, es verfälschte sonst seine Kantenerkennung. Hier
// wird der Hof nur noch linear abgetastet (zwischen den Spielpunkten
// glatt, nach außen nie zunehmend).
//
// Regler (rahmen.regler[0]):
//   x  staerke   0 = aus (Voreinstellung; der Ausführer läßt die Stufe weg) … 1
//   y  weite     (liest nur die Zusatzstufe)
//   z  schwelle  (liest nur die Zusatzstufe)
//
// Das Leuchten kommt in linearem Licht wie beim »Aufhellen« (screen) dazu:
// q + s · h · (1 − q) — es überstrahlt nie über Weiß hinaus und läßt Schwarz
// außerhalb des Hofs schwarz.

@group(0) @binding(3) var original: texture_2d<f32>;

fn linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

fn kodiert(l: vec3<f32>) -> vec3<f32> {
    let c = clamp(l, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

@fragment
fn punkt(e: Ecke) -> @location(0) vec4<f32> {
    let p = clamp(vec2<i32>(floor(e.uv * rahmen.quelle.xy)), vec2<i32>(0), vec2<i32>(rahmen.quelle.xy) - vec2<i32>(1));
    let q = linear(textureLoad(quelle, p, 0).rgb);
    let staerke = clamp(rahmen.regler[0].x, 0.0, 1.0);
    let h = linear(textureSampleLevel(original, abtaster, e.uv, 0.0).rgb);
    return vec4<f32>(kodiert(q + staerke * h * (vec3<f32>(1.0) - q)), 1.0);
}
