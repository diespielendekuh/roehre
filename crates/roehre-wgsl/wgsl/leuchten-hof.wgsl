// roehre – Stufe „leuchten-hof“ (Wunsch E3b des Terranigma-Ports): der Hof
// des Leuchtens in der Auflösung des Originals; Zusatzstufe von „leuchten“.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Regler (rahmen.regler[0]):
//   x  weite     Standardabweichung σ des Gauß-Kerns in Spielpunkten
//   y  schwelle  Leuchtdichte (linear), ab der ein Punkt leuchtet
//
// Je Punkt des Originals: Summe über alle Punkte im Umkreis von 3 σ (genau
// gelesen, kein Abtaster), gewogen mit exp(−d²/2σ²) und auf die Summe der
// Gewichte bezogen. Von jedem Punkt zählt nur der Teil über der Schwelle
// (nach Leuchtdichte BT.709, auf 0 … 1 gespreizt, Farbe bleibt). Außerhalb
// des Bildes ist es schwarz (kein Spiegeln, kein Wiederholen des Randes): das
// Gewicht dort zählt in der Bezugssumme mit, ohne Licht beizutragen — wie
// der schwarze Rand eines Spielbilds. Ergebnis sRGB-kodiert wie jede Stufe.

fn linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

fn kodiert(l: vec3<f32>) -> vec3<f32> {
    let c = clamp(l, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

@fragment
fn punkt(e: Ecke) -> @location(0) vec4<f32> {
    let sigma = clamp(rahmen.regler[0].x, 0.25, 4.0);
    let schwelle = clamp(rahmen.regler[0].y, 0.0, 0.95);
    let groesse = vec2<i32>(textureDimensions(quelle));
    let p = clamp(vec2<i32>(floor(e.uv * rahmen.quelle.xy)), vec2<i32>(0), groesse - vec2<i32>(1));
    let r = i32(ceil(3.0 * sigma));
    var summe = vec3<f32>(0.0);
    var gewicht = 0.0;
    for (var j = -r; j <= r; j++) {
        for (var i = -r; i <= r; i++) {
            let d = vec2<f32>(f32(i), f32(j));
            let dd = dot(d, d);
            if dd > 9.0 * sigma * sigma {
                continue;
            }
            let w = exp(-dd / (2.0 * sigma * sigma));
            gewicht += w;
            let t = p + vec2<i32>(i, j);
            if any(t < vec2<i32>(0)) || any(t >= groesse) {
                continue;
            }
            let c = linear(textureLoad(quelle, t, 0).rgb);
            let y = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
            summe += w * c * (max(y - schwelle, 0.0) / max(y, 1e-4) / (1.0 - schwelle));
        }
    }
    return vec4<f32>(kodiert(summe / gewicht), 1.0);
}
