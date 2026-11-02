# Fremde Arbeiten in roehre

Diese Datei listet jede fremde Arbeit, die `roehre` nutzt oder einbettet, mit Herkunft und Lizenz.
Eingebettete Dateien liegen unter `shader/fremd/<lizenz>/<name>/`, jeweils mit `LICENSE` und
`HERKUNFT.md` (Repository, Commit, Autor, Änderungsliste).

**Stand Etappe 0:** Noch ist nichts eingebettet. Die Tabelle zeigt, was laut Plan übernommen wird.

## Geplant: eingebettet

| Name | Autor | Lizenz | Quelle | Commit | Ordner (geplant) |
|---|---|---|---|---|---|
| crt-guest-advanced (inkl. ntsc, pal, hd, fast) | guest(r) | GPL-2.0-or-later | https://github.com/libretro/slang-shaders | `a02157c6ca60c982229af01ad706302761bf482c` | `shader/fremd/gpl/guest-advanced/` |
| crt-beans | Andrew Duffey | MIT | https://github.com/libretro/slang-shaders | `a02157c6ca60c982229af01ad706302761bf482c` | `shader/fremd/mit/crt-beans/` |
| Cathode Retro (slang-Portierung) | DeadlyRedCube (Original), libretro (Portierung) | MIT | https://github.com/libretro/slang-shaders, Original https://github.com/DeadlyRedCube/Cathode-Retro | `a02157c6ca60c982229af01ad706302761bf482c` | `shader/fremd/mit/cathode-retro/` |
| CRT Beam Simulator | Mark Rejhon (Blur Busters), Timothy Lottes | MIT | https://github.com/blurbusters/crt-beam-simulator | `734786a6c48f954af11cb390e38a9e06107ffdd9` | `shader/fremd/mit/crt-beam-simulator/` |
| HDR10-Hilfsfunktionen (`hdr10.h`) | Microsoft Corporation (Xbox-ATG-Samples) | MIT | https://github.com/libretro/slang-shaders (`hdr/shaders/include/hdr10.h`) | `a02157c6ca60c982229af01ad706302761bf482c` | `shader/fremd/mit/ms-hdr10/` |

## Geplant: Abhängigkeit (nicht eingebettet, über Cargo)

| Name | Autor | Lizenz | Quelle | Version |
|---|---|---|---|---|
| librashader | Ronny Chan und Mitwirkende | MPL-2.0 (wahlweise GPL-3.0) | https://github.com/SnowflakePowered/librashader | 0.12.1 (`07ac790cdb9826f37cf8e1daf5011f03e3c4daa7`) |

## Nur Referenz (kein Code, keine Grafiken übernommen)

| Name | Lizenz | Wofür |
|---|---|---|
| crt-royale (TroggleMonkey) | GPL-2.0-or-later | Abgleich der Lochmaske in hoher Auflösung, ggf. später eingebettet |
| koko-aio (kokoko3k) | GPL-3.0, Teile CC BY-SA | Ideen für Umgebungslicht und Bezel |
| Mega Bezel (HyperspaceMadness) | GPL-3.0 | Ideen für Glas und Spiegelung im Rahmen |
| blargg snes_ntsc / nes_ntsc (Shay Green) | LGPL-2.1 | CPU-Referenz für die NTSC-Signalstufe |

## Ausdrücklich nicht übernommen

| Name | Grund |
|---|---|
| Sony Megatron (HDR) | Keine Lizenzangabe in den Dateien gefunden |
| ntsc-adaptive | Keine Lizenzangabe in den Dateien gefunden |
| blargg-GPU-Portierung in slang-shaders (`ntsc/shaders/blargg`) | Datei sagt selbst „no license given“ |
