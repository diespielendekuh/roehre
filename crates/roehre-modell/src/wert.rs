//! Ein einzelner Einstellungswert.

use serde::{Deserialize, Serialize};

/// Wert einer Option. In TOML/JSON erscheint er als Zahl, Wahrheitswert oder Text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Wert {
    Schalter(bool),
    Ganzzahl(i64),
    Zahl(f64),
    Auswahl(String),
}

impl Wert {
    /// Zahlenwert, auch aus einer Ganzzahl.
    pub fn als_zahl(&self) -> Option<f64> {
        match self {
            Wert::Zahl(z) => Some(*z),
            Wert::Ganzzahl(g) => Some(*g as f64),
            _ => None,
        }
    }

    pub fn als_schalter(&self) -> Option<bool> {
        match self {
            Wert::Schalter(s) => Some(*s),
            _ => None,
        }
    }

    pub fn als_auswahl(&self) -> Option<&str> {
        match self {
            Wert::Auswahl(a) => Some(a),
            _ => None,
        }
    }
}

impl std::fmt::Display for Wert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wert::Schalter(s) => write!(f, "{}", if *s { "an" } else { "aus" }),
            Wert::Ganzzahl(g) => write!(f, "{g}"),
            Wert::Zahl(z) => write!(f, "{z}"),
            Wert::Auswahl(a) => write!(f, "{a}"),
        }
    }
}
