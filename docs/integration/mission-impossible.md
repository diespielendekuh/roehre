# Mission: Impossible (Nintendo 64)

- **Repository:** `diespielendekuh/MissionImpossibleCodes`
- **Stand:** noch nicht eingebaut. Das Projekt ist derzeit eine bytegenaue Dekompilierung
  (C, KMC GCC 2.7.2); der PC-Port und sein Renderer sind noch offen.
- **Bild:** N64 320×240 (auch 640×480, Breitbild-Hacks); im Port ggf. hochaufgelöste 3D-Bilder
  → `eingang.zeilen` (virtuelle Zeilenzahl) nutzen.
- **Schnittstelle:** braucht das C-API (Wunsch W-001), falls der Renderer nicht in Rust/wgpu ist.
- **Geplant: 3D-Monitor in einer Szene**, auf den das Spielbild projiziert wird → Flächen-Modus
  (`Roehre::zeichne_flaeche`, Maske im Material mit `shader/eigen/flaeche/roehre_maske.*`).
