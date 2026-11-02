# Etappe 4: Gehäuse, Breitbild-Röhre, automatisches Einrasten

Stand: 2026-11-02

## Was gebaut wurde

| Teil | Inhalt |
|---|---|
| Gehäuse | Vollständig berechnet im Zusammensetz-Pass (`crates/roehre-wgpu/src/zusammensetzen.wgsl`), Signed Distance Fields, **keine Bilddateien, keine fremden Vorlagen**. Arten: `nur_glas`, `fernseher` (dunkler Kunststoff, Lautsprechergitter, Betriebsanzeige), `monitor` (heller Kunststoff, schmaler Rahmen), `automat` (schwarze, matte, tiefe Blende). |
| Glas | ragt 3 % über das Bild hinaus (unbeleuchteter Leuchtstoff), abgerundete Ecken, zum Rand dunkler (Wölbung, Glasdicke), schwacher Glanz einer Lichtquelle oben links und ein Saum an der Oberkante |
| Innenfase | spiegelt das Bild an der Glaskante, mit dem Abstand unschärfer (Mipmaps der Röhrentextur, in linearem Licht erzeugt), Fresnel-Abfall |
| Licht | Raumlicht von oben (Rundung der Außenkante mit Glanzlicht oben, Schatten unten, helle Lippe zwischen Fase und Vorderseite), dazu das Licht des Bildes: Die mittlere Bildfarbe (kleinste Mip-Stufe) färbt Rahmen und Wand, mit Abfall nach außen |
| Anordnung | `crates/roehre-wgpu/src/gehaeuse.rs`: Bildfläche bleibt auf ganzen Pixeln (Maske moiréfrei), Glas und Rahmen werden darum gelegt; `gehaeuse.zoom` zwischen ganzem Gehäuse und Bild in Fensterhöhe |
| Breitbild-Röhre | `bildformat.roehrenformat = "16:9"`: 16:9-Glas, 4:3-Bild mittig, daneben schwarzer Leuchtstoff hinter dem Glas. Neue Voreinstellung `breitbild-roehre` (Flachbild-Röhre der 2000er im Fernsehgehäuse). |
| Neue Optionen | `gehaeuse.typ`, `.zoom`, `.spiegelung`, `.umgebungslicht`, `.raumlicht`, `.glanz`; `bildformat.roehrenformat` |
| Automatisches Einrasten | Bekommt eine Zeile weniger als 4 Bildschirmpixel, rastet die Bildhöhe auf ein ganzzahliges Vielfaches der **sichtbaren** Zeilen (nach Overscan) ein, wenn das Bild dabei höchstens 30 % kleiner wird. Nur bei eingeschalteter Röhre. |

Ohne Gehäuse läuft der bisherige, exakte Weg: Die neun Voreinstellungen ohne Gehäuse waren nach
dem Umbau des Zusammensetz-Passes **pixelgleich** zu ihren Referenzbildern (Abweichung 0,000).
HDR funktioniert mit Gehäuse (Weiß der Breitbild-Röhre 200 nit wie eingestellt).

## Befund: 720p

Das Gehäuse nimmt Platz; die Breitbild-Röhre bekam bei 720p nur 2,5–2,7 Pixel je Zeile und
schwebte mit 3,4–4,6 %. Das automatische Einrasten löst das allgemein:

| | vorher (720p) | nachher (720p) |
|---|---:|---:|
| scart-rgb | 1,97 % | 0,07 % |
| Composite / PAL / S-Video | 1,5–1,9 % | 0,05–0,10 % |
| arcade | 1,21 % | 0,08–0,47 % |
| breitbild-roehre (im Gehäuse) | 4,62 % | 0,11 % |

Preis: Bei 720p ist das Bild etwas kleiner (SNES: 672 statt 720 Pixel hoch). Die Referenzbilder
(gerendert bei 960×540) wurden deshalb neu geschrieben.

## Helligkeit

Unverändert für die bisherigen Voreinstellungen; `breitbild-roehre`: Weiß 100 %, Grau 91 %.

## Fotos

- [Fernseher](../bilder/etappe-4/gehaeuse_fernseher_1080p.jpg),
  [Monitor](../bilder/etappe-4/gehaeuse_monitor_1080p.jpg),
  [Automat](../bilder/etappe-4/gehaeuse_automat_1080p.jpg),
  [nur Glas](../bilder/etappe-4/gehaeuse_nur_glas_1080p.jpg),
  [Breitbild-Röhre](../bilder/etappe-4/gehaeuse_breitbild_1080p.jpg),
  [Automat mit 3D-Szene](../bilder/etappe-4/gehaeuse_automat_szene3d_1080p.jpg)
- Kanten in 4K, 1:1: [gehaeuse_kanten_4k_1zu1.png](../bilder/etappe-4/gehaeuse_kanten_4k_1zu1.png)

## Selbstkritische Abnahme

| Frage | Befund |
|---|---|
| Wirkt es wie ein Gerät? | Ja, nach einer zweiten Runde: Im ersten Wurf waren die Gehäuse flach und fast schwarz, die Wand heller als das Gehäuse. Kunststoff angehoben, Lippe und Glanzkanten ergänzt, Licht des Bildes verstärkt. |
| Spiegelung | sichtbar, aber zurückhaltend (Standard 0,5). Fehlt noch: Spiegelung des Raums im Glas selbst (nur ein Glanzfleck). |
| Moiré | Bild unverändert pixelgenau; mit Gehäuse ab 1080p ≤ 0,7 %, 720p ≤ 0,11 %. |
| Grenzen | Das Gehäuse ist nur in WGSL geschrieben, also nur über wgpu verfügbar. Für die C-API (Vulkan, D3D12) muss es mit umziehen (Wunsch W-001 ergänzt). Die Gehäuse sind bewusst schlicht; ein eigenes 3D-Modell wäre der nächste Schritt, falls die Flächenformen nicht reichen. |
