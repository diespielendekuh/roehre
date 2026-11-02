# Mission: Impossible (Nintendo 64), PC-Port

Stand dieser Notiz: 2026-11-02 (korrigiert nach Hinweis der KI des Projekts; vorher stand hier
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
