// roehre – 3D-Vorführraum: Röhrenfernseher auf einem Tisch, per Raymarching berechnet.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Diese Datei wird hinter shader/eigen/flaeche/roehre_maske.wgsl gehängt (RoehreMaske,
// roehre_maske). Alles ist berechnet: keine Modelle, keine Texturen außer dem Röhrenbild aus
// dem Flächen-Modus. Maße in Metern; der Fernseher steht im Ursprung, sein Bildschirm zeigt
// nach −z, zur Kamera hin.
//
// Bausteine:
// - Szene als Signed Distance Fields: Raum (Boden, Rückwand mit Fenster), Tisch, Gehäuse mit
//   eingelassener Bildöffnung.
// - Bildröhre analytisch als Kugelkappe in der Öffnung; Bild aus der Flächen-Textur, mal der
//   bandbegrenzten Maske (moiréfrei in jedem Abstand und Winkel).
// - Glas: Fresnel-Spiegelung des Raums (ein zweiter Strahl), leichte Tönung.
// - Licht: Lampe (warm, leicht flackernd, weiche Schatten), Licht des Bildschirms auf Tisch,
//   Gehäuse und Wand (Flächenlicht in der mittleren Bildfarbe), Straßenlicht durch das Fenster
//   mit sich bewegendem Vorhang.

struct Raum {
    // Kamera: Lage (xyz), Blickwinkel senkrecht in Grad
    auge: vec4<f32>,
    // Blickziel (xyz), Seitenverhältnis des Ziels
    ziel: vec4<f32>,
    // Zeit in Sekunden, Helligkeit des Bildschirms, Raumlicht, Flackern (0…1)
    zeit: vec4<f32>,
    // Seitenverhältnis des Bildes (Breite/Höhe), höchste Mip-Stufe, Wölbung, Ausgabe (0 SDR-Unorm, 1 SDR-sRGB)
    bild: vec4<f32>,
};

@group(0) @binding(0) var<uniform> raum: Raum;
@group(0) @binding(1) var<uniform> maske: RoehreMaske;
@group(0) @binding(2) var bildschirm: texture_2d<f32>;
@group(0) @binding(3) var abtaster: sampler;

struct Aus { @builtin(position) pos: vec4<f32>, @location(0) ndc: vec2<f32> };

@vertex
fn ecke(@builtin(vertex_index) i: u32) -> Aus {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u)) * 2.0 - 1.0;
    return Aus(vec4<f32>(p, 0.0, 1.0), p);
}

// ---------------------------------------------------------------- Maße
const TISCH_HOEHE: f32 = 0.72;
const GEHAEUSE: vec3<f32> = vec3<f32>(0.31, 0.25, 0.23);   // halbe Maße des Fernsehers
const OEFFNUNG_H: f32 = 0.165;                              // halbe Höhe der Bildöffnung
const RUECKWAND_Z: f32 = 1.1;
const LAMPE: vec3<f32> = vec3<f32>(0.9, 1.75, -0.6);

fn tv_mitte() -> vec3<f32> {
    return vec3<f32>(0.0, TISCH_HOEHE + GEHAEUSE.y + 0.012, 0.0);
}

fn oeffnung_halb() -> vec2<f32> {
    return vec2<f32>(OEFFNUNG_H * raum.bild.x, OEFFNUNG_H);
}

// ---------------------------------------------------------------- Abstandsfunktionen
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sd_rundbox(p: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    return sd_box(p, b - vec3<f32>(r)) - r;
}

fn sd_rundrechteck(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Szene: x = Abstand, y = Material (1 Boden, 2 Wand, 3 Tisch, 4 Gehäuse, 5 Fensterrahmen)
fn szene(p: vec3<f32>) -> vec2<f32> {
    var d = vec2<f32>(p.y, 1.0);
    let wand = RUECKWAND_Z - p.z;
    if (wand < d.x) { d = vec2<f32>(wand, 2.0); }
    // Tisch: Platte und vier Beine
    let platte = sd_rundbox(p - vec3<f32>(0.0, TISCH_HOEHE - 0.02, 0.05), vec3<f32>(0.75, 0.02, 0.4), 0.01);
    var beine = 1e9;
    for (var i = 0; i < 4; i++) {
        let s = vec2<f32>(select(-1.0, 1.0, (i & 1) == 1), select(-1.0, 1.0, (i & 2) == 2));
        beine = min(beine, sd_box(p - vec3<f32>(0.68 * s.x, (TISCH_HOEHE - 0.04) * 0.5, 0.05 + 0.33 * s.y), vec3<f32>(0.03, (TISCH_HOEHE - 0.04) * 0.5, 0.03)));
    }
    let tisch = min(platte, beine);
    if (tisch < d.x) { d = vec2<f32>(tisch, 3.0); }
    // Gehäuse mit eingelassener Bildöffnung (Vertiefung 3 cm) und hinten verjüngt
    let q = p - tv_mitte();
    var gehaeuse = sd_rundbox(q, GEHAEUSE, 0.035);
    let hinten = sd_rundbox(q - vec3<f32>(0.0, 0.0, 0.18), vec3<f32>(GEHAEUSE.x * 0.7, GEHAEUSE.y * 0.75, 0.2), 0.05);
    gehaeuse = min(gehaeuse, hinten);
    let halb = oeffnung_halb();
    let vertiefung = max(sd_rundrechteck(q.xy, halb + vec2<f32>(0.012), 0.025), abs(q.z + GEHAEUSE.z) - 0.03);
    gehaeuse = max(gehaeuse, -vertiefung);
    if (gehaeuse < d.x) { d = vec2<f32>(gehaeuse, 4.0); }
    return d;
}

fn normale(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.0008, 0.0);
    return normalize(vec3<f32>(
        szene(p + e.xyy).x - szene(p - e.xyy).x,
        szene(p + e.yxy).x - szene(p - e.yxy).x,
        szene(p + e.yyx).x - szene(p - e.yyx).x,
    ));
}

fn marsch(o: vec3<f32>, r: vec3<f32>, max_t: f32, schritte: i32) -> vec2<f32> {
    var t = 0.0;
    for (var i = 0; i < schritte; i++) {
        let h = szene(o + r * t);
        if (h.x < 0.0005 * t) { return vec2<f32>(t, h.y); }
        t += h.x;
        if (t > max_t) { break; }
    }
    return vec2<f32>(-1.0, 0.0);
}

fn weicher_schatten(o: vec3<f32>, r: vec3<f32>, max_t: f32) -> f32 {
    var s = 1.0;
    var t = 0.02;
    for (var i = 0; i < 24; i++) {
        let h = szene(o + r * t).x;
        s = min(s, 10.0 * h / t);
        t += clamp(h, 0.02, 0.25);
        if (s < 0.01 || t > max_t) { break; }
    }
    return clamp(s, 0.0, 1.0);
}

// ---------------------------------------------------------------- Bildröhre
// Kugelkappe in der Öffnung: Mitte der Kugel hinter dem Bildschirm, Scheitel 3 cm hinter der
// Gehäusefront minus Wölbung.
fn roehre_kugel() -> vec4<f32> {
    let r = mix(4.0, 0.9, raum.bild.z);                     // Kugelradius (flach … stark gewölbt)
    let scheitel_z = tv_mitte().z - GEHAEUSE.z + 0.03 - 0.025 * raum.bild.z;
    return vec4<f32>(tv_mitte().x, tv_mitte().y, scheitel_z + r, r);
}

// Schnitt mit der Bildröhre: t und uv, oder t < 0
fn roehre_schnitt(o: vec3<f32>, r: vec3<f32>) -> vec3<f32> {
    let k = roehre_kugel();
    let oc = o - k.xyz;
    let b = dot(oc, r);
    let c = dot(oc, oc) - k.w * k.w;
    let disk = b * b - c;
    if (disk < 0.0) { return vec3<f32>(-1.0, 0.0, 0.0); }
    let t = -b - sqrt(disk);
    if (t < 0.0) { return vec3<f32>(-1.0, 0.0, 0.0); }
    let p = o + r * t - tv_mitte();
    let halb = oeffnung_halb();
    if (sd_rundrechteck(p.xy, halb, 0.02) > 0.0) { return vec3<f32>(-1.0, 0.0, 0.0); }
    let uv = vec2<f32>(p.x / (2.0 * halb.x) + 0.5, 0.5 - p.y / (2.0 * halb.y));
    return vec3<f32>(t, uv);
}

fn mittlere_bildfarbe() -> vec3<f32> {
    return textureSampleLevel(bildschirm, abtaster, vec2<f32>(0.5), raum.bild.y).rgb * raum.zeit.y;
}

// ---------------------------------------------------------------- Licht
fn lampe_farbe() -> vec3<f32> {
    // leichtes Flackern aus zwei langsamen Sinuswellen und etwas Zufall
    let t = raum.zeit.x;
    let flackern = 1.0 - raum.zeit.w * (0.06 * sin(t * 7.3) * sin(t * 2.1) + 0.04 * fract(sin(floor(t * 12.0) * 91.7) * 4375.5));
    return vec3<f32>(1.0, 0.72, 0.45) * 1.6 * raum.zeit.z * flackern;
}

// Fenster: Nachthimmel mit Straßenlaterne, Rahmen mit Sprossen, davor ein halb zugezogener
// Vorhang, der sich im Luftzug leicht bewegt und von hinten durchleuchtet wird.
fn fenster(p: vec3<f32>) -> vec4<f32> {
    let lokal = vec2<f32>(p.x + 1.15, p.y - 1.45);
    let aussen = sd_rundrechteck(lokal, vec2<f32>(0.42, 0.45), 0.005);
    if (aussen > 0.0 || p.z < RUECKWAND_Z - 0.01) { return vec4<f32>(0.0); }
    let t = raum.zeit.x;
    // Rahmen und Sprossen (dunkles Holz, nur schwach beleuchtet)
    let rahmen = -aussen < 0.035 || abs(lokal.x) < 0.015 || abs(lokal.y - 0.1) < 0.012;
    if (rahmen) { return vec4<f32>(vec3<f32>(0.010, 0.008, 0.006), 1.0); }
    let himmel = mix(vec3<f32>(0.004, 0.006, 0.014), vec3<f32>(0.012, 0.010, 0.016), smoothstep(0.45, -0.45, lokal.y));
    let laterne = vec3<f32>(1.0, 0.55, 0.2) * (0.6 * exp(-length(lokal - vec2<f32>(0.22, -0.05)) * 40.0)
        + 0.04 * exp(-length(lokal - vec2<f32>(0.22, -0.05)) * 6.0));
    let draussen = himmel + laterne;
    // Vorhang: deckt etwa 60 % von rechts, weiche Falten wandern langsam
    let welle = 0.025 * sin(lokal.y * 5.0 + t * 0.7) + 0.012 * sin(lokal.y * 11.0 - t * 1.1);
    let kante = lokal.x + 0.08 + welle;
    let vorhang = smoothstep(-0.015, 0.015, kante);
    let falten = 0.75 + 0.25 * sin((lokal.x + welle * 1.5) * 28.0);
    // Stoff: dunkles Rot, von hinten vom Straßenlicht durchleuchtet
    let stoff = vec3<f32>(0.05, 0.015, 0.012) * falten * (0.3 + 4.0 * dot(laterne, vec3<f32>(0.33)));
    return vec4<f32>(mix(draussen, stoff, vorhang), 1.0);
}

fn albedo(m: f32, p: vec3<f32>) -> vec3<f32> {
    if (m < 1.5) {
        // Holzboden in Dielen
        let diele = floor(p.x * 5.0);
        let maser = 0.85 + 0.15 * sin(p.z * 40.0 + diele * 3.1 + sin(p.z * 7.0) * 2.0);
        return vec3<f32>(0.22, 0.14, 0.08) * maser * (0.9 + 0.1 * fract(sin(diele * 12.9) * 437.5));
    }
    if (m < 2.5) { return vec3<f32>(0.32, 0.30, 0.27); }
    if (m < 3.5) {
        let maser = 0.85 + 0.15 * sin(p.x * 30.0 + sin(p.z * 9.0) * 2.0);
        return vec3<f32>(0.12, 0.07, 0.04) * maser;
    }
    return vec3<f32>(0.035, 0.035, 0.038);
}

// Beleuchtete Oberfläche (ohne Spiegelungen außer einem Glanzlicht)
fn beleuchte(p: vec3<f32>, n: vec3<f32>, m: f32, blick: vec3<f32>, mit_schatten: bool) -> vec3<f32> {
    let fl = fenster(p);
    if (fl.w > 0.5) { return fl.rgb; }
    let a = albedo(m, p);
    var c = a * vec3<f32>(0.012, 0.014, 0.02);                     // Grundlicht der Nacht
    // Lampe
    let zur_lampe = LAMPE - p;
    let abst = length(zur_lampe);
    let l = zur_lampe / abst;
    var schatten = 1.0;
    if (mit_schatten) { schatten = weicher_schatten(p + n * 0.002, l, abst); }
    let diffus = max(dot(n, l), 0.0) / (abst * abst);
    c += a * lampe_farbe() * diffus * schatten;
    if (m > 3.5) {
        // Kunststoff: Glanzlicht der Lampe
        let h = normalize(l - blick);
        c += lampe_farbe() * pow(max(dot(n, h), 0.0), 60.0) * 0.08 * schatten / (abst * abst);
    }
    // Licht des Bildschirms: Flächenlicht vor der Röhre, Leuchtdichte = mittlere Bildfarbe
    let quelle = tv_mitte() - vec3<f32>(0.0, 0.0, GEHAEUSE.z + 0.02);
    let zur_quelle = quelle - p;
    let aq = length(zur_quelle);
    let lq = zur_quelle / aq;
    let abstrahl = max(dot(-lq, vec3<f32>(0.0, 0.0, -1.0)), 0.0);   // nur nach vorn
    let flaeche = 4.0 * oeffnung_halb().x * oeffnung_halb().y;
    c += a * mittlere_bildfarbe() * max(dot(n, lq), 0.0) * abstrahl * flaeche / (aq * aq + 0.02);
    // Straßenlicht durch das Fenster (sehr schwach, bläulich, von hinten links)
    c += a * vec3<f32>(0.05, 0.06, 0.1) * max(dot(n, normalize(vec3<f32>(-1.15, 1.45, RUECKWAND_Z) - p)), 0.0) * 0.3;
    return c;
}

// Spiegelung des Raums im Glas: ein Strahl, ohne Schatten
fn umgebung(o: vec3<f32>, r: vec3<f32>) -> vec3<f32> {
    let h = marsch(o, r, 6.0, 48);
    if (h.x < 0.0) { return vec3<f32>(0.01); }
    let p = o + r * h.x;
    return beleuchte(p, normale(p), h.y, r, false);
}

// ---------------------------------------------------------------- Hauptteil
fn linear_zu_srgb(c: vec3<f32>) -> vec3<f32> {
    let c2 = max(c, vec3<f32>(0.0));
    return select(1.055 * pow(c2, vec3<f32>(1.0 / 2.4)) - 0.055, c2 * 12.92, c2 <= vec3<f32>(0.0031308));
}

@fragment
fn farbe(e: Aus) -> @location(0) vec4<f32> {
    // Kamerastrahl
    let vorn = normalize(raum.ziel.xyz - raum.auge.xyz);
    let rechts = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), vorn));
    let oben = cross(vorn, rechts);
    let f = 1.0 / tan(radians(raum.auge.w) * 0.5);
    let r = normalize(rechts * e.ndc.x * raum.ziel.w + oben * e.ndc.y + vorn * f);
    let o = raum.auge.xyz;

    let szene_treffer = marsch(o, r, 12.0, 160);
    let roehre = roehre_schnitt(o, r);

    var c: vec3<f32>;
    if (roehre.x > 0.0 && (szene_treffer.x < 0.0 || roehre.x < szene_treffer.x)) {
        let p = o + r * roehre.x;
        let k = roehre_kugel();
        let n = normalize(p - k.xyz) * -1.0;                       // nach außen, zur Kamera
        let uv = roehre.yz;
        // Leuchtendes Bild mit bandbegrenzter Maske
        let bild = textureSample(bildschirm, abtaster, uv).rgb * roehre_maske(uv, maske) * raum.zeit.y;
        // Glas: Fresnel (Schlick, n = 1,52) und Spiegelung des Raums
        let cos_t = clamp(dot(-r, n), 0.0, 1.0);
        let fresnel = 0.043 + (1.0 - 0.043) * pow(1.0 - cos_t, 5.0);
        let spiegel = umgebung(p + n * 0.003, reflect(r, n));
        // Glas leicht grau getönt, Rand der Öffnung etwas dunkler (Schatten der Fase)
        let rand = smoothstep(0.0, 0.04, min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y)));
        c = bild * 0.92 * mix(0.7, 1.0, rand) + spiegel * fresnel + vec3<f32>(0.004);
    } else if (szene_treffer.x > 0.0) {
        let p = o + r * szene_treffer.x;
        c = beleuchte(p, normale(p), szene_treffer.y, r, true);
    } else {
        c = vec3<f32>(0.0);
    }
    // Weiche Begrenzung (Glanzlichter) und Vignette des Objektivs
    c = c / (1.0 + 0.15 * c);
    c *= 1.0 - 0.25 * dot(e.ndc * 0.7, e.ndc * 0.7);
    if (raum.bild.w > 0.5) {
        return vec4<f32>(c, 1.0);
    }
    return vec4<f32>(linear_zu_srgb(c), 1.0);
}
