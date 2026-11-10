# Ghost Return (Goemon's Great Adventure, Nintendo 64), statische Rekompilierung

Stand dieser Notiz: 2026-11-10, flacher Modus und 3D-Raum laufen im Spiel (D3D12, RTX 5070 Ti).

- **Repository:** `diespielendekuh/MysticalNinja2_recomp` (privat), roehre als Untermodul `lib/roehre`
  (Zweig `ki/ghost-return-bgra`).
- **Technik:** C++, N64Recomp / N64ModernRuntime / RecompFrontend mit **RT64** als Renderer
  (Untermodul aus dem Goemon64Recomp-Fork, Zweig `rsp-fixes-plume-rebase`), unter Windows D3D12.
- **Einbau ohne Eingriff in RT64.** Anders als in der Notiz zu Mission: Impossible vorgeschlagen,
  wird `rt64_present_queue.cpp` nicht angefasst. Der Einbau hängt am **Zeichen-Hook**, den RT64
  ohnehin anbietet (`RT64::SetRenderHooks`): Er läuft, nachdem RT64 das Spielbild in die
  Swapchain gezeichnet hat, in derselben, noch offenen Befehlsliste. Code: `src/game/ga_roehre.cpp`.

## Ablauf je Bild (im Zeichen-Hook von Bild N)

1. Das EINGANG von Bild N−1 ist inzwischen übergeben: `queue->Signal(zaun, bereit)`,
   `roehre_austausch_zeichnen`, `queue->Wait(zaun, fertig)`. Das Warten liegt damit vor der
   Befehlsliste von Bild N.
2. In die Befehlsliste von Bild N (über `plume::D3D12CommandList::d3d`): Swapchain → EINGANG
   (`CopyTextureRegion`, nur der Teil mit dem Spielbild), danach ZIEL (Bild N−1) → Swapchain
   (`CopyResource`). Zustände der Swapchain-Textur danach wieder `RENDER_TARGET`, damit plume
   nichts merkt; die Oberfläche des Spiels wird anschließend darübergezeichnet.

**Preis:** Das Röhrenbild ist ein Bild hinter dem Spiel (etwa 17 ms bei 60 Hz). Wer das nicht
will, braucht den Eingriff in RT64 wie bei Mission: Impossible beschrieben.

**Zugriff auf die nativen Objekte:** Gerät und Adapter aus dem `RenderDevice` des Init-Hooks
(`plume::D3D12Device`, LUID über `adapter->GetDesc1()`), die Warteschlange aus
`app->presentGraphicsWorker->commandQueue`, die Swapchain-Textur aus
`plume::D3D12Framebuffer::colorTargets[0]`.

## Was dabei an roehre geändert wurde

- **Kanalfolge BGRA** (`RoehreAustauschParameter::kanalfolge`, `ROEHRE_KANALFOLGE_BGRA`): Die
  Swapchain von RT64/plume ist `B8G8R8A8_UNORM`. Mit den RGBA-Bildern des Austauschs wäre
  `CopyResource` nicht erlaubt gewesen.
- **Shader-Compiler fest auf FXC** im D3D12-Weg: RT64 lädt eine ältere `dxcompiler.dll` in den
  Prozess. wgpu nimmt im Modus „Auto“ jede `dxcompiler.dll`, die es findet, verlangt aber
  mindestens v1.8.2502; jede Pipeline scheiterte mit „DXC compile error: error reading …“.

## Einstellungen im Spiel

- `bildformat.seitenverhaeltnis` wird vom Spiel fest auf `pixelgenau` gesetzt, EINGANG ist der
  Bildteil der Swapchain (4:3 mittig oder das ganze Fenster bei Breitbild), Pixel-Seitenverhältnis 1.
  So behält das Bild seine Form. Im 3D-Raum landet ein Breitbild allerdings gestaucht auf der
  4:3-Röhre; dem Spieler wird „Original“ (4:3) empfohlen.
- Gespeichert in `crt.toml` im Datenordner (TOML von roehre, davor eine Kommentarzeile des Spiels
  mit an/aus, Modus und Voreinstellung).
- Menü bisher: an / aus / 3D-Raum und sieben Voreinstellungen. Das volle Menü aus
  `roehre_katalog_json` steht noch aus.

## Ausliefern

`roehre.dll` und der Ordner `shader/` stecken in der einen EXE des Spiels und werden beim
Installieren daneben ausgepackt (`roehre/shader/`). `roehre.dll` wird verzögert geladen.

## Offen / ungeprüft

- Leistung im Spiel nicht gemessen (nur gesehen: läuft flüssig bei 2826×1696).
- Größenwechsel des Fensters, Vollbild, HDR, 120 Hz und mehr (Strahlsimulation) nicht geprüft.
- Vulkan-Weg nicht eingebaut (das Spiel läuft unter Windows mit D3D12).
