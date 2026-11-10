// roehre – 3D-Vorführraum: Wohnzimmer bei Nacht mit Röhrenfernseher (Rasterung).
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Diese Datei wird hinter shader/eigen/flaeche/roehre_maske.wgsl gehängt. Drei Pässe:
// 1. `schatten_*`: Tiefe aus Sicht der Lampe (Schattenkarte).
// 2. `haupt_*`: Szene in linearem Licht (HDR), 4-fach kantengeglättet.
// 3. `nach_*`: Tiefenunschärfe, Glühen, Tonwert, Farbstimmung, Filmkorn, Vignette.
//    Der Hauptpass schreibt dafür den Abstand zur Kamera in den Alphakanal.
//
// Stil: späte PS2-Zeit (Silent Hill 2): wenige Polygone, dichte Materialien, Nebel, Korn,
// warmes Lampenlicht gegen kühle Nacht, das Bild des Fernsehers als Lichtquelle im Raum.

struct Szene {
    ansicht: mat4x4<f32>,
    licht: mat4x4<f32>,
    // Kamera (xyz), Zeit in Sekunden
    auge: vec4<f32>,
    // Lampe (xyz), Helligkeit
    lampe: vec4<f32>,
    // Mitte des Bildschirms (xyz), Helligkeit des Bildschirms
    bild_mitte: vec4<f32>,
    // halbe Breite/Höhe der Bildröhre, höchste Mip-Stufe des Röhrenbilds, Flackern der Lampe
    bild_halb: vec4<f32>,
    // Nebeldichte, Filmkorn, Ausgabe (1 = Ziel mit sRGB), Belichtung
    nachbild: vec4<f32>,
    // Abstand der Schärfeebene (m), Stärke der Tiefenunschärfe, Stärke des Glühens, frei
    fokus: vec4<f32>,
    // Rohpixel (1 = an), Stärke der Vignette, Leben der Szene (0 … 1), frei
    raumbild: vec4<f32>,
    // Park: Laternen 0–2 (xyz, Index ihrer Kulisse), [3].x Zahl der Kulissen, ab 4 je Kulisse
    // drei Vektoren (Mitte x, Fuß y, z, Breite), (Höhe, u0, du, Ebene), (v0, dv, –, –)
    park: array<vec4<f32>, 56>,
    // Sicht des Bildschirms für seine Schattenkarte
    licht_bild: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> szene: Szene;
@group(0) @binding(1) var<uniform> maske: RoehreMaske;
@group(0) @binding(2) var farben: texture_2d_array<f32>;
@group(0) @binding(3) var eigenschaften: texture_2d_array<f32>;
@group(0) @binding(4) var kachel: sampler;
@group(0) @binding(5) var bildschirm: texture_2d<f32>;
@group(0) @binding(6) var rand: sampler;
@group(0) @binding(7) var schattenkarte: texture_depth_2d;
@group(0) @binding(8) var vergleich: sampler_comparison;
@group(0) @binding(9) var bildschattenkarte: texture_depth_2d;

// Materialnummern (wie texturen.rs)
const M_KUNSTSTOFF_SCHWARZ: u32 = 1u;
const M_CHROM: u32 = 2u;
const M_BLATT: u32 = 10u;
const M_LAMPENSCHIRM: u32 = 17u;
const M_HIMMEL: u32 = 19u;
const M_GLUEHBIRNE: u32 = 20u;
const M_LED: u32 = 21u;
const M_BILDROEHRE: u32 = 26u;
const M_WOLKEN: u32 = 40u;
const M_PARK: u32 = 41u;
const M_BAUM: u32 = 42u;
const M_FERNE: u32 = 43u;
const M_LATERNE: u32 = 44u;
const M_AUTO: u32 = 45u;
const M_PARKHOLZ: u32 = 48u;
const M_PARKEISEN: u32 = 49u;

// Draußen (Himmel, Park, Bäume, Ferne, Laternen, Auto): gemalt, leuchtet selbst, ohne
// Zimmerlicht und ohne Zimmernebel
fn draussen(m: u32) -> bool {
    return m == M_HIMMEL || (m >= 40u && m <= 45u) || m == M_PARKHOLZ || m == M_PARKEISEN;
}

// Leben der Szene (Option raum.leben): Wolken, Auto, Bäume im Wind
fn leben() -> f32 {
    return szene.raumbild.z;
}

// Ein Auto fährt alle 21 s auf der Straße hinter dem Park von rechts nach links (6,5 s).
// Ergebnis: Lage (x verschiebt die Kulisse) und Stärke seines Lichts am Fenster (0 außerhalb
// der Fahrt, in der Mitte am hellsten).
fn auto_licht(t: f32) -> vec4<f32> {
    let periode = 21.0;
    let dauer = 6.5;
    let ph = (t + 9.0) - floor((t + 9.0) / periode) * periode;
    if (ph > dauer || leben() <= 0.0) {
        return vec4<f32>(60.0, 1.5, 14.3, 0.0);
    }
    let s = ph / dauer;
    let staerke = pow(sin(3.14159265 * s), 2.0) * leben();
    return vec4<f32>(7.0 - 18.0 * s, 1.5, 14.3, staerke);
}

// Richtung zum Mond und Wolkenlage am Himmel (aus der Blickrichtung, von der Textur
// unabhängig). Die Wolken ziehen sehr langsam nach links.
const MOND: vec3<f32> = vec3<f32>(-0.2577, 0.1909, 0.9472);
fn wolken_uv(d: vec3<f32>) -> vec2<f32> {
    let t = szene.auge.w;
    return vec2<f32>(d.x / d.z, d.y / max(d.z, 0.05)) * vec2<f32>(0.8, 1.6) + vec2<f32>(t * 0.006 * leben() + 0.31, 0.12);
}
// Wie dicht die Wolken gerade vor dem Mond stehen (0 … 1): dimmt das Mondlicht im Zimmer
fn wolke_vor_mond() -> f32 {
    return textureSampleLevel(farben, kachel, wolken_uv(MOND), M_WOLKEN, 3.0).a;
}

// Wie viel Licht von `l` bei `p` ankommt (0 … 1): der Strahl wird gegen jede Kulisse des
// Parks (Bäume, Laternen, Bank) geprüft; trifft er sie innerhalb ihres Rechtecks, hält sie
// so viel Licht ab, wie ihre Textur dort deckt (Alpha, leicht verwischt aus Mip-Stufe 1).
// `ohne` ist die Kulisse, zu der das Licht selbst gehört (die Laterne verschattet sich nicht).
fn park_schatten(p: vec3<f32>, l: vec3<f32>, ohne: i32) -> f32 {
    let anzahl = i32(szene.park[3].x);
    var s = 1.0;
    for (var i = 0; i < anzahl; i++) {
        if (i == ohne) { continue; }
        let a = szene.park[4 + 3 * i];
        let b = szene.park[5 + 3 * i];
        let v = szene.park[6 + 3 * i];
        let dp = p.z - a.z;
        let dl = l.z - a.z;
        if (dp * dl >= 0.0) { continue; }
        let h = p + (l - p) * (dp / (dp - dl));
        let su = (h.x - a.x) / a.w + 0.5;
        let q = (h.y - a.y) / b.x;
        if (su < 0.0 || su > 1.0 || q < 0.0 || q > 1.0) { continue; }
        let uv = vec2<f32>(b.y + su * b.z, v.x + (1.0 - q) * v.y);
        s *= 1.0 - 0.92 * textureSampleLevel(farben, kachel, uv, i32(b.w), 1.0).a;
    }
    // Kästen der Bank (gedreht um y): Strahl im Rahmen des Kastens, Plattentest
    let kaesten = i32(szene.park[3].y);
    let basis = 4 + 3 * anzahl;
    for (var j = 0; j < kaesten; j++) {
        let m = szene.park[basis + 2 * j];
        let h = szene.park[basis + 2 * j + 1].xyz;
        let c = cos(m.w);
        let sn = sin(m.w);
        let zu_lokal = mat3x3<f32>(vec3<f32>(c, 0.0, sn), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(-sn, 0.0, c));
        let o = zu_lokal * (p - m.xyz);
        let d = zu_lokal * (l - p);
        let inv = 1.0 / select(d, vec3<f32>(1e-6), abs(d) < vec3<f32>(1e-6));
        let t1 = (-h - o) * inv;
        let t2 = (h - o) * inv;
        let nah = max(max(min(t1.x, t2.x), min(t1.y, t2.y)), min(t1.z, t2.z));
        let fern = min(min(max(t1.x, t2.x), max(t1.y, t2.y)), max(t1.z, t2.z));
        if (nah < fern && fern > 0.002 && nah < 1.0) { s *= 0.08; }
    }
    return s;
}

// Licht im Park bei `p` mit Normale `n`: Himmel, Mond (von Wolken gedämpft) und die drei
// Laternen, jeweils mit den Schatten der Kulissen. Kulissen (`karte`) sind flach: sie nehmen
// Licht auch von hinten halb an (Laub lässt es durch), damit ein Baum vor einer Laterne
// nicht schwarz ausgestanzt wirkt.
fn park_licht(p: vec3<f32>, n: vec3<f32>, karte: bool) -> vec3<f32> {
    var e = vec3<f32>(0.010, 0.012, 0.020) * select(0.55 + 0.45 * n.y, 0.6, karte);
    let mond = vec3<f32>(0.06, 0.075, 0.11) * mix(1.0, 0.4, wolke_vor_mond());
    let nm = select(max(dot(n, MOND), 0.0), max(dot(n, MOND) * 0.5 + 0.5, 0.0) * 0.6, karte);
    e += mond * nm * park_schatten(p, p + MOND * 30.0, -1);
    for (var i = 0; i < 3; i++) {
        let lage = szene.park[i];
        // der eigene Mast liegt neben dem Glas: von dort käme nur Streulicht, keine Beleuchtung
        let eigene = szene.park[4 + 3 * i];
        if (karte && abs(p.z - eigene.z) < 0.02 && abs(p.x - eigene.x) < eigene.w * 0.5) { continue; }
        let d = lage.xyz - p;
        let dd = dot(d, d);
        let l = d * inverseSqrt(dd);
        let nl = select(max(dot(n, l), 0.0), max(dot(n, l) * 0.5 + 0.5, 0.0) * 0.8, karte);
        // Der Laternenkopf hat oben eine Kappe: Licht fällt vor allem nach unten, so entsteht
        // um jeden Mast eine Lichtinsel von gut drei Metern statt eines hellen Hangs
        let kegel = mix(0.03, 1.0, smoothstep(0.55, 0.92, l.y));
        e += vec3<f32>(1.0, 0.70, 0.36) * 4.0 / (dd + 0.25) * kegel * nl * park_schatten(p, lage.xyz, i32(lage.w));
    }
    return e;
}

fn himmel(d: vec3<f32>) -> vec3<f32> {
    let t = szene.auge.w;
    let hoehe = clamp(d.y / max(d.z, 0.05) * 2.2 + 0.15, 0.0, 1.0);
    // Verlauf: unten der warme Schein der Stadt, oben tiefes Nachtblau
    var c = mix(vec3<f32>(0.030, 0.026, 0.030), vec3<f32>(0.004, 0.006, 0.016), sqrt(hoehe));
    // Sterne: ein Punkt je Zelle mit geringer Wahrscheinlichkeit, leicht funkelnd
    let q = vec2<f32>(d.x / d.z, d.y / d.z) * 260.0;
    let zelle = floor(q);
    let h = fract(sin(dot(zelle, vec2<f32>(127.1, 311.7))) * 43758.5453);
    if (h > 0.985) {
        let lage = vec2<f32>(fract(h * 17.0), fract(h * 31.0)) * 0.6 + 0.2;
        let punkt = 1.0 - smoothstep(0.05, 0.16, length(fract(q) - lage));
        let funkeln = 0.75 + 0.25 * sin(t * (2.0 + h * 5.0) + h * 40.0);
        c += vec3<f32>(0.55, 0.58, 0.7) * punkt * (h - 0.985) / 0.015 * funkeln * hoehe;
    }
    // Mond mit Hof und dunkleren Meeren
    let w = acos(clamp(dot(d, MOND), -1.0, 1.0));
    let r = 0.0105;
    let scheibe = 1.0 - smoothstep(r * 0.92, r, w);
    let ort = (d - MOND * dot(d, MOND)) / r;
    let meere = 0.82 + 0.18 * sin(ort.x * 5.0 + 1.0) * sin(ort.y * 4.0 + ort.x * 2.0);
    c = mix(c, vec3<f32>(0.95, 0.93, 0.85) * meere, scheibe);
    c += vec3<f32>(0.10, 0.11, 0.13) * exp(-w * 22.0) + vec3<f32>(0.03, 0.035, 0.045) * exp(-w * 5.0);
    // Wolken, vom Mond her aufgehellt
    let wo = textureSampleLevel(farben, kachel, wolken_uv(d), M_WOLKEN, 1.0);
    let lichtsaum = 0.6 + 1.6 * exp(-w * 9.0);
    return mix(c, wo.rgb * lichtsaum, wo.a);
}

struct Ecke {
    @location(0) lage: vec3<f32>,
    @location(1) normale: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) material: u32,
    @location(4) wind: f32,
};

fn bewegt(e: Ecke) -> vec3<f32> {
    // Vorhang: sachte Wellen im Luftzug, unten stärker
    let t = szene.auge.w;
    let w = e.wind * (0.018 * sin(t * 0.7 + e.lage.x * 4.0) + 0.008 * sin(t * 1.3 + e.lage.y * 6.0));
    // Bäume im Park: die Krone wiegt sich seitlich, jeder Baum im eigenen Takt
    if (e.material == M_BAUM) {
        let b = e.wind * leben() * (0.012 * sin(t * 0.55 + e.lage.z * 1.7) + 0.005 * sin(t * 1.4 + e.lage.x));
        return e.lage + vec3<f32>(b, 0.0, 0.0);
    }
    // Auto: entlang der Straße verschoben
    if (e.material == M_AUTO) {
        return e.lage + vec3<f32>(auto_licht(t).x, 0.0, 0.0);
    }
    return e.lage + vec3<f32>(0.0, 0.0, w);
}

// ---------------------------------------------------------------- 1. Schatten
@vertex
fn schatten_ecke(e: Ecke) -> @builtin(position) vec4<f32> {
    if (e.material == M_LAMPENSCHIRM || e.material == M_GLUEHBIRNE) {
        return vec4<f32>(0.0, 0.0, -2.0, 1.0);    // wirft keinen Schatten (Licht sitzt darin)
    }
    return szene.licht * vec4<f32>(bewegt(e), 1.0);
}

// Schattenkarte des Bildschirms: das Glas selbst wirft keinen Schatten
@vertex
fn bildschatten_ecke(e: Ecke) -> @builtin(position) vec4<f32> {
    if (e.material == M_BILDROEHRE) {
        return vec4<f32>(0.0, 0.0, -2.0, 1.0);
    }
    return szene.licht_bild * vec4<f32>(bewegt(e), 1.0);
}

// Wie viel Bildschirmlicht bei `welt` ankommt (0 … 1), aus der Schattenkarte des Bildschirms.
// Nur vor dem Gerät; Bildtunnel und Gehäuse bekommen ihr Licht ohne Karte.
fn bild_schatten(welt: vec3<f32>, n: vec3<f32>) -> f32 {
    if (welt.z > szene.bild_mitte.z - 0.04) { return 1.0; }
    let p = szene.licht_bild * vec4<f32>(welt + n * 0.006, 1.0);
    let ndc = p.xyz / p.w;
    if (p.w <= 0.0 || ndc.z > 1.0 || any(abs(ndc.xy) > vec2<f32>(1.0))) { return 1.0; }
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let pixel = 1.0 / 2048.0;
    var s = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            s += textureSampleCompareLevel(bildschattenkarte, vergleich, uv + vec2<f32>(f32(x), f32(y)) * pixel * 2.0, ndc.z - 0.0008);
        }
    }
    return s / 9.0;
}

// ---------------------------------------------------------------- 2. Szene
struct Zwischen {
    @builtin(position) pos: vec4<f32>,
    @location(0) welt: vec3<f32>,
    @location(1) normale: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material: u32,
};

@vertex
fn haupt_ecke(e: Ecke) -> Zwischen {
    let p = bewegt(e);
    return Zwischen(szene.ansicht * vec4<f32>(p, 1.0), p, e.normale, e.uv, e.material);
}

// `n` ist die Normale der Fläche ohne Relief (das Relief würde den Versatz verrauschen).
fn schatten(welt: vec3<f32>, n: vec3<f32>) -> f32 {
    let p = szene.licht * vec4<f32>(welt + n * 0.01, 1.0);
    let ndc = p.xyz / p.w;
    if (ndc.z > 1.0 || any(abs(ndc.xy) > vec2<f32>(1.0))) { return 1.0; }
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let pixel = 1.0 / 2048.0;
    var s = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            s += textureSampleCompareLevel(schattenkarte, vergleich, uv + vec2<f32>(f32(x), f32(y)) * pixel * 1.5, ndc.z - 0.0015);
        }
    }
    return s / 9.0;
}

fn lampe_farbe() -> vec3<f32> {
    let t = szene.auge.w;
    let flackern = 1.0 - szene.bild_halb.w * (0.05 * sin(t * 7.3) * sin(t * 2.1) + 0.03 * fract(sin(floor(t * 14.0) * 91.7) * 4375.5));
    return vec3<f32>(1.0, 0.70, 0.42) * szene.lampe.w * flackern;
}

// Helligkeit des Bildes auf der Röhre. Der Raum wird mit hoher Belichtung (nachbild.w, 2,6) und
// einem filmischen Tonwert abgebildet, weil er dunkel ist. Ginge das Röhrenbild ungeschützt
// denselben Weg, läge mittleres Grau (0,5) nach dem Tonwert bei 0,85 und Weiß bei 0,94: das Bild
// fräße aus, helle Szenen wären nur noch weiß (Rückmeldung des Anwenders, 2026-11-10).
// Deshalb wird hier festgelegt, welcher Wert am Ende auf dem Bildschirm des Spielers stehen soll,
// und die Strahldichte dazu aus der Umkehrung von Tonwert und Belichtung berechnet:
//   Sollwert = Röhrenbild × raum.bildhelligkeit, bis 0,8 unverändert, darüber eine weiche
//   Schulter gegen 1 (Weiß bei Helligkeit 1 landet bei 0,93: hell, aber mit Zeichnung).
fn bild_schulter(v: vec3<f32>) -> vec3<f32> {
    let knie = vec3<f32>(0.8);
    let oben = knie + 0.2 * (vec3<f32>(1.0) - exp(-(max(v, knie) - knie) / 0.2));
    return select(v, oben, v > knie);
}

// Umkehrung des filmischen Tonwerts aus `nach_*` (c·(2,51c+0,03)/(c·(2,43c+0,59)+0,14)).
fn tonwert_umkehr(y_roh: vec3<f32>) -> vec3<f32> {
    let y = clamp(y_roh, vec3<f32>(0.0), vec3<f32>(0.985));
    let a = vec3<f32>(2.51) - 2.43 * y;
    let b = vec3<f32>(0.03) - 0.59 * y;
    return (-b + sqrt(b * b + 4.0 * a * 0.14 * y)) / (2.0 * a);
}

fn bild_strahldichte(soll: vec3<f32>) -> vec3<f32> {
    return tonwert_umkehr(bild_schulter(soll)) / max(szene.nachbild.w, 0.01);
}

fn bildlicht() -> vec3<f32> {
    // mittlere Farbe des Bildes (kleinste Mip-Stufe) als Leuchtdichte des Bildschirms
    return textureSampleLevel(bildschirm, rand, vec2<f32>(0.5), szene.bild_halb.z).rgb * szene.bild_mitte.w;
}

// Grobe Umgebung für Spiegelungen (Chrom, Glas, Lack): Raum, Lampe, Fenster, Bildschirm
fn umgebung(r: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    var c = mix(vec3<f32>(0.004, 0.0035, 0.003), vec3<f32>(0.012, 0.012, 0.014), r.y * 0.5 + 0.5);
    let zur_lampe = normalize(szene.lampe.xyz - p);
    c += lampe_farbe() * (pow(max(dot(r, zur_lampe), 0.0), 400.0) * 3.0 + pow(max(dot(r, zur_lampe), 0.0), 8.0) * 0.03);
    let zum_bild = normalize(szene.bild_mitte.xyz - p);
    c += bildlicht() * pow(max(dot(r, zum_bild), 0.0), 30.0) * 0.4;
    let zum_fenster = normalize(vec3<f32>(-1.35, 1.55, 1.1) - p);
    c += vec3<f32>(0.02, 0.025, 0.04) * pow(max(dot(r, zum_fenster), 0.0), 20.0);
    return c;
}

fn umgebungslicht(n: vec3<f32>) -> vec3<f32> {
    // Himmel durch das Fenster (kühl) gegen warmen Widerschein vom Boden
    return mix(vec3<f32>(0.012, 0.010, 0.008), vec3<f32>(0.020, 0.024, 0.034), n.y * 0.5 + 0.5);
}

fn sd_rund(q: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let d = abs(q) - b + vec2<f32>(r);
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

@fragment
fn haupt_farbe(e: Zwischen) -> @location(0) vec4<f32> {
    let blick = normalize(e.welt - szene.auge.xyz);
    // Zweiseitige Flächen (Blätter, Vorhang): Normale immer zum Betrachter ausrichten.
    // Nicht über front_facing, das hängt vom Umlaufsinn der Dreiecke ab.
    var n = normalize(e.normale);
    if (dot(n, blick) > 0.0) { n = -n; }
    let entfernung = length(e.welt - szene.auge.xyz);
    var c: vec3<f32>;
    var strahlt = false;

    if (e.material == M_BILDROEHRE) {
        // Bildröhre: abgerundete Ecken, Bild mit bandbegrenzter Maske, Glas mit Spiegelung
        let a = szene.bild_halb.x / szene.bild_halb.y;
        if (sd_rund((e.uv - 0.5) * vec2<f32>(a, 1.0) * 2.0, vec2<f32>(a, 1.0), 0.12) > 0.0) { discard; }
        let soll = textureSample(bildschirm, rand, e.uv).rgb * roehre_maske(e.uv, maske) * szene.bild_mitte.w;
        let bild = bild_strahldichte(soll);
        let rand_dunkel = smoothstep(0.0, 0.06, min(min(e.uv.x, 1.0 - e.uv.x) * a, min(e.uv.y, 1.0 - e.uv.y)));
        let cos_t = clamp(dot(-blick, n), 0.0, 1.0);
        let fresnel = 0.04 + 0.96 * pow(1.0 - cos_t, 5.0);
        strahlt = true;
        c = bild * mix(0.55, 1.0, rand_dunkel) + umgebung(reflect(blick, n), e.welt) * fresnel + vec3<f32>(0.003, 0.0035, 0.004);
    } else {
        // Rohpixel (Option raum.rohpixel): die Koordinate rastet auf die Texelmitte ein (aus der
        // Nähe sieht man die Pixel der 256er-Texturen wie auf einer PS2), die Ableitungen bleiben
        // die echten, damit Mip-Stufe und anisotrope Filterung in der Ferne gleich arbeiten.
        // Ohne Rohpixel wird bilinear geglättet.
        let tg = vec2<f32>(textureDimensions(farben));
        let uv_roh = select(e.uv, (floor(e.uv * tg) + 0.5) / tg, szene.raumbild.x > 0.5 && !draussen(e.material));
        let duv_x = dpdx(e.uv);
        let duv_y = dpdy(e.uv);
        let farbe = textureSampleGrad(farben, kachel, uv_roh, e.material, duv_x, duv_y);
        if (farbe.a < 0.5) { discard; }
        let eig = textureSampleGrad(eigenschaften, kachel, uv_roh, e.material, duv_x, duv_y);
        // Das Relief kommt immer aus der geglätteten Höhe: aus der gerasterten springt die
        // Ableitung an jeder Texelgrenze, die Normale kippt kurz, und ein scharfes Glanzlicht
        // blitzt dort als einzelner heller Punkt auf (an Gehäuse und Rahmen des Fernsehers).
        let hoehe = textureSampleGrad(eigenschaften, kachel, e.uv, e.material, duv_x, duv_y).r;
        // Wie viele Texel fallen auf ein Pixel? Im flachen Blickwinkel viele: dann kann kein Pixel
        // das Relief auflösen, es erzeugte nur ein flimmerndes Punktmuster und wird ausgeblendet.
        let texel_je_pixel = max(length(duv_x * tg), length(duv_y * tg));
        let relief_anteil = clamp(1.5 / max(texel_je_pixel, 1e-4) - 0.25, 0.0, 1.0);
        // Schmutz und Verschattung in den Raumkanten (Boden/Wand, Wand/Decke, Wand/Wand). Der
        // Abstand zu der Fläche, auf der der Punkt selbst liegt, zählt nicht mit.
        let gross = 1000.0;
        let ab_x = select(min(e.welt.x + 2.4, 2.4 - e.welt.x), gross, abs(n.x) > 0.7);
        let ab_z = select(1.05 - e.welt.z, gross, abs(n.z) > 0.7);
        let ab_y = select(vec2<f32>(e.welt.y, 2.6 - e.welt.y), vec2<f32>(gross), abs(n.y) > 0.7);
        let ecke = smoothstep(0.0, 0.45, min(ab_x, ab_z)) * mix(0.55, 1.0, smoothstep(0.0, 0.35, ab_y.x))
            * mix(0.6, 1.0, smoothstep(0.0, 0.4, ab_y.y));
        let kante = mix(0.5, 1.0, ecke);
        let albedo = farbe.rgb * mix(0.75, 1.0, ecke);
        let rauheit_textur = eig.g;
        let metall = eig.b;

        let n_flaeche = n;
        // Relief ohne Tangenten (Ableitungen der Höhe in Bildschirmrichtung)
        let dpx = dpdx(e.welt);
        let dpy = dpdy(e.welt);
        let dhx = dpdx(hoehe);
        let dhy = dpdy(hoehe);
        let r1 = cross(dpy, n);
        let r2 = cross(n, dpx);
        let det = dot(dpx, r1);
        // Relieftiefe in Metern (Höhe 0 … 1 der Eigenschaftstextur)
        let tiefe = select(0.0015, 0.0004, e.material == M_BLATT) * relief_anteil;
        let grad = sign(det) * (dhx * r1 + dhy * r2) * tiefe;
        let nb = abs(det) * n - grad;
        if (abs(det) > 1e-12) { n = normalize(nb); }

        // Glanz gegen Flimmern: dreht sich die fertige Normale schneller, als ein Pixel auflösen
        // kann (gerundete Kanten, kleine Teile, Relief), wird das Glanzlicht breiter (Toksvig).
        let streuung = dot(dpdx(n), dpdx(n)) + dot(dpdy(n), dpdy(n));
        let rauheit = clamp(sqrt(rauheit_textur * rauheit_textur + min(2.0 * streuung, 0.35)), 0.0, 1.0);
        // Exponent höchstens 512: noch schärfer bleibt auf 1–2 Pixel beschränkt und flimmert
        let glanz = exp2(min(10.0 * (1.0 - rauheit) + 1.0, 9.0));
        let f0 = mix(vec3<f32>(0.04), albedo, metall);
        let diffus = albedo * (1.0 - metall);

        // Lampe: Licht durch den Schirm nach oben und unten stärker als zur Seite
        let zur_lampe = szene.lampe.xyz - e.welt;
        let dl = length(zur_lampe);
        let l = zur_lampe / dl;
        let schirm = mix(0.3, 1.0, smoothstep(0.25, 0.65, abs(l.y)));
        let ll = lampe_farbe() * schirm / (dl * dl) * schatten(e.welt, n_flaeche);
        let h = normalize(l - blick);
        c = diffus * ll * max(dot(n, l), 0.0) + f0 * ll * pow(max(dot(n, h), 0.0), glanz) * (glanz + 2.0) / 8.0 * max(dot(n, l), 0.0);

        // Blätter im Gegenlicht: ein Teil des Lampenlichts scheint gelbgrün hindurch
        if (e.material == M_BLATT) {
            c += albedo * ll * max(-dot(n, l), 0.0) * vec3<f32>(0.9, 1.4, 0.4);
        }

        // Bildschirm als Flächenlicht nach vorn (−z). Der Ursprung liegt hinter dem Glas: so
        // bekommen auch die Wände des Bildtunnels neben dem Glas ihr Licht ab, wie von einer
        // ausgedehnten Fläche (ein Punkt auf der Glasmitte sähen sie nur streifend).
        let zum_bild = szene.bild_mitte.xyz + vec3<f32>(0.0, 0.0, 0.05) - e.welt;
        let db = length(zum_bild);
        let lb = zum_bild / db;
        // Der Bildschirm strahlt nach −z: Flächen davor sehen ihn in Richtung +z
        let nach_vorn = max(lb.z, 0.0);
        let flaeche = 4.0 * szene.bild_halb.x * szene.bild_halb.y;
        let bl = bildlicht() * nach_vorn * flaeche / (db * db + flaeche) * bild_schatten(e.welt, n_flaeche);
        let hb = normalize(lb - blick);
        c += diffus * bl * max(dot(n, lb), 0.0) * 8.0 + f0 * bl * pow(max(dot(n, hb), 0.0), glanz) * 0.5 * nach_vorn;

        // Fenster: kaltes, schwaches Straßenlicht fällt schräg herein (Flächenlicht ohne Schatten)
        let zum_fenster = vec3<f32>(-1.35, 1.55, 1.15) - e.welt;
        let df = length(zum_fenster);
        let lf = zum_fenster / df;
        // Ziehen Wolken vor den Mond, wird es etwas dunkler; fährt draußen ein Auto vorbei,
        // hellt sein Licht den Schein des Fensters kurz warm auf
        let mondlicht = mix(1.0, 0.45, wolke_vor_mond());
        let fahrt = auto_licht(szene.auge.w).w;
        let fenster_licht = (vec3<f32>(0.30, 0.36, 0.50) * mondlicht + vec3<f32>(0.55, 0.50, 0.40) * fahrt)
            * max(lf.z, 0.0) / (df * df + 0.6);
        c += diffus * fenster_licht * max(dot(n, lf), 0.0) * kante;

        // Mondlicht fällt durchs Fenster und zeichnet Scheiben und Sprossen als kühlen Fleck
        // auf den Teppich vor dem Heizkörper. Die Richtung ist bewusst steiler als der Mond,
        // den man im Fenster sieht (sonst fiele der Fleck hinter die Kamera). Ob ein Punkt im Licht liegt, entscheidet der
        // Durchstoß des Strahls zum Mond durch die Wandebene: innerhalb der Scheiben, nicht auf
        // Rahmen oder Sprossen (Schatten der Möbel davor rechnet das nicht).
        if (e.welt.z < 1.05) {
            let zum_mond = normalize(vec3<f32>(-0.2, 1.0, 1.0));
            let q = e.welt + zum_mond * ((1.05 - e.welt.z) / zum_mond.z);
            let in_x = smoothstep(0.0, 0.02, min(q.x + 1.82, -0.88 - q.x));
            let in_y = smoothstep(0.0, 0.02, min(q.y - 0.98, 2.12 - q.y));
            let sprossen = smoothstep(0.016, 0.03, abs(q.x + 1.35)) * smoothstep(0.016, 0.03, abs(q.y - 1.70));
            // die Tischplatte (x ±0,81, z −0,47 … 0,37, 0,69 hoch) wirft ihren Schatten hinein
            var tisch = 1.0;
            if (e.welt.y < 0.66) {
                let r = e.welt + zum_mond * ((0.69 - e.welt.y) / zum_mond.y);
                tisch = 1.0 - smoothstep(-0.02, 0.02, min(0.81 - abs(r.x), min(r.z + 0.47, 0.37 - r.z)));
            }
            let mond = vec3<f32>(0.28, 0.36, 0.55) * 0.7 * mondlicht;
            c += diffus * mond * max(dot(n, zum_mond), 0.0) * in_x * in_y * sprossen * tisch;
        }

        // Umgebung und Spiegelung
        c += diffus * umgebungslicht(n) * kante;
        // Rücklicht der Lampe: die hell angestrahlten Wände werfen warmes Licht zurück in den
        // Raum. Ohne diesen Anteil lag alles, was die Lampe nicht direkt sieht (die Front des
        // Fernsehers mit Knöpfen und Aufdrucken), im völligen Schwarz.
        let von_waenden = normalize(vec3<f32>(0.35, 0.25, 1.0));
        c += diffus * lampe_farbe() * (0.016 + 0.012 * max(dot(n, -von_waenden), 0.0)) * kante;
        let spiegel_staerke = (1.0 - rauheit) * (1.0 - rauheit);
        c += f0 * umgebung(reflect(blick, n), e.welt) * spiegel_staerke * 1.5;

        // Selbstleuchten
        var leuchten = eig.a;
        if (e.material == M_LAMPENSCHIRM) { leuchten *= szene.lampe.w * 0.9; }
        if (e.material == M_GLUEHBIRNE) { leuchten *= szene.lampe.w * 4.0; }
        if (e.material == M_LED) { leuchten *= 1.5; }
        c += albedo * leuchten;
        strahlt = leuchten > 0.01;
        if (draussen(e.material)) {
            // Leuchtendes (Laternenglas, Scheinwerfer, Fenster in der Ferne) bleibt hell; Hang,
            // Bäume, Laternenmasten, Bank und Auto bekommen Licht und Schatten; Himmel und
            // Ferne sind gemalt
            let hell = max(farbe.r, max(farbe.g, farbe.b)) > 0.5;
            let beleuchtet = e.material == M_PARK || e.material == M_BAUM || e.material == M_LATERNE || e.material == M_AUTO
                || e.material == M_PARKHOLZ || e.material == M_PARKEISEN;
            if (e.material == M_HIMMEL) {
                c = himmel(blick);
            } else if (hell) {
                c = farbe.rgb * 1.3;
            } else if (beleuchtet) {
                let flach = e.material == M_BAUM || e.material == M_LATERNE || e.material == M_AUTO;
                let nf = select(n_flaeche, vec3<f32>(0.0, 0.0, -1.0), flach);
                c = farbe.rgb * park_licht(e.welt + nf * 0.004, nf, flach);
            } else {
                c = farbe.rgb * 0.35;
            }
            // Dunst: je weiter weg, desto mehr im Grau der Nacht
            let dunst = 1.0 - exp(-max(entfernung - 5.0, 0.0) * 0.04);
            c = mix(c, vec3<f32>(0.010, 0.012, 0.018), dunst * select(0.6, 0.25, hell));
            strahlt = true;
        }
    }

    // Nebel: dichter Dunst, leicht grünlich-grau, vom Lampenlicht etwas aufgehellt
    let nebel = select(1.0 - exp(-entfernung * szene.nachbild.x), 0.0, draussen(e.material));
    let nebelfarbe = vec3<f32>(0.010, 0.012, 0.011) + lampe_farbe() * 0.004;
    c = mix(c, nebelfarbe, nebel);
    // Alpha trägt die Entfernung für die Tiefenunschärfe; negativ heißt »leuchtet selbst« und
    // gibt dem Glühen die Quelle (Bildschirm, Schirm, Birne, LEDs, Fenster).
    return vec4<f32>(c, select(entfernung, -entfernung, strahlt));
}

// ---------------------------------------------------------------- 3. Nachbearbeitung
@group(0) @binding(0) var<uniform> nach_szene: Szene;
@group(0) @binding(1) var hdr: texture_2d<f32>;
// nur Nachbearbeitung: verkleinerte helle Anteile für das Glühen
@group(0) @binding(2) var glanz_bild: texture_2d<f32>;
@group(0) @binding(3) var weich: sampler;
// nur Glanzpass: Abtaster für das volle Bild
@group(0) @binding(2) var glanz_abtaster: sampler;

@fragment
fn glanz_farbe(e: NachAus) -> @location(0) vec4<f32> {
    // Glanzpass: jeder Punkt des auf 1/8 verkleinerten Ziels mittelt einen 8×8-Block. Es strahlen
    // nur selbstleuchtende Punkte (Alpha negativ); angestrahlte Flächen erst weit über Weiß,
    // sonst trüge das Glühen das Tapetenmuster neben der Lampe als helle Wolken mit.
    let groesse = vec2<i32>(textureDimensions(hdr));
    let basis = vec2<i32>(floor(e.pos.xy)) * 8;
    var summe = vec3<f32>(0.0);
    for (var y = 0; y < 8; y++) {
        for (var x = 0; x < 8; x++) {
            let f = textureLoad(hdr, min(basis + vec2<i32>(x, y), groesse - vec2<i32>(1)), 0);
            let l = dot(f.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
            let anteil = select(smoothstep(2.0, 4.0, l), 1.0, f.a < 0.0);
            summe += max(f.rgb - vec3<f32>(0.05), vec3<f32>(0.0)) * anteil;
        }
    }
    return vec4<f32>(summe / 64.0, 1.0);
}

struct NachAus { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn nach_ecke(@builtin(vertex_index) i: u32) -> NachAus {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return NachAus(vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0), vec2<f32>(p.x, 1.0 - p.y));
}

fn zufall(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn linear_zu_srgb(c: vec3<f32>) -> vec3<f32> {
    let c2 = max(c, vec3<f32>(0.0));
    return select(1.055 * pow(c2, vec3<f32>(1.0 / 2.4)) - 0.055, c2 * 12.92, c2 <= vec3<f32>(0.0031308));
}

// Scheibe aus 16 Punkten (Spirale im Goldenen Winkel), Radius 0 … 1
fn scheibe(i: i32) -> vec2<f32> {
    let r = sqrt((f32(i) + 0.5) / 16.0);
    let w = f32(i) * 2.39996;
    return vec2<f32>(cos(w), sin(w)) * r;
}

@fragment
fn nach_farbe(e: NachAus) -> @location(0) vec4<f32> {
    let groesse = vec2<f32>(textureDimensions(hdr));
    let gmax = vec2<i32>(groesse) - vec2<i32>(1);
    let p = vec2<i32>(e.uv * groesse);
    let mitte = textureLoad(hdr, p, 0);
    var c = mitte.rgb;

    // Tiefenunschärfe: Zerstreuungskreis aus dem Abstand zur Schärfeebene (dem Fernseher).
    // Nur Punkte, die selbst unscharf sind, tragen bei, damit scharfe Kanten nicht ausbluten.
    // Das Muster ist je Punkt gedreht: statt Doppelkanten entsteht feines Rauschen, das im
    // Filmkorn aufgeht.
    let drehung = fract(52.9829189 * fract(dot(vec2<f32>(p), vec2<f32>(0.06711056, 0.00583715)))) * 6.2831853;
    let dreh = mat2x2<f32>(cos(drehung), sin(drehung), -sin(drehung), cos(drehung));
    let fokus = nach_szene.fokus.x;
    let zk = nach_szene.fokus.y * abs(abs(mitte.a) - fokus) / max(abs(mitte.a), 0.05) * groesse.y * 0.02;
    let radius = min(zk, groesse.y * 0.008);
    if (radius > 0.75) {
        var summe = c;
        var gewicht = 1.0;
        for (var i = 0; i < 16; i++) {
            let q = clamp(p + vec2<i32>(dreh * scheibe(i) * radius), vec2<i32>(0), gmax);
            let s = textureLoad(hdr, q, 0);
            let zk_s = nach_szene.fokus.y * abs(abs(s.a) - fokus) / max(abs(s.a), 0.05) * groesse.y * 0.02;
            let w = smoothstep(0.0, 1.0, zk_s / max(radius, 1e-3));
            summe += s.rgb * w;
            gewicht += w;
        }
        c = summe / gewicht;
    }

    // Umgebungsverdeckung (Kontaktschatten) aus der Entfernung im Alpha: in Kehlen, unter
    // Gegenständen und an Kanten liegen die Nachbarn im Mittel näher als der Punkt selbst.
    // Je Paar gegenüberliegender Nachbarn: liegt ihr Mittel deutlich vor dem Punkt (auf einer
    // ebenen Fläche liegt es genau auf ihm), ist der Punkt verdeckt. Fernes (über 0,25 m
    // Tiefensprung) zählt nicht, sonst bekämen Umrisse einen dunklen Hof.
    let tiefe = abs(mitte.a);
    if (mitte.a > 0.0 && tiefe < 12.0) {
        let radius = clamp(0.07 * groesse.y * 1.45 / tiefe, 3.0, 48.0);
        var verdeckt = 0.0;
        for (var i = 0; i < 8; i++) {
            let w = f32(i) * 0.392699 + drehung;
            let r = radius * (0.35 + 0.65 * f32((i * 5) % 8) / 7.0);
            let o = vec2<i32>(vec2<f32>(cos(w), sin(w)) * r);
            let d1 = abs(textureLoad(hdr, clamp(p + o, vec2<i32>(0), gmax), 0).a);
            let d2 = abs(textureLoad(hdr, clamp(p - o, vec2<i32>(0), gmax), 0).a);
            let vor = (tiefe - 0.5 * (d1 + d2)) / tiefe;
            let nah = step(abs(d1 - tiefe), 0.25) * step(abs(d2 - tiefe), 0.25);
            verdeckt += smoothstep(0.002, 0.02, vor) * nah;
        }
        c *= 1.0 - 0.55 * verdeckt / 8.0;
    }

    // Glühen: helle Stellen (Bildschirm, Lampe) strahlen in den Dunst ab. Aus dem auf 1/8
    // verkleinerten Glanzbild, weich gefiltert über zwei Ringe.
    if (nach_szene.fokus.z > 0.0) {
        let t = 1.0 / vec2<f32>(textureDimensions(glanz_bild));
        // drei versetzte Ringe zu je 12 Punkten, damit kein Ring- oder Blockmuster bleibt
        var glanz = textureSampleLevel(glanz_bild, weich, e.uv, 0.0).rgb * 0.16;
        for (var i = 0; i < 12; i++) {
            let w = f32(i) * 0.523599;
            let r = vec2<f32>(cos(w), sin(w)) * t;
            let r2 = vec2<f32>(cos(w + 0.2618), sin(w + 0.2618)) * t;
            glanz += textureSampleLevel(glanz_bild, weich, e.uv + r * 1.2, 0.0).rgb * 0.035;
            glanz += textureSampleLevel(glanz_bild, weich, e.uv + r2 * 2.6, 0.0).rgb * 0.025;
            glanz += textureSampleLevel(glanz_bild, weich, e.uv + r * 4.5, 0.0).rgb * 0.0167;
        }
        glanz *= 1.5;
        c += glanz * nach_szene.fokus.z * 0.9;
    }
    c *= nach_szene.nachbild.w;
    // Filmischer Tonwert (sanfte Schulter)
    c = c * (2.51 * c + 0.03) / (c * (2.43 * c + 0.59) + 0.14);
    // Farbstimmung: etwas entsättigt, Schatten leicht grünlich-kühl
    let grau = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    c = mix(vec3<f32>(grau), c, 0.88);
    c += vec3<f32>(-0.004, 0.003, 0.002) * (1.0 - smoothstep(0.0, 0.25, grau));
    // Vignette (Option raum.vignette, 0 = aus)
    let d = e.uv - 0.5;
    c *= 1.0 - 0.55 * dot(d, d) * 1.6 * nach_szene.raumbild.y;
    // Filmkorn, im Dunkeln stärker, ändert sich je Bild
    let k = zufall(e.pos.xy + vec2<f32>(fract(nach_szene.auge.w * 7.0) * 1000.0, 0.0)) - 0.5;
    c += k * nach_szene.nachbild.y * (0.02 + 0.06 * (1.0 - smoothstep(0.0, 0.3, grau)));
    c = max(c, vec3<f32>(0.0));
    if (nach_szene.nachbild.z > 0.5) {
        return vec4<f32>(c, 1.0);
    }
    return vec4<f32>(linear_zu_srgb(c), 1.0);
}
