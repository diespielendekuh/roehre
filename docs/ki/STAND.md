# Stand von roehre (für KIs und Menschen)

**Zuletzt aktualisiert:** 2026-11-02, nach Etappe 6 · **Standardbranch:** `claude/exciting-brahmagupta-5fyjr0`

Diese Datei ist die eine Stelle, an der der aktuelle Stand steht. Wer roehre ändert,
aktualisiert sie (Regeln: [AGENTS.md](../../AGENTS.md)).

## 1. Was roehre ist

Eine Bibliothek, die das Bild eines Spiels wie auf einer echten Röhre darstellt: Signal (RGB,
S-Video, Composite, Antenne; NTSC/PAL), Strahl und Zeilen, Phosphormaske, Leuchtstoff,
Licht im Glas, Wölbung, Helligkeitsausgleich, HDR. Unterbau ist
[librashader](https://github.com/SnowflakePowered/librashader) 0.12.1, das RetroArch-„slang“-
Shaderketten ausführt; die Röhre selbst ist crt-guest-advanced (GPL-2.0+), ergänzt um eigene
Stufen davor und danach.

```
Spielbild → [Vorstufe: virtuelle Zeilen] → [Röhre: guest-advanced] → [Zusammensetzen: Lage, Hintergrund, SDR/HDR] → Ziel
            eigen (MIT/Apache)              fremd (GPL)                eigen (MIT/Apache)
```

## 2. Was fertig ist

| Bereich | Stand | Wo |
|---|---|---|
| Einstellungsmodell (70 Optionen, 13 Gruppen, Grenzen, deutsche Texte, TOML/JSON, Versionierung) | fertig | `crates/roehre-modell` |
| Voreinstellungen `aus`, `scart-rgb`, `wohnzimmer-90er-composite`, `wohnzimmer-90er-pal`, `svideo`, `pvm`, `bvm`, `arcade`, `pc-vga`, `breitbild-roehre` | fertig, auf Helligkeit und Moiré abgestimmt | `crates/roehre-modell/voreinstellungen/*.toml` |
| Breitbild-Röhre (16:9-Glas, 4:3-Inhalt) | fertig (`bildformat.roehrenformat`) | `gehaeuse.rs` |
| Permissive Variante ohne GPL-Shader (crt-beans) | **fehlt** (Wunsch W-003) | |
| Prüflauf: Referenzbilder, Moiré-, Helligkeitsgrenzen | fertig; GitHub-Workflow noch nie gelaufen | `roehre-pruefstand pruefe`, `.github/workflows/pruefung.yml` |
| Rust-Schnittstelle für wgpu 30 | fertig | `crates/roehre-wgpu` |
| Seitenverhältnis, Pixel-Seitenverhältnis, ganzzahlige Skalierung, Füllung | fertig | `crates/roehre-wgpu/src/flaeche.rs` |
| Virtuelle Zeilenzahl für hochaufgelöste 3D-Bilder | fertig | `shader/eigen/virtuelle-zeilen.slang` |
| HDR (scRGB, HDR10) mit Helligkeitseichung | fertig, nur gemessen, nicht auf HDR-Bildschirm angesehen | `crates/roehre-wgpu/src/eichung.rs`, `zusammensetzen.wgsl` |
| Flächen-Modus (Röhrenbildschirm als Objekt in einer 3D-Szene) | fertig; HLSL-Fassung ungeprüft | `bildschirmflaeche.rs`, `shader/eigen/flaeche/` |
| Umschalten ohne Ruckeln (Kettenbau im Hintergrund) | fertig | `Roehre::uebernehme` |
| Gehäuse (nur Glas, Fernseher, Monitor, Automat), berechnet, mit Spiegelung und Licht des Bildes | fertig; nur wgpu (WGSL) | `gehaeuse.rs`, `zusammensetzen.wgsl` |
| Bewegungsschärfe für 120–480 Hz: Strahlsimulation (Blur Busters, mit echtem Vorgängerbild) und Schwarzbilder | fertig; am echten Bildschirm ungeprüft | `Roehre::zeichne_im_takt`, `zusammensetzen.wgsl` |
| Vorschau-Programm mit Fenster (Testbild/PNG/Folge, A/B-Vergleich, Lupe, Bildschirmfoto, alle Optionen) | fertig | `crates/roehre-vorschau` |
| 3D-Vorführraum (Wohnzimmer der 80er, Röhrenfernseher auf Tisch, späte PS2-Optik, Rohpixel-Texturen 256², eigenes Wandbild, Tiefenunschärfe, Glühen) | fertig in Rust/wgpu; am echten Bildschirm und in der Leistung ungeprüft | `crates/roehre-raum`, `shader/eigen/raum/raum3d.wgsl` |
| Kamera des Raums (Nähe weit/mittel/nah, Blickwinkel, Schwenk, Lage im Fenster per Lens Shift, 4:3–32:9) als Optionen der Gruppe `raum` mit Festwerten (`festwerte()`) | fertig, mit Prüfläufen (nie abgeschnitten, Anteil, Schwenk hält das Gerät fest) | `Kamera`, `Ansicht::aus_einstellungen`, `roehre-pruefstand kamera` |
| C-API (Vulkan, D3D12) für C/C++-Spiele, auch für den 3D-Raum; Launcher-Vorschaubild | **fehlt** (Etappe 7, W-001/W-007) | |
| Leistungsmessung auf echter GPU (Filter / 3D-Raum, 1080p / 4K, Kosten je Regler) | **fehlt** (Etappe 8, W-008) | |

## 3. Einbauen (Rust, wgpu)

**Abhängigkeiten:** wgpu **30** (librashader 0.12 setzt das voraus; egui-wgpu 0.36/0.37 passt
dazu). Ein Spiel mit älterem wgpu muss anheben.

```toml
[dependencies]
roehre-wgpu = { git = "https://github.com/diespielendekuh/roehre", branch = "claude/exciting-brahmagupta-5fyjr0" }
```

```rust
use roehre_wgpu::{Roehre, Eingang, Einstellungen, Ausgabe};

// Gerät: roehre_wgpu::empfohlene_features() anfordern, soweit der Adapter sie hat.
let einstellungen = Einstellungen::voreinstellung("scart-rgb")?;     // oder aus TOML/JSON
let mut roehre = Roehre::neu(&device, &queue, "shader", einstellungen,
                             Ausgabe::sdr(surface_format))?;         // oder Ausgabe::scrgb() / hdr10()
let eingang = Eingang { breite: 256, hoehe: 224, pixel_seitenverhaeltnis: 8.0 / 7.0 };

// je Bild:
let rechteck = roehre.zeichne(&mut encoder, &spielbild_textur, &eingang, &ziel_ansicht, (breite, hoehe))?;
// Bildschirm mit mehr Hertz als das Spiel (z. B. 240 Hz, 60 Bilder/s), Bewegungsschärfe an:
// je Bildschirmauffrischung aufrufen, neues_spielbild nur bei der ersten nach einem neuen Bild
let takt = roehre_wgpu::Takt { neues_spielbild, teilbilder_je_spielbild: 240.0 / 60.0 };
roehre.zeichne_im_takt(&mut encoder, &spielbild_textur, &eingang, &ziel_ansicht, (breite, hoehe), takt)?;
// Einstellung geändert (z. B. aus dem Menü):
roehre.uebernehme(neue_einstellungen);    // Regler sofort, Kettenwechsel im Hintergrund
```

- `spielbild_textur`: das unvergrößerte Spielbild (`Rgba8Unorm`, sRGB-kodierte Werte, wie sie
  das Spiel erzeugt), Verwendung `TEXTURE_BINDING`.
- `zeichne` füllt das **ganze** Ziel (Röhrenfläche mittig, Rest Hintergrund `roehre.hintergrund`,
  linear; mit `gehaeuse.typ` zusätzlich Glas und Gehäuse). Ein Menü des Spiels wird danach
  darübergezeichnet. `roehre.anordnung(…)` liefert Bildfläche, Glas und Gehäuseform.
- Der Rückgabewert ist die Röhrenfläche in Pixeln (z. B. für Mauskoordinaten).
- **Shader-Ordner:** `shader/` aus diesem Repository wird **als Ordner neben dem Spiel**
  ausgeliefert (GPL-Shader werden bewusst nicht eingebettet). Pfad bei `Roehre::neu` angeben.

### Flächen-Modus (Bildschirm in einer 3D-Szene)

```rust
let ziel = roehre_wgpu::FlaechenZiel::neu(&device, 1440, 1080);
let werte = roehre.zeichne_flaeche(&mut encoder, &spielbild_textur, &eingang, &ziel)?;
// Im Material des Bildschirm-Modells: Farbe aus ziel.ansicht_srgb (trilinear, anisotrop),
// multipliziert mit roehre_maske(uv, werte) aus shader/eigen/flaeche/roehre_maske.wgsl
// (oder .glsl/.hlsl); werte.als_bytes() ist das Uniform-Struct.
```

Vorführung und Beispielcode: `crates/roehre-pruefstand/src/szene3d.rs`.

### Einstellungen im Menü des Spiels

roehre liefert **keine Oberfläche**. Das Spiel baut sein Menü aus dem Katalog:

```rust
for o in roehre_wgpu::modell::katalog() {
    // o.schluessel ("maske.typ"), o.gruppe.name().de, o.name.de, o.beschreibung.de,
    // o.art (Zahl{min,max,schritt,standard,einheit} | Ganzzahl | Schalter | Auswahl{werte}),
    // o.kosten, o.wirkt_wenn (nur anzeigen, wenn …)
}
let mut e = roehre.einstellungen().clone();
e.setze("maske.staerke", roehre_wgpu::Wert::Zahl(0.5));   // prüft Grenzen, liefert Hinweise
roehre.uebernehme(e);
let datei = roehre.einstellungen().als_toml(true);        // speichern (nur Abweichungen)
```

Voreinstellungen: `roehre_wgpu::modell::voreinstellungen()` (Schlüssel, Name, Beschreibung,
Lizenzmerkmal).

## 4. Wichtige Optionen (Auszug)

| Schlüssel | Bedeutung |
|---|---|
| `allgemein.roehre` | Röhre an/aus (aus = nur sauber eingepasst) |
| `signal.art`, `signal.norm` | rgb/svideo/composite/antenne, ntsc/pal |
| `maske.typ`, `maske.staerke`, `maske.triaden` | Maskenart, Stärke, Tripel über die Breite |
| `geometrie.woelbung_x/_y`, `geometrie.ecken`, `geometrie.overscan` | Glas und Rand |
| `helligkeit.ausgleich_*`, `helligkeit.nachher` | Helligkeitsausgleich SDR |
| `helligkeit.hdr_weiss`, `helligkeit.hdr_spitze` | HDR in nit |
| `bildformat.seitenverhaeltnis`, `bildformat.ganzzahlig`, `bildformat.fuellung` | Einpassen |
| `eingang.zeilen`, `eingang.waagerecht` | virtuelle Zeilenzahl (0 = automatisch) |
| `gehaeuse.typ`, `gehaeuse.zoom`, `bildformat.roehrenformat` | Gehäuse, Nähe, 16:9-Glas |
| `bewegung.modus`, `bewegung.helligkeit` | Strahlsimulation/Schwarzbilder ab 120 Hz |

Vollständig: `crates/roehre-modell/src/katalog.rs`.

## 5. Bekannte Grenzen

- Bekommt eine Zeile weniger als 4 Pixel (z. B. 720p), rastet die Bildhöhe automatisch auf ein
  ganzzahliges Vielfaches ein (Bild bis 30 % kleiner); sonst schweben die Zeilen.
- PVM, BVM, VGA skalieren ganzzahlig (harte Zeilen schweben sonst); bei 1080p ist das Bild
  dadurch 896 Pixel hoch.
- Composite: blaue Schrift auf Schwarz bleibt weich (wie am echten Gerät).
- Alles bisher nur auf der CPU (lavapipe) gerendert; Leistung auf echter GPU ungemessen.
- Nur Rust/wgpu. C/C++-Spiele (Vulkan, D3D12) brauchen das C-API (Wunsch W-001).

## 6. Änderungen an Schnittstellen

| Datum | Änderung |
|---|---|
| 2026-11-02 | Etappe 5: `Roehre::zeichne_im_takt`, `Takt`; Optionen `bewegung.*`. |
| 2026-11-02 | Etappe 4: Optionen `gehaeuse.*`, `bildformat.roehrenformat`; `Roehre::anordnung`; automatisches Einrasten unter 4 px je Zeile (Bildgröße bei kleinen Fenstern etwas kleiner als vorher). |
| 2026-11-02 | Standard von `helligkeit.gamma_ausgang` 1,95 → 2,4 und von `maske.staerke_dunkel` 1,1 → 0,4 (Mitteltöne waren zu dunkel). Gespeicherte Einstellungen mit den alten Werten bleiben gültig. |
| 2026-11-02 | Neue Optionen `strahl.zeilenverdopplung`, `signal.bandbreite`. |
| 2026-11-02 | `Roehre::neu` nimmt `impl Into<Ausgabe>` statt `wgpu::TextureFormat` (ein Format allein heißt weiterhin SDR). |
| 2026-11-02 | `roehre_kette::Kontext` hat neue Felder `hdr`, `flaeche_modus`; `virtuelle_groesse` braucht die Zielhöhe. |

## 7. Wo was liegt

| Pfad | Inhalt |
|---|---|
| `crates/roehre-modell` | Optionen, Einstellungen, Voreinstellungen (ohne GPU) |
| `crates/roehre-kette` | Einstellungen → Shaderkette und Parameter (Zuordnung zu guest-advanced) |
| `crates/roehre-wgpu` | Rust-Schnittstelle, Zusammensetzen, HDR-Eichung, Flächen-Modus |
| `crates/roehre-testbilder` | selbst erzeugte Testbilder |
| `crates/roehre-pruefstand` | Fotos, Moiré-, HDR- und Umschaltmessung ohne Fenster |
| `shader/eigen` | eigene Shader |
| `shader/fremd` | übernommene Shader mit Lizenz und Herkunft |
| `docs/ki` | Stand, Wunschliste, Logbuch, Vorlage für Prompts |
| `docs/integration` | Einbau je Spielprojekt |
| `docs/etappen` | Berichte mit Messwerten und Fotos |
