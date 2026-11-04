// roehre – Stufe „scharf-bilinear“ als WGSL (W-009), übertragen aus
// shader/eigen/scharf-bilinear.slang (eigene Arbeit von roehre).
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Vergrößert pixelgenau ohne Unschärfe, auch bei nicht ganzzahligen Faktoren:
// innerhalb eines Quellpunkts wird nicht gefiltert, nur auf dem schmalen Saum
// zwischen zwei Punkten (Breite genau ein Bildpunkt, je Achse) wird linear
// überblendet. Erwartet einen linear filternden Abtaster.
//
// Einziger Unterschied zur slang-Fassung: der Faktor bezieht sich auf den
// gezeigten Ausschnitt (`rahmen.ausschnitt`), nicht auf die ganze Textur —
// bei (0,0,1,1) rechnet sie gleich. Am Rand des Ausschnitts liest sie keinen
// Punkt von außerhalb: der erste Bildpunkt trifft t = 1, also die Mitte des
// ersten gezeigten Quellpunkts.
//
// Regler (rahmen.regler[0].x) »Weichheit« 0 … 1: verbreitert den Saum von
// genau einem Bildpunkt (0, Voreinstellung, gleich der slang-Fassung) bis zu
// einem ganzen Quellpunkt (1, dann rechnet die Stufe dasselbe wie lineares
// Abtasten). Dazwischen linear in der Saumbreite.

@fragment
fn punkt(e: Ecke) -> @location(0) vec4<f32> {
    let texel = e.uv * rahmen.quelle.xy;
    // Bildpunkte je Quellpunkt, getrennt nach Achsen.
    let gezeigt = (rahmen.ausschnitt.zw - rahmen.ausschnitt.xy) * rahmen.quelle.xy;
    let faktor = max(rahmen.ziel.xy / gezeigt, vec2<f32>(1.0));
    let ganz = floor(texel - 0.5);
    let bruch = texel - 0.5 - ganz;
    // Überblendung nur in einem Saum von 1/faktor Breite um die Punktgrenze.
    let weichheit = clamp(rahmen.regler[0].x, 0.0, 1.0);
    let saum = mix(0.5 / faktor, vec2<f32>(0.5), weichheit);
    let t = clamp((bruch - 0.5 + saum) / (2.0 * saum), vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = (ganz + 0.5 + t) * rahmen.quelle.zw;
    return vec4<f32>(textureSampleLevel(quelle, abtaster, uv, 0.0).rgb, 1.0);
}
