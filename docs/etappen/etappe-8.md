# Etappe 8: Leistung, Feinschliff, englische Texte (Zwischenstand)

## Gebaut

* **Messbefehl** `roehre-pruefstand leistung [-o leistung.md] [-n 120] [--nur-1080p]`
  (`crates/roehre-pruefstand/src/leistung.rs`): Zeitstempel der Grafikkarte am Anfang und Ende
  jedes Bilds (sonst Wanduhr, wird ausgewiesen), Median und 95-%-Wert, Spielbild 320×240.
  1. Nur Filter: alle zehn Voreinstellungen, 1080p und 4K, Anteil am 60-Hz-Budget.
  2. 3D-Raum: Standard, ohne Tiefenunschärfe, ohne Glühen, ohne beides, Nähe weit.
  3. Kosten je Regler gegenüber `scart-rgb`: Röhre aus, Leistungsstufe leicht, Composite,
     S-Video, Maske aus, Licht im Glas aus, ohne Wölbung, Gehäuse, HDR, Strahlsimulation 240 Hz
     (Mittel je Auffrischung).
* **Windows-Paket** (MinGW-Querbau): Prüfstand, Vorschau, `roehre.dll`, D3D12-Beispiel.
* **Englische Texte** für alle Optionen, Auswahlwerte und Voreinstellungen, mit Wachen.
* **Feinschliff:** Clippy ohne Warnung; Prüfstand und Vorschau finden `shader/` neben dem
  Programm.
* **3D-Raum:** Konsole im Stil des Super Famicom mit Controller (Wunsch des Anwenders).

## Offen

* Die Zahlen: gemessen werden muss auf der Grafikkarte des Anwenders (`leistung.md`). Hier gibt
  es nur lavapipe (Software); dessen Werte (200 ms je Bild bei 1080p) sagen nichts über echte
  Grafikkarten und stehen deshalb nirgends als Ergebnis.
* D3D12-Austausch unter Windows ausführen (`d3d12_spiel.exe`).
* Danach: Tabelle in `STAND.md`, Empfehlungen je Grafikkartenklasse, ggf. teure Teile
  beschleunigen.
