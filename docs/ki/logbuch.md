# Logbuch: wer hat was an roehre geändert

Neue Einträge **oben**. Je Eintrag: Datum, wer (Projekt bzw. KI), was, warum, was ungeprüft ist.

## 2026-11-02 · roehre-Hauptsitzung · 3D-Raum nach Rückmeldung (Kamera, Rekorder, Tisch)
- Neue Kamerastufe „Ganz nah“ (`raum.naehe` = 1): Bildschirm samt schmalem Rahmen füllt 84 %
  der Höhe, oben und unten nur Gehäuse. Festwerte jetzt weit 0 / mittel 0,5 / nah 0,75 /
  ganz nah 1 (vorher war 1 „nah“). Prüfläufe angepasst (Bildschirm nie abgeschnitten).
- Super Famicom entfernt (zu hell im Lampenlicht); Videorekorder ohne Marke mit Leuchtanzeige
  „CH 03 · 12:00“; statt der Zeitschrift ein Stapel Videohüllen mit lesbaren Rückentiteln
  (eigene 5×7-Pixelschrift, erfundene Titel im Stil von Verleihkassetten, eine selbst bespielte
  „SOMMER '88“).
- Tisch: massive Eiche neu (Bretter entlang der Tischlänge – vorher quer –, feine Faserlinien,
  dunkles Spätholz, Poren, dunkleres Braun, flaches Relief gegen Glitzern), Platte mit
  gerundeter Oberkante und abgesetzter Unterkante, Kopfleisten mit quer laufender Faser,
  gedrechselte Beine mit Wulst. Parkett nutzt dieselbe Eiche.

## 2026-11-02 · roehre-Hauptsitzung · Vulkan unter Windows
- `vulkan_spiel.c` jetzt auch für Windows (Win32-Handles, Vulkan zur Laufzeit geladen).
- Lauf beim Anwender (RTX 5070 Ti): Bilder flach und Raum korrekt. Ob mit geteilten Semaphoren:
  letzte Ausgabezeile noch nicht gesehen.

## 2026-11-02 · roehre-Hauptsitzung · GPL-freie Variante (W-003)
- Neue Option `allgemein.lizenz` (gpl/permissiv), Ketten BeansRgb/BeansSignal/BeansSchnell
  aus crt-beans (MIT) mit eigener Übersetzung (`plane_beans`), eigener Helligkeitspass
  `shader/eigen/beans/helligkeit.slang`, Voreinstellungen `mit-rgb`, `mit-composite`.
- Ohne `shader/fremd/gpl/` schaltet roehre selbst auf die MIT-Kette (Wache
  `ohne_gpl_ordner_rechnet_die_mit_kette`).
- Abstimmung gemessen: Fleck 1,00/0,80 statt 0,90/0,40 (sonst Zeilenschwebung bis 3,25 % bei
  1080p), Verstärkung 1,15 (Weiß 100–102 %, Grau 97 %). Prüflauf bestanden.
- Grenzen: kein gewölbtes Glas, keine Farbsäume/Regenbogen, kein Nachleuchten u. a.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 8 abgeschlossen
- D3D12-Austausch beim Anwender (RTX 5070 Ti) in Ordnung, flach und Raum.
- Alle geplanten Etappen 0–8 abgeschlossen. Offen bleiben: W-003 (permissive Variante ohne
  GPL-Shader), Vulkan mit geteilten Semaphoren auf echter Hardware, Messung auf Mittelklasse.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 8, Messwerte und D3D12-Fehler
- Messung des Anwenders (RTX 5070 Ti, Vulkan): Filter 0,15–0,66 ms (1080p), 0,48–1,00 ms (4K);
  3D-Raum 1,17 / 2,52 ms. Composite/S-Video +0,4 ms bei 4K, alles andere unter 0,2 ms.
- Erster D3D12-Lauf brach ab: `enable_cache: true` ließ librashader einen Pipeline-Cache anlegen,
  den wgpu unter D3D12 nicht kennt. Jetzt nur, wenn das Gerät `PIPELINE_CACHE` hat. Neue Wache
  `roehre-wgpu/tests/ohne_zusatzfaehigkeiten.rs` (schlägt ohne die Korrektur genau so fehl).
- Offen: zweiter D3D12-Lauf, Messung unter D3D12.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 8 (Teil 1 und 2)
- Messbefehl `roehre-pruefstand leistung` (GPU-Zeitstempel; Filter je Voreinstellung, 3D-Raum,
  Kosten je Regler, 1080p/4K, Anteil am 60-Hz-Budget). Auf lavapipe nur auf Funktion geprüft.
- Windows-Paket (MinGW-Querbau) mit Prüfstand, Vorschau, DLL und Beispielen für den Anwender.
- Englische Texte für alle Optionen und Voreinstellungen, mit Wachen; Clippy sauber.
- 3D-Raum: Konsole im Stil des Super Famicom mit Controller (Wunsch des Anwenders, ohne Logo).
- Mission: Impossible kann mit dem Einbau beginnen; Schnittstelle wird nur noch ergänzt.
- Offen: Messwerte und D3D12-Lauf vom Rechner des Anwenders.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 7
- Neue Kiste `roehre-c` mit `include/roehre.h`: Katalog als JSON, Einstellungen (laden, setzen,
  speichern), Vorschaubild als RGBA für Launcher (W-007), Austausch mit Vulkan- und
  D3D12-Spielen (W-001), beide Modi (flach, 3D-Raum), Strahlsimulation im Takt.
- Entscheidung: eigenes Gerät auf derselben Grafikkarte mit geteiltem Speicher statt Rechnen auf
  dem Gerät des Spiels (wgpu übernimmt kein fremdes D3D12-Gerät; ein fremdes Vulkan-Gerät nur mit
  passenden Funktionen, die RT64/plume nicht einschaltet). Kostet zwei Kopien je Bild.
- Geprüft: `beispiele/vorschau.c` und `beispiele/vulkan_spiel.c` laufen auf lavapipe (dort ohne
  geteilte Semaphoren: Notweg über die CPU). Windows-DLL quer gebaut, `beispiele/d3d12_spiel.cpp`
  übersetzt und gelinkt, nicht ausgeführt.
- Einbauanleitungen: Mission: Impossible mit Einstiegsstelle in RT64
  (`rt64_present_queue.cpp`), Terranigma (wgpu 24 → 30).
- Ungeprüft: D3D12 und Vulkan mit Semaphoren auf echter Hardware, Kosten.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 6 abgeschlossen
- Kamera des Raums (W-005) als Optionen der neuen Gruppe `raum` (W-006): Nähe als Anteil der
  Fensterhöhe (weit 24 %, mittel 45 %, nah 80 %), Blickwinkel ±30°, Schwenk um das Gerät,
  Lage im Fenster per außermittiger Projektion (Lens Shift), passt bei 4:3 bis 32:9.
  Prüfläufe ohne GPU: Gerät nie abgeschnitten, Anteil stimmt, Schwenk hält das Gerät fest.
- Tiefenunschärfe (Abstand im Alphakanal, gedrehtes Muster) und Glühen (1/8-Zwischenstufe).
- Bücherregal, Straßenlicht durchs Fenster, vordere Wand weiter weg (Weitwinkel bei 32:9).
- Fehler behoben: die Eckenverschattung dunkelte jede Wand ganz ab (Abstand zur eigenen Fläche).
- Wünsche W-005 bis W-008 des Mission:-Impossible-Ports eingeplant (C-API Etappe 7,
  Messung Etappe 8); nichts vorgezogen.
- Ungeprüft: Wirkung und Leistung auf echter GPU; Vorschau-Fenster nur gestartet, nicht bedient.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 6, Teil 4 (3D-Raum nach Rückmeldung)
- Rückmeldung des Anwenders umgesetzt: Gummibaum als Ficus elastica (drei Triebe, große
  gefaltete Blätter mit Stiel, durchscheinend im Gegenlicht, rote Triebhülle), Stehlampe in
  die Zimmerecke, Pflanze neben den Tisch; Wildeiche für Tisch und Parkett (Äste, Risse,
  Farbspiel, matt geölt); schlanke Fernbedienung mit Tasten als Textur; schmale dunkle
  Fußleisten; Goldrahmen mit Profil und Passepartout; neblige Nacht vor dem Fenster;
  Schmutz und Verschattung in den Raumkanten.
- Texturen jetzt 256² und als Rohpixel (Koordinate rastet auf die Texelmitte, Mip und
  Anisotropie bleiben), wie vom Anwender gewünscht.
- Neu: `Raum::wandbild_setzen` (eigenes Bild im Rahmen). Prüfstand `raum --wandbild <png>`,
  Vorschau liest `ROEHRE_WANDBILD` oder `lokal/wandbild.png` (`lokal/` ist nicht im Repository).
- Ungeprüft: Wirkung und Leistung auf echter GPU.

## 2026-11-02 · roehre-Hauptsitzung · Korrektur Mission: Impossible
- Hinweis der KI des Mission:-Impossible-Projekts (über den Anwender): Integrationsnotiz war
  veraltet. Jetzt: Repository MissionImpossibleDecomp, Port in C++ mit RT64 (D3D12/Vulkan),
  Projektregel „kleine Eingriffe in Upstream-Code“, Wünsche W-001 und W-004 präzisiert.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 5
- Bewegungsschärfe: Strahlsimulation nach Blur Busters mit echtem Vorgängerbild, Schwarzbilder,
  Schnittstelle `zeichne_im_takt`. Lichtsumme bei 120/144/165/240/480 Hz auf 0,5 % genau.
- Befund: Strahl darf bei nicht ganzzahligem Verhältnis nicht je Spielbild neu beginnen
  (bis 20 % Licht fehlten).
- Ungeprüft: Wirkung am echten Bildschirm mit hoher Bildwiederholrate.
- Bericht: `docs/etappen/etappe-5.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 4
- Berechnete Gehäuse (nur Glas, Fernseher, Monitor, Automat) mit Glas, Innenfase mit
  Spiegelung, Licht des Bildes auf Rahmen und Wand; Breitbild-Röhre (16:9-Glas).
- Automatisches Einrasten unter 4 px je Zeile: 720p-Zeilenschwebung aller Voreinstellungen
  von bis zu 4,6 % auf ≤ 0,5 %. Referenzbilder neu geschrieben (Bild bei 540p kleiner).
- Ungeprüft: Gehäuse nur in WGSL (nicht in der künftigen C-API).
- Bericht: `docs/etappen/etappe-4.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 3
- Sechs neue Voreinstellungen, Optionen `strahl.zeilenverdopplung` und `signal.bandbreite`.
- Mitteltöne waren zu dunkel (Gamma 1,95, Maske in dunklen Teilen zu stark): Standards
  geändert, alle Voreinstellungen automatisch abgestimmt (Weiß 97–103 %, Grau 87–101 %).
- `strahl.schaerfe` wirkt jetzt auch in den NTSC-/PAL-Ketten (`SIGMA_HOR`).
- Prüflauf `roehre-pruefstand pruefe` mit Referenzbildern, Moiré- und Helligkeitsgrenzen;
  GitHub-Workflow `pruefung.yml` (noch nie gelaufen).
- Bericht: `docs/etappen/etappe-3.md`.

## 2026-11-02 · roehre-Hauptsitzung · Übergabe für andere KIs
- `AGENTS.md`, `CLAUDE.md`, `docs/ki/` (STAND, Wunschliste, Logbuch, Prompt-Vorlage) angelegt,
  damit KIs anderer Spielprojekte roehre verstehen, einbauen und selbst verbessern können.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 2
- Virtuelle Zeilenzahl, Moiré-Messung, HDR mit Eichung, Flächen-Modus für 3D-Szenen.
- Fehler behoben: Voreinstellungen waren seit Etappe 1 beschädigt; Overscan halbiert.
- Ungeprüft: HLSL-Fassung der Flächen-Maske; HDR nicht auf echtem HDR-Bildschirm angesehen.
- Bericht: `docs/etappen/etappe-2.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 1
- Grundgerüst: Einstellungsmodell, Kettenzuordnung, wgpu-Schnittstelle, Testbilder, Prüfstand.
- Bericht: `docs/etappen/etappe-1.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 0
- Recherche und Plan: `docs/PLAN.md`.
