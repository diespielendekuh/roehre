//! Die benannten Voreinstellungen.
//!
//! Jede Voreinstellung ist eine TOML-Datei unter `voreinstellungen/`, die ins Programm
//! eingebettet wird. Sie enthält nur die Werte, die vom Standard abweichen, und einen
//! `[meta]`-Block mit Name, Beschreibung und Lizenzmerkmal der verwendeten Shader.

/// Eine benannte Voreinstellung.
#[derive(Clone, Copy, Debug)]
pub struct Voreinstellung {
    /// Fester Schlüssel, z. B. `scart-rgb`.
    pub schluessel: &'static str,
    /// Inhalt der TOML-Datei.
    pub toml: &'static str,
}

/// Lizenzmerkmal der Shader, die eine Voreinstellung benutzt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lizenzmerkmal {
    /// Nur eigene und MIT-lizenzierte Shader.
    Permissiv,
    /// Nutzt GPL-Shader (werden als eigene Dateien ausgeliefert).
    Gpl,
}

impl Voreinstellung {
    fn meta(&self, feld: &str) -> String {
        let t: toml::Table = toml::from_str(self.toml).expect("Voreinstellung ist gültiges TOML");
        t.get("meta")
            .and_then(|m| m.get(feld))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }

    /// Anzeigename (deutsch).
    pub fn name(&self) -> String {
        self.meta("name")
    }

    /// Beschreibung (deutsch).
    pub fn beschreibung(&self) -> String {
        self.meta("beschreibung")
    }

    pub fn lizenz(&self) -> Lizenzmerkmal {
        match self.meta("lizenz").as_str() {
            "permissiv" => Lizenzmerkmal::Permissiv,
            _ => Lizenzmerkmal::Gpl,
        }
    }
}

macro_rules! voreinstellungen {
    ($($name:literal),* $(,)?) => {
        &[$(Voreinstellung { schluessel: $name, toml: include_str!(concat!("../voreinstellungen/", $name, ".toml")) }),*]
    };
}

static ALLE: &[Voreinstellung] = voreinstellungen![
    "aus",
    "scart-rgb",
    "wohnzimmer-90er-composite",
];

/// Alle Voreinstellungen in Menüreihenfolge.
pub fn voreinstellungen() -> &'static [Voreinstellung] {
    ALLE
}

/// Voreinstellung nach Schlüssel.
pub fn voreinstellung(schluessel: &str) -> Option<&'static Voreinstellung> {
    ALLE.iter().find(|v| v.schluessel == schluessel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Einstellungen;

    #[test]
    fn alle_voreinstellungen_laden_ohne_hinweise() {
        for v in voreinstellungen() {
            let (_, hinweise) = Einstellungen::aus_toml(v.toml).unwrap();
            assert!(hinweise.is_empty(), "{}: {hinweise:?}", v.schluessel);
            assert!(!v.name().is_empty(), "{} ohne Namen", v.schluessel);
            assert!(!v.beschreibung().is_empty(), "{} ohne Beschreibung", v.schluessel);
        }
    }
}
