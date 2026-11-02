# Fremde Arbeiten in roehre

Diese Datei listet jede fremde Arbeit, die `roehre` nutzt oder einbettet, mit Herkunft und Lizenz.
Eingebettete Dateien liegen unter `shader/fremd/<lizenz>/<name>/`, jeweils mit `LICENSE` und
`HERKUNFT.md` (Repository, Commit, Autor, Änderungsliste).

**Stand Etappe 1:** Die Shader unten liegen als eigene Dateien im Repository (nicht ins Programm
eingebettet) und werden mit dem Spiel als Ordner `shader/` ausgeliefert.

## Übernommen (als Dateien unter `shader/fremd/`)

| Name | Autor | Lizenz | Quelle | Commit | Ordner (geplant) |
|---|---|---|---|---|---|
| crt-guest-advanced (inkl. ntsc, pal, hd, fast) | guest(r) | GPL-2.0-or-later | https://github.com/libretro/slang-shaders | `a02157c6ca60c982229af01ad706302761bf482c` | `shader/fremd/gpl/guest-advanced/` |
| crt-beans | Andrew Duffey | MIT (`blue_noise.png`: CC0) | https://github.com/libretro/slang-shaders | `a02157c6ca60c982229af01ad706302761bf482c` | `shader/fremd/mit/crt-beans/` |
| Cathode Retro (slang-Portierung) | Joshua Jersild / DeadlyRedCube (Original), hunterk (Portierung) | MIT (Stand aus der MIT-Zeit des Originals, siehe HERKUNFT.md; das Original steht seit 2025-12-01 unter BSL-1.0) | https://github.com/libretro/slang-shaders, Original https://github.com/DeadlyRedCube/Cathode-Retro | `a02157c6ca60c982229af01ad706302761bf482c` | `shader/fremd/mit/cathode-retro/` |
| CRT Beam Simulator (Shaderdatei; Algorithmus zusätzlich übertragen nach `crates/roehre-wgpu/src/zusammensetzen.wgsl`, mit Lizenztext) | Mark Rejhon (Blur Busters), Timothy Lottes | MIT | https://github.com/blurbusters/crt-beam-simulator | `734786a6c48f954af11cb390e38a9e06107ffdd9` | `shader/fremd/mit/crt-beam-simulator/` |

## Abhängigkeiten über Cargo (nicht im Repository)

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
| `hdr10.h` aus slang-shaders | Nur ein kleiner Teil ist als MIT (Microsoft) gekennzeichnet, der Rest hat keine Lizenzangabe. Die HDR-Formeln schreibt roehre selbst nach SMPTE ST 2084 und ITU-R BT.2087. |
| blargg-GPU-Portierung in slang-shaders (`ntsc/shaders/blargg`) | Datei sagt selbst „no license given“ |

Weitere Rust-Abhängigkeiten (wgpu, image, serde, toml, thiserror, pollster) sind übliche
Crates unter MIT und/oder Apache-2.0; die genaue Liste ergibt `cargo tree`.

## Eigene Arbeiten

`crates/` und `shader/eigen/` sind eigene Arbeit von roehre unter MIT OR Apache-2.0.
