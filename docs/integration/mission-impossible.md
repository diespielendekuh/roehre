# Mission: Impossible (Nintendo 64), PC-Port

Stand dieser Notiz: 2026-11-02, Wünsche des Projekts eingetragen (korrigiert nach Hinweis der KI des Projekts; vorher stand hier
das Vorgänger-Repository und „Renderer offen“).

- **Repository:** `diespielendekuh/MissionImpossibleDecomp`. Decomp auf `main`; der PC-Port liegt
  (Stand 2026-11-02) auf dem Branch `claude/admiring-bardeen-opoj7i` im Ordner `port/`.
  Das ältere `diespielendekuh/MissionImpossibleCodes` ist nicht mehr maßgeblich.
- **Technik:** C++, statische Rekompilierung mit **N64Recomp / N64ModernRuntime / RecompFrontend**
  und **RT64** als Renderer (**D3D12 und Vulkan**; Untermodule unter `port/lib/`). Grundlage ist
  `mi_n64_exe` (US-Recomp, GPL-3.0); der Port ist GPL-3.0.
- **Dauerregel des Projekts:** Neue Stände von `mi_n64_exe` müssen sich jederzeit übernehmen
  lassen. Eigene Funktionen in eigene Dateien und hinter eigene Optionen, in übernommenem Code
  (auch RT64) nur kleine, markierte Einstiegspunkte (`port/LIESMICH.md`). Für roehre heißt das:
  Der Einbau braucht eine Schnittstelle, die mit einem kleinen Eingriff hinter der RT64-Ausgabe
  auskommt.
- **Bild:** N64 320×240 bzw. 640×480; RT64 rendert hochaufgelöst und mit Breitbild-Erweiterung.
  Für die Röhre die virtuelle Zeilenzahl nutzen (`eingang.zeilen`, Automatik 240/480).
  Fenster-Seitenverhältnisse laut Agenda: 4:3, 16:9, 16:10, 21:9, 32:9, Auto.
- **Stand des Einbaus:** noch nicht eingebaut. **Blockiert durch das C-API** (Wunsch W-001):
  roehre ist bisher nur Rust/wgpu, und die eigenen Pässe (Zusammensetzen mit Gehäuse, HDR,
  Strahlsimulation, Mipmaps, Eichung) liegen nur als WGSL vor.
- **Geplanter Einbau (Agenda des Projekts, `notes/agenda_port.md`, Punkt 4):** Nachbearbeitung
  des Bildes: RT64-Ausgabe → roehre → Bildschirm, Stufen im Menü des Ports.
- **TV-Erlebnis (Agenda Punkt 5, Wunsch W-004):** echtzeitgerenderte 3D-Szene, alter
  Röhrenfernseher auf einem Tisch, das Spielbild mit Röhre auf der gewölbten Bildröhre,
  Glasreflexionen mit sachtem, abschaltbarem Kameraschwenk, leichte Tiefenunschärfe,
  Lichtflackern, Bewegung im Hintergrund (Fenster mit Vorhang/Straßenlicht), geringe Last,
  im Launcher wählbar. Baustein von roehre: Flächen-Modus (`zeichne_flaeche`, Maske im
  Material). Das Konzept stimmt das Projekt vor dem Bau mit dem Anwender ab.
- **Lizenz:** Port GPL-3.0, Röhre (guest-advanced) GPL-2.0-or-later, roehre-Code MIT OR
  Apache-2.0 – verträglich; Shader-Ordner wird neben dem Spiel ausgeliefert.
- **Absprache:** Die KI des Projekts ändert roehre derzeit nicht („nur lesen, solange die andere
  KI daran arbeitet“). Rückmeldungen bitte in `docs/ki/wuensche.md`.

## Stand der Wünsche des Projekts (2026-11-02)

| Wunsch | Was | Stand |
|---|---|---|
| W-001 | C-API (Vulkan/D3D12) mit Flächen-Modus **und** 3D-Raum, „Textur + Ziel je Bild“ | offen, **Etappe 7** (nächste nach Etappe 6) |
| W-004 | 3D-Vorführraum, TV-Erlebnis | **in Arbeit (Etappe 6)**: Raum, Fernseher, Licht, Nebel, Rohpixel-Texturen, eigenes Wandbild gehen in Rust/wgpu (`crates/roehre-raum`, Vorschau „3D-Raum“) |
| W-005 | Kamera: Nähe, Blickwinkel, Schwenk, Lage im Fenster (Lens Shift), 4:3–32:9 | offen, **Abschluss Etappe 6**; heute gibt es nur Nähe und Schwenk als Regler in der Vorschau, ohne Festwerte und ohne Lens Shift |
| W-006 | Kamera-Optionen im Katalog, speicherbar | offen, **Abschluss Etappe 6** |
| W-007 | Filtervorschau im Launcher (Software-Zeichnung) | offen, **Etappe 7**: Vorschaubild als RGBA-Puffer aus der C-API, Rückfall fertige PNGs je Voreinstellung |
| W-008 | Leistung nur Filter / 3D-Raum, 1080p / 4K, Kosten je Regler | offen, **Etappe 8**: Messbefehl im Prüfstand, Messung auf dem Rechner des Anwenders |

Was heute schon geht: alles aus `docs/ki/STAND.md` Abschnitt 2 – aber nur über Rust/wgpu 30.
Für den C++-Port mit RT64 ist bis zur C-API noch nichts einbaubar.
