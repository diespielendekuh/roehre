// roehre – Stufen „naechster“ und „linear“: das Bild einfach eingepaßt, mit
// dem Abtaster der Stufe (nächster Punkt bzw. linear). Für Spiele, die alle
// Filter über denselben Ausführer fahren wollen.
// SPDX-License-Identifier: MIT OR Apache-2.0

@fragment
fn punkt(e: Ecke) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSampleLevel(quelle, abtaster, e.uv, 0.0).rgb, 1.0);
}
