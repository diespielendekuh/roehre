// roehre-wgsl – gemeinsamer Kopf jeder Stufe (Wunsch W-009).
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Die Vereinbarung zwischen einer Stufe und ihrem Ausführer, gleich für jede
// wgpu-Version (Terranigma: 24, roehre: 30):
//   @group(0) @binding(0)  Rahmen (uniform, 128 Byte, siehe `roehre_wgsl::Rahmen`)
//   @group(0) @binding(1)  Quelle (texture_2d<f32>, Rgba8Unorm, sRGB-kodierte Werte)
//   @group(0) @binding(2)  Abtaster (linear oder nächster Punkt, je Stufe)
// Gezeichnet wird ein einziges Dreieck über den Bildausschnitt des Ziels
// (drei Ecken, kein Eckenpuffer); der Ausführer setzt dafür den Viewport.

struct Rahmen {
    // Größe der Quelltextur in Punkten: (breite, höhe, 1/breite, 1/höhe).
    quelle: vec4<f32>,
    // Größe des Ziels (des Viewports) in Bildpunkten, ebenso.
    ziel: vec4<f32>,
    // Gezeigter Ausschnitt der Quelle in Texturkoordinaten (u0, v0, u1, v1);
    // das ganze Bild ist (0, 0, 1, 1). Ein Spiel mit beschnittenem Overscan
    // zeigt nur einen Teil seiner Textur.
    ausschnitt: vec4<f32>,
    // x: Bildzähler; y–w frei.
    bild: vec4<u32>,
    // Regler der Stufe, je nach Stufe belegt (siehe ihre Beschreibung).
    regler: array<vec4<f32>, 4>,
}

@group(0) @binding(0) var<uniform> rahmen: Rahmen;
@group(0) @binding(1) var quelle: texture_2d<f32>;
@group(0) @binding(2) var abtaster: sampler;

struct Ecke {
    @builtin(position) lage: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Ein Dreieck, das den Viewport ganz deckt: Ecken (0,0), (2,0), (0,2) im
// Einheitsmaß; die Texturkoordinate läuft über den Ausschnitt (und darüber
// hinaus, wo das Dreieck übersteht — dort wird nicht gezeichnet).
@vertex
fn ecke(@builtin(vertex_index) i: u32) -> Ecke {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var e: Ecke;
    e.lage = vec4<f32>(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
    e.uv = mix(rahmen.ausschnitt.xy, rahmen.ausschnitt.zw, p);
    return e;
}
