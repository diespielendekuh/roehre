# Logbuch: wer hat was an roehre geändert

Neue Einträge **oben**. Je Eintrag: Datum, wer (Projekt bzw. KI), was, warum, was ungeprüft ist.

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
