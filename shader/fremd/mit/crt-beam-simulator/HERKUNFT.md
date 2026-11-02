# Blur Busters CRT Beam Simulator: Herkunft

- **Autoren:** Mark Rejhon (Blur Busters), Timothy Lottes
- **Lizenz:** MIT (Kopf in `shaders/crt-beam-simulator.slang`; `LICENSE` aus dem Original-Repository)
- **Original:** https://github.com/blurbusters/crt-beam-simulator, Commit `734786a6c48f954af11cb390e38a9e06107ffdd9`
- **slang-Fassung:** https://github.com/libretro/slang-shaders, Commit `a02157c6ca60c982229af01ad706302761bf482c`
- **Übernommen am:** 2026-11-02

## Übernommene Dateien

| Ursprung | Hier |
|---|---|
| `subframe-bfi/crt-beam-simulator.slangp` | Ordnerwurzel |
| `subframe-bfi/shaders/crt-beam-simulator.slang` | `shaders/` |
| `LICENSE` (Original-Repository) | `LICENSE` |

## Änderungen

Keine.

## Weiterverwendung in eigenem Code

Der Kern des Algorithmus (Photonenbudget je Bildpunkt, Überlappung mit dem Strahlfenster,
Nachleuchten über zwei Auffrischungszyklen) ist nach `crates/roehre-wgpu/src/zusammensetzen.wgsl`
(Funktionen `strahl_*`, `ueberlappung`, `roehre_roh`) übertragen, mit dem MIT-Lizenztext im
Kommentar. Abweichungen von der Vorlage:

| Was | Vorlage | roehre | Warum |
|---|---|---|---|
| Vorherige Bilder | liest das aktuelle Bild noch einmal (Demo-Fassung) | echtes vorheriges Spielbild | richtiges Nachleuchten bei Bewegung |
| Drittes Bild („Curr“) | eigenes Bild | aktuelles Bild | überlappt bei ganzzahligem Verhältnis nie; keine zusätzliche Latenz |
| Strahllage | Auffrischung mod N | ebenso; bei ganzzahligem N beim neuen Spielbild auf den Zyklusanfang eingerastet | ausgelassene Spielbilder verschieben den Strahl nicht |
| LCD-Schutz | N + 0,001 bei geradem N | Phase wandert um 0,001 Teilbilder je Spielbild | gleiche Wirkung, passt zum eingerasteten Strahl |

Gemessen: Das Mittel der Teilbilder ist bei N = 2; 2,4; 2,75; 4; 8 auf 0,005 genau
Original × Verstärkung (`roehre-pruefstand bewegung`, Teil von `pruefe`).
