# Cathode Retro (slang-Portierung): Herkunft

- **Autor des Originals:** Joshua Jersild (DeadlyRedCube), https://github.com/DeadlyRedCube/Cathode-Retro
- **Portierung nach slang:** hunterk (libretro), nach einem Stand von etwa November 2023
- **Lizenz:** MIT. Die Dateiköpfe der Portierung sagen „license: MIT“. Das Original stand von
  2022-02-15 (Commit `f8cf442`) bis 2025-12-01 unter MIT; erst danach (Commit `4ae960a`)
  wurde die LICENSE-Datei des Originals auf Boost Software License 1.0 geändert. Der hier
  übernommene Stand stammt aus der MIT-Zeit. `LICENSE` ist der Text aus `f8cf442`.
- **Quelle:** https://github.com/libretro/slang-shaders
- **Commit:** `a02157c6ca60c982229af01ad706302761bf482c` (2026-10-31)
- **Übernommen am:** 2026-11-02

## Übernommene Dateien

| Ursprung | Hier |
|---|---|
| `crt/cathode-retro_no-signal.slangp` | Ordnerwurzel |
| `crt/shaders/cathode-retro/**` | `shaders/cathode-retro/**` |

## Änderungen

| Datei | Änderung | Grund |
|---|---|---|
| `cathode-retro_no-signal.slangp` | `shader3 = ../stock.slang` → `../../../eigen/durchreichen.slang` | `stock.slang` liegt außerhalb des Ordners; ersetzt durch einen gleichwertigen eigenen Durchreich-Pass |
| `shaders/cathode-retro/signal_test.slangp` | `shader8 = ../../../stock.slang` → `../../../../../eigen/durchreichen.slang` | wie oben |
