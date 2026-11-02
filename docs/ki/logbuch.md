# Logbuch: wer hat was an roehre geändert

Neue Einträge **oben**. Je Eintrag: Datum, wer (Projekt bzw. KI), was, warum, was ungeprüft ist.

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
