// roehre – Flächen-Maske für 3D-Szenen (WGSL)
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Für einen Röhrenbildschirm als Objekt in einer 3D-Szene: Das Röhrenbild (Signal, Strahl,
// Zeilen, Leuchtstoff, Licht) kommt als Textur aus roehre (Flächen-Modus, ohne Maske, mit
// Mipmaps). Die Maske rechnet diese Funktion im Material des Bildschirm-Modells aus den
// UV-Koordinaten.
//
// Warum nicht einfach eine Maskentextur? Die Maske hat Hunderte Streifen über die
// Bildbreite. Aus der Entfernung oder schräg betrachtet werden sie feiner als ein
// Bildschirmpixel, und jedes Abtasten erzeugt dann Moiré. Diese Funktion beschreibt die Maske
// als wenige Sinus-Anteile und blendet jeden Anteil weich aus, bevor er die Abtastgrenze
// erreicht (Bandbegrenzung über die Ableitungen der UV-Koordinaten). Aus der Nähe sieht man
// die Farbstreifen, aus der Ferne eine gleichmäßige Fläche – in keinem Abstand Moiré.
//
// Der Mittelwert über eine Periode ist 1: Die Maske ändert die Helligkeit im Mittel nicht.
// Ergebnis mit der linearen Farbe des Röhrenbilds multiplizieren.

struct RoehreMaske {
    // Farbtripel über die Bildbreite (u von 0 bis 1)
    triaden: f32,
    // Stärke der Streifen, 0 … 1
    staerke: f32,
    // Stärke der Schlitze (0 = Streifenmaske, sonst Schlitzmaske)
    schlitz: f32,
    // Breite / Höhe der Bildfläche
    seitenverhaeltnis: f32,
    // Schlitzabstand senkrecht in Tripelbreiten
    schlitz_abstand: f32,
    // 1 = Reihenfolge B, G, R
    bgr: f32,
};

const ROEHRE_TAU: f32 = 6.283185307179586;

// 1 bei groben Mustern, 0 ab der Abtastgrenze (0,5 Perioden je Pixel), weich dazwischen.
fn roehre_bandgrenze(perioden_je_pixel: f32) -> f32 {
    return 1.0 - smoothstep(0.2, 0.45, perioden_je_pixel);
}

fn roehre_maske(uv: vec2<f32>, m: RoehreMaske) -> vec3<f32> {
    // Koordinaten in Tripeln (waagerecht) und Schlitzreihen (senkrecht)
    let t = uv.x * m.triaden;
    let v = uv.y * m.triaden / (m.seitenverhaeltnis * max(m.schlitz_abstand, 0.1));
    // Fußabdruck eines Bildschirmpixels in Tripeln bzw. Reihen (auch bei schrägem Blick)
    let wt = length(vec2<f32>(dpdx(t), dpdy(t)));
    let wv = length(vec2<f32>(dpdx(v), dpdy(v)));

    // Streifen: Grundwelle und erste Oberwelle (schmalere Streifen aus der Nähe)
    let a1 = m.staerke * 0.8 * roehre_bandgrenze(wt);
    let a2 = m.staerke * 0.3 * roehre_bandgrenze(2.0 * wt);
    var phase = vec3<f32>(0.0, 1.0 / 3.0, 2.0 / 3.0);
    if (m.bgr > 0.5) {
        phase = phase.zyx;
    }
    let x = ROEHRE_TAU * (vec3<f32>(t) - phase);
    let streifen = max(vec3<f32>(1.0) + a1 * cos(x) + a2 * cos(2.0 * x), vec3<f32>(0.0));

    // Schlitze: dunkle Querstege, in benachbarten Tripeln um eine halbe Reihe versetzt
    // (Produkt zweier Wellen: senkrecht je Reihe, waagerecht mit halber Tripelfrequenz)
    let s = m.schlitz * roehre_bandgrenze(wv) * roehre_bandgrenze(0.5 * wt);
    let schlitz = 1.0 + s * 0.5 * cos(ROEHRE_TAU * v) * cos(0.5 * ROEHRE_TAU * t);

    return streifen * schlitz;
}
