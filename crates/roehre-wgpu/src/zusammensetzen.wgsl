// roehre – Zusammensetzen: setzt die Röhrenfläche pixelgenau ins Ziel, füllt den Rest und
// kodiert für die Ausgabe (SDR, scRGB oder HDR10).
// SPDX-License-Identifier: MIT OR Apache-2.0

struct Daten {
    // Röhrenfläche im Ziel: x, y, Breite, Höhe in Pixeln
    flaeche: vec4<f32>,
    // Hintergrundfarbe (linear, 0…1) und Ausgabeart:
    // 0 = SDR (Ziel ohne sRGB), 1 = SDR (Ziel mit sRGB), 2 = scRGB, 3 = HDR10
    hintergrund: vec4<f32>,
    // HDR: Gamma der Röhrenausgabe, Leuchtdichte von Röhrenwert 1,0 in nit,
    //      Spitzenleuchtdichte in nit, Leuchtdichte des Hintergrunds 1,0 in nit
    hdr: vec4<f32>,
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

// Weiche Begrenzung auf die Spitzenleuchtdichte: bis 75 % linear, darüber asymptotisch.
fn begrenze(nit: vec3<f32>, spitze: f32) -> vec3<f32> {
    let knie = 0.75 * spitze;
    let rest = spitze - knie;
    let weich = knie + rest * (1.0 - exp(-(nit - knie) / rest));
    return select(nit, weich, nit > vec3<f32>(knie));
}

// SMPTE ST 2084 (PQ), Eingang in nit
fn pq(nit: vec3<f32>) -> vec3<f32> {
    let m1 = 2610.0 / 16384.0;
    let m2 = 2523.0 / 4096.0 * 128.0;
    let c1 = 3424.0 / 4096.0;
    let c2 = 2413.0 / 4096.0 * 32.0;
    let c3 = 2392.0 / 4096.0 * 32.0;
    let y = pow(clamp(nit / 10000.0, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(m1));
    return pow((c1 + c2 * y) / (1.0 + c3 * y), vec3<f32>(m2));
}

// ITU-R BT.2087: lineares BT.709 → lineares BT.2020 (Zeilen der Matrix)
fn bt709_zu_bt2020(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(vec3<f32>(0.6274, 0.3293, 0.0433), c),
        dot(vec3<f32>(0.0691, 0.9195, 0.0114), c),
        dot(vec3<f32>(0.0164, 0.0880, 0.8956), c),
    );
}

@fragment
fn farbe(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let art = u32(daten.hintergrund.a + 0.5);
    let p = pos.xy - daten.flaeche.xy;
    let innen = !(p.x < 0.0 || p.y < 0.0 || p.x >= daten.flaeche.z || p.y >= daten.flaeche.w);

    if (art <= 1u) {
        // SDR: Werte der Röhre unverändert durchreichen
        if (!innen) {
            return vec4<f32>(daten.hintergrund.rgb, 1.0);
        }
        var c = textureLoad(roehre, vec2<i32>(p), 0).rgb;
        if (art == 1u) {
            c = srgb_zu_linear(c);
        }
        return vec4<f32>(c, 1.0);
    }

    // HDR: in Leuchtdichte (nit) umrechnen
    var nit: vec3<f32>;
    if (innen) {
        let c = textureLoad(roehre, vec2<i32>(p), 0).rgb;
        nit = pow(c, vec3<f32>(daten.hdr.x)) * daten.hdr.y;
    } else {
        nit = daten.hintergrund.rgb * daten.hdr.w;
    }
    nit = begrenze(nit, daten.hdr.z);
    if (art == 2u) {
        // scRGB: linear BT.709, 1,0 = 80 nit
        return vec4<f32>(nit / 80.0, 1.0);
    }
    // HDR10: BT.2020 mit PQ
    return vec4<f32>(pq(max(bt709_zu_bt2020(nit), vec3<f32>(0.0))), 1.0);
}
