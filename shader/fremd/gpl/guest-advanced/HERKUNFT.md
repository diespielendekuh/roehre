# crt-guest-advanced: Herkunft

- **Autor:** guest(r) (guest.r@gmail.com); NTSC-Pässe nach Themaister, PAL-Pässe von guest(r)
- **Lizenz:** GPL-2.0-or-later (Lizenzkopf in den Hauptdateien; Volltext in `LICENSE`)
- **Quelle:** https://github.com/libretro/slang-shaders
- **Commit:** `a02157c6ca60c982229af01ad706302761bf482c` (2026-10-31)
- **Übernommen am:** 2026-11-02

## Übernommene Dateien (Ursprungspfad → Pfad hier)

| Ursprung | Hier |
|---|---|
| `crt/crt-guest-advanced.slangp`, `-ntsc`, `-pal`, `-hd`, `-fast`, `-fastest` | Ordnerwurzel |
| `crt/shaders/guest/advanced/**` | `shaders/guest/advanced/**` |
| `crt/shaders/guest/hd/**` | `shaders/guest/hd/**` |
| `crt/shaders/guest/fast/**` | `shaders/guest/fast/**` |

Die Ordnerstruktur ist so gewählt, dass die relativen Pfade in den `.slangp`-Dateien
unverändert gelten.

## Nicht übernommen

- `shaders/guest/advanced/grade/` (Dogway, wird von keinem der übernommenen Presets genutzt)
- `shaders/guest/advanced/README_old`

## Hinweise zur Lizenz einzelner Dateien

- `shaders/guest/advanced/ntsc/ntsc-pass1..3.slang` tragen keinen eigenen Lizenzkopf
  („NTSC-Adaptive, based on Themaister's NTSC shader“). Sie sind Teil des Pakets
  crt-guest-advanced-ntsc und werden hier wie dieses unter GPL-2.0-or-later geführt.
- `shaders/guest/advanced/pal/pal-pass1..3.slang` („by guest.r“) tragen keinen eigenen
  Lizenzkopf und werden wie das Paket geführt.
- `stock.slang` in den drei Unterordnern ist ein trivialer Durchreich-Pass ohne Lizenzkopf.
- Die Farb-LUTs unter `shaders/guest/advanced/lut/` gehören zum Paket.

## Änderungen

Keine. Alle Dateien liegen im Originalzustand vor. `roehre` setzt nur Parameterwerte zur
Laufzeit.
