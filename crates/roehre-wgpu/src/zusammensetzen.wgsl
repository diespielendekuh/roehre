// roehre – Zusammensetzen: setzt die Röhrenfläche pixelgenau ins Ziel und füllt den Rest.
// SPDX-License-Identifier: MIT OR Apache-2.0

struct Daten {
    // Röhrenfläche im Ziel: x, y, Breite, Höhe in Pixeln
    flaeche: vec4<f32>,
    // Hintergrundfarbe (linear) und 1.0, wenn das Ziel ein sRGB-Format ist
    hintergrund: vec4<f32>,
};

@group(0) @binding(0) var<uniform> daten: Daten;
@group(0) @binding(1) var roehre: texture_2d<f32>;

@vertex
fn ecke(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // Ein Dreieck, das den ganzen Bildschirm überdeckt
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

// sRGB-Dekodierung (IEC 61966-2-1): die Hardware kodiert beim Schreiben wieder,
// sodass im Ziel genau die Werte landen, die die Röhre berechnet hat.
fn srgb_zu_linear(c: vec3<f32>) -> vec3<f32> {
    let klein = c / 12.92;
    let gross = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(gross, klein, c <= vec3<f32>(0.04045));
}

@fragment
fn farbe(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pos.xy - daten.flaeche.xy;
    if (p.x < 0.0 || p.y < 0.0 || p.x >= daten.flaeche.z || p.y >= daten.flaeche.w) {
        return vec4<f32>(daten.hintergrund.rgb, 1.0);
    }
    var c = textureLoad(roehre, vec2<i32>(p), 0).rgb;
    if (daten.hintergrund.a > 0.5) {
        c = srgb_zu_linear(c);
    }
    return vec4<f32>(c, 1.0);
}
