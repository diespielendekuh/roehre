//! C-Schnittstelle von roehre. Die Beschreibung jeder Funktion steht in `include/roehre.h`;
//! hier steht, wie sie umgesetzt ist.
//!
//! Grundsätze:
//! * Keine Panik überschreitet die Grenze nach C: jede Funktion läuft in `catch_unwind`.
//! * Fehler liefern einen Rückgabewert < 0 bzw. NULL; den Text dazu liefert
//!   `roehre_letzter_fehler()` (je Thread).
//! * Texte, die roehre herausgibt und der Aufrufer freigeben muss, gibt er mit
//!   `roehre_text_freigeben` zurück. Statische Texte (Version, Katalog) nicht freigeben.
//! * roehre rechnet auf einem eigenen wgpu-Gerät. Mit Vulkan- und D3D12-Spielen tauscht es
//!   Bilder über geteilten Grafikspeicher aus (`austausch.rs`), damit am Gerät des Spiels nichts
//!   geändert werden muss (bei D3D12) bzw. nur zwei Erweiterungen (bei Vulkan).

#![allow(clippy::missing_safety_doc)]

mod arbeitsplatz;
mod einstellungen;
mod fehler;
mod katalog;
mod vorschau;

pub use arbeitsplatz::Modus;
pub use einstellungen::RoehreEinstellungen;
pub use vorschau::RoehreVorschau;

use std::ffi::{c_char, CString};

/// Version der Bibliothek, z. B. "0.1.0".
#[unsafe(no_mangle)]
pub extern "C" fn roehre_version() -> *const c_char {
    static V: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    V.as_ptr().cast()
}

/// Text des letzten Fehlers dieses Threads (leer, wenn keiner).
#[unsafe(no_mangle)]
pub extern "C" fn roehre_letzter_fehler() -> *const c_char {
    fehler::letzter()
}

/// Gibt einen Text frei, den roehre herausgegeben hat.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_text_freigeben(text: *mut c_char) {
    if !text.is_null() {
        drop(unsafe { CString::from_raw(text) });
    }
}

/// Katalog aller Optionen, Gruppen, Festwerte und Voreinstellungen als JSON (UTF-8).
#[unsafe(no_mangle)]
pub extern "C" fn roehre_katalog_json() -> *const c_char {
    katalog::json().as_ptr()
}

pub(crate) fn text_heraus(s: String) -> *mut c_char {
    CString::new(s.replace('\0', " ")).expect("ohne Nullbyte").into_raw()
}
