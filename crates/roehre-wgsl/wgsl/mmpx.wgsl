// roehre – Stufe „mmpx“ als WGSL (W-009, E3b des Terranigma-Ports).
// SPDX-License-Identifier: MIT
//
// MMPX nach Morgan McGuire und Mara Gagiu, »MMPX Style-Preserving Pixel-Art
// Magnification«, JCGT 10(2), 2021. Copyright 2020 Morgan McGuire & Mara
// Gagiu, MIT-Lizenz (Text in LIZENZ-MMPX neben dieser Datei).
//
// Verdoppelt das Bild: jeder Quellpunkt E wird zu vier Punkten J K / L M, und
// jeder davon ist eine Farbe der Nachbarschaft — es entsteht keine neue Farbe,
// nur Treppen an Kanten mit Steigung 1:1 und 2:1 werden geglättet.
//
// Der Ausführer zeichnet in ein Ziel der doppelten Größe der Quelle
// (`Groesse::QuelleMal(2)`), Abtaster nächster Punkt (gelesen wird ohnehin
// mit textureLoad). Nachbarn außerhalb des Ausschnitts sind der nächste
// Randpunkt im Ausschnitt: ein beschnittener Overscan wirkt nicht hinein.
//
// Die Regeln sind maschinell aus der Rust-Fassung `roehre_wgsl::mmpx`
// übertragen (dieselbe Reihenfolge, dieselben Zuweisungen); die Prüfung
// `mmpx_gpu_gleich_cpu` im Prüfstand verlangt Punktgleichheit.
//
// Nachbarschaft um E:
//       P
//     A B C
//   Q D E F R
//     G H I
//       S

var<private> mitte: vec2<i32>;
var<private> unten_links: vec2<i32>;
var<private> oben_rechts: vec2<i32>;

fn n(dx: i32, dy: i32) -> vec3<f32> {
    let p = clamp(mitte + vec2<i32>(dx, dy), unten_links, oben_rechts);
    return textureLoad(quelle, p, 0).rgb;
}

fn gl(a: vec3<f32>, b: vec3<f32>) -> bool {
    return all(a == b);
}

// Helligkeit für die Vergleiche: r + g + b in Stufen 0–255, ganzzahlig
// gerundet — sonst könnten gleiche Summen aus verschiedenen Farben durch
// Rundung der Gleitkommazahlen ungleich ausfallen.
fn hell(a: vec3<f32>) -> f32 {
    return round(dot(a, vec3<f32>(255.0)));
}

@fragment
fn punkt(ein: Ecke) -> @location(0) vec4<f32> {
    let ziel = vec2<i32>(floor(ein.lage.xy));
    mitte = ziel / 2;
    let viertel = ziel % 2;
    unten_links = vec2<i32>(round(rahmen.ausschnitt.xy * rahmen.quelle.xy));
    oben_rechts = vec2<i32>(round(rahmen.ausschnitt.zw * rahmen.quelle.xy)) - vec2<i32>(1);
    let b = n(0, -1);
    let d = n(-1, 0);
    let e = n(0, 0);
    let f = n(1, 0);
    let h = n(0, 1);
    // Fläche: sind B, D, F, H gleich E, greift keine Regel.
    if gl(b, e) && gl(d, e) && gl(f, e) && gl(h, e) {
        return vec4<f32>(e, 1.0);
    }
    let a = n(-1, -1);
    let c = n(1, -1);
    let g = n(-1, 1);
    let i = n(1, 1);
    let p = n(0, -2);
    let q = n(-2, 0);
    let r = n(2, 0);
    let s = n(0, 2);
    let hb = hell(b);
    let hd = hell(d);
    let he = hell(e);
    let hf = hell(f);
    let hh = hell(h);
    var j = e;
    var k = e;
    var l = e;
    var m = e;

    // 1:1-Regeln (»1:1 slope rules«)
    if gl(d, b)
        && !gl(d, h)
        && !gl(d, f)
        && (he >= hd || gl(e, a))
        && (gl(e, a) || gl(e, c) || gl(e, g))
        && (he < hd || !gl(a, d) || !gl(e, p) || !gl(e, q))
    {
        j = d;
    }
    if gl(b, f)
        && !gl(b, d)
        && !gl(b, h)
        && (he >= hb || gl(e, c))
        && (gl(e, a) || gl(e, c) || gl(e, i))
        && (he < hb || !gl(c, b) || !gl(e, p) || !gl(e, r))
    {
        k = b;
    }
    if gl(h, d)
        && !gl(h, f)
        && !gl(h, b)
        && (he >= hh || gl(e, g))
        && (gl(e, a) || gl(e, g) || gl(e, i))
        && (he < hh || !gl(g, h) || !gl(e, s) || !gl(e, q))
    {
        l = h;
    }
    if gl(f, h)
        && !gl(f, b)
        && !gl(f, d)
        && (he >= hf || gl(e, i))
        && (gl(e, c) || gl(e, g) || gl(e, i))
        && (he < hf || !gl(i, h) || !gl(e, r) || !gl(e, s))
    {
        m = f;
    }

    // Kreuzungsregeln (»Intersection rules«)
    if !gl(e, f)
        && gl(e, c)
        && gl(e, i)
        && gl(e, d)
        && gl(e, q)
        && gl(f, b)
        && gl(f, h)
        && !gl(f, n(3, 0))
    {
        k = f;
        m = f;
    }
    if !gl(e, d)
        && gl(e, a)
        && gl(e, g)
        && gl(e, f)
        && gl(e, r)
        && gl(d, b)
        && gl(d, h)
        && !gl(d, n(-3, 0))
    {
        j = d;
        l = d;
    }
    if !gl(e, h) && gl(e, g) && gl(e, i) && gl(e, b) && gl(e, p) && gl(h, d) && gl(h, f) && !gl(h, n(0, 3))
    {
        l = h;
        m = h;
    }
    if !gl(e, b) && gl(e, a) && gl(e, c) && gl(e, h) && gl(e, s) && gl(b, d) && gl(b, f) && !gl(b, n(0, -3)) {
        j = b;
        k = b;
    }
    if hb < he
        && gl(e, g)
        && gl(e, h)
        && gl(e, i)
        && gl(e, s)
        && !gl(e, a)
        && !gl(e, d)
        && !gl(e, c)
        && !gl(e, f)
    {
        j = b;
        k = b;
    }
    if hh < he
        && gl(e, a)
        && gl(e, b)
        && gl(e, c)
        && gl(e, p)
        && !gl(e, d)
        && !gl(e, g)
        && !gl(e, i)
        && !gl(e, f)
    {
        l = h;
        m = h;
    }
    if hf < he
        && gl(e, a)
        && gl(e, d)
        && gl(e, g)
        && gl(e, q)
        && !gl(e, b)
        && !gl(e, c)
        && !gl(e, i)
        && !gl(e, h)
    {
        k = f;
        m = f;
    }
    if hd < he
        && gl(e, c)
        && gl(e, f)
        && gl(e, i)
        && gl(e, r)
        && !gl(e, b)
        && !gl(e, a)
        && !gl(e, g)
        && !gl(e, h)
    {
        j = d;
        l = d;
    }

    // 2:1-Regeln (»2:1 slope rules«)
    if !gl(h, b) {
        if !gl(h, a) && !gl(h, e) && !gl(h, c) {
            if gl(h, g) && gl(h, f) && gl(h, r) && !gl(h, d) && !gl(h, n(2, -1)) {
                l = m;
            }
            if gl(h, i) && gl(h, d) && gl(h, q) && !gl(h, f) && !gl(h, n(-2, -1)) {
                m = l;
            }
        }
        if !gl(b, i) && !gl(b, g) && !gl(b, e) {
            if gl(b, a) && gl(b, f) && gl(b, r) && !gl(b, d) && !gl(b, n(2, 1)) {
                j = k;
            }
            if gl(b, c) && gl(b, d) && gl(b, q) && !gl(b, f) && !gl(b, n(-2, 1)) {
                k = j;
            }
        }
    }
    if !gl(f, d) {
        if !gl(d, i) && !gl(d, e) && !gl(d, c) {
            if gl(d, a) && gl(d, h) && gl(d, s) && !gl(d, b) && !gl(d, n(1, 2)) {
                j = l;
            }
            if gl(d, g) && gl(d, b) && gl(d, p) && !gl(d, h) && !gl(d, n(1, -2)) {
                l = j;
            }
        }
        if !gl(f, e) && !gl(f, a) && !gl(f, g) {
            if gl(f, c) && gl(f, h) && gl(f, s) && !gl(f, b) && !gl(f, n(-1, 2)) {
                k = m;
            }
            if gl(f, i) && gl(f, b) && gl(f, p) && !gl(f, h) && !gl(f, n(-1, -2)) {
                m = k;
            }
        }
    }

    if viertel.y == 0 {
        return vec4<f32>(select(j, k, viertel.x == 1), 1.0);
    }
    return vec4<f32>(select(l, m, viertel.x == 1), 1.0);
}
