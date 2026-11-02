//! Einstellungen als undurchsichtiger Zeiger: laden, speichern, einzelne Optionen setzen.

use std::ffi::c_char;
use std::ptr;

use roehre_modell::{option, OptionsArt};
use roehre_wgpu::{Einstellungen, Wert};

use crate::fehler::{geschuetzt, text};
use crate::text_heraus;

/// Undurchsichtiger Satz Einstellungen.
pub struct RoehreEinstellungen(pub(crate) Einstellungen);

fn heraus(e: Einstellungen) -> *mut RoehreEinstellungen {
    Box::into_raw(Box::new(RoehreEinstellungen(e)))
}

pub(crate) unsafe fn innen<'a>(e: *const RoehreEinstellungen) -> Result<&'a Einstellungen, String> {
    unsafe { e.as_ref() }.map(|e| &e.0).ok_or_else(|| "Einstellungen fehlen (NULL)".to_string())
}

/// Standard-Einstellungen (alle Optionen auf ihrem Standard).
#[unsafe(no_mangle)]
pub extern "C" fn roehre_einstellungen_standard() -> *mut RoehreEinstellungen {
    heraus(Einstellungen::standard())
}

/// Einstellungen einer Voreinstellung, z. B. "scart-rgb". NULL bei unbekanntem Namen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_einstellungen_voreinstellung(name: *const c_char) -> *mut RoehreEinstellungen {
    geschuetzt(ptr::null_mut(), || {
        let name = unsafe { text(name, "Name") }?;
        Einstellungen::voreinstellung(name).map(heraus).map_err(|e| e.to_string())
    })
}

/// Einstellungen aus TOML oder JSON (erkannt am ersten Zeichen `{`). Hinweise (unbekannte
/// Schlüssel, korrigierte Werte) gehen nicht verloren: sie stehen danach in
/// `roehre_letzter_fehler()`, der Rückgabewert ist trotzdem gültig.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_einstellungen_aus_text(t: *const c_char) -> *mut RoehreEinstellungen {
    geschuetzt(ptr::null_mut(), || {
        let t = unsafe { text(t, "Text") }?;
        let (e, hinweise) = if t.trim_start().starts_with('{') { Einstellungen::aus_json(t) } else { Einstellungen::aus_toml(t) }
            .map_err(|e| e.to_string())?;
        crate::fehler::setze(hinweise.iter().map(|h| h.to_string()).collect::<Vec<_>>().join("\n"));
        Ok(heraus(e))
    })
}

/// Kopie.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_einstellungen_kopieren(e: *const RoehreEinstellungen) -> *mut RoehreEinstellungen {
    geschuetzt(ptr::null_mut(), || Ok(heraus(unsafe { innen(e) }?.clone())))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_einstellungen_freigeben(e: *mut RoehreEinstellungen) {
    if !e.is_null() {
        drop(unsafe { Box::from_raw(e) });
    }
}

/// Als TOML (`format` 0) oder JSON (`format` 1); mit `nur_abweichungen` != 0 nur, was vom
/// Standard abweicht. Freigeben mit `roehre_text_freigeben`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_einstellungen_als_text(e: *const RoehreEinstellungen, format: i32, nur_abweichungen: i32) -> *mut c_char {
    geschuetzt(ptr::null_mut(), || {
        let e = unsafe { innen(e) }?;
        Ok(text_heraus(if format == 1 { e.als_json(nur_abweichungen != 0) } else { e.als_toml(nur_abweichungen != 0) }))
    })
}

/// Setzt eine Option, Wert als Text: Zahl ("0.5" oder "0,5"), Schalter ("an"/"aus",
/// "true"/"false", "1"/"0") oder Auswahl ("composite"). Rückgabe 0: übernommen; 1: übernommen,
/// aber korrigiert (Text in `roehre_letzter_fehler`); < 0: abgelehnt.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_einstellungen_setzen(e: *mut RoehreEinstellungen, schluessel: *const c_char, wert: *const c_char) -> i32 {
    geschuetzt(-1, || {
        let e = &mut unsafe { e.as_mut() }.ok_or("Einstellungen fehlen (NULL)")?.0;
        let k = unsafe { text(schluessel, "Schlüssel") }?;
        let w = unsafe { text(wert, "Wert") }?.trim();
        let o = option(k).ok_or_else(|| format!("unbekannte Option „{k}“"))?;
        let zahl = || w.replace(',', ".").parse::<f64>().map_err(|_| format!("„{w}“ ist keine Zahl ({k})"));
        let wert = match o.art {
            OptionsArt::Zahl { .. } => Wert::Zahl(zahl()?),
            OptionsArt::Ganzzahl { .. } => Wert::Ganzzahl(zahl()?.round() as i64),
            OptionsArt::Schalter { .. } => Wert::Schalter(match w {
                "an" | "true" | "1" | "ja" => true,
                "aus" | "false" | "0" | "nein" => false,
                _ => return Err(format!("„{w}“ ist kein Schalterwert ({k})")),
            }),
            OptionsArt::Auswahl { werte, .. } => {
                if !werte.iter().any(|(s, _)| *s == w) {
                    return Err(format!("„{w}“ ist für {k} nicht erlaubt"));
                }
                Wert::Auswahl(w.to_string())
            }
        };
        let hinweise = e.setze(k, wert);
        if hinweise.is_empty() {
            Ok(0)
        } else {
            crate::fehler::setze(hinweise.iter().map(|h| h.to_string()).collect::<Vec<_>>().join("\n"));
            Ok(1)
        }
    })
}

/// Wert einer Option als Text (wie in TOML). Freigeben mit `roehre_text_freigeben`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_einstellungen_wert(e: *const RoehreEinstellungen, schluessel: *const c_char) -> *mut c_char {
    geschuetzt(ptr::null_mut(), || {
        let e = unsafe { innen(e) }?;
        let k = unsafe { text(schluessel, "Schlüssel") }?;
        option(k).ok_or_else(|| format!("unbekannte Option „{k}“"))?;
        Ok(text_heraus(match e.wert(k) {
            Wert::Auswahl(a) => a.clone(),
            w => w.to_string(),
        }))
    })
}
