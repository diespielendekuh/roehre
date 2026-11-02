# Etappe 8: Leistung, Feinschliff, englische Texte

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

## Messwerte (Anwender, RTX 5070 Ti, Vulkan)

Siehe `docs/ki/STAND.md` Abschnitt 2a und `docs/messungen/2026-11-02_rtx5070ti_vulkan.md`.
Kurz: jede Voreinstellung bleibt bei 4K unter 1 ms (Ziel war unter 2 ms), der 3D-Raum braucht
2,5 ms bei 4K. Teuer sind nur die Signalwege Composite und S-Video (+0,4 ms); Tiefenunschärfe
und Glühen kosten im Raum zusammen 0,5 ms bei 4K. Nichts davon gefährdet stabile 60 FPS.

## Fehler aus dem ersten Lauf unter Windows

`d3d12_spiel.exe` brach beim Anlegen ab: librashader legte einen Pipeline-Cache an, den wgpu
unter D3D12 nicht anbietet. roehre fordert ihn jetzt nur noch an, wenn das Gerät ihn kann; die
neue Wache startet alle Voreinstellungen auf einem Gerät ohne jede Zusatzfähigkeit.

## Offen

* Zweiter Lauf von `d3d12_spiel.exe` nach der Korrektur.
* Messung unter D3D12 und auf einer Mittelklassekarte (Hochrechnung: teuerste Röhre um 2 ms,
  Raum um 5 ms bei 4K).
