# roehre-c: C-Schnittstelle

`include/roehre.h` ist die ganze Schnittstelle, jede Funktion ist dort beschrieben.

## Bauen

```bash
cargo build --release -p roehre-c                                  # Linux: target/release/libroehre.so und libroehre.a
cargo build --release -p roehre-c --target x86_64-pc-windows-msvc   # Windows: roehre.dll, roehre.dll.lib
```

Ausliefern: die Bibliothek und den Ordner `shader/` (enthält GPL-Shader als eigene Dateien,
siehe `THIRD_PARTY.md`).

## Was es gibt

| Teil | Wofür | Funktionen |
|---|---|---|
| Katalog | eigenes Menü bauen (Schlüssel, Namen, Beschreibung, Grenzen, Standard, Festwerte) | `roehre_katalog_json` |
| Einstellungen | laden, setzen, speichern (TOML/JSON) | `roehre_einstellungen_*` |
| Vorschaubild | Launcher, der in Software zeichnet: RGBA-Bild in den Arbeitsspeicher | `roehre_vorschau_*` |
| Austausch | Spiel mit eigenem Vulkan- oder D3D12-Renderer: Bild rein, Röhrenbild raus | `roehre_vulkan_*`, `roehre_d3d12_*`, `roehre_austausch_*` |

Beide Modi überall: `ROEHRE_MODUS_FLACH` (Röhrenbild im Fenster) und `ROEHRE_MODUS_RAUM`
(3D-Vorführraum, Kamera über die Optionen `raum.*`).

## Wie der Austausch arbeitet

roehre rechnet auf einem eigenen Gerät derselben Grafikkarte und teilt mit dem Spiel zwei
Bilder (EINGANG, ZIEL) und eine Synchronisation (Vulkan: zwei Semaphoren, D3D12: ein Zaun).

```
Spiel:  [Bild → EINGANG] signal „bereit“ ──┐                    ┌── wait „fertig“ [ZIEL → Swapchain] present
roehre:                                    └→ wait … rechnen … signal ┘
```

Warum nicht auf dem Gerät des Spiels selbst: wgpu (auf dem roehre aufbaut) kann ein fremdes
D3D12-Gerät nicht übernehmen und ein fremdes Vulkan-Gerät nur, wenn es mit genau den Funktionen
angelegt wurde, die wgpu erwartet. So bleiben alle Pässe (Gehäuse, HDR, Strahlsimulation,
3D-Raum) ohne Portierung nutzbar. Preis: zwei Kopien je Bild auf der Grafikkarte.

Voraussetzungen am Gerät des Spiels:

* **D3D12:** keine.
* **Vulkan:** Version 1.1+, Erweiterungen `VK_KHR_external_memory_fd` und
  `VK_KHR_external_semaphore_fd` (Windows: `…_win32`). Ohne geteilte Semaphoren (Mesa lavapipe)
  gibt es einen Notweg mit Ordnung über die CPU – nur zum Prüfen.

## Hinweise aus dem ersten Einbau in ein Spiel (Ghost Return, RT64 unter D3D12)

* **BGRA-Swapchain:** `kanalfolge = ROEHRE_KANALFOLGE_BGRA` setzen, dann sind EINGANG und ZIEL
  BGRA8 und lassen sich mit `CopyResource`/`CopyTextureRegion` aus der und in die Swapchain
  kopieren. Mit RGBA-Bildern verbietet D3D12 diese Kopien (andere Formatfamilie).
* **Fremde `dxcompiler.dll` im Prozess:** egal, der D3D12-Weg übersetzt mit FXC.
* **Ohne Eingriff in den Renderer:** Bietet der Renderer nur eine Stelle „nach dem Spielbild, in
  der offenen Befehlsliste“ (bei RT64 der Zeichen-Hook), geht es mit einem Bild Verzögerung:
  „bereit“/Zeichnen/„fertig“ für das vorige Bild am Anfang des Hooks, die Kopien für das
  aktuelle Bild in die Liste. Ausgeführt in `docs/integration/ghost-return.md`.
* **Nie warten, wenn das Zeichnen fehlschlug:** Gibt `roehre_austausch_zeichnen` einen Fehler
  zurück, kommt „fertig“ nicht; ein `queue->Wait` darauf hielte die Warteschlange für immer an.

## Beispiele und Prüfläufe

| Datei | Was | Geprüft |
|---|---|---|
| `beispiele/vorschau.c` | Katalog, Einstellungen, Vorschaubild flach und Raum | Linux (lavapipe), läuft |
| `beispiele/vulkan_spiel.c` | Vulkan-Spiel mit eigenem Gerät, Austausch, Rücklesen | Linux (lavapipe, Notweg ohne Semaphoren), läuft |
| `beispiele/d3d12_spiel.cpp` | D3D12-Spiel mit eigenem Gerät, Austausch, Rücklesen | nur übersetzt (MinGW); auf Windows nicht ausgeführt |

```bash
cc crates/roehre-c/beispiele/vulkan_spiel.c -Icrates/roehre-c/include -Ltarget/release -lroehre -lvulkan -o vulkan_spiel
LD_LIBRARY_PATH=target/release ./vulkan_spiel shader /tmp/vk   # Rückgabe 0 = Austausch in Ordnung
```

## Grenzen

* Größenwechsel (Fenster, Auflösung von RT64): Austausch freigeben und neu anlegen.
* `roehre_austausch_einstellungen` blockiert beim Wechsel der Shaderkette (bis etwa 1 s).
* Der 3D-Raum gibt auch bei HDR-Ausgabe SDR-Helligkeiten aus.
* Der Vulkan-Weg mit geteilten Semaphoren und der ganze D3D12-Weg sind auf echter Hardware
  ungeprüft (Etappe 8).
