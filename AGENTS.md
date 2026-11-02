# AGENTS.md – Anleitung für jede KI, die mit roehre arbeitet

`roehre` ist **keine eigenständige Anwendung**, sondern eine Bibliothek, die Spiele benutzen:
Sie stellt das Bild eines Spiels so dar, als liefe es auf einem echten Röhrenfernseher oder
-monitor (CRT). Mehrere Spieleprojekte bauen darauf auf, jedes mit eigener KI. Diese Datei
sagt dir, wie du dich hier zurechtfindest, wie du roehre in dein Spiel einbaust und wie du
roehre selbst verbesserst, ohne die anderen Projekte zu beschädigen.

## 1. Zuerst lesen (in dieser Reihenfolge)

1. **[docs/ki/STAND.md](docs/ki/STAND.md)**: was fertig ist, wie man es einbaut, was fehlt.
   Immer aktuell; das ist die wichtigste Datei.
2. **[docs/ki/wuensche.md](docs/ki/wuensche.md)**: was andere Projekte brauchen und noch fehlt.
3. **[docs/ki/logbuch.md](docs/ki/logbuch.md)**: wer zuletzt was geändert hat.
4. Wenn du ein bestimmtes Spiel einbaust: **`docs/integration/<projekt>.md`**, falls vorhanden.
5. Nur bei Bedarf: [docs/PLAN.md](docs/PLAN.md) (Architektur und Begründungen),
   [docs/etappen/](docs/etappen/) (Berichte mit Messwerten und Fotos),
   [THIRD_PARTY.md](THIRD_PARTY.md) (Lizenzen fremder Shader).

## 2. Du willst roehre in dein Spiel einbauen

Lies `docs/ki/STAND.md`, Abschnitt „Einbauen“. Kurz: Rust/wgpu über `crates/roehre-wgpu`, C/C++
mit Vulkan oder D3D12 über `crates/roehre-c` (`include/roehre.h`),
Einstellungen und Menütexte über `crates/roehre-modell`, Shader-Ordner `shader/` mit dem
Spiel ausliefern.

Wenn etwas fehlt oder für dein Spiel nicht passt, hast du zwei Wege:

- **Kleine, klare Lücke** (ein Parameter fehlt, ein Format wird nicht unterstützt, ein Fehler):
  **Behebe es selbst hier im Repository** nach den Regeln in Abschnitt 3.
- **Große oder unklare Sache** (neue Schnittstelle, anderer Grafik-Unterbau, Entscheidung
  nötig): **Trage es in `docs/ki/wuensche.md` ein** (Vorlage dort) und arbeite in deinem
  Spiel vorerst ohne. Wer als Nächstes an roehre arbeitet, sieht den Wunsch.

Baue roehre **nicht** in deinem Spiel nach oder um (keine Kopie der Shader mit eigenen
Änderungen im Spielrepository). Änderungen gehören hierher, damit alle Projekte davon haben.

## 3. Du änderst roehre selbst: die Regeln

**Sprache:** Alles auf Deutsch (Code-Kommentare, Doku, Commit-Texte, Bezeichner in ASCII mit
ae/oe/ue). Moderne Fachbegriffe (Shader, Preset, HDR, Mipmap …) sind erwünscht. Kommentare
erklären *warum*, mit der Zahl, an der ein Befund hängt.

**Erst messen, dann behaupten.** Es gibt einen Prüfstand, der ohne Grafikkarte rendert
(Mesa lavapipe). Wer etwas am Bild ändert, misst vorher und nachher:

```bash
cargo test --workspace                                   # alle Prüfläufe
cargo build --release -p roehre-pruefstand
./target/release/roehre-pruefstand pruefe                # Pflicht vor jedem Push: Referenzbilder, Moiré, Helligkeit
./target/release/roehre-pruefstand bogen -g 1920x1080 -o /tmp/bogen   # Fotos aller Voreinstellungen
./target/release/roehre-pruefstand moire                 # Moiré-Messung (Ziel: ab 1080p ≤ 1,1 %)
./target/release/roehre-pruefstand hdr                   # HDR-Leuchtdichten
./target/release/roehre-pruefstand umschalten            # Kettenwechsel ohne Ruckeln
```

Unter Linux ohne GPU vorher: `apt-get install -y mesa-vulkan-drivers`.

**Nichts darf still schlechter werden.** `roehre-pruefstand pruefe` muss bestehen. Ändert
sich das Bild gewollt, schreibe die Referenzen mit `pruefe --neu` neu, sieh dir die Bilder an
(`crates/roehre-pruefstand/referenz/`) und begründe die Änderung im Logbuch. Die Moiré- und Helligkeitswerte in
`docs/etappen/` sind der Maßstab. Verschlechtert deine Änderung für ein anderes Projekt etwas
(z. B. andere Voreinstellung, andere Auflösung), mach es über eine Einstellung oder einen
Schalter, nicht über einen geänderten Standard.

**Lizenzen sind streng getrennt.**
- Eigener Code und eigene Shader: `crates/`, `shader/eigen/` – Lizenz MIT OR Apache-2.0.
- Fremde Shader: `shader/fremd/<gpl|mit>/<name>/` mit `LICENSE` und `HERKUNFT.md`
  (Repository, Commit, Autor, Änderungsliste). Jede Änderung an einer fremden Datei kommt in
  deren `HERKUNFT.md`. Neue fremde Arbeiten nur mit geprüfter Lizenz und Eintrag in
  `THIRD_PARTY.md`. Keine Datei ohne klare Lizenz übernehmen.
- Keine Spielgrafik, keine ROM-Inhalte, keine Fotos fremder Geräte ins Repository. Testbilder
  werden in `crates/roehre-testbilder` selbst erzeugt.

**Schnittstellen sind ein Versprechen an andere Spiele.** Wenn du eine öffentliche Funktion
oder einen Optionsschlüssel (`gruppe.name`) änderst oder entfernst, prüfe, wer ihn nutzt
(`docs/integration/`), und schreibe es in `docs/ki/STAND.md` unter „Änderungen an
Schnittstellen“. Optionsschlüssel werden nie still umbenannt; dafür gibt es die Migration in
`crates/roehre-modell/src/einstellungen.rs` (`version` hochzählen).

## 4. Nach jeder Änderung: Stand hinterlegen (Pflicht)

Damit die nächste KI weiterarbeiten kann:

1. **`docs/ki/STAND.md`** aktualisieren: was jetzt geht, was sich an Schnittstellen geändert
   hat, was offen ist. Datum oben anpassen.
2. **`docs/ki/logbuch.md`**: oben einen Eintrag (Datum, wer – Projekt bzw. KI –, was, warum,
   was ungeprüft ist).
3. **`docs/ki/wuensche.md`**: erledigte Wünsche als erledigt markieren (mit Commit),
   neue eintragen.
4. Wenn du ein Spiel eingebaut hast: `docs/integration/<projekt>.md` anlegen oder
   aktualisieren (wie eingebaut, welche Einstellungen, was dort besonders ist).
5. Commit mit deutschem Text, der den **Befund** erzählt, nicht die Dateiliste.

## 5. Git

- Arbeite auf einem **eigenen Branch** (z. B. `ki/<projekt>-<thema>`), nicht direkt auf dem
  Standardbranch, außer der Anwender erlaubt es ausdrücklich.
- Erstelle danach einen Pull Request gegen den Standardbranch, wenn der Anwender das wünscht;
  sonst sag ihm, auf welchem Branch deine Änderung liegt.
- Keine Builds (Binärdateien) ins Repository.
