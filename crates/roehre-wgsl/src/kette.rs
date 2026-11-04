//! **Erzeugte Ketten** (W-009, Teil 2): fremde slang-Ketten (heute crt-beans,
//! MIT), die librashader beim Erzeugen nach WGSL übersetzt hat, samt allem,
//! was ein Ausführer braucht, um sie **genau wie librashader** laufen zu
//! lassen — ohne librashader und ohne wgpu-Version.
//!
//! Erzeugt von `roehre-pruefstand wgsl-erzeugen`; die Daten stehen in
//! `src/erzeugt/` und `wgsl/erzeugt/` und werden nicht von Hand geändert.
//!
//! **Was ein Ausführer nachbilden muß** (gelesen in librashader 0.12.1,
//! `librashader-runtime-wgpu` und `librashader-runtime`):
//!
//! * Bindungen: Gruppe 0 = Bilder (`texture_2d<f32>`, filterbar) und die
//!   Puffer [`Pass::ubo`]/[`Pass::push`] (beide als uniform), Gruppe 1 = je
//!   Bild ein Abtaster unter **derselben** Nummer.
//! * Eckdaten: vier Ecken als Dreiecksstreifen, je Ecke `vec4` Lage (Ort 0)
//!   und `vec2` Texturkoordinate (Ort 1), 24 Byte: Zwischenpässe
//!   [`ECKEN_ZWISCHEN`] mit [`MVP_EINHEIT`], der letzte Pass [`ECKEN_LETZTER`]
//!   mit [`MVP_LETZTER`] in den Viewport des Ziels.
//! * Größen: beginnend bei der Größe des Spielbilds wird je Pass
//!   `max(round(Bezug · Faktor), 1)` gerechnet ([`Skala`]); der letzte Pass
//!   zeichnet in den Viewport, gleich was seine Skala sagt.
//! * Abtaster: `Original` mit Filter und Rand des **ersten** Passes, `Source`
//!   mit denen des **lesenden** Passes, `PassOutput[i]` mit denen des
//!   **erzeugenden** Passes i, ein Nachschlagebild mit seinen eigenen.
//!   Mip-Filter = Filter; `ClampToBorder` nur mit dem Merkmal
//!   `ADDRESS_MODE_CLAMP_TO_BORDER`, sonst `ClampToEdge` (Rand durchsichtig
//!   schwarz).
//! * Werte: [`Wert`] je Feld, `vec4`-Größen als `(b, h, 1/b, 1/h)`.

/// Eine Ecke: Lage (x, y, z, w) und Texturkoordinate (u, v).
pub type Eckdaten = ([f32; 4], [f32; 2]);

/// Die vier Ecken der Zwischenpässe (librashader `OFFSCREEN_VBO_DATA`).
pub const ECKEN_ZWISCHEN: [Eckdaten; 4] = [
    ([-1.0, -1.0, 0.0, 1.0], [0.0, 0.0]),
    ([-1.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
    ([1.0, -1.0, 0.0, 1.0], [1.0, 0.0]),
    ([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
];

/// Die vier Ecken des letzten Passes (librashader `FINAL_VBO_DATA`).
pub const ECKEN_LETZTER: [Eckdaten; 4] = [
    ([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
    ([0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
    ([1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
    ([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
];

/// MVP der Zwischenpässe (librashader `IDENTITY_MVP`), spaltenweise.
pub const MVP_EINHEIT: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// MVP des letzten Passes (librashader `DEFAULT_MVP`), spaltenweise.
pub const MVP_LETZTER: [f32; 16] = [
    2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 1.0,
];

/// Wie groß ein Pass rechnet (eine Achse).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Skala {
    /// Größe des vorigen Passes (beim ersten: des Spielbilds) mal Faktor.
    Quelle(f32),
    /// Größe des Viewports mal Faktor.
    Ziel(f32),
    /// Feste Punktzahl.
    Fest(u32),
    /// Größe des Spielbilds mal Faktor.
    Original(f32),
}

impl Skala {
    /// `max(round(Bezug · Faktor), 1)` wie librashader.
    pub fn rechne(self, vorher: u32, ziel: u32, original: u32) -> u32 {
        let w = match self {
            Skala::Quelle(f) => vorher as f32 * f,
            Skala::Ziel(f) => ziel as f32 * f,
            Skala::Fest(n) => n as f32,
            Skala::Original(f) => original as f32 * f,
        };
        (w.round() as u32).max(1)
    }
}

/// Format eines Zwischenbilds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba16Float,
    Rgba32Float,
    Rgb10a2Unorm,
}

/// Verhalten am Rand der Textur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rand {
    /// Außen durchsichtig schwarz (braucht `ADDRESS_MODE_CLAMP_TO_BORDER`,
    /// sonst wie [`Rand::Kante`]).
    Schwarz,
    Kante,
    Wiederholen,
    Spiegeln,
}

/// Welches Bild eine Bindung zeigt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bild {
    /// Das Spielbild (`Original`).
    Original,
    /// Die Ausgabe des vorigen Passes (`Source`; beim ersten das Spielbild).
    Quelle,
    /// Die Ausgabe von Pass n (`PassOutput[n]`, auch über seinen Alias).
    Pass(u32),
    /// Nachschlagebild n aus [`Erzeugt::luts`].
    Lut(u32),
}

/// Eine Bildbindung: Bild unter `nummer` in Gruppe 0, sein Abtaster unter
/// `nummer` in Gruppe 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bindung {
    pub nummer: u32,
    pub bild: Bild,
}

/// In welchem Puffer ein Feld liegt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ort {
    Ubo,
    Push,
}

/// Was in ein Feld geschrieben wird.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wert {
    /// `mat4`: [`MVP_EINHEIT`] bzw. [`MVP_LETZTER`].
    Mvp,
    /// `vec4`: Größe des Ziels dieses Passes.
    Ausgabe,
    /// `vec4`: Größe des Viewports.
    Endausgabe,
    /// `u32`: Bildzähler (modulo [`Pass::bildzaehler_modulo`], wenn > 0).
    Bildzaehler,
    /// `i32`: Laufrichtung, 1.
    Richtung,
    /// `f32`: Seitenverhältnis des Spielbilds (Breite/Höhe in Punkten).
    Seitenverhaeltnis,
    /// `vec4`: Größe eines Bilds.
    Groesse(Bild),
    /// `f32`: ein Regler nach Name ([`Erzeugt::regler`]).
    Regler(&'static str),
}

/// Ein Feld in einem Puffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Feld {
    pub ort: Ort,
    /// Lage in Byte.
    pub lage: u32,
    pub wert: Wert,
}

/// Ein uniform-Puffer eines Passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Puffer {
    pub nummer: u32,
    pub groesse: u32,
    /// Sichtbar in der Eckenfunktion.
    pub ecke: bool,
    /// Sichtbar in der Punktfunktion.
    pub punkt: bool,
}

/// Ein Pass einer erzeugten Kette.
#[derive(Debug, Clone, Copy)]
pub struct Pass {
    /// Dateistamm des slang-Shaders.
    pub name: &'static str,
    pub ecke: &'static str,
    pub ecke_einstieg: &'static str,
    pub punkt: &'static str,
    pub punkt_einstieg: &'static str,
    pub skala: (Skala, Skala),
    pub format: Format,
    pub linear: bool,
    pub rand: Rand,
    pub bildzaehler_modulo: u32,
    pub ubo: Option<Puffer>,
    pub push: Option<Puffer>,
    pub felder: &'static [Feld],
    pub bilder: &'static [Bindung],
}

/// Ein Nachschlagebild (RGBA, 8 Bit, zeilenweise von oben).
#[derive(Debug, Clone, Copy)]
pub struct Lut {
    pub name: &'static str,
    pub breite: u32,
    pub hoehe: u32,
    pub rgba: &'static [u8],
    pub linear: bool,
    pub rand: Rand,
    pub mipmap: bool,
}

/// Ein Regler der Kette mit dem Wert, den das Preset setzt (sonst dem des
/// Shaders), und den Grenzen aus `#pragma parameter`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Regler {
    pub name: &'static str,
    pub standard: f32,
    pub kleinst: f32,
    pub groesst: f32,
}

/// Eine erzeugte Kette.
#[derive(Debug, Clone, Copy)]
pub struct Erzeugt {
    /// Kurzname (ASCII).
    pub name: &'static str,
    /// Preset, aus dem sie erzeugt ist (relativ zu `shader/`).
    pub preset: &'static str,
    pub lizenz: &'static str,
    pub paesse: &'static [Pass],
    pub luts: &'static [Lut],
    pub regler: &'static [Regler],
}
