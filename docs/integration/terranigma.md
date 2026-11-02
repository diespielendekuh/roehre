# Terranigma PC-Port

- **Repository:** `diespielendekuh/Tenchi-Souzou-Deutsch-mod-tool`, Kiste `crates/terranigma-spiel`
- **Stand:** noch nicht eingebaut (Wunsch W-002)
- **Bild:** 256×224, Pixel 8:7 (NTSC); der Port beschneidet den Rand selbst (Werksbeschnitt 8/8/0/1)
- **Grafik:** winit 0.30, egui 0.31, egui-wgpu 0.31, **wgpu 24** (D3D12 unter Windows,
  Vulkan/GL unter Linux). roehre braucht wgpu 30 → Port auf egui 0.37 / wgpu 30 anheben.
- **Heute:** Das Spielbild wird auf der CPU um den Faktor k vergrößert und als egui-Textur
  `spielfeld` hochgeladen (`crates/terranigma-spiel/src/portmenue.rs`).
- **Geplanter Einbau:** Spielbild unvergrößert als wgpu-Textur hochladen, `Roehre::zeichne` in
  die Swapchain-Textur (oder eine Zieltextur, die egui als native Textur zeigt), danach egui
  (Menü) darüber. Die CPU-Vergrößerung entfällt.
- **Voreinstellung:** `scart-rgb` als Standard (europäischer Fernseher, RGB).
