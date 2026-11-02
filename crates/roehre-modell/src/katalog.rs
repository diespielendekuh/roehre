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
}

impl Gruppe {
    pub const ALLE: [Gruppe; 9] = [
        Gruppe::Allgemein,
        Gruppe::Signal,
        Gruppe::Strahl,
        Gruppe::Maske,
        Gruppe::Leuchtstoff,
        Gruppe::Licht,
        Gruppe::Geometrie,
        Gruppe::Helligkeit,
        Gruppe::Bildformat,
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
        beschreibung: Texte::de("Schaltet die ganze Röhre ein oder aus. Aus heißt: Das Bild wird nur sauber und unverzerrt eingepasst."),
        art: OptionsArt::Schalter { standard: true },
        kosten: Kosten::Hoch,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "allgemein.leistung",
        gruppe: Gruppe::Allgemein,
        name: Texte::de_en("Leistungsstufe", "Performance level"),
        beschreibung: Texte::de("„Voll“ ist die genaueste Darstellung. „Leicht“ spart Rechenzeit für Laptops und schwache Grafikkarten, mit etwas weniger Licht im Glas."),
        art: OptionsArt::Auswahl {
            werte: &[("voll", Texte::de_en("Voll", "Full")), ("leicht", Texte::de_en("Leicht", "Light"))],
            standard: "voll",
        },
        kosten: Kosten::Hoch,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Signal
    OptionBeschreibung {
        schluessel: "signal.art",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Anschluss", "Connection"),
        beschreibung: Texte::de("Wie das Bild zum Fernseher kommt. RGB ist am saubersten, Composite und Antenne mischen Helligkeit und Farbe in ein Signal und erzeugen Farbsäume und Kantenflimmern."),
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
        beschreibung: Texte::de("NTSC (Japan, USA) zeigt bei Composite wandernde Kantenmuster (Dot-Crawl). PAL (Europa) mittelt die Farbe über zwei Zeilen und wirkt dadurch ruhiger, aber farblich weicher."),
        art: OptionsArt::Auswahl {
            werte: &[("ntsc", Texte::de("NTSC")), ("pal", Texte::de("PAL"))],
            standard: "ntsc",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.artefakte",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Farbartefakte", "Colour artifacts"),
        beschreibung: Texte::de("Stärke der Regenbogen- und Musterbildung an feinen Details, wenn Helligkeit als Farbe missverstanden wird."),
        art: zahl(0.0, 3.0, 0.1, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.farbsaum",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Farbsäume", "Colour fringing"),
        beschreibung: Texte::de("Farbe, die an Kanten in die Nachbarschaft ausläuft."),
        art: zahl(0.0, 3.0, 0.1, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.saettigung",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Farbsättigung des Signals", "Signal saturation"),
        beschreibung: Texte::de("Farbstärke nach dem Dekodieren des Signals."),
        art: zahl(0.0, 2.0, 0.01, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: NICHT_RGB,
    },
    OptionBeschreibung {
        schluessel: "signal.rauschen",
        gruppe: Gruppe::Signal,
        name: Texte::de_en("Rauschen", "Noise"),
        beschreibung: Texte::de("Bildrauschen eines schwachen Signals, etwa über Antenne oder ein langes Kabel."),
        art: zahl(0.0, 0.5, 0.01, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Strahl
    OptionBeschreibung {
        schluessel: "strahl.form_dunkel",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Zeilen bei dunklen Bildteilen", "Scanlines in dark areas"),
        beschreibung: Texte::de("Wie schmal der Strahl bei wenig Helligkeit ist. Höher heißt schmalere Zeilen mit deutlicheren Lücken."),
        art: zahl(0.5, 4.0, 0.05, 1.3),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.form_hell",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Zeilen bei hellen Bildteilen", "Scanlines in bright areas"),
        beschreibung: Texte::de("Wie schmal der Strahl bei voller Helligkeit ist. Echte Röhren haben bei hellen Bildteilen breitere, fast verschmelzende Zeilen."),
        art: zahl(0.5, 2.5, 0.025, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.aufblaehen",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Breitere helle Zeilen", "Wider bright scanlines"),
        beschreibung: Texte::de("Wie stark der Strahl bei hellen Bildteilen zusätzlich anschwillt."),
        art: zahl(0.0, 1.0, 0.05, 0.6),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.zeilenfarbe",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Farbsättigung der Zeilenränder", "Scanline edge saturation"),
        beschreibung: Texte::de("Wie kräftig die Farbe zum Rand jeder Zeile wird."),
        art: zahl(0.0, 3.0, 0.1, 0.5),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.schaerfe",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Waagerechte Schärfe", "Horizontal sharpness"),
        beschreibung: Texte::de("Wie scharf der Strahl entlang einer Zeile zeichnet."),
        art: zahl(0.5, 10.0, 0.1, 5.2),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.zeilensprung",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Zeilensprung (480i/576i)", "Interlacing"),
        beschreibung: Texte::de("Wie Bilder mit hoher Zeilenzahl dargestellt werden: als Halbbilder wie auf der Röhre oder geglättet."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("aus", Texte::de("Aus")),
                ("halbbilder", Texte::de("Halbbilder")),
                ("geglaettet", Texte::de("Geglättet")),
            ],
            standard: "halbbilder",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_rot",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Rot (waagerecht)", "Red convergence (horizontal)"),
        beschreibung: Texte::de("Versatz des roten Strahls gegenüber Grün, in Ausgabepixeln."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_blau",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Blau (waagerecht)", "Blue convergence (horizontal)"),
        beschreibung: Texte::de("Versatz des blauen Strahls gegenüber Grün, in Ausgabepixeln."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_rot_y",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Rot (senkrecht)", "Red convergence (vertical)"),
        beschreibung: Texte::de("Senkrechter Versatz des roten Strahls, in Ausgabepixeln."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_blau_y",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler Blau (senkrecht)", "Blue convergence (vertical)"),
        beschreibung: Texte::de("Senkrechter Versatz des blauen Strahls, in Ausgabepixeln."),
        art: zahl_einheit(-3.0, 3.0, 0.25, 0.0, "px"),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "strahl.konvergenz_rand",
        gruppe: Gruppe::Strahl,
        name: Texte::de_en("Konvergenzfehler zum Rand", "Convergence error towards edges"),
        beschreibung: Texte::de("0 heißt: überall gleich. Höhere Werte lassen den Fehler zur Mitte hin verschwinden und zum Rand wachsen, wie bei einer echten Röhre."),
        art: zahl(0.0, 0.75, 0.05, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Maske
    OptionBeschreibung {
        schluessel: "maske.typ",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Maskentyp", "Mask type"),
        beschreibung: Texte::de("Die Anordnung der roten, grünen und blauen Leuchtpunkte. Streifenmaske: senkrechte Streifen (Trinitron, PVM). Schlitzmaske: Streifen mit versetzten Unterbrechungen (die meisten Fernseher). Lochmaske: versetzte Punkte (PC-Monitore, Arcade)."),
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
        beschreibung: Texte::de("Wie deutlich die Maske in hellen Bildteilen zu sehen ist."),
        art: zahl(0.0, 1.0, 0.025, 0.3),
        kosten: Kosten::Keine,
        wirkt_wenn: MIT_MASKE,
    },
    OptionBeschreibung {
        schluessel: "maske.staerke_dunkel",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Maskenstärke dunkel", "Mask strength (dark)"),
        beschreibung: Texte::de("Wie deutlich die Maske in dunklen Bildteilen zu sehen ist."),
        art: zahl(0.0, 1.5, 0.05, 1.1),
        kosten: Kosten::Keine,
        wirkt_wenn: MIT_MASKE,
    },
    OptionBeschreibung {
        schluessel: "maske.triaden",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Feinheit der Maske", "Mask density"),
        beschreibung: Texte::de("Gewünschte Zahl der Farbtripel über die Bildbreite. Fernseher haben etwa 300 bis 500, Profi-Monitore 600 bis 1000. Die Maske wird immer auf ganze Bildschirmpixel gelegt, damit kein Moiré entsteht; der tatsächliche Wert hängt deshalb von der Fenstergröße ab."),
        art: OptionsArt::Ganzzahl { min: 150, max: 1200, standard: 450 },
        kosten: Kosten::Keine,
        wirkt_wenn: MIT_MASKE,
    },
    OptionBeschreibung {
        schluessel: "maske.subpixel",
        gruppe: Gruppe::Maske,
        name: Texte::de_en("Subpixel-Reihenfolge des Bildschirms", "Display subpixel order"),
        beschreibung: Texte::de("Reihenfolge der Farbpunkte im eigenen Bildschirm. Die meisten sind RGB; manche Fernseher und Monitore sind BGR."),
        art: OptionsArt::Auswahl {
            werte: &[("rgb", Texte::de("RGB")), ("bgr", Texte::de("BGR"))],
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
        beschreibung: Texte::de("Die Grundfarben der Röhre. EBU war in Europa üblich, SMPTE-C in den USA, P22 bei vielen Fernsehern und Monitoren."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("aus", Texte::de_en("Unverändert (sRGB)", "Unchanged (sRGB)")),
                ("ebu", Texte::de("EBU")),
                ("p22", Texte::de("P22")),
                ("smpte_c", Texte::de("SMPTE-C")),
                ("philips", Texte::de("Philips")),
                ("trinitron", Texte::de("Trinitron")),
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
        beschreibung: Texte::de("Negative Werte machen das Weiß wärmer (gelblicher), positive kühler (bläulicher). Japanische Fernseher waren oft deutlich kühler eingestellt."),
        art: zahl_einheit(-100.0, 100.0, 5.0, 0.0, "%"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "leuchtstoff.saettigung",
        gruppe: Gruppe::Leuchtstoff,
        name: Texte::de_en("Farbsättigung", "Saturation"),
        beschreibung: Texte::de("Gesamte Farbstärke des Bildes."),
        art: zahl(0.0, 2.0, 0.05, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "leuchtstoff.nachgluehen",
        gruppe: Gruppe::Leuchtstoff,
        name: Texte::de_en("Nachglühen", "Afterglow"),
        beschreibung: Texte::de("Wie lange ein heller Punkt nach dem Abschalten noch schwach leuchtet. Sichtbar als kurze Schweife bei schneller Bewegung auf dunklem Grund."),
        art: zahl(0.0, 0.6, 0.01, 0.2),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Licht
    OptionBeschreibung {
        schluessel: "licht.glow",
        gruppe: Gruppe::Licht,
        name: Texte::de_en("Glühen", "Glow"),
        beschreibung: Texte::de("Weiches Leuchten um helle Bildteile, aus der Streuung im Leuchtstoff."),
        art: zahl(0.0, 1.0, 0.01, 0.08),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "licht.bloom",
        gruppe: Gruppe::Licht,
        name: Texte::de_en("Überstrahlen", "Bloom"),
        beschreibung: Texte::de("Helle Bildteile strahlen in ihre Umgebung und füllen die Lücken zwischen den Zeilen."),
        art: zahl(0.0, 1.0, 0.05, 0.0),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "licht.halation",
        gruppe: Gruppe::Licht,
        name: Texte::de_en("Lichthof (Halation)", "Halation"),
        beschreibung: Texte::de("Licht, das im dicken Frontglas der Röhre zurückgeworfen wird und einen weiten, schwachen Hof um helle Flächen legt."),
        art: zahl(0.0, 1.0, 0.025, 0.0),
        kosten: Kosten::Gering,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Geometrie
    OptionBeschreibung {
        schluessel: "geometrie.woelbung_x",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Wölbung waagerecht", "Horizontal curvature"),
        beschreibung: Texte::de("0 ist flach. Fernseher der 80er und 90er liegen etwa bei 0,02 bis 0,05."),
        art: zahl(0.0, 0.15, 0.005, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.woelbung_y",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Wölbung senkrecht", "Vertical curvature"),
        beschreibung: Texte::de("0 ist flach. Bei Streifenmasken-Röhren (Trinitron) ist die Röhre senkrecht flach."),
        art: zahl(0.0, 0.15, 0.005, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.ecken",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Abgerundete Ecken", "Rounded corners"),
        beschreibung: Texte::de("Radius der Bildecken."),
        art: zahl(0.0, 0.15, 0.005, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.randweich",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Weicher Bildrand", "Soft picture edge"),
        beschreibung: Texte::de("Breite des weichen Übergangs am Bildrand."),
        art: zahl(0.0, 1.0, 0.01, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.vignette",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Randabdunklung", "Vignette"),
        beschreibung: Texte::de("Wie stark das Bild zu den Rändern dunkler wird."),
        art: zahl(0.0, 1.0, 0.05, 0.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "geometrie.overscan",
        gruppe: Gruppe::Geometrie,
        name: Texte::de_en("Überstand (Overscan)", "Overscan"),
        beschreibung: Texte::de("Anteil des Bildes, der wie bei einem Fernseher hinter dem Rahmen verschwindet, in Prozent je Seite."),
        art: zahl_einheit(0.0, 8.0, 0.5, 0.0, "%"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Helligkeit
    OptionBeschreibung {
        schluessel: "helligkeit.ausgleich_dunkel",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Ausgleich dunkle Bildteile", "Compensation dark areas"),
        beschreibung: Texte::de("Hebt dunkle Bildteile an, damit Maske und Zeilen das Bild nicht verdunkeln."),
        art: zahl(0.5, 4.0, 0.05, 1.4),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "helligkeit.ausgleich_hell",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Ausgleich helle Bildteile", "Compensation bright areas"),
        beschreibung: Texte::de("Hebt helle Bildteile an."),
        art: zahl(0.5, 3.0, 0.025, 1.1),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "helligkeit.nachher",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Gesamthelligkeit", "Overall brightness"),
        beschreibung: Texte::de("Helligkeit am Ende der Röhre."),
        art: zahl(0.5, 3.0, 0.01, 1.0),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "helligkeit.gamma_ausgang",
        gruppe: Gruppe::Helligkeit,
        name: Texte::de_en("Gamma des Bildschirms", "Display gamma"),
        beschreibung: Texte::de("Gamma, mit dem das Ergebnis für den eigenen Bildschirm kodiert wird."),
        art: zahl(1.5, 3.0, 0.05, 1.95),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    // ---------------------------------------------------------------- Bildformat
    OptionBeschreibung {
        schluessel: "bildformat.seitenverhaeltnis",
        gruppe: Gruppe::Bildformat,
        name: Texte::de_en("Seitenverhältnis", "Aspect ratio"),
        beschreibung: Texte::de("4:3 wie auf einer Röhre. „Pixelgenau“ nutzt das Seitenverhältnis der Pixel aus dem Spiel (bei SNES 8:7). 16:9 entspricht einer späten Breitbild-Röhre. „Ganzes Fenster“ füllt das Fenster und verzerrt dabei."),
        art: OptionsArt::Auswahl {
            werte: &[
                ("4:3", Texte::de("4:3")),
                ("pixelgenau", Texte::de_en("Pixelgenau", "Pixel exact")),
                ("16:9", Texte::de("16:9")),
                ("fenster", Texte::de_en("Ganzes Fenster (verzerrt)", "Whole window (stretched)")),
            ],
            standard: "4:3",
        },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "bildformat.ganzzahlig",
        gruppe: Gruppe::Bildformat,
        name: Texte::de_en("Ganzzahlige Skalierung", "Integer scaling"),
        beschreibung: Texte::de("Jede Bildzeile bekommt gleich viele Bildschirmzeilen. Das Bild wird dafür etwas kleiner, Zeilen wirken aber völlig gleichmäßig."),
        art: OptionsArt::Schalter { standard: false },
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
    OptionBeschreibung {
        schluessel: "bildformat.fuellung",
        gruppe: Gruppe::Bildformat,
        name: Texte::de_en("Bildgröße", "Picture size"),
        beschreibung: Texte::de("Anteil der Fensterhöhe, den das Bild höchstens einnimmt."),
        art: zahl_einheit(50.0, 100.0, 1.0, 100.0, "%"),
        kosten: Kosten::Keine,
        wirkt_wenn: None,
    },
];

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
    fn schluessel_eindeutig_und_zur_gruppe_passend() {
        let mut gesehen = HashSet::new();
        for o in katalog() {
            assert!(gesehen.insert(o.schluessel), "doppelt: {}", o.schluessel);
            let (gruppe, _) = o.schluessel.split_once('.').expect("Schlüssel ohne Punkt");
            assert_eq!(gruppe, o.gruppe.schluessel(), "{}", o.schluessel);
        }
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
