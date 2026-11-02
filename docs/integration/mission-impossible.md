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
- **Stand des Einbaus:** noch nicht eingebaut; die C-API dafür steht seit Etappe 7 (Anleitung unten).
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
| W-001 | C-API (Vulkan/D3D12) mit Flächen-Modus **und** 3D-Raum, „Textur + Ziel je Bild“ | **gebaut (Etappe 7)**: `crates/roehre-c`, Austausch über geteilten Grafikspeicher; D3D12 und Vulkan-mit-Semaphoren auf echter Hardware ungeprüft |
| W-004 | 3D-Vorführraum, TV-Erlebnis | **fertig in Rust/wgpu (Etappe 6)**: Raum, Fernseher, Licht, Nebel, Rohpixel-Texturen, eigenes Wandbild, Tiefenunschärfe, Glühen (`crates/roehre-raum`, Vorschau „3D-Raum“); für RT64 mit der C-API |
| W-005 | Kamera: Nähe, Blickwinkel, Schwenk, Lage im Fenster (Lens Shift), 4:3–32:9 | **fertig in Rust/wgpu (Etappe 6)**, Bilder: `docs/bilder/etappe-6/kamera_bogen.jpg` |
| W-006 | Kamera-Optionen im Katalog, speicherbar | **fertig (Etappe 6)**: Gruppe `raum`, Festwerte über `festwerte()` |
| W-007 | Filtervorschau im Launcher (Software-Zeichnung) | **gebaut (Etappe 7)**: `roehre_vorschau_zeichnen` liefert ein RGBA-Bild, flach oder Raum |
| W-008 | Leistung nur Filter / 3D-Raum, 1080p / 4K, Kosten je Regler | **Messbefehl fertig** (`roehre-pruefstand leistung`); Zahlen von der Grafikkarte des Anwenders stehen aus |


## Einbau über die C-API (Etappe 7)

**Der Einbau kann beginnen (Stand 2026-11-02).** Die Schnittstelle in `roehre.h` gilt als fest:
Änderungen daran werden nur noch ergänzt (neue Funktionen, neue Felder am Ende von
`RoehreAustauschParameter`, erkennbar an `struktur_groesse`), nicht umgebaut. Was noch aussteht –
der D3D12-Lauf und die Messung auf echter Hardware –, kann nur Fehler im Inneren von roehre
aufdecken; die werden dort behoben, ohne dass sich der Einbau ändert. Empfohlene Reihenfolge:
Launcher (Katalog-JSON, Einstellungen speichern, Vorschaubild) zuerst, weil er unabhängig von
RT64 ist; danach der Austausch in `rt64_present_queue.cpp`.

Schnittstelle: `crates/roehre-c/include/roehre.h`, Überblick `crates/roehre-c/LIESMICH.md`.
roehre rechnet auf einem eigenen Gerät derselben Grafikkarte; RT64 und roehre teilen zwei Bilder
(EINGANG, ZIEL) und eine Synchronisation. **Unter D3D12 braucht RT64/plume dafür keinerlei
Änderung am Gerät**, unter Vulkan zwei Geräteerweiterungen (siehe unten).

### Einstiegsstelle in RT64

`src/hle/rt64_present_queue.cpp`, `PresentQueue::threadPresent` (RT64-Stand des Ports,
Untermodul `6f1c2d9`). Dort zeichnet heute `viRenderer->render(renderParams)` das Spielbild in die
Swapchain, danach `RenderHookDraw` (Oberfläche des Ports) und der Inspector. Vorschlag für einen
kleinen, markierten Eingriff (`// roehre:`), alles Weitere in eigenen Dateien des Ports:

1. **Bild nach EINGANG:** statt in die Swapchain zeichnet der VI-Renderer in ein eigenes
   RGBA8-Zwischenbild des Ports (so bleiben VI-Beschnitt und Filter von RT64 erhalten) und kopiert
   es nach EINGANG; EINGANG danach in den Lesezustand. Liste beenden, ausführen, **„bereit“**
   melden (D3D12: `queue->Signal(zaun, bereit)` mit den Werten aus `roehre_d3d12_zaunwerte`;
   Vulkan: Semaphore 0 signalisieren).
2. `roehre_austausch_zeichnen(a, modus, zeit)` (bzw. `…_im_takt` bei 120 Hz und mehr).
3. **Neue Liste,** die auf **„fertig“** wartet (D3D12: `queue->Wait(zaun, fertig)`; Vulkan:
   Semaphore 1 als Wartesemaphore), ZIEL in die Swapchain kopieren bzw. zeichnen, dann wie bisher
   `RenderHookDraw`, Inspector, Barriere nach PRESENT, ausführen, präsentieren.

Native Objekte holt der Port aus plume (`plume::D3D12Device::d3d`, `D3D12CommandQueue::d3d`,
`D3D12CommandList::d3d`, bzw. die Vulkan-Gegenstücke); die geöffneten roehre-Bilder kopiert er
mit nativen Befehlen (`CopyTextureRegion`, `vkCmdBlitImage`), weil plume fremde Ressourcen nicht
einhüllen kann. Das ist Arbeit im Code des Ports, nicht in RT64.

* **EINGANG-Größe:** die Größe des Zwischenbilds (z. B. 320×240 oder die Auflösung von RT64);
  `pixel_seitenverhaeltnis` so, dass Breite × Seitenverhältnis / Höhe = 4/3 (bzw. das
  Breitbildverhältnis von RT64). Bei hoher Auflösung rechnet roehre selbst auf die virtuelle
  Zeilenzahl herunter (`eingang.zeilen`, Automatik 240/480).
* **ZIEL-Größe:** Swapchain-Größe. Bei Größenänderung den Austausch neu anlegen.
* **Ausgabe:** SDR = RGBA8 sRGB; mit HDR-Swapchain `ROEHRE_AUSGABE_SCRGB` bzw. `_HDR10`.
* **Vulkan:** plume muss beim Anlegen des Geräts `VK_KHR_external_memory_fd` und
  `VK_KHR_external_semaphore_fd` (Windows: `…_win32`) einschalten – eine Zeile in
  `OptionalDeviceExtensions` von `plume_vulkan.cpp`, also ein Eingriff in ein Untermodul. Unter
  Windows mit D3D12 entfällt das.
* **Launcher-Vorschau (W-007):** `roehre_vorschau_erstellen` einmal, dann bei jeder
  Regleränderung (entprellt) `roehre_vorschau_zeichnen` mit einem Standbild des Spiels; das
  RGBA-Ergebnis zeigt der Launcher wie ein gewöhnliches Bild.
* **Menü:** `roehre_katalog_json` liefert alle Optionen mit deutschem Namen, Beschreibung,
  Grenzen, Standard und Festwerten (z. B. Nähe weit/mittel/nah); gespeichert wird mit
  `roehre_einstellungen_als_text`.

### Offen

* Der D3D12-Weg und der Vulkan-Weg mit Semaphoren sind auf echter Hardware nicht ausgeführt
  (Etappe 8; Beispiel `beispiele/d3d12_spiel.cpp` zum Ausprobieren unter Windows).
* Die Kosten (zwei Kopien je Bild, Röhre, Raum) sind auf echter Grafikkarte ungemessen (W-008).
