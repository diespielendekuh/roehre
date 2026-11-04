# Wunschliste: was Projekte von roehre brauchen

Hier trägt jede KI (oder jeder Mensch) ein, was einem Spielprojekt fehlt. Wer roehre
weiterbaut, arbeitet diese Liste mit ab. Neue Einträge **oben** einfügen, fortlaufende
Nummer, erledigte nicht löschen, sondern mit Commit markieren.

## Vorlage

```markdown
### W-NNN: <kurzer Titel>
- **Von:** <Projekt / KI>, <Datum>
- **Status:** offen | in Arbeit (<wer>) | erledigt (<Commit>) | abgelehnt (<Grund>)
- **Was fehlt:** <konkret: Funktion, Format, Fehlerbild mit Werten>
- **Warum:** <wofür das Spiel es braucht>
- **Vorschlag:** <wie es gehen könnte, falls bekannt>
```

## Einträge

### W-009: Die MIT-Kette ohne wgpu-Version (für Terranigma auf wgpu 24)
- **Von:** Terranigma-Port (Helfer-KI der Oberfläche, über den Anwender), 2026-11-04
- **Status:** in Arbeit (Helfer-KI des Terranigma-Ports, Branch `helfer/terranigma`): Kiste
  `crates/roehre-wgsl` steht mit `scharf-bilinear` (gegen librashader ±1 in jeder Größe) und
  `mmpx` (GPU punktgleich mit der CPU-Referenz), Prüfbefehl `roehre-pruefstand wgsl`;
  offen: crt-beans (`mit-rgb`, `mit-composite`), `virtuelle-zeilen`, Regler-Beschreibung.
- **Was fehlt:** Eine Fassung der GPL-freien Kette (crt-beans und die eigenen Stufen
  `scharf-bilinear`, `virtuelle-zeilen`), die **keine wgpu- und keine librashader-Version**
  voraussetzt: die Pässe als WGSL-Text, dazu eine Beschreibung der Pass-Folge (Größe und Format
  jedes Zwischenbilds, Abtastung, welche Eingaben ein Pass liest) und der Parameter (Schlüssel,
  Grenzen, Voreinstellung, deutscher Text aus `roehre-modell`), z. B. als Kiste `roehre-wgsl`
  ohne GPU-Abhängigkeit.
- **Warum:** Der Port soll vorerst **nicht** auf wgpu 30 / egui 0.37 angehoben werden (Entscheid
  seines Leiters steht aus, W-002). Mit Shader-Text und Pass-Beschreibung kann der Port die
  Röhre auf seinem wgpu 24 selbst ausführen — ohne Kopie und Umbau der Shader im Spielrepository
  (AGENTS.md §2): der Port liest sie aus roehre.
- **Vorschlag:** crt-beans ist slang; für die MIT-Voreinstellungen `mit-rgb` und `mit-composite`
  die Pässe einmal nach WGSL übertragen (naga kann slang→SPIR-V→WGSL, Handarbeit wohl nötig) und
  mit `roehre-pruefstand pruefe` gegen dieselben Referenzbilder halten wie die librashader-Kette.
  GPL-Teile bleiben außen vor.
- **Ansprechpartner im Port:** `tools/koordination/helfer/bericht-helfer.md`, Abschnitt
  »Wünsche an roehre«.

### W-008: Leistung getrennt messen (nur Filter / 3D-Raum, 1080p / 4K, Kosten je Regler)
- **Von:** Mission: Impossible (KI des Projekts, über den Anwender), 2026-11-02
- **Status:** erledigt für Vulkan auf RTX 5070 Ti (Etappe 8): `docs/ki/STAND.md` Abschnitt 2a,
  Tabelle `docs/messungen/`. Offen: Messung unter D3D12 und auf einer Mittelklassekarte
- **Was fehlt:** Messung auf echter Grafikkarte, getrennt für „nur Filter“ und „3D-Raum“, je
  1080p und 4K, mit dem Ziel stabiler 60 FPS; dazu eine Tabelle, welcher Regler wie viel kostet.
- **Warum:** Der Port muss stabil 60 FPS halten.
- **Einordnung (roehre):** Hier im Container gibt es nur eine Software-GPU (lavapipe), deren
  Zeiten nichts über echte Hardware sagen. roehre liefert in Etappe 8 einen Messbefehl im
  Prüfstand (`roehre-pruefstand leistung`), der genau diese Tabelle als Datei schreibt; laufen
  lassen muss ihn der Anwender auf seinem Rechner. Die Zahlen kommen dann in `STAND.md`.

### W-007: Kleine Filtervorschau im Launcher (Launcher zeichnet in Software)
- **Von:** Mission: Impossible (KI des Projekts, über den Anwender), 2026-11-02
- **Status:** erledigt (Etappe 7): `roehre_vorschau_erstellen` / `roehre_vorschau_zeichnen`
  (RGBA in den Arbeitsspeicher, flach oder Raum); Weg 2 (fertige PNGs) nicht gebaut, weil Weg 1
  ohne Fenster auf jeder Grafikkarte läuft
- **Was fehlt:** Eine Möglichkeit, im Launcher eine Vorschau der Einstellungen zu zeigen,
  obwohl der Launcher selbst keine Shader ausführen kann.
- **Vorschlag (roehre):** Zwei Wege, beide werden gebaut:
  1. **Vorschau als Bild aus der C-API** (empfohlen): `roehre_vorschau_rgba(einstellungen,
     bild, breite, hoehe, puffer)` erzeugt ohne Fenster auf der GPU ein Vorschaubild
     (Testbild oder ein vom Launcher übergebenes Standbild des Spiels, wahlweise im 3D-Raum)
     und liefert es als RGBA-Puffer in den Arbeitsspeicher. Der Launcher zeigt es als
     gewöhnliches Bild an; bei einer Regleränderung wird neu angefordert (wenige ms bis
     ~100 ms, mit Entprellen). Die GPU ist ja vorhanden, nur der Launcher nutzt sie nicht.
  2. **Fertige Vorschaubilder je Voreinstellung** als Rückfall (z. B. ohne GPU-Zugang beim
     Start): der Prüfstand rendert sie, der Port liefert sie als PNG mit. Zeigt nur
     Voreinstellungen, keine einzelnen Regler.

### W-006: Kamera-Optionen im Katalog, speicherbar
- **Von:** Mission: Impossible (KI des Projekts, über den Anwender), 2026-11-02
- **Status:** erledigt (Etappe 6): Gruppe `raum` mit zehn Optionen, Festwerte über
  `roehre_modell::festwerte(schluessel)`; über die C-API erreichbar ab Etappe 7
- **Was fehlt:** Die Kamera des 3D-Raums (W-005) als gewöhnliche Optionen im Katalog
  (Schlüssel, deutscher Name, Beschreibung, Grenzen, Standard), speicherbar in TOML/JSON wie
  die übrigen Einstellungen. Die Oberfläche baut der Launcher selbst.
- **Vorschlag (roehre):** neue Gruppe `raum` im Katalog (Nähe, Blickwinkel, Schwenk an/aus und
  Stärke, Lage des Fernsehers, Licht, Flackern, Korn, Tiefenunschärfe). Festwerte als
  benannte Stufen mit Zahlenwert, damit Menü und stufenloser Regler dieselbe Option nutzen.

### W-005: Kamera im 3D-Raum (Nähe, Blickwinkel, Schwenk, Lage im Fenster, alle Formate)
- **Von:** Mission: Impossible (KI des Projekts, über den Anwender), 2026-11-02
- **Status:** erledigt (Etappe 6) in Rust/wgpu: `roehre_raum::Kamera`, `Ansicht::aus_einstellungen`;
  Kontaktbogen `docs/bilder/etappe-6/kamera_bogen.jpg`; für C++ ab Etappe 7
- **Was fehlt:** frei kombinierbar:
  - **Nähe** stufenlos mit Festwerten Nah (Fernseher füllt das Fenster weitgehend, oben und
    unten bleibt Rand, Gerät sichtbar), Mittel (etwas Tisch), Weit (mehr Raum).
  - **Blickwinkel** stufenlos mit Festwerten von vorn, leicht links, leicht rechts.
  - **Schwenk** aus (völlig still) oder an (langsam links/rechts, Stärke einstellbar); die
    Kamera bleibt dabei auf den Fernseher gerichtet, er wandert nicht durchs Bild.
  - **Lage des Fernsehers im Fenster** stufenlos mit Festwerten Links/Mitte/Rechts bis an den
    Fensterrand (Platz für eine Webcam beim Streamen), auch in der Nah-Stufe, ohne
    Abschneiden, ohne Änderung des Blickwinkels und ohne Verzerrung.
  - **Alle Seitenverhältnisse 4:3 bis 32:9**: Rand und Lage stimmen, der Raum hat keine
    sichtbaren Enden; bei 4:3 wird der Ausschnitt kleiner.
- **Warum:** TV-Erlebnis im Port, Streaming mit Webcam neben dem Fernseher.
- **Einordnung (roehre):**
  - **Außermittige Projektion (Lens Shift) ist der richtige Weg** für die Lage im Fenster: die
    Bildebene wird verschoben, die Kamera nicht gedreht. Die Vorderseite des Fernsehers bleibt
    parallel zur Bildebene und damit unverzerrt; Blickwinkel und Spiegelungen bleiben gleich.
  - **Nähe wird als Höhe des Geräts im Fenster** festgelegt, nicht als Kameraabstand. Nur so
    stimmt „oben und unten etwas Rand“ bei jedem Format.
  - **Grenzfall:** bei 4:3 in der Nah-Stufe füllt das Gerät schon fast die ganze Breite, eine
    Verschiebung zum Rand ist dort kaum möglich. Damit nichts abgeschnitten wird, gilt: erst
    einpassen (die Nähe wird bei schmalen Fenstern so weit zurückgenommen, dass das Gerät in
    der Breite passt), dann verschieben, höchstens bis das Gerät den Rand berührt. Der
    Streamer bekommt in Nah also nur bei breiten Fenstern viel Platz – das ist geometrisch
    nicht anders lösbar, ohne das Gerät zu verkleinern.
  - Der Schwenk kreist um die Mitte des Geräts und hält es per Lens Shift an seiner Stelle.
  - Für 32:9 wird der Raum seitlich so erweitert, dass in Weit keine Kante sichtbar ist.

### W-004: 3D-Vorführraum mit einem berechneten Röhrenfernseher
- **Von:** Anwender, 2026-11-02 (Frage nach einer 3D-Szene mit Bildschirmgerät)
- **Status:** erledigt in Rust/wgpu (Etappe 6): `crates/roehre-raum`, in der Vorschau unter „3D-Raum“; für D3D12/Vulkan mit der C-API (Etappe 7)
- **Was fehlt:** Eine 3D-Szene mit einem selbst erzeugten Fernseher (gewölbtes Glas als
  Kugelkappe, Gehäuse als Körper), Kamera frei beweglich, Bild über den Flächen-Modus.
- **Warum:** Prüfstein und Vorlage für das „TV-Erlebnis“ im Mission:-Impossible-Port
  (Agenda Punkt 5: Fernseher auf einem Tisch, Glasreflexionen mit sachtem Kameraschwenk,
  Tiefenunschärfe, Lichtflackern, Fenster mit Vorhang/Straßenlicht im Hintergrund, geringe
  Last). Die Vorlage soll so gebaut sein, dass sie sich später auf D3D12/Vulkan (RT64-Umfeld)
  übertragen lässt.

### W-003: Permissive Variante ohne GPL-Shader
- **Von:** roehre (Planung), 2026-11-02
- **Status:** erledigt (2026-11-02): crt-beans mit eigener Übersetzung der Optionen,
  Option `allgemein.lizenz`, Voreinstellungen `mit-rgb` / `mit-composite`, automatischer Rückfall
  ohne GPL-Ordner. Cathode Retro nicht benutzt: der Signalteil unserer Kopie ist fest auf
  224 Zeilen eingestellt
- **Was fehlt:** Eine Kette auf Basis von crt-beans (MIT) und Cathode Retro (MIT), auf die
  dieselben Optionen abgebildet werden, für Spiele, die keine GPL-Shader ausliefern wollen.
  Die Shader liegen bereits unter `shader/fremd/mit/`.
- **Warum:** Lizenzfreiheit für Projekte, die das brauchen.

### W-002: Einbauanleitung und Versionsanhebung für den Terranigma-Port
- **Von:** roehre (Planung), 2026-11-02
- **Status:** offen (geplant Etappe 7)
- **Was fehlt:** Der Port nutzt wgpu 24 / egui 0.31, roehre braucht wgpu 30. Anleitung für das
  Anheben (egui 0.37) und den Einbau (Spielbild unvergrößert hochladen, roehre zeichnet, egui
  darüber), siehe `docs/PLAN.md` Abschnitt 11.1.
- **Warum:** Terranigma ist das erste Spiel, das roehre benutzen soll.

### W-001: C-API mit Vulkan- und D3D12-Weg
- **Von:** roehre (Planung, für Mission: Impossible), 2026-11-02
- **Status:** gebaut (Etappe 7): `crates/roehre-c`, Austausch über geteilten Grafikspeicher
  (eigenes Gerät, D3D12 ohne Eingriff am Spielgerät, Vulkan mit zwei Erweiterungen); auf echter
  Hardware ungeprüft (Etappe 8). Abweichung vom Vorschlag: nicht „Textur des Spiels + Ziel“
  direkt, weil wgpu fremde Geräte nicht (D3D12) bzw. nur mit passenden Funktionen (Vulkan)
  übernehmen kann – siehe `crates/roehre-c/LIESMICH.md`
- **Was fehlt:** Eine C-Schnittstelle (`roehre.h`) für C/C++-Spiele: erstellen mit
  Vulkan-Gerät oder D3D12-Gerät, Einstellungen laden/setzen, Katalog auflisten, zeichnen in
  eine Zieltextur, Flächen-Modus. **Ergänzt 2026-11-02 (Wunsch des Projekts):** auch der
  3D-Raum (`roehre-raum`) samt Kamera-Optionen (W-005/W-006) über die C-API; je Bild nur
  „fertige Textur + Ziel“, alles Weitere erledigt roehre. Für den Engpass bei Mission:
  Impossible ist die C-API die nächste Etappe; sie wird nicht vor den Abschluss von
  Etappe 6 gezogen (Entscheid des Anwenders).
- **Warum:** Der PC-Port von Mission: Impossible ist C++ mit **RT64 (D3D12/Vulkan)** auf
  N64ModernRuntime (`diespielendekuh/MissionImpossibleDecomp`, `port/`). Der Eingriff in RT64
  muss klein bleiben (Projektregel: Upstream jederzeit übernehmbar); ideal ist eine Funktion,
  die nach der RT64-Ausgabe eine Textur (Spielbild unvergrößert oder hochaufgelöst) und ein
  Ziel bekommt und alles Weitere selbst erledigt.
- **Vorschlag:** librashader hat Vulkan- und D3D12-Laufzeiten und eine C-API; die eigenen
  Pässe (Zusammensetzen mit Gehäuse und HDR, Mipmaps, Eichung) liegen bisher nur als WGSL vor
  und müssen dafür als slang-Pässe oder je Backend vorliegen.
