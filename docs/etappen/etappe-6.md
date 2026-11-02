# Etappe 6: Vorschau-Programm und 3D-Vorführraum (Zwischenbericht)

## Was gebaut ist

* **Vorschau** (`crates/roehre-vorschau`, eframe/wgpu): Testbild, eigene PNGs, Bildfolgen,
  Schwenktest, A/B-Vergleich mit Schieber, Lupe, Bildschirmfoto, alle Optionen aus dem
  Katalog; Ansicht „3D-Raum“ mit Regler für Nähe, Schwenk, Licht, Flackern und Korn.
* **3D-Raum** (`crates/roehre-raum`): ein Wohnzimmer der späten 80er bei Nacht, Vorlage für das
  „TV-Erlebnis“ des Mission:-Impossible-Ports (Wunsch W-004). Alles berechnet: Geometrie aus
  `szene.rs`, Texturen aus `texturen.rs`, keine Fremddaten. Der Bildschirm ist die Röhre selbst
  (Flächen-Modus), er beleuchtet den Raum.

## Rückmeldungen des Anwenders und was daraus wurde

| Rückmeldung | Umsetzung |
|---|---|
| Fernseher zu klein, zu rund, nicht wie ein 80er-Gerät | Neubau als Raster-Szene: hohles Furniergehäuse, Bedienfeld, Antenne, gewölbte Röhre |
| Richtung Silent Hill 2/3, späte PS2, nicht Comic, nicht fotorealistisch | Nebel, Korn, Vignette, entsättigte Farben, Schmutz in den Raumkanten, Wasserflecken |
| Pflanze zu künstlich, Blätter zu kurz (Vorbild: Gummibaum aus Sims 1) | Ficus elastica: drei Triebe, große ovale, längs gefaltete Blätter an Stielen, durchscheinend |
| Lampe in die Ecke, Pflanze neben den Tisch | `LAMPE` hinten rechts, `PFLANZE` rechts neben dem Tisch |
| Bilderrahmen langweilig, eigenes Foto als Platzhalter | Goldrahmen mit Profil, Passepartout; `Raum::wandbild_setzen` (Foto bleibt lokal, nicht im Repository) |
| Fernbedienung zu knubbelig, lieber Texturen als Polygone | flaches Gehäuse, Tasten als Textur |
| Parkett und Tisch wie Laminat, Spiegelung nicht wie Holz | Wildeiche mit Ästen, Rissen, Farbspiel; matt geölt |
| Texturen pixelig mit sichtbaren Rohpixeln, nicht verschmiert | 256², Abtastung rastet auf die Texelmitte ein (Mip/Anisotropie bleiben) |
| Blick aus dem Fenster zu comichaft | Neblige Nacht: Laterne mit Hof, schemenhafte Häuser, kahler Baum, wenige trübe Fenster |

## Bilder

`docs/bilder/etappe-6/raum_*.jpg` (Prüfstand `roehre-pruefstand raum`, 1920×1080, lavapipe).

## Offen

* Wirkung und Leistung auf echter GPU (Anwender).
* Tiefenunschärfe, Glühen um helle Stellen; weitere Details (Regal, Uhr).
* Übertragung auf D3D12/Vulkan (Etappe 7, C-API).
