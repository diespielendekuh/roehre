# Etappe 1: Fundament

Stand: 2026-11-02

## Was gebaut wurde

| Teil | Inhalt |
|---|---|
| `crates/roehre-modell` | Einstellungsmodell: 44 Optionen in 9 Gruppen mit Schlüssel, deutschem (teils englischem) Namen, Beschreibung, Grenzen, Schrittweite, Standard, Leistungskosten und Abhängigkeit („wirkt nur bei Composite“). Laden/Speichern als TOML und JSON, Vererbung über `basis = "…"`, Version mit Migrationsrahmen. Ungültige Werte werden mit Hinweis korrigiert, nie stillschweigend. |
| `crates/roehre-modell/voreinstellungen/` | `aus`, `scart-rgb`, `wohnzimmer-90er-composite` |
| `crates/roehre-kette` | Übersetzt Einstellungen in Shaderkette und Parameter von crt-guest-advanced. Wählt die Maske so, dass ein Farbtripel immer eine ganze Zahl von Ausgabepixeln breit ist (kein Moiré). |
| `crates/roehre-wgpu` | Rust-Schnittstelle: `Roehre::neu`, `zeichne`, `uebernehme`. Röhrenfläche auf ganzen Pixeln, Röhre zeichnet in eine Zwischentextur genau dieser Größe, eigener Pass setzt sie 1:1 ins Ziel. Kettenwechsel im Hintergrund-Thread. |
| `crates/roehre-testbilder` | Selbst erzeugte Testbilder: Pixelszene (256×224, 8:7), Farbbalken, Graukeil, Zonenplatte, Kreuzgitter, Schriftprobe (eigene 5×7-Schrift), 3D-Szene (eigener Strahlverfolger). |
| `crates/roehre-pruefstand` | Rendern ohne Fenster: `foto`, `bogen` (Kontaktbogen mit 1:1-Ausschnitten), `testbilder`, `umschalten`. |
| `shader/fremd/` | guest-advanced (GPL-2.0+), crt-beans, Cathode Retro, CRT Beam Simulator (MIT), je mit `LICENSE` und `HERKUNFT.md` |
| `shader/eigen/` | `scharf-bilinear.slang` (Voreinstellung „Sauber/aus“), `durchreichen.slang` |

## Fotos

Gerendert auf der CPU (lavapipe), deshalb ohne Zeitangaben.

- Übersicht 1080p: [uebersicht_1080p.jpg](../bilder/etappe-1/uebersicht_1080p.jpg). Je Zeile eine Voreinstellung,
  je Testbild verkleinerte Gesamtansicht und 1:1-Ausschnitt.
- Übersicht 4K: [uebersicht_4k.jpg](../bilder/etappe-1/uebersicht_4k.jpg)
- 1:1-Ausschnitte (PNG, verlustfrei): `docs/bilder/etappe-1/*_1zu1.png`

## Selbstkritische Abnahme

| Frage | Befund | Folge |
|---|---|---|
| Zu dunkel? | **Ja, im ersten Durchlauf.** 75-%-Weiß hatte nur 37 % (Scart) bzw. 57 % (Composite) der Leuchtdichte des sauberen Bildes. Ursache: Schlitzmaske und Scanlines nehmen zusammen fast zwei Drittel des Lichts weg. | Helligkeitsausgleich in beiden Voreinstellungen angehoben (`ausgleich_dunkel/hell`, `nachher`, breiterer Strahl bei hellen Stellen). Jetzt 100 % (0,52 linear wie das saubere Bild), Maske weiterhin deutlich sichtbar. Die automatische Helligkeitsprüfung kommt in Etappe 3 in den Prüfstand. |
| Moiré? | Nicht sichtbar. Die Maske liegt auf ganzen Pixeln: 1080p → 3 px je Tripel (480 Tripel), 4K → 6 px (480 Tripel). | Zonenplatten-Messung folgt in Etappe 2. |
| Farben falsch? | Farbbalken bleiben farbtreu; bei Composite laufen Farben an Kanten aus, wie gewollt. Gelb und Cyan werden durch die Maskenspitzen etwas gesättigter. | Feinabstimmung der Farbprofile in Etappe 3. |
| Schrift bei Composite? | Rote und blaue Schrift auf Schwarz war im ersten Durchlauf unlesbar verschmiert. | Farbartefakte 0,8 und Farbsäume 0,7 statt 1,0. Bleibt sichtbar weich, wie am echten Gerät. |
| Ränder unsauber? | Abgerundete Ecken und weicher Rand sind sauber. | – |
| Ruckelt das Umschalten? | Nein. Kettenwechsel läuft im Hintergrund (auf der CPU 0,3–1,1 s), die alte Kette zeichnet währenddessen weiter (`roehre-pruefstand umschalten`). | – |
| 3D-Szene in 1080p | Die Röhre arbeitet noch mit 1080 Eingangszeilen: Man sieht nur die Maske, keine Zeilen. | **Virtuelle Zeilenzahl ist Aufgabe von Etappe 2.** |

## Offen für die nächsten Etappen

- Etappe 2: Eingangsstufe (virtuelle Zeilenzahl, Overscan aus der Eingangsstufe statt guest),
  HDR-Ausgabe, Moiré-Messung, Flächen-Modus für den 3D-Monitor.
- Etappe 3: restliche Voreinstellungen, leichte und permissive Varianten, Referenzbilder in CI.
