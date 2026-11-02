# Etappe 3: Voreinstellungen, Helligkeitsabstimmung, Prüflauf

Stand: 2026-11-02

## Was gebaut wurde

| Teil | Inhalt |
|---|---|
| Neue Voreinstellungen | `wohnzimmer-90er-pal`, `svideo`, `pvm`, `bvm`, `arcade`, `pc-vga` (zusammen 9 mit `aus`, `scart-rgb`, `wohnzimmer-90er-composite`) |
| Neue Optionen | `strahl.zeilenverdopplung` (31-kHz-Monitor; nur wenn jede Zeile ≥ 4 px bekommt), `signal.bandbreite` (Feinzeichnung des Signals, NTSC und PAL) |
| Prüfstand | `helligkeit` (Weiß und Mittelgrau je Voreinstellung), `pruefe` (Referenzbilder, Moiré- und Helligkeitsgrenzen, Fehlercode und Differenzbilder bei Verstoß; `--neu` schreibt die Referenzen neu) |
| Referenzbilder | `crates/roehre-pruefstand/referenz/*.png`, je Voreinstellung 1:1-Ausschnitt + Gesamtansicht (zusammen 1,6 MB). Lauf ist reproduzierbar (Abweichung 0,000); eine Änderung der Maskenstärke um 0,15 schlägt an. |
| GitHub-Workflow | `.github/workflows/pruefung.yml`: Tests und `pruefe` mit lavapipe, bei Pull Requests und von Hand. **Noch nie auf GitHub gelaufen**; ob die Mesa-Fassung des Runners innerhalb der Toleranz rendert, zeigt erst der erste Lauf. |

## Befunde und Korrekturen

1. **Mitteltöne waren viel zu dunkel.** Neue Messung des 40-%-Grau: nur 64–67 % des sauberen
   Bildes, obwohl Weiß stimmte. Zwei Ursachen:
   - Das Ausgabegamma stand auf 1,95 (aus der falschen guest-Datei übernommen); guest selbst
     nimmt 2,4. Neuer Standard 2,4.
   - Die Maskenstärke für dunkle Bildteile (`maske.staerke_dunkel`, guest `mcut`) stand auf
     1,0–1,2: Dunkle und mittlere Töne bekamen eine viel kräftigere Maske als helle. Eine echte
     Maske lässt bei jeder Helligkeit gleich viel Licht durch. Jetzt gleich der Stärke für helle
     Bildteile (Standard 0,4). Gemessen bei Arcade: Grau 64 % → 93 % bei gleichem Weiß.
   - Danach je Voreinstellung automatisch abgestimmt (Nachverstärkung für 100 % Weiß, kleinster
     Ausgleich dunkler Teile für ≥ 88 % Grau).
2. **`strahl.schaerfe` wirkte bei NTSC und PAL nicht.** Die Ketten mit Signalstufe filtern
   waagerecht mit `SIGMA_HOR` statt `h_sharp`; der Regler wird jetzt dorthin übersetzt.
3. **Composite-Schrift.** Die Unschärfe kam von der Signalbandbreite, nicht von den Artefakten.
   Mit `signal.bandbreite = 1,4` sind Magenta und Gelb auf Schwarz wieder lesbar, Blau auf
   Schwarz bleibt weich (Blau trägt kaum Helligkeit, am echten Gerät ebenso).
4. **Harte Zeilen schweben bei krummen Faktoren.** PVM, BVM und VGA zeigten bei 1080p bis 5,8 %
   Zeilenschwebung, unabhängig von der Maske. Diese drei Voreinstellungen skalieren deshalb
   ganzzahlig (≤ 0,25 % überall). Preis: Bei 1080p ist das Bild 896 statt 1080 Pixel hoch.
5. **Zeilenverdopplung auf 1080p** (2,4 px je Zeile) schwebte mit 8,5 %; sie greift jetzt nur,
   wenn jede Zeile mindestens 4 Pixel bekommt (bei 224 Zeilen ab 4K).

## Messwerte

### Helligkeit (SDR, 1080p, Anteil am sauberen Bild)

| Voreinstellung | 75-%-Weiß | Anteil | 40-%-Grau | Anteil |
|---|---:|---:|---:|---:|
| aus | 0.521 | 100 % | 0.133 | 100 % |
| scart-rgb | 0.516 | 99 % | 0.119 | 90 % |
| wohnzimmer-90er-composite | 0.524 | 101 % | 0.134 | 101 % |
| wohnzimmer-90er-pal | 0.529 | 102 % | 0.134 | 101 % |
| svideo | 0.522 | 100 % | 0.124 | 93 % |
| pvm | 0.515 | 99 % | 0.117 | 88 % |
| bvm | 0.537 | 103 % | 0.118 | 89 % |
| arcade | 0.507 | 97 % | 0.118 | 89 % |
| pc-vga | 0.521 | 100 % | 0.116 | 87 % |

Grenzen im Prüflauf: Weiß 95–105 %, Grau ≥ 85 %.

### Moiré (höchster Wert über SNES 256×224, N64 320×240, PAL 256×239)

| Voreinstellung | Zeilen ab 1080p (höchster Wert) | Zeilen bei 720p | Maske (höchster Wert) |
|---|---:|---:|---:|
| scart-rgb | 1.01 % | 1.97 % | 0.16 % |
| wohnzimmer-90er-composite | 0.67 % | 1.59 % | 0.13 % |
| wohnzimmer-90er-pal | 0.55 % | 1.53 % | 0.28 % |
| svideo | 0.79 % | 1.94 % | 0.01 % |
| pvm | 0.16 % | 0.12 % | 0.00 % |
| bvm | 0.18 % | 0.23 % | 0.00 % |
| arcade | 0.74 % | 1.21 % | 0.13 % |
| pc-vga | 0.13 % | 0.14 % | 0.08 % |

Grenzen im Prüflauf: Zeilen ab 1080p ≤ 1,2 %, bei 720p ≤ 2,5 %, Maske ≤ 0,5 %.
Vollständige Tabelle: `roehre-pruefstand moire`.

## Fotos

- Alle Voreinstellungen, 1080p: [uebersicht_1080p.jpg](../bilder/etappe-3/uebersicht_1080p.jpg)
- Pixelszene in 4K: [uebersicht_4k_pixelszene.jpg](../bilder/etappe-3/uebersicht_4k_pixelszene.jpg)
- Composite, S-Video, RGB nebeneinander (1:1): [composite_svideo_rgb_1080p_1zu1.png](../bilder/etappe-3/composite_svideo_rgb_1080p_1zu1.png)
- Schrift Composite NTSC und PAL (1:1): [schrift_composite_ntsc_pal_1080p_1zu1.png](../bilder/etappe-3/schrift_composite_ntsc_pal_1080p_1zu1.png)

## Selbstkritische Abnahme

| Frage | Befund |
|---|---|
| Zu dunkel? | Nein mehr: Weiß 97–103 %, Mittelgrau 87–101 % in allen Voreinstellungen. |
| Moiré? | Ab 1080p höchstens 1,01 %, Maske höchstens 0,28 %. 720p bis 2 % bei den Fernsehern. |
| Farben? | Unverändert; die Farbprofile der Leuchtstoffe sind noch nicht gegen Messwerte geprüft. |
| Unterscheidbarkeit? | Composite → S-Video → RGB sichtbar abgestuft; PVM/BVM mit harten Zeilen; Arcade hell und weich. |
| Ungeprüft | Der GitHub-Workflow (noch nie gelaufen); alle Voreinstellungen nur auf der CPU gerendert. |

## Nicht in dieser Etappe (verschoben)

- **Breitbild-Röhre (16:9-Glas mit 4:3-Inhalt):** braucht eine eigene Stufe, die den Inhalt auf
  der Röhre mit Balken einpasst; kommt mit dem Gehäuse (Etappe 4).
- **Leichte Leistungsstufe:** Die Option `allgemein.leistung = "leicht"` schaltet bei RGB auf
  crt-guest-advanced-fast; eigene leichte Voreinstellungen sind nicht nötig. Messung der
  Ersparnis erst auf echter GPU.
- **Permissive Variante (nur MIT-Shader, crt-beans):** Zuordnung der Optionen zu crt-beans fehlt
  noch; vorgemerkt als Wunsch W-003.
