//! # roehre-modell
//!
//! Das Einstellungsmodell von `roehre`. Es kennt jede Option mit Schlüssel, Gruppe,
//! Namen, Beschreibung, Grenzen und Standardwert ([`katalog`]), lädt und speichert
//! Einstellungen als TOML oder JSON ([`Einstellungen`]) und liefert die benannten
//! Voreinstellungen ([`voreinstellungen`]).
//!
//! Das Modell hat keine Abhängigkeit zur GPU. Die Spiele nutzen es, um ihr eigenes Menü
//! zu bauen: Die Bibliothek liefert Namen, Beschreibungen und Gruppen, aber keine
//! Oberfläche.

mod einstellungen;
pub mod katalog;
mod texte;
pub mod voreinstellungen;
mod wert;

pub use einstellungen::{Einstellungen, Fehler, Hinweis, AKTUELLE_VERSION};
pub use katalog::{katalog, option, Gruppe, Kosten, OptionBeschreibung, OptionsArt};
pub use texte::{Sprache, Texte};
pub use voreinstellungen::{voreinstellung, voreinstellungen, Lizenzmerkmal, Voreinstellung};
pub use wert::Wert;
