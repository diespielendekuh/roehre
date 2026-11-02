# Eigene Shader von roehre

Alle Dateien in diesem Ordner sind eigene Arbeit von roehre und stehen unter
**MIT OR Apache-2.0** (siehe `LICENSE-MIT` und `LICENSE-APACHE` im Hauptordner).

| Datei | Zweck |
|---|---|
| `durchreichen.slang` | Reicht das Bild unverändert weiter (ersetzt `stock.slang` in Cathode-Retro-Presets) |
| `scharf-bilinear.slang` | Pixelgenaues, gleichmäßiges Vergrößern ohne Unschärfe |
| `sauber.slangp` | Kette der Voreinstellung „Sauber/aus“ |
| `virtuelle-zeilen.slang` | Flächentreues Verkleinern hochaufgelöster Bilder auf die Zeilenzahl der Röhre |
| `virtuelle-zeilen.slangp` | Kette der Vorstufe |
| `flaeche/roehre_maske.wgsl` (`.glsl`, `.hlsl`) | Bandbegrenzte Maske als Materialfunktion für Röhrenbildschirme in 3D-Szenen |
| `raum/raum3d.wgsl` | 3D-Vorführraum: Wohnzimmer mit Röhrenfernseher, Rasterung mit Schatten, Relief, Nebel, Filmkorn (crate `roehre-raum`) |
