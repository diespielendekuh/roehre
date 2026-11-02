# Vorlage: Prompt für eine KI, die roehre in ein Spiel einbauen soll

Diesen Text einer KI geben, die an einem Spiel arbeitet und den Röhreneffekt (CRT) einbauen
soll. Die eckigen Klammern ersetzen. Die KI braucht Lesezugriff auf das Repository
`diespielendekuh/roehre` (privat); zum Ändern auch Schreibzugriff.

---

```text
Für den Röhreneffekt (CRT-Simulation) gibt es die Bibliothek „roehre“ im GitHub-Repository
diespielendekuh/roehre. Sie ist die gemeinsame Grundlage für mehrere meiner Spiele und wird
auch von anderen KIs weiterentwickelt.

1. Hole dir das Repository (Standardbranch) und lies zuerst AGENTS.md, dann docs/ki/STAND.md.
   Dort steht, was fertig ist, wie man es einbaut und was noch fehlt. Lies auch
   docs/ki/wuensche.md und, falls vorhanden, docs/integration/[projektname].md.
2. Baue den Effekt in [Spiel / Repository] ein. Nutze roehre so, wie es STAND.md beschreibt;
   kopiere die Shader nicht verändert in mein Spiel.
3. Wenn roehre für mein Spiel etwas fehlt oder nicht passt: Kleine, klare Dinge behebst du
   direkt in roehre, nach den Regeln in AGENTS.md (eigener Branch, messen mit dem Prüfstand,
   nichts für andere Spiele verschlechtern, Lizenzen getrennt halten). Große oder unklare
   Dinge trägst du in docs/ki/wuensche.md ein und sagst es mir.
4. Wenn du in roehre etwas geändert hast: Aktualisiere docs/ki/STAND.md und
   docs/ki/logbuch.md, lege docs/integration/[projektname].md an oder aktualisiere sie, und
   sag mir, auf welchem Branch die Änderung liegt.

Besonderheiten meines Spiels: [z. B. Auflösung 256×224, Pixel 8:7, wgpu-Version, Renderer,
Menü des Spiels, HDR ja/nein, 3D-Monitor in der Szene …]
```
