# Wunschliste: was Projekte von roehre brauchen

Hier trägt jede KI (oder jeder Mensch) ein, was einem Spielprojekt fehlt. Wer roehre
weiterbaut, arbeitet diese Liste mit ab. Neue Einträge **oben** einfügen, fortlaufende
Nummer, erledigte nicht löschen, sondern mit Commit markieren.

## Vorlage

```markdown
### W-NNN: <kurzer Titel>
- **Von:** <Projekt / KI>, <Datum>
- **Status:** offen | in Arbeit (<wer>) | erledigt (<Commit>) | abgelehnt (<Grund>)
- **Was fehlt:** <konkret: Funktion, Format, Fehlerbild mit Werten>
- **Warum:** <wofür das Spiel es braucht>
- **Vorschlag:** <wie es gehen könnte, falls bekannt>
```

## Einträge

### W-004: 3D-Vorführraum mit einem berechneten Röhrenfernseher
- **Von:** Anwender, 2026-11-02 (Frage nach einer 3D-Szene mit Bildschirmgerät)
- **Status:** geplant für Etappe 6 (Vorschau-Programm)
- **Was fehlt:** Eine 3D-Szene mit einem selbst erzeugten Fernseher (gewölbtes Glas als
  Kugelkappe, Gehäuse als Körper), Kamera frei beweglich, Bild über den Flächen-Modus.
- **Warum:** Prüfstein und Vorlage für den 3D-Monitor im Mission:-Impossible-Port.

### W-003: Permissive Variante ohne GPL-Shader
- **Von:** roehre (Planung), 2026-11-02
- **Status:** offen
- **Was fehlt:** Eine Kette auf Basis von crt-beans (MIT) und Cathode Retro (MIT), auf die
  dieselben Optionen abgebildet werden, für Spiele, die keine GPL-Shader ausliefern wollen.
  Die Shader liegen bereits unter `shader/fremd/mit/`.
- **Warum:** Lizenzfreiheit für Projekte, die das brauchen.

### W-002: Einbauanleitung und Versionsanhebung für den Terranigma-Port
- **Von:** roehre (Planung), 2026-11-02
- **Status:** offen (geplant Etappe 7)
- **Was fehlt:** Der Port nutzt wgpu 24 / egui 0.31, roehre braucht wgpu 30. Anleitung für das
  Anheben (egui 0.37) und den Einbau (Spielbild unvergrößert hochladen, roehre zeichnet, egui
  darüber), siehe `docs/PLAN.md` Abschnitt 11.1.
- **Warum:** Terranigma ist das erste Spiel, das roehre benutzen soll.

### W-001: C-API mit Vulkan- und D3D12-Weg
- **Von:** roehre (Planung, für Mission: Impossible), 2026-11-02
- **Status:** offen (geplant Etappe 7)
- **Was fehlt:** Eine C-Schnittstelle (`roehre.h`) für C/C++-Spiele: erstellen mit
  Vulkan-Gerät oder D3D12-Gerät, Einstellungen laden/setzen, Katalog auflisten, zeichnen in
  eine Zieltextur, Flächen-Modus.
- **Warum:** Der PC-Port von Mission: Impossible entsteht in C (Renderer noch offen).
- **Vorschlag:** librashader hat Vulkan- und D3D12-Laufzeiten und eine C-API; die eigenen
  Pässe (Zusammensetzen mit Gehäuse und HDR, Mipmaps, Eichung) liegen bisher nur als WGSL vor
  und müssen dafür als slang-Pässe oder je Backend vorliegen.
