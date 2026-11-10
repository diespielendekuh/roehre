// roehre – Stufe „farbe“ (Wunsch E3b des Terranigma-Ports): Farbkorrektur
// hinter jedem Filter, Punkt für Punkt (Ziel = Quelle, textureLoad).
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Regler (rahmen.regler[0]):
//   x  helligkeit  Faktor auf das Licht (1 = unverändert)
//   y  kontrast    um Mittelgrau 0,18 in linearem Licht (1 = unverändert)
//   z  saettigung  0 = grau, 1 = unverändert, 2 = doppelt
//   w  waerme      −1 kühl … +1 warm (0 = unverändert)
//
// Gerechnet wird in linearem Licht (sRGB dekodiert, danach wieder kodiert):
// Helligkeit und Kontrast wirken dann wie am Bildschirm, Sättigung mischt
// mit der Leuchtdichte nach BT.709. Wärme verschiebt Rot und Blau
// gegeneinander um bis zu ±10 %, Grün bleibt — eine einfache Annäherung an
// eine andere Farbtemperatur, keine Farbmetrik.

fn linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

fn kodiert(l: vec3<f32>) -> vec3<f32> {
    let c = clamp(l, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

@fragment
fn punkt(e: Ecke) -> @location(0) vec4<f32> {
    let r = rahmen.regler[0];
    let p = clamp(vec2<i32>(floor(e.uv * rahmen.quelle.xy)), vec2<i32>(0), vec2<i32>(rahmen.quelle.xy) - vec2<i32>(1));
    var l = linear(textureLoad(quelle, p, 0).rgb) * r.x;
    l = max(0.18 * pow(max(l, vec3<f32>(0.0)) / 0.18, vec3<f32>(r.y)), vec3<f32>(0.0));
    let y = dot(l, vec3<f32>(0.2126, 0.7152, 0.0722));
    l = mix(vec3<f32>(y), l, r.z);
    l = l * vec3<f32>(1.0 + 0.1 * r.w, 1.0, 1.0 - 0.1 * r.w);
    return vec4<f32>(kodiert(l), 1.0);
}
