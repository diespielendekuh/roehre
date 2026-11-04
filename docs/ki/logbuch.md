# Logbuch: wer hat was an roehre geändert

Neue Einträge **oben**. Je Eintrag: Datum, wer (Projekt bzw. KI), was, warum, was ungeprüft ist.

## 2026-11-05 · Terranigma-Port (Helfer-KI der Oberfläche) · Zeit je Bild der versionsfreien Ketten
- Neuer Befehl `roehre-pruefstand wgsl-leistung [-o datei.md] [-n 60]`: jede Kette aus `KETTEN`,
  die erzeugten (crt-beans, ScaleFX, OmniScale) und die Stufen hinter einem Filter (Leuchten bei
  Weite 1,5 und 4, Farbe), Eingang 256 × 224, Ziel 4:3 in 1080p und 4K; Messung wie `leistung`
  (Zeitstempel, sonst Wanduhr), Pipelines vorab gebaut. `Messer`/`Kennzahl` dafür `pub(crate)`.
- Hier nur lavapipe gemessen (Zahlen sagen nichts über echte Karten); laufen lassen muß ihn der
  Anwender auf seinem Rechner.

## 2026-11-05 · Terranigma-Port (Helfer-KI der Oberfläche) · Weichheit und Leuchten
- `scharf-bilinear` hat den Regler **»weichheit«** (0 … 1, Voreinstellung 0 = Saum ein Bildpunkt,
  unverändert): 1 rechnet wie `linear` (größte Abweichung 1), 0,5 liegt dazwischen.
- Neue Stufen **`leuchten`** (leichtes Leuchten/Bloom hinter einem Filter, Stärke 0 = aus) und
  **`leuchten-hof`** (der Hof in Spielauflösung, Gauß bis 3 σ, Teil über der Schwelle). Die
  Vereinbarung bekommt dafür das Feld `Stufe::zusatz` und die optionale Bindung 3: das Ergebnis
  der Zusatzstufe auf dem Original. Der Hof entsteht in Spielpunkten — unabhängig von der
  Fenstergröße und unsichtbar für Kantenglätter davor.
- `roehre-pruefstand wgsl`: Stärke 0 = Bild unverändert (0), Punktprobe symmetrisch (±1), nach
  außen nie zunehmend, jenseits 3 σ schwarz, Halbwertsbreite in Spielpunkten bei ×3 und ×6 gleich
  (2,67); Grau unter der Schwelle unverändert. Ein erster Entwurf in einem Pass (9 × 9 Abtastungen
  im Abstand 0,75 σ) zeigte Wellen im Hof — verworfen.
- Ungeprüft: echte Grafikkarte.

## 2026-11-05 · Terranigma-Port (Helfer-KI der Oberfläche) · ScaleFX und OmniScale
- Lizenzen der Pixelkunst-Vergrößerer in libretro `slang-shaders` `3685eda` aus den Dateiköpfen
  gelesen: xBR/Super-xBR, HQx, SABR, 2xSaI/Eagle, xSoft **GPL** (nicht übernommen); **ScaleFX**
  (Sp00kyFox) und **OmniScale** (Lior Halphon, slang von hunterk) **MIT** — unverändert unter
  `shader/fremd/mit/{scalefx,omniscale}`, Presets `shader/eigen/{scalefx,omniscale}.slangp`
  (ScaleFX: `stock.slang` durch das eigene `durchreichen.slang` ersetzt).
- `wgsl-erzeugen` kennt Gruppen; neue Datei `src/erzeugt/pixelkunst.rs` (`SCALEFX`, 6 Pässe;
  `OMNISCALE`, 1 Pass). Gegen librashader: **0 Abweichung** (zwei Testbilder, drei Größen).
- `pruefe` bestanden.

## 2026-11-05 · Terranigma-Port (Helfer-KI der Oberfläche) · MMPX gegen die Fassung der Autoren
- `shader/fremd/mit/mmpx/mmpx.slang` (MIT, McGuire & Gagiu, slang von hunterk, unverändert aus
  libretro `3685eda`) als **Prüfstein**; Preset `shader/eigen/mmpx-pruefung.slangp` (ein Pass,
  doppelte Größe, `clamp_to_edge` wie die eigene Stufe).
- `roehre-pruefstand wgsl` fährt sie über librashader und vergleicht mit `mmpx.wgsl`: **0 Punkte
  anders** auf drei Testbildern. Dafür wiegt die Helligkeit für die Vergleiche jetzt nach BT.709
  wie dort (vorher r+g+b aus dem Gedächtnis — die Referenz war beim Bau nicht erreichbar), in
  ganzen Zahlen (2126·r + 7152·g + 722·b, in f32 exakt, GPU = CPU).
- `pruefe` bestanden.

## 2026-11-05 · Terranigma-Port (Helfer-KI der Oberfläche) · Stufe `farbe`
- Eigene Stufe `crates/roehre-wgsl/wgsl/farbe.wgsl` (MIT OR Apache-2.0): Farbkorrektur hinter
  jedem Filter, 1:1 — Helligkeit, Kontrast (um Mittelgrau 0,18), Sättigung (BT.709), Wärme
  (Rot/Blau gegeneinander ±10 %, eine Annäherung, keine Farbmetrik), in linearem Licht.
  `Stufe::neutral` sagt, wann ein Ausführer sie weglassen darf.
- `roehre-pruefstand wgsl`: neutral 0 Abweichung, Sättigung 0 exakt grau. `pruefe` bestanden.

## 2026-11-05 · Terranigma-Port (Helfer-KI der Oberfläche) · Stufe `lcd-raster`, Regler je Stufe
- Neue eigene Stufe `crates/roehre-wgsl/wgsl/lcd-raster.wgsl` (MIT OR Apache-2.0) für die
  Bildfilter-Vielfalt (Wunsch E3b des Terranigma-Anwenders): Handheld-/LCD-Bild, jeder
  Spielpunkt eine Zelle mit dunklen Lücken, wahlweise RGB-Streifen. Gegen Moiré wird der Anteil
  der Lücken und Streifen am Ausgabepunkt je Achse **exakt integriert** (Kastenfilter), nicht
  abgetastet. Kette `lcd` in `KETTEN`.
- Stufen tragen jetzt beschriebene Regler (`StufenRegler`: Name, deutscher Titel und Satz,
  Grenzen, Schritt, Voreinstellung); `Rahmen::mit_reglern` legt sie in `rahmen.regler`.
- `roehre-pruefstand wgsl` misst das Raster auf einer weißen Fläche: Helligkeit über genau vier
  Zellperioden (gebrochene Fenster mit Bruchteilen gewogen) — 1440×1080 0,00 %, 935×701 0,84 %
  (Rest = 8-Bit-Rundung; Grenze 1 %, roehres Moiré-Grenze ist 1,1 %). Fotos mit `WGSL_FOTOS`.
- `pruefe` bestanden.

## 2026-11-04 nachts · Terranigma-Port (Helfer-KI der Oberfläche) · W-009 Teil 2: crt-beans erzeugt
- `roehre-pruefstand wgsl-erzeugen` übersetzt die drei MIT-Ketten (`eigen/beans/rgb.slangp`,
  `signal.slangp`, `schnell.slangp`: 13, 13 und 4 Pässe samt `helligkeit.slang`) mit
  **librashaders eigener Übersetzung** (slang → SPIR-V → naga → WGSL, dieselben Optionen wie
  sein wgpu-Unterbau) und schreibt WGSL-Texte nach `crates/roehre-wgsl/wgsl/erzeugt/` und die
  Pass-Beschreibung (Größenregel, Format, Abtaster, Bindungen, Lage jedes Werts und Reglers,
  Nachschlagebild BlueNoiseTex roh als RGBA) nach `src/erzeugt/beans.rs`. Nichts ist von Hand
  abgeschrieben; was ein Ausführer nicht kennt (Verlauf, Rückkopplung, Mipmaps, seltene
  Werte), bricht das Erzeugen ab. Lizenztext von crt-beans liegt in der Kiste.
- `roehre_wgsl::kette` beschreibt, was ein Ausführer von librashader nachbilden muß (gelesen
  in librashader 0.12.1): Eckdaten und MVP, Größenrechnung, welcher Abtaster zu welchem Bild.
- `crates/roehre-pruefstand/src/wgsl_kette.rs`: ein Ausführer auf wgpu 30. **Prüfstein:**
  `roehre-pruefstand wgsl` zeichnet jede Kette einmal über librashaders `FilterChain`, einmal
  über den Ausführer — zwei Testbilder, 960×720, 1440×1080, 935×701, Standardregler und die
  von `roehre_kette::plane` für `mit-rgb`/`mit-composite` geplanten: **0 Abweichung** in jedem
  Fall (Rgba8Unorm, lavapipe).
- `roehre-kette`: librashader hinter das Standard-Merkmal `librashader` (Standard an, nichts
  ändert sich für bestehende Nutzer); ohne es bleiben Kettenwahl und `plane` — ein Spiel
  plant die Regler damit genau wie roehre, ohne librashader zu ziehen.
- Ungeprüft: echte GPU; ältere naga-Fassungen (Terranigma: wgpu 24) prüft der Port.

## 2026-11-04 abends · Terranigma-Port (Helfer-KI der Oberfläche) · W-009 Teil 1: Kiste `roehre-wgsl`
- Neue Kiste `crates/roehre-wgsl` **ohne Abhängigkeiten**: WGSL-Text der Stufen samt
  Beschreibung (Größe des Ziels, Abtastung, Lizenz), Ketten für das Menü eines Spiels
  (`scharf`, `mmpx`), das Layout der Werte als 128 Byte (`Rahmen::als_bytes`). Die WGSL-Dateien
  liegen **in** der Kiste (`wgsl/`), damit ein Spiel sie mitpacken kann (`cargo vendor` kopiert
  nur den Kistenordner).
- `scharf-bilinear.wgsl`: aus `shader/eigen/scharf-bilinear.slang` übertragen; einzige
  Erweiterung ein gezeigter Ausschnitt (Overscan), bei (0,0,1,1) gleich.
- `mmpx.wgsl` + `roehre_wgsl::mmpx` (CPU): MMPX nach McGuire & Gagiu (2021, MIT,
  `wgsl/LIZENZ-MMPX`) — für die Bildfilter-Vielfalt (Wunsch des Terranigma-Anwenders, »der
  Sammelort ist roehre«). Die Regeln stammen aus der Rust-Abschrift `mmpx` 0.2.0, deren Fehler
  (Nachbarn P/S nur in Spalte 0) nicht übernommen ist; die WGSL-Regeln sind maschinell aus der
  Rust-Fassung übertragen.
- `roehre-pruefstand wgsl`: führt die Stufen auf wgpu 30 aus (ein schlichter Ausführer mit dem
  Layout der Vereinbarung) und vergleicht `scharf-bilinear` mit der Voreinstellung `aus`
  (librashader) in fünf Größen und drei Testbildern — größte Abweichung 1 —, `mmpx` mit der
  CPU-Referenz — 0 Punkte verschieden, auch mit beschnittenem Ausschnitt.
- Befund für Ausführer: das Bindungslayout muß ausdrücklich angelegt werden; ein abgeleitetes
  Layout läßt den Abtaster weg, wo eine Stufe ihn nicht liest (mmpx).
- Ungeprüft: die Referenz der MMPX-Autoren selbst (Seite beim Bau nicht erreichbar); naga 24
  (Terranigma) prüft der Port beim Einbinden.

## 2026-11-04 · Terranigma-Port (Helfer-KI der Oberfläche) · Wunsch W-009
- Nur Doku: Wunsch W-009 eingetragen (die MIT-Kette als WGSL ohne wgpu-/librashader-Version),
  weil der Port vorerst auf wgpu 24 bleibt; `docs/integration/terranigma.md` nennt den zweiten
  Weg. Am Code und an den Shadern von roehre nichts geändert.

## 2026-11-03 · roehre-Hauptsitzung · 3D-Raum: Schattenwurf draußen und drinnen
- Rückmeldung: Bank mitten auf dem Weg ohne Schatten, quer zum Weg; insgesamt zu künstlich,
  es fehlen Schatten.
- Park mit echtem Licht statt aufgemalter Helligkeit: Hang, Bäume, Laternenmasten, Bank und
  Auto haben nur noch Grundfarben; der Shader beleuchtet sie mit Himmel, Mond (von Wolken
  gedämpft) und den drei Laternen (Lichtkegel nach unten, Inseln von gut drei Metern). Jeder
  Lichtstrahl wird gegen alle Kulissen (Alpha der Textur) und die Kästen der Bank geprüft:
  Schatten von Bäumen, Masten und Bank. Daten im Uniform `park` (`park_kulissen`,
  `parkbank_kaesten`). Dunst nach hinten.
- Bank ist jetzt ein 3D-Modell (Holzlatten, gusseiserne Wangen, Armlehnen), steht auf dem
  Rasen neben dem Weg, längs zu ihm und mit der Sitzfläche zum Weg (`szene::parkbank`).
  Ein junger Baum vorn rechts dazu.
- Drinnen: zweite Schattenkarte für das Bildschirmlicht (Tasse, Fernbedienung, Kassetten
  werfen Schatten vom Fernseher weg), dazu Umgebungsverdeckung im Nachbearbeitungspass
  (Kontaktschatten unter Gegenständen, in Kehlen und am Bildtunnel; aus der Entfernung im
  Alpha, Paare gegenüberliegender Nachbarn, Tiefensprünge über 0,25 m zählen nicht).
- Ehrlich: Laternen und Bank stehen nahe beieinander, der Schatten der Bank liegt daher
  großteils unter ihr; die Mondschatten sind gegen das Laternenlicht schwach. Leistung
  ungeprüft (ein Schattenpass mehr, Strahltests draußen nur im Fensterausschnitt, 16
  Abtastungen je Pixel für die Verdeckung).

## 2026-11-03 · roehre-Hauptsitzung · 3D-Raum: Park vor dem Fenster, Leben in der Szene
- Draußen statt neblig-leerer Häuserfront ein kleiner Park am Hang bei Nacht (`szene::park`):
  Himmel mit Verlauf, Mond samt Hof, funkelnden Sternen und langsam ziehenden Wolken (im
  Shader aus der Blickrichtung, scharf unabhängig von der Texturgröße), Häuser mit
  erleuchteten Fenstern und eine Baumreihe auf der Kuppe, Straße, Rasenhang mit Kiesweg,
  drei Laternen mit Lichtinseln, eine Bank, neun Bäume als Kulissen. Alles gemalt und
  selbstleuchtend, ohne Zimmerlicht und Zimmernebel, ohne Rohpixel; Silhouetten mit weichem
  Alpha (glatte Kanten statt Treppen). Kamera reicht jetzt 40 m weit.
- Neue Option `raum.leben` (0 … 1, Standard 1): Wolken ziehen am Mond vorbei und dimmen das
  Mondlicht im Zimmer, alle 21 s fährt hinter dem Park ein Auto vorbei (Scheinwerfer, das
  Fensterlicht wird dabei kurz warm heller), die Bäume wiegen sich. 0 = still.
- Mondlicht fällt durchs Fenster und zeichnet Scheiben und Sprossen auf den Teppich, mit dem
  Schatten der Tischplatte. Die Richtung ist steiler als der sichtbare Mond (sonst läge der
  Fleck hinter der Kamera) – bewusste Freiheit.
- Vorhänge hingen in der Fensterbank: jetzt an einer Stange mit Haltern 0,17 m vor der Wand,
  bodenlang, Falten unten tiefer.
- Heizkörper neu: Gliederheizkörper aus Gusseisen (15 Glieder aus je zwei Säulen mit Naben
  und Kerben, Standfüße, Thermostatventil mit Rippenkopf, Vor- und Rücklauf), Lack
  vergilbt mit Läufern und Abplatzern (`Heizlack`).
- Fernbedienung neu: Deckplatte aus satiniertem Aluminium statt Rauschtextur, erhabene
  Gummitasten (rote Netztaste, zwölf Ziffern, zwei Wippen, vier Farbtasten), IR-Fenster.
- Prüfstand `raum`: Ansichten `fenster`, `fernbedienung`, `heizung`, `auto`; `--zeit <s>`
  setzt die Zeit aller Ansichten.
- Ungeprüft: Leistung (Himmel je Pixel im Fenster, ein Texturzugriff mehr je Raumpunkt für
  die Wolken vor dem Mond).

## 2026-11-03 · roehre-Hauptsitzung · 3D-Raum: Fernseher neu nach Art früher 80er-Tischgeräte
- Rückmeldung: Rahmen lief in die Bedienleiste, Bildöffnung ohne richtige Ecken, Bedienfeld
  zu hell. Neu, in grober Anlehnung an US-Tischgeräte der frühen 80er (Referenzfoto des
  Anwenders, nicht nachgebaut, bewusst dunkel):
  - Nussbaumgehäuse mit sichtbarem Holzrand, darunter ein zurückgesetzter schwarzer Sockel.
  - Bildblende aus dunkel eloxiertem Aluminium (neues Material `Blende`), mittig um das Bild,
    mit abgeschrägter Kante und tiefem Tunnel mit weit gerundeten Ecken bis zur Gummidichtung
    am Glas (neue Hilfen `rund_umfang`/`rund_ring`).
  - Eigene Bediensäule hinter einer Fuge: Kopfteil mit Drucktaste NETZ, Zugknopf LAUTSTÄRKE
    und Kontrolllampe; schwarze Leiste mit vier Rändelrädern (Bildlage, Schärfe, Helligkeit,
    Farbton) und Farbknopf; Wählerplatte mit VHF-Kanalring 2–13 (neues Material
    `Kanalskala`) und UHF-Feinskala, beide mit Knebelknopf; unten 14 Lautsprecherlamellen.
  - Aufdrucke hell auf dunkel; Programmtasten, silbernes Feld und gelochtes Gitter entfallen.
- `BILD_MITTE.z` liegt jetzt auf dem Scheitel des Glases (−0,18 statt −0,215); das Licht des
  Bildschirms kommt von hinter dem Glas, damit die Tunnelwände es abbekommen. Rücklicht der
  Wände etwas stärker. Gerätebreite für die Kamera 0,68 m.
- Ungeprüft: Leistung (mehr Dreiecke am Gerät, unerheblich erwartet).

## 2026-11-02 · roehre-Hauptsitzung · 3D-Raum: Fernseher im Detail, Szene feingeschliffen
- Bedienfeld neu: silberne Blende statt schwarzer Fläche; zwei Drehknöpfe mit 36 Rippen,
  Aluminiumkappe, weißem Zeiger und Skalenring (LAUTSTÄRKE, HELLIGKEIT); acht Programmtasten
  mit hellen Ziffern, Taste 3 gedrückt mit grüner LED (PROGRAMM); Netztaste in Chromfassung mit
  roter LED (NETZ), Fernbedienungsfenster; gelochtes Lautsprechergitter mit Chromrahmen;
  Aufdruck STEREO unter dem Bildschirm. Beschriftung als eigene 5×7-Pixelschrift, keine Marke.
- Rücklicht der Lampe (warmes Licht von den angestrahlten Wänden): vorher lag die Front des
  Fernsehers mit Knöpfen und Aufdrucken im völligen Schwarz.
- Eiche des Tisches ruhiger: weniger Kontrast, mehr Jahresringe je Brett.
- Glühen nur noch aus selbstleuchtenden Flächen (Bildschirm, Schirm, Birne, LEDs, Fenster;
  im Alpha der Raumfarbe als negatives Vorzeichen der Entfernung markiert). Vorher strahlte
  jede helle Fläche über einer festen Schwelle: die angestrahlte Tapete neben der Lampe glühte
  mit und trug ihr Muster als fleckige helle Wolken. Angestrahlte Flächen glühen erst weit über
  Weiß. Glanzbild mittelt jetzt alle 64 Punkte seines Blocks, weich gefiltert über drei
  versetzte Ringe zu je 12 Punkten.
- Ungeprüft: Leistung nach diesen Änderungen (Glanzpass liest jetzt jeden Punkt einmal, das
  entspricht dem vorherigen Aufwand in etwa); Bilder in `docs/bilder/feinschliff/`.

## 2026-11-02 · roehre-Hauptsitzung · 3D-Raum: Rohpixel und Vignette schaltbar, Gehäusekante ruhig
- Neue Optionen `raum.rohpixel` (an/aus, Standard an) und `raum.vignette` (0 … 1, Standard 1).
  Mit Korn 0 und Glühen 0 liegt über dem Raum nur noch die Tiefenunschärfe; die Farbstimmung
  (leicht entsättigt) bleibt.
- Spur heller Punkte an der rechten Gehäusekante („ganz nah“): Ursache waren feine Spalten
  zwischen Front, Rückwand und Holzmantel; durch sie blitzten Lampe und Wand dahinter
  pixelweise hindurch. Gefunden durch Abschalten: Lampe aus → weg, Schatten aus → stärker,
  Relief flach → unverändert. Front und Rückwand ragen jetzt in den Mantel hinein.
  Erst vermutete Ursachen (Schattenakne, Streiflicht, Relief des Furniers) geprüft und verworfen.
- Ruhigere Glanzlichter: Relief immer aus der geglätteten Höhe (die gerasterte springt an jeder
  Texelgrenze), Relief bei starker Verkleinerung ausgeblendet, Glanzbreite aus der Streuung der
  Normale (Toksvig), Exponent höchstens 512. Gemessen (Pixel > 0,25 heller als ihre Umgebung,
  1080p): „ganz nah“ auf Gehäuse und Rahmen 766 → 516, „nah“ 1013 → 560 (ohne Rohpixel 297),
  „seitlich“ 854 → 649.
- Prüfstand `raum`: Schalter `--ohne-rohpixel`, `--ohne-korn`, `--nur-unschaerfe`.

## 2026-11-02 · roehre-Hauptsitzung · 3D-Raum nach Rückmeldung (Kamera, Rekorder, Tisch)
- Neue Kamerastufe „Ganz nah“ (`raum.naehe` = 1): Bildschirm samt schmalem Rahmen füllt 84 %
  der Höhe, oben und unten nur Gehäuse. Festwerte jetzt weit 0 / mittel 0,5 / nah 0,75 /
  ganz nah 1 (vorher war 1 „nah“). Prüfläufe angepasst (Bildschirm nie abgeschnitten).
- Super Famicom entfernt (zu hell im Lampenlicht); Videorekorder ohne Marke mit Leuchtanzeige
  „CH 03 · 12:00“; statt der Zeitschrift ein Stapel Videohüllen mit lesbaren Rückentiteln
  (eigene 5×7-Pixelschrift, erfundene Titel im Stil von Verleihkassetten, eine selbst bespielte
  „SOMMER '88“).
- Tisch: massive Eiche neu (Bretter entlang der Tischlänge – vorher quer –, feine Faserlinien,
  dunkles Spätholz, Poren, dunkleres Braun, flaches Relief gegen Glitzern), Platte mit
  gerundeter Oberkante und abgesetzter Unterkante, Kopfleisten mit quer laufender Faser,
  gedrechselte Beine mit Wulst. Parkett nutzt dieselbe Eiche.

## 2026-11-02 · roehre-Hauptsitzung · Vulkan unter Windows
- `vulkan_spiel.c` jetzt auch für Windows (Win32-Handles, Vulkan zur Laufzeit geladen).
- Lauf beim Anwender (RTX 5070 Ti): Bilder flach und Raum korrekt. Ob mit geteilten Semaphoren:
  letzte Ausgabezeile noch nicht gesehen.

## 2026-11-02 · roehre-Hauptsitzung · GPL-freie Variante (W-003)
- Neue Option `allgemein.lizenz` (gpl/permissiv), Ketten BeansRgb/BeansSignal/BeansSchnell
  aus crt-beans (MIT) mit eigener Übersetzung (`plane_beans`), eigener Helligkeitspass
  `shader/eigen/beans/helligkeit.slang`, Voreinstellungen `mit-rgb`, `mit-composite`.
- Ohne `shader/fremd/gpl/` schaltet roehre selbst auf die MIT-Kette (Wache
  `ohne_gpl_ordner_rechnet_die_mit_kette`).
- Abstimmung gemessen: Fleck 1,00/0,80 statt 0,90/0,40 (sonst Zeilenschwebung bis 3,25 % bei
  1080p), Verstärkung 1,15 (Weiß 100–102 %, Grau 97 %). Prüflauf bestanden.
- Grenzen: kein gewölbtes Glas, keine Farbsäume/Regenbogen, kein Nachleuchten u. a.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 8 abgeschlossen
- D3D12-Austausch beim Anwender (RTX 5070 Ti) in Ordnung, flach und Raum.
- Alle geplanten Etappen 0–8 abgeschlossen. Offen bleiben: W-003 (permissive Variante ohne
  GPL-Shader), Vulkan mit geteilten Semaphoren auf echter Hardware, Messung auf Mittelklasse.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 8, Messwerte und D3D12-Fehler
- Messung des Anwenders (RTX 5070 Ti, Vulkan): Filter 0,15–0,66 ms (1080p), 0,48–1,00 ms (4K);
  3D-Raum 1,17 / 2,52 ms. Composite/S-Video +0,4 ms bei 4K, alles andere unter 0,2 ms.
- Erster D3D12-Lauf brach ab: `enable_cache: true` ließ librashader einen Pipeline-Cache anlegen,
  den wgpu unter D3D12 nicht kennt. Jetzt nur, wenn das Gerät `PIPELINE_CACHE` hat. Neue Wache
  `roehre-wgpu/tests/ohne_zusatzfaehigkeiten.rs` (schlägt ohne die Korrektur genau so fehl).
- Offen: zweiter D3D12-Lauf, Messung unter D3D12.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 8 (Teil 1 und 2)
- Messbefehl `roehre-pruefstand leistung` (GPU-Zeitstempel; Filter je Voreinstellung, 3D-Raum,
  Kosten je Regler, 1080p/4K, Anteil am 60-Hz-Budget). Auf lavapipe nur auf Funktion geprüft.
- Windows-Paket (MinGW-Querbau) mit Prüfstand, Vorschau, DLL und Beispielen für den Anwender.
- Englische Texte für alle Optionen und Voreinstellungen, mit Wachen; Clippy sauber.
- 3D-Raum: Konsole im Stil des Super Famicom mit Controller (Wunsch des Anwenders, ohne Logo).
- Mission: Impossible kann mit dem Einbau beginnen; Schnittstelle wird nur noch ergänzt.
- Offen: Messwerte und D3D12-Lauf vom Rechner des Anwenders.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 7
- Neue Kiste `roehre-c` mit `include/roehre.h`: Katalog als JSON, Einstellungen (laden, setzen,
  speichern), Vorschaubild als RGBA für Launcher (W-007), Austausch mit Vulkan- und
  D3D12-Spielen (W-001), beide Modi (flach, 3D-Raum), Strahlsimulation im Takt.
- Entscheidung: eigenes Gerät auf derselben Grafikkarte mit geteiltem Speicher statt Rechnen auf
  dem Gerät des Spiels (wgpu übernimmt kein fremdes D3D12-Gerät; ein fremdes Vulkan-Gerät nur mit
  passenden Funktionen, die RT64/plume nicht einschaltet). Kostet zwei Kopien je Bild.
- Geprüft: `beispiele/vorschau.c` und `beispiele/vulkan_spiel.c` laufen auf lavapipe (dort ohne
  geteilte Semaphoren: Notweg über die CPU). Windows-DLL quer gebaut, `beispiele/d3d12_spiel.cpp`
  übersetzt und gelinkt, nicht ausgeführt.
- Einbauanleitungen: Mission: Impossible mit Einstiegsstelle in RT64
  (`rt64_present_queue.cpp`), Terranigma (wgpu 24 → 30).
- Ungeprüft: D3D12 und Vulkan mit Semaphoren auf echter Hardware, Kosten.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 6 abgeschlossen
- Kamera des Raums (W-005) als Optionen der neuen Gruppe `raum` (W-006): Nähe als Anteil der
  Fensterhöhe (weit 24 %, mittel 45 %, nah 80 %), Blickwinkel ±30°, Schwenk um das Gerät,
  Lage im Fenster per außermittiger Projektion (Lens Shift), passt bei 4:3 bis 32:9.
  Prüfläufe ohne GPU: Gerät nie abgeschnitten, Anteil stimmt, Schwenk hält das Gerät fest.
- Tiefenunschärfe (Abstand im Alphakanal, gedrehtes Muster) und Glühen (1/8-Zwischenstufe).
- Bücherregal, Straßenlicht durchs Fenster, vordere Wand weiter weg (Weitwinkel bei 32:9).
- Fehler behoben: die Eckenverschattung dunkelte jede Wand ganz ab (Abstand zur eigenen Fläche).
- Wünsche W-005 bis W-008 des Mission:-Impossible-Ports eingeplant (C-API Etappe 7,
  Messung Etappe 8); nichts vorgezogen.
- Ungeprüft: Wirkung und Leistung auf echter GPU; Vorschau-Fenster nur gestartet, nicht bedient.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 6, Teil 4 (3D-Raum nach Rückmeldung)
- Rückmeldung des Anwenders umgesetzt: Gummibaum als Ficus elastica (drei Triebe, große
  gefaltete Blätter mit Stiel, durchscheinend im Gegenlicht, rote Triebhülle), Stehlampe in
  die Zimmerecke, Pflanze neben den Tisch; Wildeiche für Tisch und Parkett (Äste, Risse,
  Farbspiel, matt geölt); schlanke Fernbedienung mit Tasten als Textur; schmale dunkle
  Fußleisten; Goldrahmen mit Profil und Passepartout; neblige Nacht vor dem Fenster;
  Schmutz und Verschattung in den Raumkanten.
- Texturen jetzt 256² und als Rohpixel (Koordinate rastet auf die Texelmitte, Mip und
  Anisotropie bleiben), wie vom Anwender gewünscht.
- Neu: `Raum::wandbild_setzen` (eigenes Bild im Rahmen). Prüfstand `raum --wandbild <png>`,
  Vorschau liest `ROEHRE_WANDBILD` oder `lokal/wandbild.png` (`lokal/` ist nicht im Repository).
- Ungeprüft: Wirkung und Leistung auf echter GPU.

## 2026-11-02 · roehre-Hauptsitzung · Korrektur Mission: Impossible
- Hinweis der KI des Mission:-Impossible-Projekts (über den Anwender): Integrationsnotiz war
  veraltet. Jetzt: Repository MissionImpossibleDecomp, Port in C++ mit RT64 (D3D12/Vulkan),
  Projektregel „kleine Eingriffe in Upstream-Code“, Wünsche W-001 und W-004 präzisiert.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 5
- Bewegungsschärfe: Strahlsimulation nach Blur Busters mit echtem Vorgängerbild, Schwarzbilder,
  Schnittstelle `zeichne_im_takt`. Lichtsumme bei 120/144/165/240/480 Hz auf 0,5 % genau.
- Befund: Strahl darf bei nicht ganzzahligem Verhältnis nicht je Spielbild neu beginnen
  (bis 20 % Licht fehlten).
- Ungeprüft: Wirkung am echten Bildschirm mit hoher Bildwiederholrate.
- Bericht: `docs/etappen/etappe-5.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 4
- Berechnete Gehäuse (nur Glas, Fernseher, Monitor, Automat) mit Glas, Innenfase mit
  Spiegelung, Licht des Bildes auf Rahmen und Wand; Breitbild-Röhre (16:9-Glas).
- Automatisches Einrasten unter 4 px je Zeile: 720p-Zeilenschwebung aller Voreinstellungen
  von bis zu 4,6 % auf ≤ 0,5 %. Referenzbilder neu geschrieben (Bild bei 540p kleiner).
- Ungeprüft: Gehäuse nur in WGSL (nicht in der künftigen C-API).
- Bericht: `docs/etappen/etappe-4.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 3
- Sechs neue Voreinstellungen, Optionen `strahl.zeilenverdopplung` und `signal.bandbreite`.
- Mitteltöne waren zu dunkel (Gamma 1,95, Maske in dunklen Teilen zu stark): Standards
  geändert, alle Voreinstellungen automatisch abgestimmt (Weiß 97–103 %, Grau 87–101 %).
- `strahl.schaerfe` wirkt jetzt auch in den NTSC-/PAL-Ketten (`SIGMA_HOR`).
- Prüflauf `roehre-pruefstand pruefe` mit Referenzbildern, Moiré- und Helligkeitsgrenzen;
  GitHub-Workflow `pruefung.yml` (noch nie gelaufen).
- Bericht: `docs/etappen/etappe-3.md`.

## 2026-11-02 · roehre-Hauptsitzung · Übergabe für andere KIs
- `AGENTS.md`, `CLAUDE.md`, `docs/ki/` (STAND, Wunschliste, Logbuch, Prompt-Vorlage) angelegt,
  damit KIs anderer Spielprojekte roehre verstehen, einbauen und selbst verbessern können.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 2
- Virtuelle Zeilenzahl, Moiré-Messung, HDR mit Eichung, Flächen-Modus für 3D-Szenen.
- Fehler behoben: Voreinstellungen waren seit Etappe 1 beschädigt; Overscan halbiert.
- Ungeprüft: HLSL-Fassung der Flächen-Maske; HDR nicht auf echtem HDR-Bildschirm angesehen.
- Bericht: `docs/etappen/etappe-2.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 1
- Grundgerüst: Einstellungsmodell, Kettenzuordnung, wgpu-Schnittstelle, Testbilder, Prüfstand.
- Bericht: `docs/etappen/etappe-1.md`.

## 2026-11-02 · roehre-Hauptsitzung · Etappe 0
- Recherche und Plan: `docs/PLAN.md`.
