//! Einstellungen: laden, prüfen, speichern.
//!
//! Dateiformat (TOML; JSON hat dieselbe Struktur):
//!
//! ```toml
//! version = 1
//! basis = "scart-rgb"        # optional: erbt alle Werte dieser Voreinstellung
//!
//! [maske]
//! typ = "streifenmaske"
//! staerke = 0.4
//! ```
//!
//! Beim Laden gilt: unbekannte Schlüssel werden mit Hinweis übergangen, Werte außerhalb der
//! Grenzen werden mit Hinweis auf die Grenze gesetzt, Werte des falschen Typs werden mit
//! Hinweis durch den Standard ersetzt. Ein Fehler entsteht nur bei unlesbarer Datei,
//! unbekannter Basis oder einer Version aus der Zukunft.

use std::collections::BTreeMap;

use crate::katalog::{katalog, option, OptionsArt};
use crate::voreinstellungen::voreinstellung;
use crate::wert::Wert;

/// Aktuelle Version des Dateiformats.
pub const AKTUELLE_VERSION: u32 = 1;

/// Fehler beim Laden von Einstellungen.
#[derive(Debug, thiserror::Error)]
pub enum Fehler {
    #[error("TOML nicht lesbar: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("JSON nicht lesbar: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Einstellungen haben Version {0}, diese roehre kennt nur bis {AKTUELLE_VERSION}")]
    ZukuenftigeVersion(u32),
    #[error("unbekannte Voreinstellung „{0}“")]
    UnbekannteVoreinstellung(String),
    #[error("Aufbau ungültig: {0}")]
    Aufbau(String),
}

/// Hinweis beim Laden: etwas wurde übergangen oder korrigiert.
#[derive(Clone, Debug, PartialEq)]
pub enum Hinweis {
    UnbekannterSchluessel(String),
    FalscherTyp { schluessel: String, erwartet: &'static str },
    UnbekannteAuswahl { schluessel: String, wert: String },
    AufGrenzeGesetzt { schluessel: String, gelesen: f64, gesetzt: f64 },
    Migriert { von: u32, nach: u32 },
}

impl std::fmt::Display for Hinweis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hinweis::UnbekannterSchluessel(s) => write!(f, "unbekannte Option „{s}“ übergangen"),
            Hinweis::FalscherTyp { schluessel, erwartet } => {
                write!(f, "„{schluessel}“ erwartet {erwartet}, Standardwert verwendet")
            }
            Hinweis::UnbekannteAuswahl { schluessel, wert } => {
                write!(f, "„{schluessel}“ kennt den Wert „{wert}“ nicht, Standardwert verwendet")
            }
            Hinweis::AufGrenzeGesetzt { schluessel, gelesen, gesetzt } => {
                write!(f, "„{schluessel}“ = {gelesen} liegt außerhalb der Grenzen, auf {gesetzt} gesetzt")
            }
            Hinweis::Migriert { von, nach } => write!(f, "von Version {von} auf {nach} umgestellt"),
        }
    }
}

/// Vollständige, geprüfte Einstellungen: Jede Option des Katalogs hat einen gültigen Wert.
#[derive(Clone, Debug, PartialEq)]
pub struct Einstellungen {
    werte: BTreeMap<&'static str, Wert>,
    /// Name der Voreinstellung, von der diese Einstellungen abstammen (nur zur Anzeige).
    pub basis: Option<String>,
}

impl Default for Einstellungen {
    fn default() -> Self {
        Self::standard()
    }
}

impl Einstellungen {
    /// Alle Optionen auf ihrem Standardwert.
    pub fn standard() -> Self {
        let werte = katalog().iter().map(|o| (o.schluessel, standardwert(&o.art))).collect();
        Self { werte, basis: None }
    }

    /// Einstellungen einer benannten Voreinstellung.
    pub fn voreinstellung(name: &str) -> Result<Self, Fehler> {
        let v = voreinstellung(name).ok_or_else(|| Fehler::UnbekannteVoreinstellung(name.into()))?;
        // Voreinstellungen werden in den Tests geprüft; hier gibt es keine Hinweise mehr.
        let (e, _) = Self::aus_toml(v.toml)?;
        Ok(e)
    }

    /// Lädt Einstellungen aus TOML.
    pub fn aus_toml(text: &str) -> Result<(Self, Vec<Hinweis>), Fehler> {
        let wurzel: toml::Table = toml::from_str(text)?;
        let json = serde_json::to_value(wurzel).map_err(|e| Fehler::Aufbau(e.to_string()))?;
        Self::aus_baum(json)
    }

    /// Lädt Einstellungen aus JSON.
    pub fn aus_json(text: &str) -> Result<(Self, Vec<Hinweis>), Fehler> {
        Self::aus_baum(serde_json::from_str(text)?)
    }

    fn aus_baum(wurzel: serde_json::Value) -> Result<(Self, Vec<Hinweis>), Fehler> {
        let serde_json::Value::Object(mut wurzel) = wurzel else {
            return Err(Fehler::Aufbau("oberste Ebene ist keine Tabelle".into()));
        };
        let mut hinweise = Vec::new();

        let version = match wurzel.remove("version") {
            None => AKTUELLE_VERSION,
            Some(v) => v
                .as_u64()
                .map(|v| v as u32)
                .ok_or_else(|| Fehler::Aufbau("„version“ ist keine Zahl".into()))?,
        };
        if version > AKTUELLE_VERSION {
            return Err(Fehler::ZukuenftigeVersion(version));
        }
        migriere(version, &mut wurzel, &mut hinweise);

        let mut ergebnis = match wurzel.remove("basis") {
            None => Self::standard(),
            Some(serde_json::Value::String(name)) => {
                let mut e = Self::voreinstellung(&name)?;
                e.basis = Some(name);
                e
            }
            Some(_) => return Err(Fehler::Aufbau("„basis“ ist kein Text".into())),
        };
        // Metadaten der Voreinstellungsdateien gehören nicht zu den Werten.
        wurzel.remove("meta");

        for (gruppe, inhalt) in wurzel {
            let serde_json::Value::Object(tabelle) = inhalt else {
                hinweise.push(Hinweis::UnbekannterSchluessel(gruppe));
                continue;
            };
            for (name, roh) in tabelle {
                let schluessel = format!("{gruppe}.{name}");
                match option(&schluessel) {
                    None => hinweise.push(Hinweis::UnbekannterSchluessel(schluessel)),
                    Some(o) => {
                        let wert = pruefe(o.schluessel, &o.art, &roh, &mut hinweise);
                        ergebnis.werte.insert(o.schluessel, wert);
                    }
                }
            }
        }
        Ok((ergebnis, hinweise))
    }

    /// Speichert als TOML. Mit `nur_abweichungen` werden nur Werte geschrieben, die von der
    /// Basis (oder vom Standard) abweichen.
    pub fn als_toml(&self, nur_abweichungen: bool) -> String {
        let baum = self.als_baum(nur_abweichungen);
        let tabelle: toml::Table = serde_json::from_value(baum).expect("Baum ist immer eine Tabelle");
        toml::to_string_pretty(&tabelle).expect("Einstellungen sind immer als TOML darstellbar")
    }

    /// Speichert als JSON.
    pub fn als_json(&self, nur_abweichungen: bool) -> String {
        serde_json::to_string_pretty(&self.als_baum(nur_abweichungen)).expect("immer darstellbar")
    }

    fn als_baum(&self, nur_abweichungen: bool) -> serde_json::Value {
        let vergleich = match (&self.basis, nur_abweichungen) {
            (_, false) => None,
            (Some(b), true) => Self::voreinstellung(b).ok(),
            (None, true) => Some(Self::standard()),
        };
        let mut wurzel = serde_json::Map::new();
        wurzel.insert("version".into(), AKTUELLE_VERSION.into());
        if let Some(b) = &self.basis {
            wurzel.insert("basis".into(), b.clone().into());
        }
        for (schluessel, wert) in &self.werte {
            if let Some(v) = &vergleich {
                if v.werte.get(schluessel) == Some(wert) {
                    continue;
                }
            }
            let (gruppe, name) = schluessel.split_once('.').expect("Schlüssel mit Punkt");
            let tabelle = wurzel
                .entry(gruppe)
                .or_insert_with(|| serde_json::Value::Object(Default::default()));
            tabelle
                .as_object_mut()
                .expect("Gruppe ist Tabelle")
                .insert(name.into(), serde_json::to_value(wert).expect("Wert darstellbar"));
        }
        serde_json::Value::Object(wurzel)
    }

    /// Wert einer Option. Unbekannte Schlüssel sind ein Programmierfehler.
    pub fn wert(&self, schluessel: &str) -> &Wert {
        self.werte
            .get(schluessel)
            .unwrap_or_else(|| panic!("unbekannte Option „{schluessel}“"))
    }

    pub fn zahl(&self, schluessel: &str) -> f64 {
        self.wert(schluessel).als_zahl().expect("Option ist keine Zahl")
    }

    pub fn schalter(&self, schluessel: &str) -> bool {
        self.wert(schluessel).als_schalter().expect("Option ist kein Schalter")
    }

    pub fn auswahl(&self, schluessel: &str) -> &str {
        self.wert(schluessel).als_auswahl().expect("Option ist keine Auswahl")
    }

    /// Setzt einen Wert mit denselben Prüfungen wie beim Laden.
    pub fn setze(&mut self, schluessel: &str, wert: Wert) -> Vec<Hinweis> {
        let mut hinweise = Vec::new();
        match option(schluessel) {
            None => hinweise.push(Hinweis::UnbekannterSchluessel(schluessel.into())),
            Some(o) => {
                let roh = serde_json::to_value(&wert).expect("Wert darstellbar");
                let w = pruefe(o.schluessel, &o.art, &roh, &mut hinweise);
                self.werte.insert(o.schluessel, w);
            }
        }
        hinweise
    }

    /// Alle Werte in Katalogreihenfolge.
    pub fn alle(&self) -> impl Iterator<Item = (&'static str, &Wert)> {
        katalog().iter().map(|o| (o.schluessel, &self.werte[o.schluessel]))
    }
}

fn standardwert(art: &OptionsArt) -> Wert {
    match *art {
        OptionsArt::Zahl { standard, .. } => Wert::Zahl(standard),
        OptionsArt::Ganzzahl { standard, .. } => Wert::Ganzzahl(standard),
        OptionsArt::Schalter { standard } => Wert::Schalter(standard),
        OptionsArt::Auswahl { standard, .. } => Wert::Auswahl(standard.into()),
    }
}

fn pruefe(schluessel: &str, art: &OptionsArt, roh: &serde_json::Value, hinweise: &mut Vec<Hinweis>) -> Wert {
    let falsch = |erwartet, hinweise: &mut Vec<Hinweis>| {
        hinweise.push(Hinweis::FalscherTyp { schluessel: schluessel.into(), erwartet });
        standardwert(art)
    };
    match *art {
        OptionsArt::Zahl { min, max, .. } => match roh.as_f64() {
            None => falsch("eine Zahl", hinweise),
            Some(z) if !z.is_finite() => falsch("eine endliche Zahl", hinweise),
            Some(z) => {
                let g = z.clamp(min, max);
                if g != z {
                    hinweise.push(Hinweis::AufGrenzeGesetzt { schluessel: schluessel.into(), gelesen: z, gesetzt: g });
                }
                Wert::Zahl(g)
            }
        },
        OptionsArt::Ganzzahl { min, max, .. } => match roh.as_f64() {
            None => falsch("eine ganze Zahl", hinweise),
            Some(z) if !z.is_finite() => falsch("eine endliche Zahl", hinweise),
            Some(z) => {
                let g = (z.round() as i64).clamp(min, max);
                if g as f64 != z {
                    hinweise.push(Hinweis::AufGrenzeGesetzt { schluessel: schluessel.into(), gelesen: z, gesetzt: g as f64 });
                }
                Wert::Ganzzahl(g)
            }
        },
        OptionsArt::Schalter { .. } => match roh.as_bool() {
            Some(b) => Wert::Schalter(b),
            None => falsch("an/aus (true/false)", hinweise),
        },
        OptionsArt::Auswahl { werte, .. } => match roh.as_str() {
            None => falsch("einen Text", hinweise),
            Some(s) if werte.iter().any(|(w, _)| *w == s) => Wert::Auswahl(s.into()),
            Some(s) => {
                hinweise.push(Hinweis::UnbekannteAuswahl { schluessel: schluessel.into(), wert: s.into() });
                standardwert(art)
            }
        },
    }
}

/// Hebt ältere Dateiversionen auf die aktuelle an. Version 1 ist die erste; künftige
/// Umbenennungen oder Umrechnungen kommen hier als eigene Schritte hinzu.
fn migriere(version: u32, _wurzel: &mut serde_json::Map<String, serde_json::Value>, hinweise: &mut Vec<Hinweis>) {
    if version < AKTUELLE_VERSION {
        hinweise.push(Hinweis::Migriert { von: version, nach: AKTUELLE_VERSION });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_ist_vollstaendig() {
        let e = Einstellungen::standard();
        assert_eq!(e.alle().count(), katalog().len());
    }

    #[test]
    fn grenzen_und_typen_werden_geprueft() {
        let (e, h) = Einstellungen::aus_toml(
            r#"
            version = 1
            [maske]
            staerke = 7.0
            typ = "dreieck"
            [licht]
            glow = "viel"
            [gibtsnicht]
            x = 1
            "#,
        )
        .unwrap();
        assert_eq!(e.zahl("maske.staerke"), 1.0);
        assert_eq!(e.auswahl("maske.typ"), "schlitzmaske");
        assert_eq!(e.zahl("licht.glow"), 0.08);
        assert_eq!(h.len(), 4, "{h:?}");
    }

    #[test]
    fn zukunftsversion_wird_abgelehnt() {
        assert!(matches!(Einstellungen::aus_toml("version = 99"), Err(Fehler::ZukuenftigeVersion(99))));
    }

    #[test]
    fn hin_und_zurueck_toml_und_json() {
        let mut e = Einstellungen::voreinstellung("scart-rgb").unwrap();
        e.basis = Some("scart-rgb".into());
        e.setze("maske.staerke", Wert::Zahl(0.55));
        let t = e.als_toml(true);
        assert!(t.contains("staerke = 0.55"), "{t}");
        let (zurueck, h) = Einstellungen::aus_toml(&t).unwrap();
        assert!(h.is_empty(), "{h:?}");
        assert_eq!(zurueck, e);
        let (zurueck, _) = Einstellungen::aus_json(&e.als_json(false)).unwrap();
        assert_eq!(zurueck, e);
    }
}
