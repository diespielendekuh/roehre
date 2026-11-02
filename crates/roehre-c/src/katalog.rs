//! Der Katalog als JSON, damit ein Menü in C/C++ (z. B. ein Launcher) alle Optionen mit
//! Schlüssel, deutschem Namen, Beschreibung, Grenzen, Standard und Festwerten bauen kann.

use std::ffi::CString;
use std::sync::OnceLock;

use roehre_modell::{festwerte, katalog, voreinstellungen, Gruppe, Kosten, Lizenzmerkmal, OptionsArt, Texte};
use serde_json::{json, Value};

fn texte(t: &Texte) -> Value {
    json!({ "de": t.de, "en": t.en })
}

fn bauen() -> Value {
    let gruppen: Vec<Value> = Gruppe::ALLE
        .iter()
        .map(|g| json!({ "schluessel": g.schluessel(), "name": texte(&g.name()) }))
        .collect();
    let optionen: Vec<Value> = katalog()
        .iter()
        .map(|o| {
            let mut v = json!({
                "schluessel": o.schluessel,
                "gruppe": o.gruppe.schluessel(),
                "name": texte(&o.name),
                "beschreibung": texte(&o.beschreibung),
                "kosten": match o.kosten { Kosten::Keine => "keine", Kosten::Gering => "gering", Kosten::Hoch => "hoch" },
                "wirkt_wenn": o.wirkt_wenn.map(|(s, w)| json!({ "option": s, "werte": w })),
            });
            let art = match o.art {
                OptionsArt::Zahl { min, max, schritt, standard, einheit } => json!({
                    "art": "zahl", "min": min, "max": max, "schritt": schritt, "standard": standard, "einheit": einheit,
                    "festwerte": festwerte(o.schluessel).iter()
                        .map(|(k, n, w)| json!({ "schluessel": k, "name": texte(n), "wert": w }))
                        .collect::<Vec<_>>(),
                }),
                OptionsArt::Ganzzahl { min, max, standard } => json!({ "art": "ganzzahl", "min": min, "max": max, "standard": standard }),
                OptionsArt::Schalter { standard } => json!({ "art": "schalter", "standard": standard }),
                OptionsArt::Auswahl { werte, standard } => json!({
                    "art": "auswahl", "standard": standard,
                    "werte": werte.iter().map(|(k, n)| json!({ "schluessel": k, "name": texte(n) })).collect::<Vec<_>>(),
                }),
            };
            if let (Value::Object(a), Value::Object(b)) = (&mut v, art) {
                a.extend(b);
            }
            v
        })
        .collect();
    let vor: Vec<Value> = voreinstellungen()
        .iter()
        .map(|v| {
            json!({
                "schluessel": v.schluessel,
                "name": v.name(),
                "beschreibung": v.beschreibung(),
                "lizenz": match v.lizenz() { Lizenzmerkmal::Permissiv => "permissiv", Lizenzmerkmal::Gpl => "gpl" },
            })
        })
        .collect();
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "einstellungsversion": roehre_modell::AKTUELLE_VERSION,
        "gruppen": gruppen,
        "optionen": optionen,
        "voreinstellungen": vor,
    })
}

pub(crate) fn json() -> &'static CString {
    static J: OnceLock<CString> = OnceLock::new();
    J.get_or_init(|| CString::new(serde_json::to_string_pretty(&bauen()).expect("JSON")).expect("ohne Nullbyte"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn katalog_ist_vollstaendig() {
        let v: serde_json::Value = serde_json::from_str(super::json().to_str().unwrap()).unwrap();
        assert_eq!(v["optionen"].as_array().unwrap().len(), roehre_modell::katalog().len());
        let naehe = v["optionen"].as_array().unwrap().iter().find(|o| o["schluessel"] == "raum.naehe").unwrap();
        assert_eq!(naehe["festwerte"].as_array().unwrap().len(), 3);
        assert!(v["voreinstellungen"].as_array().unwrap().len() >= 10);
    }
}
