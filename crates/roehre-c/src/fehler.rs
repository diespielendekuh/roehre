//! Letzter Fehler je Thread und Schutz gegen Panik an der C-Grenze.

use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

thread_local! {
    static LETZTER: RefCell<CString> = RefCell::new(CString::default());
}

pub(crate) fn setze(text: impl Into<String>) {
    let t = CString::new(text.into().replace('\0', " ")).expect("ohne Nullbyte");
    LETZTER.with(|l| *l.borrow_mut() = t);
}

pub(crate) fn letzter() -> *const c_char {
    LETZTER.with(|l| l.borrow().as_ptr())
}

/// Führt `f` aus; Fehler und Panik werden zu `fehlwert` mit gesetztem Fehlertext.
pub(crate) fn geschuetzt<T>(fehlwert: T, f: impl FnOnce() -> Result<T, String>) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            setze(e);
            fehlwert
        }
        Err(p) => {
            let text = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unbekannte Panik".into());
            setze(format!("interner Fehler in roehre: {text}"));
            fehlwert
        }
    }
}

/// Liest einen C-Text (UTF-8) oder meldet, was fehlt.
pub(crate) unsafe fn text<'a>(p: *const c_char, was: &str) -> Result<&'a str, String> {
    if p.is_null() {
        return Err(format!("{was} fehlt (NULL)"));
    }
    unsafe { std::ffi::CStr::from_ptr(p) }.to_str().map_err(|_| format!("{was} ist kein UTF-8"))
}
