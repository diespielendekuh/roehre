# Terranigma PC-Port

- **Repository:** `diespielendekuh/Tenchi-Souzou-Deutsch-mod-tool`, Kiste `crates/terranigma-spiel`
- **Stand:** noch nicht eingebaut (Wunsch W-002); Einbauweg unten. **Zweiter Weg ohne Anheben**
  (Wunsch W-009, 2026-11-04): die MIT-Kette als WGSL, vom Port auf wgpu 24 selbst ausgeführt —
  ob angehoben wird, entscheidet der Leiter des Ports
- **Bild:** 256×224, Pixel 8:7 (NTSC); der Port beschneidet den Rand selbst (Werksbeschnitt 8/8/0/1)
- **Grafik:** winit 0.30, egui 0.31, egui-wgpu 0.31, **wgpu 24** (D3D12 unter Windows,
  Vulkan/GL unter Linux). roehre braucht wgpu 30 → Port auf egui 0.37 / wgpu 30 anheben.
- **Heute:** Das Spielbild wird auf der CPU um den Faktor k vergrößert und als egui-Textur
  `spielfeld` hochgeladen (`crates/terranigma-spiel/src/portmenue.rs`).
- **Geplanter Einbau:** Spielbild unvergrößert als wgpu-Textur hochladen, `Roehre::zeichne` in
  die Swapchain-Textur (oder eine Zieltextur, die egui als native Textur zeigt), danach egui
  (Menü) darüber. Die CPU-Vergrößerung entfällt.
- **Voreinstellung:** `scart-rgb` als Standard (europäischer Fernseher, RGB).

## Einbauweg (Stand Etappe 7)

Terranigma ist Rust mit wgpu – es braucht die C-API nicht, sondern bindet `roehre-wgpu` direkt
ein (Beispiel für eframe/egui 0.37 mit wgpu 30: `crates/roehre-vorschau/src/main.rs`).

1. **Anheben:** wgpu 24 → 30, egui/egui-wgpu/egui-winit 0.31 → 0.37 (librashader 0.12 setzt wgpu
   30 voraus). Das ist die größte Arbeit; sie betrifft nur `crates/terranigma-spiel`.
2. **Gerät:** beim Anlegen `roehre_wgpu::empfohlene_features()` anfordern, soweit der Adapter sie
   hat.
3. **Bild:** das Spielbild 256×224 unvergrößert als `Rgba8Unorm`-Textur hochladen (die
   CPU-Vergrößerung in `portmenue.rs` entfällt), `Eingang { breite: 256, hoehe: 224,
   pixel_seitenverhaeltnis: 8.0 / 7.0 }`. Der Werksbeschnitt 8/8/0/1 bleibt Sache des Ports
   (vor dem Hochladen beschneiden oder roehre mit `geometrie.overscan` arbeiten lassen).
4. **Zeichnen:** `Roehre::zeichne` in die Swapchain-Textur, danach egui (Menü) darüber; ab 120 Hz
   `zeichne_im_takt` (Strahlsimulation).
5. **Menü:** die Optionen aus `roehre_modell::katalog()` (deutsche Namen, Grenzen, Festwerte);
   Einstellungen als TOML im Spielstandordner.
6. **3D-Raum** (falls gewünscht): `roehre_raum::Raum` mit `Ansicht::aus_einstellungen`.

## Zweiter Weg ohne Anheben: `roehre-wgsl` (W-009, ab 2026-11-04 auf `helfer/terranigma`)

Der Port bleibt vorerst auf wgpu 24. Statt `roehre-wgpu` liest er die Stufen als WGSL-Text aus
`crates/roehre-wgsl` und führt sie selbst aus (Vorbild: `crates/roehre-pruefstand/src/wgsl.rs`):

1. Bindungslayout **ausdrücklich** nach der Vereinbarung anlegen (Rahmen 128 Byte uniform,
   Quelle `texture_2d<f32>` filterbar, Abtaster filternd) — nicht ableiten lassen.
2. Je Stufe der Kette ein Ziel der Größe `Rahmen::zielgroesse` (MMPX: Quelle × 2, Format
   `Rgba8Unorm`), die letzte Stufe in den Bildausschnitt des Fensters (Viewport).
3. Werksbeschnitt 8/8/0/1 über `Rahmen::ausschnitt` (Texturkoordinaten); `scharf-bilinear`
   und `mmpx` lesen keinen Punkt von außerhalb.
4. Wie die Kiste in den offline gebauten Port kommt (Git-Abhängigkeit, beim Packen
   mitgelegt), entscheidet der Leiter des Ports.
5. **Röhre ohne librashader (Teil 2):** `roehre_wgsl::erzeugt::beans::{BEANS_RGB, BEANS_SIGNAL,
   BEANS_SCHNELL}` mit dem Ausführer nach `crates/roehre-pruefstand/src/wgsl_kette.rs`
   (gegen librashader punktgleich). Die Regler plant heute `roehre_kette::plane`; solange
   diese Kiste librashader zieht, braucht der Port dafür einen Weg ohne (siehe W-009).
