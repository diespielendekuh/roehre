//! **roehre-wgsl** — die GPL-freien Stufen von roehre als WGSL-Text mit
//! Pass-Beschreibung, **ohne wgpu- und ohne librashader-Version** (Wunsch
//! W-009). Ein Spiel mit älterem wgpu (Terranigma: 24) führt die Stufen damit
//! selbst aus, ohne anzuheben und ohne die Shader bei sich zu kopieren.
//!
//! Die Kiste hat keine Abhängigkeiten. Sie liefert:
//!
//! * [`Stufe`]: ein Pass — WGSL-Text (Kopf [`GEMEINSAM`] + Rumpf), Größe des
//!   Ziels, Art des Abtasters;
//! * [`Kette`]: Stufen in Reihenfolge, wie ein Spiel sie als Bildfilter
//!   anbietet ([`KETTEN`]);
//! * [`Rahmen`]: die Werte, die jede Stufe liest, samt ihrem Layout als
//!   128 Byte ([`Rahmen::als_bytes`]) — der Ausführer rät nichts;
//! * [`mmpx`]: die CPU-Referenz der Stufe `mmpx`.
//!
//! **Die Vereinbarung** (steht auch im Kopf jeder Stufe): `@group(0)`,
//! `@binding(0)` der Rahmen (uniform), `@binding(1)` die Quelle
//! (`texture_2d<f32>`, `Rgba8Unorm` mit sRGB-kodierten Werten),
//! `@binding(2)` der Abtaster. Eckenfunktion `ecke`, Punktfunktion `punkt`,
//! drei Ecken ohne Eckenpuffer, Viewport = Bildausschnitt des Ziels.
//!
//! Die Dateien liegen **in** der Kiste (`wgsl/`), nicht unter `shader/`:
//! wer die Kiste mitpackt (`cargo vendor`), bekommt nur ihren Ordner.

pub mod erzeugt;
pub mod kette;
pub mod mmpx;

/// Der gemeinsame Kopf jeder Stufe: Rahmen, Bindungen, Eckenfunktion.
pub const GEMEINSAM: &str = include_str!("../wgsl/gemeinsam.wgsl");

/// Wie groß das Ziel einer Stufe ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Groesse {
    /// Der Bildausschnitt des Ziels (die letzte Stufe einer Kette).
    Ziel,
    /// Die Quelle mal n in jeder Achse (z. B. MMPX: 2).
    QuelleMal(u32),
}

/// Wie die Quelle abgetastet wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abtastung {
    /// Linear (Klemmen am Rand).
    Linear,
    /// Nächster Punkt (Klemmen am Rand).
    Naechster,
}

/// Ein Pass.
#[derive(Debug, Clone, Copy)]
pub struct Stufe {
    /// Kurzname (ASCII), wie im Dateinamen.
    pub name: &'static str,
    /// Der Rumpf in WGSL (ohne Kopf); ganz: [`Stufe::wgsl`].
    pub rumpf: &'static str,
    pub groesse: Groesse,
    pub abtastung: Abtastung,
    /// Lizenz der Stufe (SPDX).
    pub lizenz: &'static str,
    /// Regler der Stufe in der Reihenfolge von `rahmen.regler` (je `vec4`
    /// vier Werte: Regler 0 → `regler[0].x`, 1 → `.y` …).
    pub regler: &'static [StufenRegler],
}

/// Ein Regler einer Stufe, mit Text für das Menü eines Spiels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StufenRegler {
    /// Schlüssel (ASCII).
    pub name: &'static str,
    /// Anzeigename, deutsch.
    pub titel: &'static str,
    /// Ein Satz, deutsch.
    pub beschreibung: &'static str,
    pub kleinst: f32,
    pub groesst: f32,
    pub schritt: f32,
    pub standard: f32,
}

impl Stufe {
    /// Der ganze WGSL-Text: Kopf und Rumpf.
    pub fn wgsl(&self) -> String {
        format!("{GEMEINSAM}\n{}", self.rumpf)
    }
}

/// Pixelgenau ohne Unschärfe einpassen, Saum je Achse genau ein Bildpunkt —
/// aus `shader/eigen/scharf-bilinear.slang`.
pub const SCHARF_BILINEAR: Stufe = Stufe {
    name: "scharf-bilinear",
    rumpf: include_str!("../wgsl/scharf-bilinear.wgsl"),
    groesse: Groesse::Ziel,
    abtastung: Abtastung::Linear,
    lizenz: "MIT OR Apache-2.0",
    regler: &[],
};

/// MMPX: verdoppeln, Treppen an schrägen Kanten glätten, keine neue Farbe.
pub const MMPX: Stufe = Stufe {
    name: "mmpx",
    rumpf: include_str!("../wgsl/mmpx.wgsl"),
    groesse: Groesse::QuelleMal(2),
    abtastung: Abtastung::Naechster,
    lizenz: "MIT",
    regler: &[],
};

/// Handheld-/LCD-Raster: Zellen mit dunklen Lücken, wahlweise RGB-Streifen;
/// gegen Moiré exakt über die Fläche des Bildpunkts integriert.
pub const LCD_RASTER: Stufe = Stufe {
    name: "lcd-raster",
    rumpf: include_str!("../wgsl/lcd-raster.wgsl"),
    groesse: Groesse::Ziel,
    abtastung: Abtastung::Naechster,
    lizenz: "MIT OR Apache-2.0",
    regler: &[
        StufenRegler { name: "gitter", titel: "Raster", beschreibung: "Wie dunkel die Lücken zwischen den Punkten sind.", kleinst: 0.0, groesst: 1.0, schritt: 0.05, standard: 0.35 },
        StufenRegler { name: "breite", titel: "Rasterbreite", beschreibung: "Breite der Lücken als Anteil eines Spielpunkts.", kleinst: 0.0, groesst: 0.5, schritt: 0.025, standard: 0.15 },
        StufenRegler { name: "subpixel", titel: "Farbstreifen", beschreibung: "Senkrechte Rot-Grün-Blau-Streifen in jedem Punkt wie bei einem echten LCD.", kleinst: 0.0, groesst: 1.0, schritt: 0.05, standard: 0.0 },
    ],
};

/// Nächster Punkt, ohne weitere Rechnung (bei krummen Faktoren ungleich
/// breite Punkte).
pub const NAECHSTER: Stufe = Stufe { name: "naechster", rumpf: include_str!("../wgsl/abtasten.wgsl"), groesse: Groesse::Ziel, abtastung: Abtastung::Naechster, lizenz: "MIT OR Apache-2.0", regler: &[] };

/// Linear eingepaßt (weich).
pub const LINEAR: Stufe = Stufe { name: "linear", rumpf: include_str!("../wgsl/abtasten.wgsl"), groesse: Groesse::Ziel, abtastung: Abtastung::Linear, lizenz: "MIT OR Apache-2.0", regler: &[] };

/// Farbkorrektur hinter jedem Filter (1:1): Helligkeit, Kontrast, Sättigung,
/// Wärme, in linearem Licht. Bei den Voreinstellungen unverändert bis auf die
/// 8-Bit-Rundung; ein Ausführer läßt sie dann weg ([`Stufe::neutral`]).
pub const FARBE: Stufe = Stufe {
    name: "farbe",
    rumpf: include_str!("../wgsl/farbe.wgsl"),
    groesse: Groesse::Ziel,
    abtastung: Abtastung::Naechster,
    lizenz: "MIT OR Apache-2.0",
    regler: &[
        StufenRegler { name: "helligkeit", titel: "Helligkeit", beschreibung: "Mehr oder weniger Licht für das ganze Bild.", kleinst: 0.5, groesst: 1.5, schritt: 0.05, standard: 1.0 },
        StufenRegler { name: "kontrast", titel: "Kontrast", beschreibung: "Spreizt helle und dunkle Töne um Mittelgrau.", kleinst: 0.5, groesst: 1.5, schritt: 0.05, standard: 1.0 },
        StufenRegler { name: "saettigung", titel: "Sättigung", beschreibung: "0 ist grau, 1 die Farben des Spiels, darüber kräftiger.", kleinst: 0.0, groesst: 2.0, schritt: 0.05, standard: 1.0 },
        StufenRegler { name: "waerme", titel: "Farbwärme", beschreibung: "Kühler (blauer) oder wärmer (röter), wie die Farbtemperatur eines Fernsehers.", kleinst: -1.0, groesst: 1.0, schritt: 0.1, standard: 0.0 },
    ],
};

impl Stufe {
    /// Stehen alle Regler auf ihrer Voreinstellung? (Dann darf ein Ausführer
    /// eine Stufe wie [`FARBE`] weglassen.)
    pub fn neutral(&self, werte: &[(&str, f32)]) -> bool {
        self.regler.iter().all(|r| werte.iter().find(|(n, _)| *n == r.name).is_none_or(|(_, w)| (w - r.standard).abs() < r.schritt / 2.0))
    }
}

/// Stufen in Reihenfolge, als ein Bildfilter.
#[derive(Debug, Clone, Copy)]
pub struct Kette {
    /// Kurzname (ASCII).
    pub name: &'static str,
    /// Anzeigename, deutsch.
    pub titel: &'static str,
    /// Ein Satz für das Menü, deutsch.
    pub beschreibung: &'static str,
    pub stufen: &'static [Stufe],
}

/// Die Bildfilter, die diese Kiste heute liefert.
pub const KETTEN: &[Kette] = &[
    Kette {
        name: "scharf",
        titel: "Scharf",
        beschreibung: "Harte Kanten, gleich große Punkte bei jeder Größe.",
        stufen: &[SCHARF_BILINEAR],
    },
    Kette {
        name: "mmpx",
        titel: "Glatt (MMPX)",
        beschreibung: "Treppen an schrägen Kanten geglättet, ohne neue Farben; danach scharf eingepaßt.",
        stufen: &[MMPX, SCHARF_BILINEAR],
    },
    Kette { name: "pixel", titel: "Pixel", beschreibung: "Nächster Punkt — bei krummen Faktoren ungleich breite Punkte.", stufen: &[NAECHSTER] },
    Kette { name: "weich", titel: "Weich", beschreibung: "Linear geglättet.", stufen: &[LINEAR] },
    Kette {
        name: "lcd",
        titel: "LCD-Raster",
        beschreibung: "Wie ein Handheld-Bildschirm: Punkte mit feinen dunklen Lücken.",
        stufen: &[LCD_RASTER],
    },
];

/// Eine Kette nach Kurzname.
pub fn kette(name: &str) -> Option<&'static Kette> {
    KETTEN.iter().find(|k| k.name == name)
}

/// Die Werte, die jede Stufe liest (`struct Rahmen` in `gemeinsam.wgsl`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rahmen {
    /// Größe der Quelltextur in Punkten.
    pub quelle: (u32, u32),
    /// Größe des Ziels (Viewport) in Bildpunkten.
    pub ziel: (u32, u32),
    /// Gezeigter Ausschnitt der Quelle in Texturkoordinaten (u0, v0, u1, v1).
    pub ausschnitt: [f32; 4],
    /// Bildzähler.
    pub bild: u32,
    /// Regler der Stufe (je Stufe belegt; heute ungenutzt).
    pub regler: [f32; 16],
}

impl Rahmen {
    /// Ganzes Bild, keine Regler.
    pub fn neu(quelle: (u32, u32), ziel: (u32, u32)) -> Rahmen {
        Rahmen { quelle, ziel, ausschnitt: [0.0, 0.0, 1.0, 1.0], bild: 0, regler: [0.0; 16] }
    }

    /// Die Regler einer Stufe: ihre Voreinstellungen, überschrieben mit
    /// `werte` (Schlüssel → Wert, in die Grenzen geklemmt).
    pub fn mit_reglern(mut self, s: &Stufe, werte: &[(&str, f32)]) -> Rahmen {
        for (i, r) in s.regler.iter().enumerate().take(16) {
            let w = werte.iter().find(|(n, _)| *n == r.name).map_or(r.standard, |(_, w)| *w);
            self.regler[i] = w.clamp(r.kleinst, r.groesst);
        }
        self
    }

    /// Das Layout der WGSL-Struktur (std140-gleich: vier `vec4` zu 16 Byte,
    /// dann `array<vec4<f32>, 4>`), Little Endian, 128 Byte.
    pub fn als_bytes(&self) -> [u8; 128] {
        let mut b = [0u8; 128];
        let mut f = |i: usize, v: f32| b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        let (qb, qh) = (self.quelle.0.max(1) as f32, self.quelle.1.max(1) as f32);
        let (zb, zh) = (self.ziel.0.max(1) as f32, self.ziel.1.max(1) as f32);
        for (i, v) in [qb, qh, 1.0 / qb, 1.0 / qh, zb, zh, 1.0 / zb, 1.0 / zh].into_iter().enumerate() {
            f(i, v);
        }
        for (i, v) in self.ausschnitt.into_iter().enumerate() {
            f(8 + i, v);
        }
        for (i, v) in self.regler.into_iter().enumerate() {
            f(16 + i, v);
        }
        b[48..52].copy_from_slice(&self.bild.to_le_bytes());
        b
    }

    /// Die Größe des Ziels einer Stufe für diesen Rahmen.
    pub fn zielgroesse(&self, s: &Stufe) -> (u32, u32) {
        match s.groesse {
            Groesse::Ziel => self.ziel,
            Groesse::QuelleMal(n) => (self.quelle.0 * n, self.quelle.1 * n),
        }
    }
}

#[cfg(test)]
mod proben {
    use super::*;

    /// Jede Stufe ist gültiges WGSL mit den vereinbarten Bindungen und
    /// Einstiegen — geprüft mit naga (dem Übersetzer in wgpu).
    #[test]
    fn jede_stufe_ist_gueltiges_wgsl() {
        for s in KETTEN.iter().flat_map(|k| k.stufen.iter()).chain([&FARBE]) {
            {
                let text = s.wgsl();
                let modul = naga::front::wgsl::parse_str(&text).unwrap_or_else(|e| panic!("{}: {}", s.name, e.emit_to_string(&text)));
                naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty())
                    .validate(&modul)
                    .unwrap_or_else(|e| panic!("{}: {e:?}", s.name));
                let einstiege: Vec<_> = modul.entry_points.iter().map(|e| e.name.as_str()).collect();
                assert_eq!(einstiege, ["ecke", "punkt"], "{}", s.name);
            }
        }
    }

    /// Das Byte-Layout paßt zur WGSL-Struktur: naga rechnet die Lage jedes
    /// Felds, und die Gesamtgröße ist 128.
    #[test]
    fn der_rahmen_hat_das_layout_der_wgsl_struktur() {
        let modul = naga::front::wgsl::parse_str(GEMEINSAM).expect("Kopf");
        let (_, t) = modul.types.iter().find(|(_, t)| t.name.as_deref() == Some("Rahmen")).expect("struct Rahmen");
        let naga::TypeInner::Struct { members, span } = &t.inner else { panic!("kein struct") };
        assert_eq!(*span, 128);
        let lage: Vec<(&str, u32)> = members.iter().map(|m| (m.name.as_deref().unwrap_or(""), m.offset)).collect();
        assert_eq!(lage, [("quelle", 0), ("ziel", 16), ("ausschnitt", 32), ("bild", 48), ("regler", 64)]);
        let r = Rahmen { bild: 7, ..Rahmen::neu((256, 224), (1024, 896)) };
        let b = r.als_bytes();
        assert_eq!(f32::from_le_bytes(b[0..4].try_into().unwrap()), 256.0);
        assert_eq!(f32::from_le_bytes(b[20..24].try_into().unwrap()), 896.0);
        assert_eq!(f32::from_le_bytes(b[40..44].try_into().unwrap()), 1.0);
        assert_eq!(u32::from_le_bytes(b[48..52].try_into().unwrap()), 7);
        assert_eq!(r.zielgroesse(&MMPX), (512, 448));
    }
}
