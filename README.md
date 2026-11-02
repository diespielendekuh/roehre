# roehre

Röhrenfernseher- und Bildschirm-Simulation als wiederverwendbare Bibliothek für Spiele:
Signal (RGB, S-Video, Composite mit NTSC/PAL), Strahl, Maske, Leuchtstoff, Licht, Geometrie,
HDR, Strahlsimulation für Monitore ab 120 Hz und auf Wunsch ein prozedural erzeugtes Gehäuse.
Rust-API für wgpu, C-API für Vulkan und D3D12.

**Stand:** Etappe 5 (Bewegungsschärfe für 120–480 Hz) fertig. Plan: [docs/PLAN.md](docs/PLAN.md), Berichte: [docs/etappen/](docs/etappen/).

**Für KIs und Mitwirkende:** Einstieg über [AGENTS.md](AGENTS.md), aktueller Stand in
[docs/ki/STAND.md](docs/ki/STAND.md), Vorlage für einen Prompt in [docs/ki/PROMPT.md](docs/ki/PROMPT.md).

Fremde Arbeiten, auf denen `roehre` aufbaut, und ihre Lizenzen: [THIRD_PARTY.md](THIRD_PARTY.md).

## Schnellstart

```bash
cargo build --release -p roehre-pruefstand
./target/release/roehre-pruefstand foto -v scart-rgb -b pixelszene -g 1920x1080 -o foto.png
./target/release/roehre-pruefstand bogen -g 1920x1080 -o bogen/
```

Ohne Grafikkarte (z. B. in CI) rendert Mesa lavapipe auf der CPU (`mesa-vulkan-drivers`).

## Aufbau

| Ordner | Inhalt |
|---|---|
| `crates/roehre-modell` | Einstellungen, Optionen, Voreinstellungen (ohne GPU) |
| `crates/roehre-kette` | Einstellungen → Shaderkette und Parameter |
| `crates/roehre-wgpu` | Rust-Schnittstelle für wgpu |
| `crates/roehre-testbilder` | selbst erzeugte Testbilder |
| `crates/roehre-pruefstand` | Fotos, Kontaktbögen, Messungen ohne Fenster |
| `shader/eigen` | eigene Shader (MIT OR Apache-2.0) |
| `shader/fremd` | übernommene Shader mit Lizenz und Herkunft |
| `docs/` | Plan, Etappenberichte, Fotos |
