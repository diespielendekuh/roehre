//! Mehrsprachige Texte für Namen und Beschreibungen.

/// Sprache der Texte. Deutsch ist vollständig, Englisch folgt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sprache {
    Deutsch,
    Englisch,
}

/// Ein Text in mehreren Sprachen. Fehlt die gewünschte Sprache, wird Deutsch geliefert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Texte {
    pub de: &'static str,
    pub en: Option<&'static str>,
}

impl Texte {
    pub const fn de(de: &'static str) -> Self {
        Self { de, en: None }
    }

    pub const fn de_en(de: &'static str, en: &'static str) -> Self {
        Self { de, en: Some(en) }
    }

    /// Text in der gewünschten Sprache, ersatzweise Deutsch.
    pub fn in_sprache(&self, sprache: Sprache) -> &'static str {
        match sprache {
            Sprache::Deutsch => self.de,
            Sprache::Englisch => self.en.unwrap_or(self.de),
        }
    }
}
