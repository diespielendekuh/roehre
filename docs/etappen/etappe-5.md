# Etappe 5: Bewegungsschärfe (Strahlsimulation, Schwarzbilder)

Stand: 2026-11-02

## Was gebaut wurde

| Teil | Inhalt |
|---|---|
| Schnittstelle | `Roehre::zeichne_im_takt(…, Takt { neues_spielbild, teilbilder_je_spielbild })`: Das Spiel ruft je Bildschirmauffrischung auf. Die Röhrenkette läuft nur bei neuen Spielbildern, die Teilbilder rechnet allein der Zusammensetz-Pass. `zeichne` bleibt wie bisher (ein Bild je Aufruf). |
| Strahlsimulation | Algorithmus des Blur Busters CRT Beam Simulator (Mark Rejhon, Timothy Lottes, MIT) in `zusammensetzen.wgsl` übertragen, mit Lizenztext. Jeder Bildpunkt gibt sein Licht ab, sobald der simulierte Strahl seine Zeile erreicht; was in eine Auffrischung nicht passt, leuchtet nach. |
| Verbesserung gegenüber der Vorlage | Die libretro-Fassung liest für die vorherigen Auffrischungen das aktuelle Bild noch einmal (Demo-Fassung). roehre gibt das echte vorherige Spielbild und braucht dafür nur zwei Bilder, ohne zusätzliche Latenz. Details: `shader/fremd/mit/crt-beam-simulator/HERKUNFT.md`. |
| Schwarzbilder | einfache Alternative: Bild nur in der ersten Auffrischung je Zyklus |
| Optionen | `bewegung.modus` (aus, strahl, schwarzbilder), `bewegung.helligkeit` (Helligkeit gegen Schärfe), `bewegung.lcd_schutz` |
| Prüfstand | `roehre-pruefstand bewegung -n <Verhältnis>`; die Lichtsumme bei 2; 2,4; 4 ist Teil von `pruefe` |

## Messung: Lichtsumme

Mittel aller Auffrischungen eines eingeschwungenen Bildes gegenüber dem Bild ohne Simulation,
16 Graustufen. Soll ist die eingestellte Verstärkung (0,7):

| Verhältnis (Bildschirm : Spiel) | Beispiel | größte Abweichung |
|---|---|---:|
| 2 | 120 Hz : 60 | 0,005 |
| 2,4 | 144 Hz : 60 | 0,003 |
| 2,75 | 165 Hz : 60 | 0,004 |
| 4 | 240 Hz : 60 | 0,002 |
| 8 | 480 Hz : 60 | 0,002 |

**Befund unterwegs:** Im ersten Entwurf begann der Strahl bei jedem neuen Spielbild oben. Bei
nicht ganzzahligen Verhältnissen (144 Hz: abwechselnd 3 und 2 Auffrischungen je Spielbild)
erreichte er so die unteren Zeilen in kurzen Spielbildern nie; es fehlten bis zu 20 % Licht.
Jetzt läuft der Strahl wie in der Vorlage durch (Lage = Auffrischung mod N) und rastet nur bei
ganzzahligem N auf den Zyklusanfang ein.

## Fotos

- Vier Auffrischungen bei 240 Hz (Graukeil): [strahl_4_teilbilder.png](../bilder/etappe-5/strahl_4_teilbilder.png).
  Der Strahl wandert von oben nach unten; dunkle Stufen leuchten kurz, helle bis in die
  nächste Auffrischung.
- Zwölf Auffrischungen bei 144 Hz (die ersten acht): [strahl_2komma4_teilbilder.png](../bilder/etappe-5/strahl_2komma4_teilbilder.png)

## Selbstkritische Abnahme

| Frage | Befund |
|---|---|
| Stimmt die Helligkeit? | Ja, auf 0,5 % genau (× Verstärkung). Mit Verstärkung 0,7 ist das Bild bewusst um 30 % dunkler; das ist der Preis der Schärfe. In HDR lässt sich das ausgleichen (Weiß höher stellen). |
| Wirkt es? | Messbar nur die Lichtverteilung; die eigentliche Bewegungsschärfe lässt sich nur am echten Bildschirm mit hoher Bildwiederholrate beurteilen. **Ungeprüft.** |
| Flimmern auf LCD | LCD-Schutz nach Vorlage; ob er auf einem bestimmten Bildschirm reicht, ist ungeprüft. |
| Gehäuse | bleibt ruhig, nur die Röhre scannt (Spiegelung im Rahmen nutzt das ganze Bild). |
| Grenzen | nur wgpu (WGSL), wie Gehäuse und HDR; für die C-API mit umziehen. |
