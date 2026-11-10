//! Der Katalog aller Optionen.
//!
//! Jede Option hat einen festen Schlüssel der Form `gruppe.name`. Der Schlüssel ist Teil
//! des Dateiformats und wird nie umbenannt; ändert sich eine Option grundlegend, gibt es
//! eine Migration (siehe [`crate::Einstellungen`]).
//!
//! Die Zahlenwerte vieler Optionen entsprechen den Parametern von crt-guest-advanced
//! (guest(r), GPL-2.0-or-later). Die Grenzen sind hier enger gefasst als dort: Sie umfassen
//! den Bereich, der eine Röhre glaubwürdig nachbildet, nicht jeden technisch möglichen Wert.

use crate::texte::Texte;

/// Gruppe einer Option, in der Reihenfolge, in der ein Menü sie zeigen sollte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gruppe {
    Allgemein,
    Signal,
    Strahl,
    Maske,
    Leuchtstoff,
    Licht,
    Geometrie,
    Helligkeit,
    Bildformat,
    Eingang,
    Gehaeuse,
    Bewegung,
    /// Kamera und Stimmung des 3D-Vorführraums (`roehre-raum`); wirkt nicht auf die Röhre.
    Raum,
}

impl Gruppe {
    pub const ALLE: [Gruppe; 13] = [
        Gruppe::Allgemein,
        Gruppe::Signal,
        Gruppe::Strahl,
        Gruppe::Maske,
        Gruppe::Leuchtstoff,
        Gruppe::Licht,
        Gruppe::Geometrie,
        Gruppe::Helligkeit,
        Gruppe::Bildformat,
        Gruppe::Eingang,
        Gruppe::Gehaeuse,
        Gruppe::Bewegung,
        Gruppe::Raum,
    ];

    /// Schlüssel der Gruppe, zugleich Tabellenname in TOML.
    pub fn schluessel(self) -> &'static str {
        match self {
            Gruppe::Allgemein => "allgemein",
            Gruppe::Signal => "signal",
            Gruppe::Strahl => "strahl",
            Gruppe::Maske => "maske",
            Gruppe::Leuchtstoff => "leuchtstoff",
            Gruppe::Licht => "licht",
            Gruppe::Geometrie => "geometrie",
            Gruppe::Helligkeit => "helligkeit",
            Gruppe::Bildformat => "bildformat",
            Gruppe::Eingang => "eingang",
            Gruppe::Gehaeuse => "gehaeuse",
            Gruppe::Bewegung => "bewegung",
            Gruppe::Raum => "raum",
        }
    }

    pub fn name(self) -> Texte {
        match self {
            Gruppe::Allgemein => Texte::de_en("Allgemein", "General"),
            Gruppe::Signal => Texte::de_en("Signal", "Signal"),
            Gruppe::Strahl => Texte::de_en("Strahl und Zeilen", "Beam and scanlines"),
            Gruppe::Maske => Texte::de_en("Maske", "Mask"),
            Gruppe::Leuchtstoff => Texte::de_en("Leuchtstoff und Farbe", "Phosphor and colour"),
            Gruppe::Licht => Texte::de_en("Licht im Glas", "Light in the glass"),
            Gruppe::Geometrie => Texte::de_en("Geometrie", "Geometry"),
            Gruppe::Helligkeit => Texte::de_en("Helligkeit", "Brightness"),
            Gruppe::Bildformat => Texte::de_en("Bildformat", "Picture format"),
            Gruppe::Eingang => Texte::de_en("Eingangsbild", "Input picture"),
            Gruppe::Gehaeuse => Texte::de_en("Gehäuse", "Cabinet"),
            Gruppe::Bewegung => Texte::de_en("Bewegungsschärfe", "Motion clarity"),
            Gruppe::Raum => Texte::de_en("3D-Raum", "3D room"),
        }
    }
}

/// Grobe Leistungskosten einer Option, damit ein Menü teure Optionen kennzeichnen kann.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kosten {
    /// Nur ein Uniform-Wert, kein messbarer Unterschied.
    Keine,
    /// Wenige zusätzliche Abtastungen.
    Gering,
    /// Zusätzliche Pässe oder viele Abtastungen.
    Hoch,
}

/// Art und Wertebereich einer Option.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionsArt {
    Zahl {
        min: f64,
        max: f64,
        schritt: f64,
        standard: f64,
        einheit: Option<&'static str>,
    },
    Ganzzahl {
        min: i64,
        max: i64,
        standard: i64,
    },
    Schalter {
        standard: bool,
    },
    Auswahl {
        werte: &'static [(&'static str, Texte)],
        standard: &'static str,
    },
}

/// Beschreibung einer Option.
#[derive(Clone, Copy, Debug)]
pub struct OptionBeschreibung {
    /// Fester Schlüssel `gruppe.name`.
    pub schluessel: &'static str,
    pub gruppe: Gruppe,
    pub name: Texte,
    pub beschreibung: Texte,
    pub art: OptionsArt,
    pub kosten: Kosten,
    /// Die Option wirkt nur, wenn die genannte Auswahl-Option einen der Werte hat.
    pub wirkt_wenn: Option<(&'static str, &'static [&'static str])>,
}

impl OptionBeschreibung {
    /// Relativer Pfad des Vorschaubilds dieser Option (Etappe 3 erzeugt die Bilder).
    pub fn vorschaubild(&self) -> String {
        format!("vorschau/{}.png", self.schluessel)
    }
}

const fn zahl(min: f64, max: f64, schritt: f64, standard: f64) -> OptionsArt {
    OptionsArt::Zahl { min, max, schritt, standard, einheit: None }
}

const fn zahl_einheit(min: f64, max: f64, schritt: f64, standard: f64, einheit: &'static str) -> OptionsArt {
    OptionsArt::Zahl { min, max, schritt, standard, einheit: Some(einheit) }
}

const NICHT_RGB: Option<(&str, &[&str])> = Some(("signal.art", &["svideo", "composite", "antenne"]));
const MIT_MASKE: Option<(&str, &[&str])> = Some(("maske.typ", &["streifenmaske", "schlitzmaske", "lochmaske"]));

static KATALOG: &[OptionBeschreibung] = &[
    // ---------------------------------------------------------------- Allgemein
    OptionBeschreibung {
        schluessel: "allgemein.roehre",
        gruppe: Gruppe::Allgemein,
        name: Texte::de_en("Röhrensimulation", "CRT simulation"),
        beschreibung: Texte::de_en("Schaltet die ganze Röhre ein oder aus. Aus heißt: Das Bild wird nur sauber und unverzerrt eingepasst.", "Turns the whole CRT simulation on or off. Off means the picture is only scaled cleanly and without distortion."),
        art: OptionsArt::Schalter { standard: true },
        kosten: Kosten::Hoch,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "allgemein.leistung",
        gruppe: Gruppe::Allgemein,
        name: Texte::de_en("Leistungsstufe", "Performance level"),
        beschreibung: Texte::de_en("„Voll“ ist die genaueste Darstellung. „Leicht“ spart Rechenzeit für Laptops und schwache Grafikkarten, mit etwas weniger Licht im Glas.", "“Full” is the most accurate rendering. “Light” saves GPU time for laptops and weak graphics cards, with a little less light in the glass."),
        art: OptionsArt::Auswahl {
            werte: &[("voll", Texte::de_en("Voll", "Full")), ("leicht", Texte::de_en("Leicht", "Light"))],
            standard: "voll",
        },
        kosten: Kosten::Hoch,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "allgemein.lizenz",
        gruppe: Gruppe::Allgemein,
        name: Texte::de_en("Röhren-Shader", "CRT shader"),
        beschreibung: Texte::de_en(
            "Welche Shaderkette die Röhre rechnet. „Volle Auswahl“ ist crt-guest-advanced (GPL-2.0): alle Optionen wirken. „Nur MIT“ ist crt-beans (MIT-Lizenz) für Spiele, die keine GPL-Shader ausliefern dürfen: schöne Zeilen und Maske, aber kein gewölbtes Glas, keine Ecken und Vignette, keine Farbsäume oder Regenbogen bei Composite (nur weichere Farbe), keine Konvergenzfehler, kein Nachleuchten, kein Helligkeitsausgleich und keine Maskenstärke. Fehlt der GPL-Ordner, rechnet roehre automatisch mit „Nur MIT“.",
            "Which shader chain renders the CRT. “Full choice” is crt-guest-advanced (GPL-2.0): every option works. “MIT only” is crt-beans (MIT licence) for games that must not ship GPL shaders: fine scanlines and mask, but no curved glass, no corners or vignette, no colour fringes or rainbows on composite (only softer colour), no convergence errors, no afterglow, no brightness compensation and no mask strength. If the GPL folder is missing, roehre automatically uses “MIT only”.",
        ),
        art: OptionsArt::Auswahl {
            werte: &[
                ("gpl", Texte::de_en("Volle Auswahl (guest-advanced, GPL)", "Full choice (guest-advanced, GPL)")),
                ("permissiv", Texte::de_en("Nur MIT (crt-beans)", "MIT only (crt-beans)")),
            ],
            standard: "gpl",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Signal
    OptionBeschreibung {
        schluessel: "signal.art",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Anschluss", "Connection"),
        beschreibung: Texte::de_en("Wie das Bild zum Fernseher kommt. RGB ist am saubersten, Composite und Antenne mischen Helligkeit und Farbe in ein Signal und erzeugen Farbsäume und Kantenflimmern.", "How the picture gets to the TV. RGB is the cleanest; composite and RF mix brightness and colour into one signal and produce colour fringes and edge shimmer."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("rgb", Texte::de_en("RGB (Scart, Komponente)", "RGB (SCART, component)")),
                ("svideo", Texte::de_en("S-Video", "S-Video")),
                ("composite", Texte::de_en("Composite (Cinch)", "Composite")),
                ("antenne", Texte::de_en("Antenne (HF)", "RF")),
            ],
            standard: "rgb",
        },
        kosten: Kosten::Hoch,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "signal.norm",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Fernsehnorm", "TV standard"),
        beschreibung: Texte::de_en("NTSC (Japan, USA) zeigt bei Composite wandernde Kantenmuster (Dot-Crawl). PAL (Europa) mittelt die Farbe über zwei Zeilen und wirkt dadurch ruhiger, aber farblich weicher.", "NTSC (Japan, USA) shows crawling edge patterns on composite (dot crawl). PAL (Europe) averages colour over two lines and looks calmer but softer in colour."),
        art: OptionsArt::Auswahl {
            werte: &[("ntsc", Texte::de_en("NTSC", "NTSC")), ("pal", Texte::de_en("PAL", "PAL"))],
            standard: "ntsc",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.artefakte",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Farbartefakte", "Colour artifacts"),
        beschreibung: Texte::de_en("Stärke der Regenbogen- und Musterbildung an feinen Details, wenn Helligkeit als Farbe missverstanden wird.", "Strength of rainbow and pattern artefacts on fine detail, when brightness is mistaken for colour."),
        art: zahl(0.0, 3.0, 0.1, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.farbsaum",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Farbsäume", "Colour fringing"),
        beschreibung: Texte::de_en("Farbe, die an Kanten in die Nachbarschaft ausläuft.", "Colour bleeding from edges into the neighbourhood."),
        art: zahl(0.0, 3.0, 0.1, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.saettigung",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Farbsättigung des Signals", "Signal saturation"),
        beschreibung: Texte::de_en("Farbstärke nach dem Dekodieren des Signals.", "Colour strength after decoding the signal."),
        art: zahl(0.0, 2.0, 0.01, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.bandbreite",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Bandbreite des Signals", "Signal bandwidth"),
        beschreibung: Texte::de_en("Wie viel Feinzeichnung das Signal trägt. 1,0 entspricht dem Signal einer Konsole am Fernseher; höhere Werte machen kleine Schrift lesbarer, Farbsäume und Regenbogen bleiben.", "How much fine detail the signal carries. 1.0 matches a console on a TV; higher values make small text more readable, colour fringes and rainbows remain."),
        art: zahl(0.5, 2.0, 0.05, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.rauschen",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Rauschen", "Noise"),
        beschreibung: Texte::de_en("Bildrauschen eines schwachen Signals, etwa über Antenne oder ein langes Kabel.", "Picture noise of a weak signal, e.g. via an antenna or a long cable."),
        art: zahl(0.0, 0.5, 0.01, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Strahl
    OptionBeschreibung {
        schluessel: "strahl.form_dunkel",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Zeilen bei dunklen Bildteilen", "Scanlines in dark areas"),
        beschreibung: Texte::de_en("Wie schmal der Strahl bei wenig Helligkeit ist. Höher heißt schmalere Zeilen mit deutlicheren Lücken.", "How narrow the beam is at low brightness. Higher means thinner lines with clearer gaps."),
        art: zahl(0.5, 4.0, 0.05, 1.3),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.form_hell",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Zeilen bei hellen Bildteilen", "Scanlines in bright areas"),
        beschreibung: Texte::de_en("Wie schmal der Strahl bei voller Helligkeit ist. Echte Röhren haben bei hellen Bildteilen breitere, fast verschmelzende Zeilen.", "How narrow the beam is at full brightness. Real CRTs draw bright areas with wider, almost merging lines."),
        art: zahl(0.5, 2.5, 0.025, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.aufblaehen",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Breitere helle Zeilen", "Wider bright scanlines"),
        beschreibung: Texte::de_en("Wie stark der Strahl bei hellen Bildteilen zusätzlich anschwillt.", "How much the beam additionally swells in bright areas."),
        art: zahl(0.0, 1.0, 0.05, 0.6),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.zeilenfarbe",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Farbsättigung der Zeilenränder", "Scanline edge saturation"),
        beschreibung: Texte::de_en("Wie kräftig die Farbe zum Rand jeder Zeile wird.", "How strong the colour becomes towards the edge of each line."),
        art: zahl(0.0, 3.0, 0.1, 0.5),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.schaerfe",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Waagerechte Schärfe", "Horizontal sharpness"),
        beschreibung: Texte::de_en("Wie scharf der Strahl entlang einer Zeile zeichnet.", "How sharply the beam draws along a line."),
        art: zahl(0.5, 10.0, 0.1, 5.2),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.zeilensprung",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Zeilensprung (480i/576i)", "Interlacing"),
        beschreibung: Texte::de_en("Wie Bilder mit hoher Zeilenzahl dargestellt werden: als Halbbilder wie auf der Röhre oder geglättet.", "How pictures with a high line count are shown: as fields like on a CRT, or smoothed."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("aus", Texte::de_en("Aus", "Off")),
                ("halbbilder", Texte::de_en("Halbbilder", "Fields")),
                ("geglaettet", Texte::de_en("Geglättet", "Smoothed")),
            ],
            standard: "halbbilder",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.zeilenverdopplung",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Zeilenverdopplung (31 kHz)", "Line doubling (31 kHz)"),
        beschreibung: Texte::de_en("Wie ein PC-Röhrenmonitor (VGA): Bilder mit wenig Zeilen (240p) werden mit jeder Zeile doppelt gezeichnet. Die Zeilen werden dadurch feiner und dichter. Wirkt nur, wenn jede Zeile mindestens 4 Bildschirmpixel bekommt (bei 224 Zeilen also ab etwa 4K); darunter würden die feinen Zeilen schweben.", "Like a PC CRT monitor (VGA): low-line pictures (240p) draw every line twice, making the lines finer and denser. Only takes effect when each line gets at least 4 screen pixels (about 4K for 224 lines); below that the fine lines would beat."),
        art: OptionsArt::Schalter { standard: false },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_rot",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Rot (waagerecht)", "Red convergence (horizontal)"),
        beschreibung: Texte::de_en("Versatz des roten Strahls gegenüber Grün, in Ausgabepixeln.", "Offset of the red beam relative to green, in output pixels."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_blau",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Blau (waagerecht)", "Blue convergence (horizontal)"),
        beschreibung: Texte::de_en("Versatz des blauen Strahls gegenüber Grün, in Ausgabepixeln.", "Offset of the blue beam relative to green, in output pixels."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_rot_y",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Rot (senkrecht)", "Red convergence (vertical)"),
        beschreibung: Texte::de_en("Senkrechter Versatz des roten Strahls, in Ausgabepixeln.", "Vertical offset of the red beam, in output pixels."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_blau_y",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Blau (senkrecht)", "Blue convergence (vertical)"),
        beschreibung: Texte::de_en("Senkrechter Versatz des blauen Strahls, in Ausgabepixeln.", "Vertical offset of the blue beam, in output pixels."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_rand",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler zum Rand", "Convergence error towards edges"),
        beschreibung: Texte::de_en("0 heißt: überall gleich. Höhere Werte lassen den Fehler zur Mitte hin verschwinden und zum Rand wachsen, wie bei einer echten Röhre.", "0 means the same everywhere. Higher values make the error vanish towards the centre and grow towards the edge, as on a real CRT."),
        art: zahl(0.0, 0.75, 0.05, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Maske
    OptionBeschreibung {
        schluessel: "maske.typ",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Maskentyp", "Mask type"),
        beschreibung: Texte::de_en("Die Anordnung der roten, grünen und blauen Leuchtpunkte. Streifenmaske: senkrechte Streifen (Trinitron, PVM). Schlitzmaske: Streifen mit versetzten Unterbrechungen (die meisten Fernseher). Lochmaske: versetzte Punkte (PC-Monitore, Arcade).", "The arrangement of the red, green and blue phosphors. Aperture grille: vertical stripes (Trinitron, PVM). Slot mask: stripes with staggered breaks (most TVs). Shadow mask: staggered dots (PC monitors, arcade)."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("aus", Texte::de_en("Aus", "Off")),
                ("streifenmaske", Texte::de_en("Streifenmaske", "Aperture grille")),
                ("schlitzmaske", Texte::de_en("Schlitzmaske", "Slot mask")),
                ("lochmaske", Texte::de_en("Lochmaske", "Shadow mask")),
            ],
            standard: "schlitzmaske",
        },
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "maske.staerke",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Maskenstärke hell", "Mask strength (bright)"),
        beschreibung: Texte::de_en("Wie deutlich die Maske in hellen Bildteilen zu sehen ist.", "How visible the mask is in bright areas."),
        art: zahl(0.0, 1.0, 0.025, 0.3),
        kosten: Kosten::Keine,
        wirkt_wenn: MIT_MASKE,
    },
    OptionBeschreibung {
        schluessel: "maske.staerke_dunkel",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Maskenstärke dunkel", "Mask strength (dark)"),
        beschreibung: Texte::de_en("Wie deutlich die Maske in dunklen Bildteilen zu sehen ist. Gleich der Stärke für helle Bildteile heißt: Die Maske lässt bei jeder Helligkeit gleich viel Licht durch, wie eine echte Maske. Höhere Werte machen dunkle und mittlere Töne deutlich dunkler (gemessen: 40-%-Grau von 88 % auf 64 % des sauberen Bildes bei 1,1).", "How visible the mask is in dark areas. Equal to the strength for bright areas means the mask lets through the same share of light at every brightness, like a real mask. Higher values make dark and mid tones much darker (measured: 40 % grey drops from 88 % to 64 % of the clean picture at 1.1)."),
        art: zahl(0.0, 1.5, 0.05, 0.4),
        kosten: Kosten::Keine,
        wirkt_wenn: MIT_MASKE,
    },
    OptionBeschreibung {
        schluessel: "maske.triaden",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Feinheit der Maske", "Mask density"),
        beschreibung: Texte::de_en("Gewünschte Zahl der Farbtripel über die Bildbreite. Fernseher haben etwa 300 bis 500, Profi-Monitore 600 bis 1000. Die Maske wird immer auf ganze Bildschirmpixel gelegt, damit kein Moiré entsteht; der tatsächliche Wert hängt deshalb von der Fenstergröße ab.", "Desired number of colour triads across the picture width. TVs have about 300 to 500, professional monitors 600 to 1000. The mask is always aligned to whole screen pixels to avoid moiré, so the actual value depends on the window size."),
        art: OptionsArt::Ganzzahl { min: 150, max: 1200, standard: 450 },
        kosten: Kosten::Keine,
        wirkt_wenn: MIT_MASKE,
    },
    OptionBeschreibung {
        schluessel: "maske.subpixel",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Subpixel-Reihenfolge des Bildschirms", "Display subpixel order"),
        beschreibung: Texte::de_en("Reihenfolge der Farbpunkte im eigenen Bildschirm. Die meisten sind RGB; manche Fernseher und Monitore sind BGR.", "Order of the subpixels of your own display. Most are RGB; some TVs and monitors are BGR."),
        art: OptionsArt::Auswahl {
            werte: &[("rgb", Texte::de_en("RGB", "RGB")), ("bgr", Texte::de_en("BGR", "BGR"))],
            standard: "rgb",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: MIT_MASKE,
    },
    // ---------------------------------------------------------------- Leuchtstoff
    OptionBeschreibung {
        schluessel: "leuchtstoff.profil",
        gruppe: Gruppe::Leuchtstoff,
        name: Texte::de_en("Leuchtstoff-Farbraum", "Phosphor gamut"),
        beschreibung: Texte::de_en("Die Grundfarben der Röhre. EBU war in Europa üblich, SMPTE-C in den USA, P22 bei vielen Fernsehern und Monitoren.", "The primaries of the CRT. EBU was common in Europe, SMPTE-C in the USA, P22 in many TVs and monitors."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("aus", Texte::de_en("Unverändert (sRGB)", "Unchanged (sRGB)")),
                ("ebu", Texte::de_en("EBU", "EBU")),
                ("p22", Texte::de_en("P22", "P22")),
                ("smpte_c", Texte::de_en("SMPTE-C", "SMPTE-C")),
                ("philips", Texte::de_en("Philips", "Philips")),
                ("trinitron", Texte::de_en("Trinitron", "Trinitron")),
            ],
            standard: "aus",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "leuchtstoff.farbtemperatur",
        gruppe: Gruppe::Leuchtstoff,
        name: Texte::de_en("Farbtemperatur", "Colour temperature"),
        beschreibung: Texte::de_en("Negative Werte machen das Weiß wärmer (gelblicher), positive kühler (bläulicher). Japanische Fernseher waren oft deutlich kühler eingestellt.", "Negative values make white warmer (more yellow), positive values cooler (more blue). Japanese TVs were often set noticeably cooler."),
        art: zahl_einheit(-100.0, 100.0, 5.0, 0.0, "%"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "leuchtstoff.saettigung",
        gruppe: Gruppe::Leuchtstoff,
        name: Texte::de_en("Farbsättigung", "Saturation"),
        beschreibung: Texte::de_en("Gesamte Farbstärke des Bildes.", "Overall colour strength of the picture."),
        art: zahl(0.0, 2.0, 0.05, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "leuchtstoff.nachgluehen",
        gruppe: Gruppe::Leuchtstoff,
        name: Texte::de_en("Nachglühen", "Afterglow"),
        beschreibung: Texte::de_en("Wie lange ein heller Punkt nach dem Abschalten noch schwach leuchtet. Sichtbar als kurze Schweife bei schneller Bewegung auf dunklem Grund.", "How long a bright dot keeps glowing faintly after it is switched off. Visible as short trails on fast motion over a dark background."),
        art: zahl(0.0, 0.6, 0.01, 0.2),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Licht
    OptionBeschreibung {
        schluessel: "licht.glow",
        gruppe: Gruppe::Licht,
        name: Texte::de_en("Glühen", "Glow"),
        beschreibung: Texte::de_en("Weiches Leuchten um helle Bildteile, aus der Streuung im Leuchtstoff.", "Soft glow around bright areas, from scattering in the phosphor."),
        art: zahl(0.0, 1.0, 0.01, 0.08),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "licht.bloom",
        gruppe: Gruppe::Licht,
        name: Texte::de_en("Überstrahlen", "Bloom"),
        beschreibung: Texte::de_en("Helle Bildteile strahlen in ihre Umgebung und füllen die Lücken zwischen den Zeilen.", "Bright areas bleed into their surroundings and fill the gaps between lines."),
        art: zahl(0.0, 1.0, 0.05, 0.0),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "licht.halation",
        gruppe: Gruppe::Licht,
        name: Texte::de_en("Lichthof (Halation)", "Halation"),
        beschreibung: Texte::de_en("Licht, das im dicken Frontglas der Röhre zurückgeworfen wird und einen weiten, schwachen Hof um helle Flächen legt.", "Light reflected inside the thick front glass of the tube, laying a wide, faint halo around bright areas."),
        art: zahl(0.0, 1.0, 0.025, 0.0),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Geometrie
    OptionBeschreibung {
        schluessel: "geometrie.woelbung_x",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Wölbung waagerecht", "Horizontal curvature"),
        beschreibung: Texte::de_en("0 ist flach. Fernseher der 80er und 90er liegen etwa bei 0,02 bis 0,05.", "0 is flat. TVs of the 80s and 90s are around 0.02 to 0.05."),
        art: zahl(0.0, 0.15, 0.005, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.woelbung_y",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Wölbung senkrecht", "Vertical curvature"),
        beschreibung: Texte::de_en("0 ist flach. Bei Streifenmasken-Röhren (Trinitron) ist die Röhre senkrecht flach.", "0 is flat. Aperture-grille tubes (Trinitron) are vertically flat."),
        art: zahl(0.0, 0.15, 0.005, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.ecken",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Abgerundete Ecken", "Rounded corners"),
        beschreibung: Texte::de_en("Radius der Bildecken.", "Radius of the picture corners."),
        art: zahl(0.0, 0.15, 0.005, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.randweich",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Weicher Bildrand", "Soft picture edge"),
        beschreibung: Texte::de_en("Breite des weichen Übergangs am Bildrand.", "Width of the soft transition at the picture edge."),
        art: zahl(0.0, 1.0, 0.01, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.vignette",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Randabdunklung", "Vignette"),
        beschreibung: Texte::de_en("Wie stark das Bild zu den Rändern dunkler wird.", "How much the picture darkens towards the edges."),
        art: zahl(0.0, 1.0, 0.05, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.overscan",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Überstand (Overscan)", "Overscan"),
        beschreibung: Texte::de_en("Anteil des Bildes, der wie bei einem Fernseher hinter dem Rahmen verschwindet, in Prozent je Seite.", "Share of the picture that disappears behind the bezel like on a TV, in percent per side."),
        art: zahl_einheit(0.0, 8.0, 0.5, 0.0, "%"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Helligkeit
    OptionBeschreibung {
        schluessel: "helligkeit.ausgleich_dunkel",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Ausgleich dunkle Bildteile", "Compensation dark areas"),
        beschreibung: Texte::de_en("Hebt dunkle Bildteile an, damit Maske und Zeilen das Bild nicht verdunkeln.", "Lifts dark areas so that mask and scanlines do not darken the picture."),
        art: zahl(0.5, 4.0, 0.05, 1.4),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "helligkeit.ausgleich_hell",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Ausgleich helle Bildteile", "Compensation bright areas"),
        beschreibung: Texte::de_en("Hebt helle Bildteile an.", "Lifts bright areas."),
        art: zahl(0.5, 3.0, 0.025, 1.1),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "helligkeit.nachher",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Gesamthelligkeit", "Overall brightness"),
        beschreibung: Texte::de_en("Helligkeit am Ende der Röhre.", "Brightness at the end of the CRT stage."),
        art: zahl(0.5, 3.0, 0.01, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "helligkeit.gamma_ausgang",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Gamma des Bildschirms", "Display gamma"),
        beschreibung: Texte::de_en("Gamma, mit dem das Ergebnis für den eigenen Bildschirm kodiert wird.", "Gamma used to encode the result for your own display."),
        art: zahl(1.8, 3.0, 0.05, 2.4),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Helligkeit (HDR)
    OptionBeschreibung {
        schluessel: "helligkeit.hdr_weiss",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("HDR: Helligkeit von Weiß", "HDR: white level"),
        beschreibung: Texte::de_en("Nur bei HDR-Ausgabe. Leuchtdichte, die eine weiße Fläche im Mittel haben soll. 200 nit entspricht einem hellen Röhrenfernseher.", "HDR output only. Average luminance a white area should have. 200 nits matches a bright CRT TV."),
        art: zahl_einheit(80.0, 600.0, 10.0, 200.0, "nit"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "helligkeit.hdr_spitze",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("HDR: Spitzenhelligkeit", "HDR: peak brightness"),
        beschreibung: Texte::de_en("Nur bei HDR-Ausgabe. Höchste Leuchtdichte einzelner Leuchtstreifen. Sie sollte der Spitzenhelligkeit des eigenen Bildschirms entsprechen; darüber wird weich begrenzt.", "HDR output only. Highest luminance of individual phosphor stripes. Should match the peak brightness of your display; above that it is soft-clipped."),
        art: zahl_einheit(200.0, 4000.0, 50.0, 1000.0, "nit"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Bildformat
    OptionBeschreibung {
        schluessel: "bildformat.seitenverhaeltnis",
        gruppe: Gruppe::Bildformat,
        name: Texte::de_en("Seitenverhältnis", "Aspect ratio"),
        beschreibung: Texte::de_en("4:3 wie auf einer Röhre. „Pixelgenau“ nutzt das Seitenverhältnis der Pixel aus dem Spiel (bei SNES 8:7). 16:9 entspricht einer späten Breitbild-Röhre. „Ganzes Fenster“ füllt das Fenster und verzerrt dabei.", "4:3 as on a CRT. “Pixel-exact” uses the game's pixel aspect ratio (8:7 on SNES). 16:9 matches a late widescreen CRT. “Whole window” fills the window and distorts."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("4:3", Texte::de_en("4:3", "4:3")),
                ("pixelgenau", Texte::de_en("Pixelgenau", "Pixel exact")),
                ("16:9", Texte::de_en("16:9", "16:9")),
                ("fenster", Texte::de_en("Ganzes Fenster (verzerrt)", "Whole window (stretched)")),
            ],
            standard: "4:3",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "bildformat.roehrenformat",
        gruppe: Gruppe::Bildformat,
        name: Texte::de_en("Format der Röhre", "Tube format"),
        beschreibung: Texte::de_en("Seitenverhältnis des Glases. „Wie das Bild“ heißt: Die Röhre ist so breit wie das Bild. 16:9 entspricht einer späten Breitbild-Röhre: Ein 4:3-Bild sitzt dann mittig, links und rechts bleibt schwarzer Leuchtstoff hinter dem Glas. Sichtbar nur mit Gehäuse (mindestens „nur Glas“).", "Aspect ratio of the glass. “Like the picture” means the tube is as wide as the picture. 16:9 matches a late widescreen CRT: a 4:3 picture then sits in the centre with black phosphor behind the glass left and right. Only visible with a cabinet (at least “glass only”)."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("wie_bild", Texte::de_en("Wie das Bild", "Like the picture")),
                ("16:9", Texte::de_en("16:9", "16:9")),
            ],
            standard: "wie_bild",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "bildformat.ganzzahlig",
        gruppe: Gruppe::Bildformat,
        name: Texte::de_en("Ganzzahlige Skalierung", "Integer scaling"),
        beschreibung: Texte::de_en("Jede Bildzeile bekommt gleich viele Bildschirmzeilen. Das Bild wird dafür etwas kleiner, Zeilen wirken aber völlig gleichmäßig. Empfohlen bei harten, schmalen Zeilen (Profi- und PC-Monitore) und bei 720p: Sonst schweben die Zeilen sichtbar.", "Every picture line gets the same number of screen lines. The picture becomes slightly smaller, but lines look perfectly even. Recommended for hard, narrow scanlines (professional and PC monitors) and at 720p, where lines otherwise visibly beat."),
        art: OptionsArt::Schalter { standard: false },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "bildformat.fuellung",
        gruppe: Gruppe::Bildformat,
        name: Texte::de_en("Bildgröße", "Picture size"),
        beschreibung: Texte::de_en("Anteil der Fensterhöhe, den das Bild höchstens einnimmt.", "Maximum share of the window height the picture takes up."),
        art: zahl_einheit(50.0, 100.0, 1.0, 100.0, "%"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Eingang
    OptionBeschreibung {
        schluessel: "eingang.zeilen",
        gruppe: Gruppe::Eingang,
        name: Texte::de_en("Zeilenzahl der Röhre", "Tube line count"),
        beschreibung: Texte::de_en("Wie viele Zeilen die Röhre zeichnet. 0 heißt automatisch: Bilder bis 600 Zeilen (SNES, N64) zeigen ihre eigenen Zeilen, höher aufgelöste Bilder (z. B. 1080p eines 3D-Spiels) werden auf 240 Zeilen gebracht, ab einer Ausgabe von 4K auf 480 (jede Zeile braucht mindestens 4 Bildschirmpixel, sonst entsteht ein Schwebungsmuster). 240 und 480 entsprechen NTSC, 288 und 576 PAL.", "How many lines the CRT draws. 0 means automatic: pictures up to 600 lines (SNES, N64) show their own lines, higher-resolution pictures (e.g. 1080p from a 3D game) are brought to 240 lines, from 4K output to 480 (each line needs at least 4 screen pixels, otherwise a beat pattern appears). 240 and 480 match NTSC, 288 and 576 PAL."),
        art: OptionsArt::Ganzzahl { min: 0, max: 1200, standard: 0 },
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "eingang.waagerecht",
        gruppe: Gruppe::Eingang,
        name: Texte::de_en("Waagerechte Auflösung", "Horizontal resolution"),
        beschreibung: Texte::de_en("Nur wenn die Zeilenzahl verringert wird: Bildpunkte je Zeile im Verhältnis zur Zeilenzahl und zum Seitenverhältnis. 2 heißt doppelt so viele Punkte wie bei quadratischen Pixeln; das hält das Bild waagerecht scharf, so wie der Strahl einer Röhre entlang der Zeile nicht in Pixel zerfällt.", "Only when the line count is reduced: dots per line relative to the line count and aspect ratio. 2 means twice as many dots as with square pixels; this keeps the picture sharp horizontally, just as a CRT beam does not break into pixels along the line."),
        art: zahl(1.0, 4.0, 0.25, 2.0),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Gehäuse
    OptionBeschreibung {
        schluessel: "gehaeuse.typ",
        gruppe: Gruppe::Gehaeuse,
        name: Texte::de_en("Gehäuse", "Cabinet"),
        beschreibung: Texte::de_en("Zeigt nicht nur das Bild, sondern den ganzen Bildschirm: Glas mit Glanz, Rahmen mit Spiegelung des Bildes und Licht, das das Bild auf Rahmen und Wand wirft. Alle Gehäuse sind berechnet, keine Fotos. „Nur Glas“ zeigt die Bildöffnung ohne Rahmen.", "Shows not just the picture but the whole screen: glass with sheen, bezel reflecting the picture, and light the picture casts on bezel and wall. All cabinets are computed, no photos. “Glass only” shows the picture opening without a bezel."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("aus", Texte::de_en("Aus", "Off")),
                ("nur_glas", Texte::de_en("Nur Glas", "Glass only")),
                ("fernseher", Texte::de_en("Fernseher", "Television")),
                ("monitor", Texte::de_en("Monitor", "Monitor")),
                ("automat", Texte::de_en("Spielautomat", "Arcade cabinet")),
            ],
            standard: "aus",
        },
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "gehaeuse.zoom",
        gruppe: Gruppe::Gehaeuse,
        name: Texte::de_en("Nähe zum Bildschirm", "Closeness"),
        beschreibung: Texte::de_en("0 zeigt das ganze Gehäuse, 1 rückt so nah heran, dass das Bild die Fensterhöhe füllt und das Gehäuse nur noch am Rand zu sehen ist.", "0 shows the whole cabinet, 1 moves so close that the picture fills the window height and the cabinet is only visible at the edge."),
        art: zahl(0.0, 1.0, 0.05, 0.3),
        kosten: Kosten::Keine,
        wirkt_wenn: Some(("gehaeuse.typ", &["fernseher", "monitor", "automat"])),
    },
    OptionBeschreibung {
        schluessel: "gehaeuse.spiegelung",
        gruppe: Gruppe::Gehaeuse,
        name: Texte::de_en("Spiegelung im Rahmen", "Reflection in the bezel"),
        beschreibung: Texte::de_en("Wie stark sich das Bild unscharf in der Innenkante des Rahmens spiegelt.", "How strongly the picture is reflected, blurred, in the inner edge of the bezel."),
        art: zahl(0.0, 1.0, 0.05, 0.5),
        kosten: Kosten::Gering,
        wirkt_wenn: Some(("gehaeuse.typ", &["fernseher", "monitor", "automat"])),
    },
    OptionBeschreibung {
        schluessel: "gehaeuse.umgebungslicht",
        gruppe: Gruppe::Gehaeuse,
        name: Texte::de_en("Licht des Bildes im Raum", "Screen light in the room"),
        beschreibung: Texte::de_en("Wie stark das Bild Rahmen und Wand in seiner Farbe beleuchtet.", "How strongly the picture lights bezel and wall in its colour."),
        art: zahl(0.0, 1.0, 0.05, 0.5),
        kosten: Kosten::Gering,
        wirkt_wenn: Some(("gehaeuse.typ", &["fernseher", "monitor", "automat"])),
    },
    OptionBeschreibung {
        schluessel: "gehaeuse.raumlicht",
        gruppe: Gruppe::Gehaeuse,
        name: Texte::de_en("Raumhelligkeit", "Room light"),
        beschreibung: Texte::de_en("Grundhelligkeit des Raums: 0 ist ein dunkles Zimmer, in dem nur das Bild leuchtet.", "Base brightness of the room: 0 is a dark room where only the picture glows."),
        art: zahl(0.0, 1.0, 0.05, 0.3),
        kosten: Kosten::Keine,
        wirkt_wenn: Some(("gehaeuse.typ", &["fernseher", "monitor", "automat"])),
    },
    OptionBeschreibung {
        schluessel: "gehaeuse.glanz",
        gruppe: Gruppe::Gehaeuse,
        name: Texte::de_en("Glanz auf dem Glas", "Glass glare"),
        beschreibung: Texte::de_en("Schwacher Widerschein einer Lichtquelle im Raum auf dem gewölbten Glas.", "Faint reflection of a room light on the curved glass."),
        art: zahl(0.0, 1.0, 0.05, 0.4),
        kosten: Kosten::Keine,
        wirkt_wenn: Some(("gehaeuse.typ", &["nur_glas", "fernseher", "monitor", "automat"])),
    },
    // ---------------------------------------------------------------- Bewegung
    OptionBeschreibung {
        schluessel: "bewegung.modus",
        gruppe: Gruppe::Bewegung,
        name: Texte::de_en("Bewegungsschärfe", "Motion clarity"),
        beschreibung: Texte::de_en("Nur für Bildschirme mit mehr Hertz als das Spiel (120, 240, 480 Hz). Eine Röhre zeigt jeden Bildpunkt nur kurz, deshalb wirken Bewegungen auf ihr scharf; ein gewöhnlicher Bildschirm hält das Bild und verwischt sie. „Strahl“ bildet den wandernden Elektronenstrahl samt Nachleuchten nach (Blur Busters CRT Beam Simulator), „Schwarzbilder“ schiebt einfach dunkle Bilder ein. Je mehr Hertz, desto besser; auf manchen LCDs kann es flimmern.", "Only for displays with more hertz than the game (120, 240, 480 Hz). A CRT shows each dot only briefly, so motion looks sharp on it; an ordinary display holds the picture and blurs it. “Beam” simulates the moving electron beam with afterglow (Blur Busters CRT Beam Simulator), “black frames” simply inserts dark frames. The more hertz, the better; some LCDs may flicker."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("aus", Texte::de_en("Aus", "Off")),
                ("strahl", Texte::de_en("Strahl (empfohlen)", "Beam (recommended)")),
                ("schwarzbilder", Texte::de_en("Schwarzbilder", "Black frames")),
            ],
            standard: "aus",
        },
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "bewegung.helligkeit",
        gruppe: Gruppe::Bewegung,
        name: Texte::de_en("Helligkeit gegen Schärfe", "Brightness vs. clarity"),
        beschreibung: Texte::de_en("Höher heißt heller, aber helle Bildteile leuchten länger nach und verwischen etwas mehr. Empfohlen: 0,7 ab 240 Hz, 0,5 bei 120 Hz.", "Higher is brighter, but bright areas glow longer and blur slightly more. Recommended: 0.7 from 240 Hz, 0.5 at 120 Hz."),
        art: zahl(0.1, 1.0, 0.025, 0.7),
        kosten: Kosten::Keine,
        wirkt_wenn: Some(("bewegung.modus", &["strahl"])),
    },
    OptionBeschreibung {
        schluessel: "bewegung.lcd_schutz",
        gruppe: Gruppe::Bewegung,
        name: Texte::de_en("Schutz für LCD-Bildschirme", "LCD protection"),
        beschreibung: Texte::de_en("Verschiebt den Strahl langsam, damit sich auf LCD-Bildschirmen bei geraden Verhältnissen (120 zu 60, 240 zu 60) kein Muster einbrennt. Bei OLED nicht nötig.", "Slowly shifts the beam so that LCDs do not burn in a pattern at even ratios (120 to 60, 240 to 60). Not needed on OLED."),
        art: OptionsArt::Schalter { standard: true },
        kosten: Kosten::Keine,
        wirkt_wenn: Some(("bewegung.modus", &["strahl", "schwarzbilder"])),
    },
    // ---------------------------------------------------------------- 3D-Raum
    OptionBeschreibung {
        schluessel: "raum.naehe",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Nähe", "Distance"),
        beschreibung: Texte::de_en("Wie groß der Fernseher im Fenster erscheint. Weit (0): viel vom Raum; Mittel (0,5): etwas vom Tisch; Nah (0,75): das Gerät füllt das Fenster weitgehend, oben und unten bleibt ein Rand; Ganz nah (1): der Bildschirm füllt das Fenster fast, oben und unten ist nur noch das Gehäuse zu sehen. Bei schmalen Fenstern wird so weit zurückgenommen, dass es in der Breite passt.", "How large the TV appears in the window. Wide (0): much of the room; medium (0.5): some of the table; close (0.75): the set mostly fills the window with a margin at top and bottom; very close (1): the screen almost fills the window, only the cabinet shows at top and bottom. In narrow windows it pulls back so it fits the width."),
        art: zahl(0.0, 1.0, 0.01, 0.75),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.blickwinkel",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Blickwinkel", "Viewing angle"),
        beschreibung: Texte::de_en("Von wo die Kamera auf den Fernseher schaut: −1 deutlich von links (30°), −0,5 leicht von links, 0 von vorn, 0,5 leicht von rechts, 1 deutlich von rechts.", "Where the camera looks at the TV from: −1 clearly from the left (30°), −0.5 slightly from the left, 0 from the front, 0.5 slightly from the right, 1 clearly from the right."),
        art: zahl(-1.0, 1.0, 0.01, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.schwenk",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Kameraschwenk", "Camera sway"),
        beschreibung: Texte::de_en("Aus: die Kamera steht völlig still. An: eine ganz leichte, langsame Bewegung nach links und rechts; die Kamera bleibt auf den Fernseher gerichtet, er bleibt an seiner Stelle im Fenster, nur Perspektive und Spiegelungen im Glas ändern sich.", "Off: the camera stands completely still. On: a very slight, slow movement left and right; the camera stays aimed at the TV, which stays in place in the window, only perspective and reflections in the glass change."),
        art: OptionsArt::Auswahl {
            werte: &[("aus", Texte::de_en("Aus", "Off")), ("an", Texte::de_en("An", "On"))],
            standard: "an",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.schwenk_staerke",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Stärke des Schwenks", "Sway strength"),
        beschreibung: Texte::de_en("Wie weit die Kamera beim Schwenk ausweicht (1 = etwa 6° zu jeder Seite).", "How far the camera moves when swaying (1 = about 6° to each side)."),
        art: zahl(0.0, 1.0, 0.01, 0.35),
        kosten: Kosten::Keine,
        wirkt_wenn: Some(("raum.schwenk", &["an"])),
    },
    OptionBeschreibung {
        schluessel: "raum.lage",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Lage des Fernsehers im Fenster", "TV position in the window"),
        beschreibung: Texte::de_en("Verschiebt den Fernseher nach links (−1, bis an den Fensterrand) oder rechts (1), z. B. um beim Streamen daneben Platz für das Webcam-Bild zu lassen. Der Bildausschnitt wird verschoben, nicht die Kamera gedreht: Blickwinkel und Form des Geräts bleiben gleich, abgeschnitten wird nichts.", "Moves the TV to the left (−1, up to the window edge) or right (1), e.g. to leave room for a webcam picture when streaming. The view is shifted, not the camera rotated: viewing angle and shape of the set stay the same, nothing is cut off."),
        art: zahl(-1.0, 1.0, 0.01, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.licht",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Licht der Stehlampe", "Floor lamp"),
        beschreibung: Texte::de_en("Helligkeit der Stehlampe in der Zimmerecke; 0 = nur der Fernseher leuchtet.", "Brightness of the floor lamp in the corner; 0 = only the TV glows."),
        art: zahl(0.0, 2.0, 0.05, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.bildhelligkeit",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Helligkeit des Bildes", "Picture brightness"),
        beschreibung: Texte::de_en(
            "Wie hell das Bild auf dem Fernseher leuchtet. 1 entspricht dem Röhrenbild ohne Raum: helle Szenen sind hell, behalten aber ihre Zeichnung. Höher wird greller (Weiß frisst langsam aus), niedriger gedämpfter wie ein heruntergeregelter Fernseher.",
            "How bright the picture on the TV glows. 1 matches the CRT picture without the room: bright scenes are bright but keep their detail. Higher is harsher (white slowly burns out), lower is dimmer like a TV turned down.",
        ),
        art: zahl(0.5, 1.5, 0.05, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.flackern",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Flackern der Lampe", "Lamp flicker"),
        beschreibung: Texte::de_en("Leichtes, unregelmäßiges Flackern der Glühbirne.", "Slight, irregular flicker of the light bulb."),
        art: zahl(0.0, 1.0, 0.05, 0.4),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.korn",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Filmkorn", "Film grain"),
        beschreibung: Texte::de_en("Körnung über dem ganzen Bild, im Dunkeln stärker.", "Grain over the whole picture, stronger in the dark."),
        art: zahl(0.0, 1.0, 0.05, 0.35),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.rohpixel",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Rohpixel-Texturen", "Raw-pixel textures"),
        beschreibung: Texte::de_en(
            "An: Die Texturen des Raums zeigen aus der Nähe ihre einzelnen Pixel, wie zur PS2-Zeit. Aus: Die Texturen werden geglättet, Holz, Gehäuse und Wände wirken ruhiger.",
            "On: the room's textures show their individual pixels up close, as in the PS2 era. Off: textures are smoothed, wood, cabinet and walls look calmer.",
        ),
        art: OptionsArt::Schalter { standard: true },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.vignette",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Vignette", "Vignette"),
        beschreibung: Texte::de_en(
            "Dunkelt die Ecken des Bilds ab wie bei einer Kamera. 0 = aus. Zusammen mit Filmkorn 0 und Glühen 0 liegt über dem Raum nur noch die Tiefenunschärfe.",
            "Darkens the corners of the picture as with a camera. 0 = off. Together with film grain 0 and glow 0, only the depth of field remains over the room.",
        ),
        art: zahl(0.0, 1.0, 0.05, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.unschaerfe",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Tiefenunschärfe", "Depth of field"),
        beschreibung: Texte::de_en("Der Fernseher bleibt scharf, was davor und dahinter liegt, verschwimmt leicht wie bei einer Kamera. 0 = alles scharf.", "The TV stays sharp; what lies in front of and behind it blurs slightly, as with a camera. 0 = everything sharp."),
        art: zahl(0.0, 1.0, 0.05, 0.3),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.gluehen",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Glühen", "Glow"),
        beschreibung: Texte::de_en("Helle Stellen wie Bildschirm und Lampe strahlen leicht in den Raum ab.", "Bright spots like the screen and the lamp glow slightly into the room."),
        art: zahl(0.0, 1.0, 0.05, 0.5),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "raum.leben",
        gruppe: Gruppe::Raum,
        name: Texte::de_en("Leben der Szene", "Scene life"),
        beschreibung: Texte::de_en(
            "Kleine Veränderungen draußen und im Raum: Wolken ziehen am Mond vorbei und dimmen sein Licht, ab und zu fährt ein Auto vorbei und sein Licht wandert durchs Zimmer, die Bäume im Park wiegen sich. 0 = still.",
            "Small changes outside and in the room: clouds drift past the moon and dim its light, now and then a car passes and its light moves through the room, the trees in the park sway. 0 = still.",
        ),
        art: zahl(0.0, 1.0, 0.05, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
];

/// Benannte Festwerte einer stufenlosen Option (für Menüs mit Stufen): Schlüssel, Name, Wert.
/// Stufen und Regler stellen dieselbe Option ein.
pub fn festwerte(schluessel: &str) -> &'static [(&'static str, Texte, f64)] {
    static NAEHE: [(&str, Texte, f64); 4] = [
        ("weit", Texte::de_en("Weit", "Wide"), 0.0),
        ("mittel", Texte::de_en("Mittel", "Medium"), 0.5),
        ("nah", Texte::de_en("Nah", "Close"), 0.75),
        ("ganz_nah", Texte::de_en("Ganz nah", "Very close"), 1.0),
    ];
    static BLICK: [(&str, Texte, f64); 3] = [
        ("links", Texte::de_en("Leicht von links", "Slightly from the left"), -0.5),
        ("vorn", Texte::de_en("Von vorn", "Front"), 0.0),
        ("rechts", Texte::de_en("Leicht von rechts", "Slightly from the right"), 0.5),
    ];
    static LAGE: [(&str, Texte, f64); 3] = [
        ("links", Texte::de_en("Links", "Left"), -1.0),
        ("mitte", Texte::de_en("Mitte", "Centre"), 0.0),
        ("rechts", Texte::de_en("Rechts", "Right"), 1.0),
    ];
    match schluessel {
        "raum.naehe" => &NAEHE,
        "raum.blickwinkel" => &BLICK,
        "raum.lage" => &LAGE,
        _ => &[],
    }
}

/// Alle Optionen in Menüreihenfolge.
pub fn katalog() -> &'static [OptionBeschreibung] {
    KATALOG
}

/// Beschreibung einer Option nach Schlüssel.
pub fn option(schluessel: &str) -> Option<&'static OptionBeschreibung> {
    KATALOG.iter().find(|o| o.schluessel == schluessel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn alle_texte_haben_eine_englische_fassung() {
        for o in katalog() {
            assert!(o.name.en.is_some(), "{}: Name ohne Englisch", o.schluessel);
            assert!(o.beschreibung.en.is_some(), "{}: Beschreibung ohne Englisch", o.schluessel);
            if let OptionsArt::Auswahl { werte, .. } = o.art {
                for (k, t) in werte {
                    assert!(t.en.is_some(), "{} = {k}: ohne Englisch", o.schluessel);
                }
            }
        }
        for g in Gruppe::ALLE {
            assert!(g.name().en.is_some(), "Gruppe {} ohne Englisch", g.schluessel());
        }
    }

    #[test]
    fn schluessel_eindeutig_und_zur_gruppe_passend() {
        let mut gesehen = HashSet::new();
        for o in katalog() {
            assert!(gesehen.insert(o.schluessel), "doppelt: {}", o.schluessel);
            let (gruppe, _) = o.schluessel.split_once('.').expect("Schlüssel ohne Punkt");
            assert_eq!(gruppe, o.gruppe.schluessel(), "{}", o.schluessel);
        }
    }

    /// docs/ki/STAND.md ist die Übersicht für andere KIs; sie darf bei der Zahl der Optionen
    /// und Gruppen nicht hinter dem Katalog zurückbleiben.
    #[test]
    fn stand_md_nennt_die_richtige_zahl_der_optionen() {
        let stand = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/ki/STAND.md"))
            .expect("docs/ki/STAND.md fehlt");
        let erwartet = format!("({} Optionen, {} Gruppen", katalog().len(), Gruppe::ALLE.len());
        assert!(stand.contains(&erwartet), "docs/ki/STAND.md muss „{erwartet}“ nennen – bitte aktualisieren");
    }

    #[test]
    fn standardwerte_liegen_in_den_grenzen() {
        for o in katalog() {
            match o.art {
                OptionsArt::Zahl { min, max, standard, schritt, .. } => {
                    assert!(min <= standard && standard <= max, "{}", o.schluessel);
                    assert!(schritt > 0.0);
                }
                OptionsArt::Ganzzahl { min, max, standard } => assert!(min <= standard && standard <= max),
                OptionsArt::Auswahl { werte, standard } => {
                    assert!(werte.iter().any(|(w, _)| *w == standard), "{}", o.schluessel)
                }
                OptionsArt::Schalter { .. } => {}
            }
            if let Some((bedingung, werte)) = o.wirkt_wenn {
                let b = option(bedingung).expect("Bedingung verweist auf unbekannte Option");
                let OptionsArt::Auswahl { werte: erlaubt, .. } = b.art else { panic!("Bedingung keine Auswahl") };
                for w in werte {
                    assert!(erlaubt.iter().any(|(e, _)| e == w), "{} → {}", o.schluessel, w);
                }
            }
        }
    }
}
