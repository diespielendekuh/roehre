// roehre – Zusammensetzen: setzt die Röhrenfläche pixelgenau ins Ziel, zeichnet auf Wunsch
// Glas und Gehäuse und kodiert für die Ausgabe (SDR, scRGB oder HDR10).
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Das Gehäuse ist vollständig berechnet (Signed Distance Fields), es gibt keine Bilddateien.
// Längen der Gehäuseform stehen in halben Glashöhen (Glas senkrecht −1 … 1).

struct Daten {
    // Bildfläche (dort zeichnet die Röhre): x, y, Breite, Höhe in Pixeln
    inhalt: vec4<f32>,
    // Glas: Mitte x, Mitte y, halbe Breite, halbe Höhe in Pixeln
    glas: vec4<f32>,
    // Hintergrundfarbe (linear) und Ausgabeart:
    // 0 = SDR (Ziel ohne sRGB), 1 = SDR (Ziel mit sRGB), 2 = scRGB, 3 = HDR10
    hintergrund: vec4<f32>,
    // HDR: Gamma der Röhrenausgabe, Leuchtdichte von Röhrenwert 1,0 in nit,
    //      Spitzenleuchtdichte in nit, Leuchtdichte von Weiß (1,0) in nit
    hdr: vec4<f32>,
    // Gehäuse: Art (0 = ohne, 1 = nur Glas, 2 = Fernseher, 3 = Monitor, 4 = Automat),
    //          Eckenradius Glas, Breite der Innenfase, Eckenradius außen
    form1: vec4<f32>,
    // Rahmenbreite seitlich, oben, unten; Rauheit
    form2: vec4<f32>,
    // Kunststofffarbe (linear), höchste Mip-Stufe der Röhrentextur
    farbe: vec4<f32>,
    // Spiegelung, Licht des Bildes, Raumlicht, Glanz
    licht: vec4<f32>,
};

@group(0) @binding(0) var<uniform> daten: Daten;
// Röhrenbild: rohe Werte (für das Bild selbst, Punkt für Punkt) …
@group(0) @binding(1) var roehre: texture_2d<f32>;
// … und mit Mipmaps über eine sRGB-Ansicht (für Spiegelung und Umgebungslicht, gefiltert)
@group(0) @binding(2) var roehre_mip: texture_2d<f32>;
@group(0) @binding(3) var abtaster: sampler;

@vertex
fn ecke(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

// sRGB-Dekodierung (IEC 61966-2-1)
fn srgb_zu_linear(c: vec3<f32>) -> vec3<f32> {
    let klein = c / 12.92;
    let gross = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(gross, klein, c <= vec3<f32>(0.04045));
}

fn linear_zu_srgb(c: vec3<f32>) -> vec3<f32> {
    let c2 = max(c, vec3<f32>(0.0));
    let klein = c2 * 12.92;
    let gross = 1.055 * pow(c2, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(gross, klein, c2 <= vec3<f32>(0.0031308));
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

// ITU-R BT.2087: lineares BT.709 → lineares BT.2020
fn bt709_zu_bt2020(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(vec3<f32>(0.6274, 0.3293, 0.0433), c),
        dot(vec3<f32>(0.0691, 0.9195, 0.0114), c),
        dot(vec3<f32>(0.0164, 0.0880, 0.8956), c),
    );
}

// Abstand zu einem Rechteck mit abgerundeten Ecken (negativ innen)
fn sd_rund(q: vec2<f32>, halb: vec2<f32>, r: f32) -> f32 {
    let d = abs(q) - halb + vec2<f32>(r);
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

// Richtung, in der der Abstand wächst (Normale in der Bildebene)
fn sd_richtung(q: vec2<f32>, halb: vec2<f32>, r: f32) -> vec2<f32> {
    let e = 0.002;
    let g = vec2<f32>(
        sd_rund(q + vec2<f32>(e, 0.0), halb, r) - sd_rund(q - vec2<f32>(e, 0.0), halb, r),
        sd_rund(q + vec2<f32>(0.0, e), halb, r) - sd_rund(q - vec2<f32>(0.0, e), halb, r),
    );
    return g / max(length(g), 1e-6);
}

fn zufall(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// Röhrenbild an einem Pixel, linear relativ zu Weiß = 1
fn roehre_linear(p: vec2<f32>) -> vec3<f32> {
    let c = textureLoad(roehre, vec2<i32>(p - daten.inhalt.xy), 0).rgb;
    if (daten.hintergrund.a >= 1.5) {
        return pow(c, vec3<f32>(daten.hdr.x)) * daten.hdr.y / daten.hdr.w;
    }
    return srgb_zu_linear(c);
}

// Röhrenbild gefiltert (Mip-Stufe `stufe`) an einer Stelle in Pixeln; außerhalb schwarz
fn roehre_weich(p: vec2<f32>, stufe: f32) -> vec3<f32> {
    let uv = (p - daten.inhalt.xy) / daten.inhalt.zw;
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return vec3<f32>(0.0);
    }
    var c = textureSampleLevel(roehre_mip, abtaster, uv, stufe).rgb;
    if (daten.hintergrund.a >= 1.5) {
        c = c * daten.hdr.y / daten.hdr.w;
    }
    return c;
}

// Ausgabe kodieren; `l` ist linear relativ zu Weiß = 1
fn ausgeben(l: vec3<f32>) -> vec4<f32> {
    let art = u32(daten.hintergrund.a + 0.5);
    if (art == 0u) {
        return vec4<f32>(linear_zu_srgb(l), 1.0);
    }
    if (art == 1u) {
        return vec4<f32>(l, 1.0);
    }
    let nit = begrenze(max(l, vec3<f32>(0.0)) * daten.hdr.w, daten.hdr.z);
    if (art == 2u) {
        return vec4<f32>(nit / 80.0, 1.0);
    }
    return vec4<f32>(pq(max(bt709_zu_bt2020(nit), vec3<f32>(0.0))), 1.0);
}

// Ohne Gehäuse: der bisherige, exakte Weg (Werte der Röhre unverändert)
fn ohne_gehaeuse(pos: vec2<f32>) -> vec4<f32> {
    let art = u32(daten.hintergrund.a + 0.5);
    let p = pos - daten.inhalt.xy;
    let innen = !(p.x < 0.0 || p.y < 0.0 || p.x >= daten.inhalt.z || p.y >= daten.inhalt.w);
    if (art <= 1u) {
        if (!innen) {
            return vec4<f32>(daten.hintergrund.rgb, 1.0);
        }
        var c = textureLoad(roehre, vec2<i32>(p), 0).rgb;
        if (art == 1u) {
            c = srgb_zu_linear(c);
        }
        return vec4<f32>(c, 1.0);
    }
    var nit: vec3<f32>;
    if (innen) {
        let c = textureLoad(roehre, vec2<i32>(p), 0).rgb;
        nit = pow(c, vec3<f32>(daten.hdr.x)) * daten.hdr.y;
    } else {
        nit = daten.hintergrund.rgb * daten.hdr.w;
    }
    nit = begrenze(nit, daten.hdr.z);
    if (art == 2u) {
        return vec4<f32>(nit / 80.0, 1.0);
    }
    return vec4<f32>(pq(max(bt709_zu_bt2020(nit), vec3<f32>(0.0))), 1.0);
}

@fragment
fn farbe(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let form = u32(daten.form1.x + 0.5);
    if (form == 0u) {
        return ohne_gehaeuse(pos.xy);
    }

    let hh = daten.glas.w;                       // halbe Glashöhe in Pixeln
    let a = daten.glas.z / hh;                   // Seitenverhältnis des Glases
    let px = 1.0 / hh;                           // ein Pixel in halben Glashöhen
    let q = (pos.xy - daten.glas.xy) / hh;       // Lage in halben Glashöhen
    let r_glas = daten.form1.y;
    let fase = daten.form1.z;
    let d_glas = sd_rund(q, vec2<f32>(a, 1.0), r_glas);
    let spiegelung = daten.licht.x;
    let bildlicht = daten.licht.y;
    let raumlicht = daten.licht.z;
    let glanz = daten.licht.w;
    let mip_max = daten.farbe.w;

    // Mittlere Farbe des Bildes (kleinste Mip-Stufe): Licht, das das Bild in den Raum wirft
    let mittel = roehre_weich(daten.inhalt.xy + 0.5 * daten.inhalt.zw, mip_max);

    // --- Glas -----------------------------------------------------------------------------
    var glas: vec3<f32>;
    {
        let p = pos.xy - daten.inhalt.xy;
        let im_bild = !(p.x < 0.0 || p.y < 0.0 || p.x >= daten.inhalt.z || p.y >= daten.inhalt.w);
        // Hinter dem Glas außerhalb des Bildes: unbeleuchteter Leuchtstoff, fast schwarz
        glas = select(vec3<f32>(0.004), roehre_linear(pos.xy), im_bild);
        // Zum Rand des Glases hin etwas dunkler (Wölbung und Glasdicke)
        glas *= 1.0 - 0.3 * exp(d_glas / 0.05);
        // Glanz: weicher Widerschein einer Lichtquelle oben links und schwacher Saum oben
        let fleck = exp(-(pow((q.x + 0.45 * a) / (0.55 * a), 2.0) + pow((q.y + 0.6) / 0.35, 2.0)));
        let saum = exp((d_glas + 0.01) / 0.02) * smoothstep(0.2, -0.9, q.y);
        glas += glanz * (0.03 * fleck + 0.015 * saum) * vec3<f32>(0.95, 0.97, 1.0);
    }
    if (form == 1u) {
        // Nur Glas: außerhalb Hintergrund, Kante kantengeglättet
        let deckung = clamp(0.5 - d_glas / px, 0.0, 1.0);
        let wand = daten.hintergrund.rgb;
        return ausgeben(mix(wand, glas, deckung));
    }

    // --- Rahmen ---------------------------------------------------------------------------
    let seite = daten.form2.x;
    let oben = daten.form2.y;
    let unten = daten.form2.z;
    let rauheit = daten.form2.w;
    let kunststoff = daten.farbe.rgb;
    let mitte_y = (unten - oben) / 2.0;
    let halb_aussen = vec2<f32>(a + seite, 1.0 + (oben + unten) / 2.0);
    let qa = q - vec2<f32>(0.0, mitte_y);
    let d_aussen = sd_rund(qa, halb_aussen, daten.form1.w);

    // Licht des Bildes auf dem Rahmen: stark an der Glaskante, nach außen abfallend
    let bildlicht_hier = mittel * bildlicht * 1.2 * exp(-max(d_glas, 0.0) * 3.5);
    var rahmen: vec3<f32>;
    if (d_glas < fase) {
        // Innenfase: schräg zum Glas hin, im Schatten des Rahmens. In ihr spiegelt sich das
        // Bild: Punkt an der Glaskante gespiegelt, mit zunehmendem Abstand unschärfer.
        let t = clamp(d_glas / max(fase, 1e-4), 0.0, 1.0);
        let n = sd_richtung(q, vec2<f32>(a, 1.0), r_glas);
        let gespiegelt = daten.glas.xy + (q - n * (2.0 * max(d_glas, 0.0) + 0.01)) * hh;
        let stufe = clamp(log2(1.0 + t * fase * hh * 0.5), 0.0, mip_max);
        let bild = roehre_weich(gespiegelt, stufe);
        let fresnel = 0.04 + 0.5 * pow(1.0 - t, 3.0);
        rahmen = kunststoff * (0.35 * raumlicht + 0.8 * bildlicht_hier)
            + bild * spiegelung * fresnel * (1.0 - 0.7 * rauheit);
    } else {
        // Vorderseite: Raumlicht von oben, Rundung an der Außenkante mit Glanzlicht oben und
        // Schatten unten, dazu das Licht des Bildes
        let rand = smoothstep(-0.04, 0.0, d_aussen);
        let n = sd_richtung(qa, halb_aussen, daten.form1.w);
        let oben_licht = rand * max(-n.y, 0.0);
        let unten_schatten = rand * max(n.y, 0.0);
        let streu = 0.55 + 0.25 * clamp(-qa.y / halb_aussen.y, -1.0, 1.0);
        // Lippe: die Kante zwischen Innenfase und Vorderseite, nach innen geneigt. Unterhalb
        // des Glases zeigt sie nach oben und fängt Raumlicht, oberhalb liegt sie im Schatten.
        let lippe = exp(-pow((d_glas - fase - 0.012) / 0.012, 2.0));
        let ng = sd_richtung(q, vec2<f32>(a, 1.0), r_glas);
        rahmen = kunststoff * raumlicht * (streu + 1.2 * oben_licht - 0.5 * unten_schatten)
            + (kunststoff + vec3<f32>(0.04 * (1.0 - rauheit))) * bildlicht_hier
            + vec3<f32>(0.08 * (1.0 - rauheit)) * raumlicht * oben_licht
            + vec3<f32>(0.10 * (1.0 - 0.5 * rauheit)) * raumlicht * lippe * max(ng.y, 0.0)
            + (kunststoff * 2.0 + vec3<f32>(0.03)) * bildlicht_hier * lippe;
        // Feine Körnung des Kunststoffs (verhindert auch Farbstufen)
        rahmen *= 1.0 + (zufall(floor(pos.xy)) - 0.5) * 0.05;

        if (form == 2u) {
            // Fernseher: Lautsprechergitter unter dem Glas (waagerechte Schlitze) …
            let y0 = 1.0 + 0.1;
            let y1 = 1.0 + unten - 0.1;
            if (q.y > y0 && q.y < y1 && abs(q.x) < a * 0.7) {
                let zeile = (q.y - y0) / 0.045;
                let ly = (fract(zeile) - 0.5) * 0.045;
                let schlitz = sd_rund(vec2<f32>(q.x, ly), vec2<f32>(a * 0.7, 0.008), 0.008);
                rahmen *= mix(0.25, 1.0, smoothstep(-px, px, schlitz));
            }
            // … und eine kleine Betriebsanzeige
            let led = length(q - vec2<f32>(a + seite * 0.45, 1.0 + unten * 0.5)) - 0.012;
            rahmen += vec3<f32>(0.9, 0.05, 0.02) * smoothstep(px, -px, led) * 0.6;
            rahmen += vec3<f32>(0.9, 0.05, 0.02) * exp(-max(led, 0.0) / 0.01) * 0.03;
        }
    }

    // --- Wand dahinter ---------------------------------------------------------------------
    let wand = daten.hintergrund.rgb + vec3<f32>(0.008, 0.008, 0.009) * raumlicht
        + mittel * bildlicht * 0.15 * exp(-max(d_aussen, 0.0) * 1.5);

    // Zusammensetzen mit Kantenglättung: Glas → Rahmen → Wand
    let auf_glas = clamp(0.5 - d_glas / px, 0.0, 1.0);
    let im_gehaeuse = clamp(0.5 - d_aussen / px, 0.0, 1.0);
    let l = mix(wand, mix(rahmen, glas, auf_glas), im_gehaeuse);
    return ausgeben(l);
}
