//! `roehre-pruefstand wgsl-erzeugen`: übersetzt die MIT-Ketten (crt-beans)
//! mit librashaders eigener Übersetzung (slang → SPIR-V → naga → WGSL, wie
//! sein wgpu-Unterbau) und schreibt WGSL-Texte und Pass-Beschreibung nach
//! `crates/roehre-wgsl` (W-009, Teil 2). Nichts wird von Hand abgeschrieben;
//! was der Ausführer nicht nachbilden kann (Verlauf, Rückkopplung, Mipmaps,
//! seltene Werte), bricht das Erzeugen ab, statt still zu fehlen.

use std::fmt::Write as _;
use std::path::Path;

use librashader::presets::{ShaderFeatures, ShaderPreset, ShaderPresetPack};
use librashader::reflect::naga::{Naga, NagaLoweringOptions};
use librashader::reflect::semantics::{BindingStage, MemberOffset, ShaderSemantics, TextureSemantics, UniqueSemantics};
use librashader::reflect::targets::WGSL;
use librashader::reflect::{CompilePresetTarget, CompileShader, ReflectShader, SpirvCompilation};
use librashader::{FilterMode, ImageFormat, WrapMode};

type Fehler = Box<dyn std::error::Error + Send + Sync>;

/// Die Ketten, die erzeugt werden: (Kurzname, Rust-Name, Preset relativ zu shader/).
/// Gruppen: (Datei unter `src/erzeugt/`, Kopfzeile der Herkunft, Ketten).
const GRUPPEN: &[(&str, &str, &[(&str, &str, &str)])] = &[
    (
        "beans",
        "aus den slang-Presets von crt-beans\n// (Andrew Duffey, MIT, shader/fremd/mit/crt-beans/LICENSE) und dem eigenen Pass\n// helligkeit.slang",
        &[("beans-rgb", "BEANS_RGB", "eigen/beans/rgb.slangp"), ("beans-signal", "BEANS_SIGNAL", "eigen/beans/signal.slangp"), ("beans-schnell", "BEANS_SCHNELL", "eigen/beans/schnell.slangp")],
    ),
    (
        "pixelkunst",
        "aus ScaleFX (Sp00kyFox, MIT) und OmniScale\n// (Lior Halphon, MIT; slang von hunterk), shader/fremd/mit/{scalefx,omniscale}",
        &[("scalefx", "SCALEFX", "eigen/scalefx.slangp"), ("omniscale", "OMNISCALE", "eigen/omniscale.slangp")],
    ),
];

fn rand(w: WrapMode) -> &'static str {
    match w {
        WrapMode::ClampToBorder => "Rand::Schwarz",
        WrapMode::ClampToEdge => "Rand::Kante",
        WrapMode::Repeat => "Rand::Wiederholen",
        WrapMode::MirroredRepeat => "Rand::Spiegeln",
    }
}

fn format(f: ImageFormat) -> Result<&'static str, Fehler> {
    Ok(match f {
        ImageFormat::Unknown | ImageFormat::R8G8B8A8Unorm => "Format::Rgba8Unorm",
        ImageFormat::R8G8B8A8Srgb => "Format::Rgba8UnormSrgb",
        ImageFormat::R16G16B16A16Sfloat => "Format::Rgba16Float",
        ImageFormat::R32G32B32A32Sfloat => "Format::Rgba32Float",
        ImageFormat::A2B10G10R10UnormPack32 => "Format::Rgb10a2Unorm",
        andere => return Err(format!("Format {andere:?} kennt der Ausführer nicht").into()),
    })
}

fn skala(s: &librashader::presets::Scaling) -> String {
    use librashader::presets::{ScaleFactor, ScaleType};
    let f = match s.factor {
        ScaleFactor::Float(f) => f,
        ScaleFactor::Absolute(n) => n as f32,
    };
    match s.scale_type {
        ScaleType::Input => format!("Skala::Quelle({f:?})"),
        ScaleType::Viewport => format!("Skala::Ziel({f:?})"),
        ScaleType::Absolute => format!("Skala::Fest({})", f as u32),
        ScaleType::Original => format!("Skala::Original({f:?})"),
    }
}

fn felder(off: &MemberOffset, wert: &str, aus: &mut Vec<String>) {
    if let Some(o) = off.ubo {
        aus.push(format!("Feld {{ ort: Ort::Ubo, lage: {o}, wert: {wert} }}"));
    }
    if let Some(o) = off.push {
        aus.push(format!("Feld {{ ort: Ort::Push, lage: {o}, wert: {wert} }}"));
    }
}

/// Erzeugt alle Ketten nach `ziel` (= `crates/roehre-wgsl`).
pub fn erzeugen(shader: &Path, ziel: &Path) -> crate::Ergebnis {
    innen(shader, ziel).map_err(|e| e as Box<dyn std::error::Error>)
}

fn innen(shader: &Path, ziel: &Path) -> Result<(), Fehler> {
    for &(datei, herkunft, ketten) in GRUPPEN {
    let mut rs = format!(
        "// ERZEUGT von `roehre-pruefstand wgsl-erzeugen` {herkunft}.\n// Nicht von Hand ändern — neu erzeugen.\n// SPDX-License-Identifier: MIT\n\nuse crate::kette::*;\n\n"
    );
    for &(kurz, rust, preset_pfad) in ketten {
        let preset = ShaderPreset::try_parse(shader.join(preset_pfad), ShaderFeatures::ORIGINAL_ASPECT_UNIFORMS | ShaderFeatures::FRAMETIME_UNIFORMS)?;
        let dateinamen: Vec<String> = preset.passes.iter().map(|p| p.path.file_stem().map_or("pass".into(), |s| s.to_string_lossy().to_string())).collect();
        let pack = ShaderPresetPack::load_from_preset::<Fehler>(preset)?;
        let regler_vorgabe: Vec<(String, f32)> = pack.parameters.iter().map(|p| (p.name.to_string(), p.value)).collect();
        let texturen = pack.textures.clone();
        let (paesse, semantik): (Vec<_>, ShaderSemantics) =
            WGSL::compile_preset_passes::<SpirvCompilation, Naga, Fehler>(pack.passes, texturen.iter().map(|t| &t.meta))?;
        let ordner = ziel.join("wgsl/erzeugt").join(kurz);
        std::fs::create_dir_all(&ordner)?;
        let mut pass_texte = vec![];
        let mut regler: Vec<(String, f32, f32, f32)> = vec![];
        let n = paesse.len();
        for (index, (config, mut reflect)) in paesse.into_iter().enumerate() {
            let meta = &config.meta;
            if meta.mipmap_input {
                return Err(format!("{kurz} Pass {index}: mipmap_input kennt der Ausführer nicht").into());
            }
            let r = reflect.reflect(index, &semantik)?;
            let wgsl = reflect.compile(NagaLoweringOptions { write_pcb_as_ubo: true, sampler_bind_group: 1, suppress_derivative_uniformity: true })?;
            let name = dateinamen.get(index).cloned().unwrap_or_else(|| format!("pass{index}"));
            let stamm = format!("{index:02}-{name}");
            let kopf = format!(
                "// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — {preset_pfad}, Pass {index} »{name}« (Herkunft: shader/fremd/mit/).\n// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.\n"
            );
            std::fs::write(ordner.join(format!("{stamm}.ecke.wgsl")), format!("{kopf}{}", wgsl.vertex))?;
            std::fs::write(ordner.join(format!("{stamm}.punkt.wgsl")), format!("{kopf}{}", wgsl.fragment))?;
            let ecke_einstieg = wgsl.context.vertex.entry_points[0].name.clone();
            let punkt_einstieg = wgsl.context.fragment.entry_points[0].name.clone();

            // Puffer
            let puffer = |b: Option<u32>, groesse: u32, maske: BindingStage| match b {
                Some(b) => format!(
                    "Some(Puffer {{ nummer: {b}, groesse: {groesse}, ecke: {}, punkt: {} }})",
                    maske.contains(BindingStage::VERTEX),
                    maske.contains(BindingStage::FRAGMENT)
                ),
                None => "None".into(),
            };
            let ubo = r.ubo.as_ref().map_or("None".into(), |u| puffer(Some(u.binding), u.size, u.stage_mask));
            let push = r.push_constant.as_ref().map_or("None".into(), |p| puffer(p.binding, p.size, p.stage_mask));
            if r.push_constant.as_ref().is_some_and(|p| p.binding.is_none()) {
                return Err(format!("{kurz} Pass {index}: Push-Block ohne Bindung").into());
            }

            // Werte
            let mut f = vec![];
            for (sem, v) in &r.meta.unique_meta {
                let wert = match sem {
                    UniqueSemantics::MVP => "Wert::Mvp",
                    UniqueSemantics::Output => "Wert::Ausgabe",
                    UniqueSemantics::FinalViewport => "Wert::Endausgabe",
                    UniqueSemantics::FrameCount => "Wert::Bildzaehler",
                    UniqueSemantics::FrameDirection => "Wert::Richtung",
                    UniqueSemantics::OriginalAspect => "Wert::Seitenverhaeltnis",
                    andere => return Err(format!("{kurz} Pass {index}: Wert {andere:?} kennt der Ausführer nicht").into()),
                };
                felder(&v.offset, wert, &mut f);
            }
            let bild = |s: &librashader::reflect::semantics::Semantic<TextureSemantics>| -> Result<String, Fehler> {
                Ok(match s.semantics {
                    TextureSemantics::Original => "Bild::Original".into(),
                    TextureSemantics::Source => "Bild::Quelle".into(),
                    TextureSemantics::PassOutput => format!("Bild::Pass({})", s.index),
                    TextureSemantics::User => format!("Bild::Lut({})", s.index),
                    andere => return Err(format!("{kurz} Pass {index}: Bild {andere:?} kennt der Ausführer nicht").into()),
                })
            };
            for (s, v) in &r.meta.texture_size_meta {
                felder(&v.offset, &format!("Wert::Groesse({})", bild(s)?), &mut f);
            }
            for (name, v) in &r.meta.parameter_meta {
                felder(&v.offset, &format!("Wert::Regler({:?})", name.as_str()), &mut f);
                let p = config.data.parameters.get(name.as_str()).ok_or_else(|| format!("Regler {name} ohne #pragma parameter"))?;
                let standard = regler_vorgabe.iter().find(|(n, _)| n == name.as_str()).map_or(p.initial, |(_, w)| *w);
                if !regler.iter().any(|(n, ..)| n == name.as_str()) {
                    regler.push((name.to_string(), standard, p.minimum, p.maximum));
                }
            }
            f.sort();
            let mut b: Vec<(u32, String)> = vec![];
            for (s, t) in &r.meta.texture_meta {
                b.push((t.binding, bild(s)?));
            }
            b.sort();

            let fmt = format(meta.get_format_override().unwrap_or(config.data.format))?;
            let mut p = String::new();
            writeln!(p, "        Pass {{")?;
            writeln!(p, "            name: {name:?},")?;
            writeln!(p, "            ecke: include_str!(\"../../wgsl/erzeugt/{kurz}/{stamm}.ecke.wgsl\"),")?;
            writeln!(p, "            ecke_einstieg: {ecke_einstieg:?},")?;
            writeln!(p, "            punkt: include_str!(\"../../wgsl/erzeugt/{kurz}/{stamm}.punkt.wgsl\"),")?;
            writeln!(p, "            punkt_einstieg: {punkt_einstieg:?},")?;
            let (sx, sy) = if meta.scaling.valid { (skala(&meta.scaling.x), skala(&meta.scaling.y)) } else { ("Skala::Quelle(1.0)".into(), "Skala::Quelle(1.0)".into()) };
            // Der letzte Pass zeichnet ohnehin in den Viewport.
            let _ = n;
            writeln!(p, "            skala: ({sx}, {sy}),")?;
            writeln!(p, "            format: {fmt},")?;
            writeln!(p, "            linear: {},", meta.filter == FilterMode::Linear)?;
            writeln!(p, "            rand: {},", rand(meta.wrap_mode))?;
            writeln!(p, "            bildzaehler_modulo: {},", meta.frame_count_mod)?;
            writeln!(p, "            ubo: {ubo},")?;
            writeln!(p, "            push: {push},")?;
            writeln!(p, "            felder: &[{}],", f.join(", "))?;
            writeln!(p, "            bilder: &[{}],", b.iter().map(|(n, w)| format!("Bindung {{ nummer: {n}, bild: {w} }}")).collect::<Vec<_>>().join(", "))?;
            writeln!(p, "        }},")?;
            pass_texte.push(p);
        }
        // Nachschlagebilder als rohe RGBA-Dateien (ohne PNG-Entpacker beim Ausführer).
        let mut luts = vec![];
        for (i, t) in texturen.iter().enumerate() {
            let bild: Option<image::RgbaImage> = t.data.clone().into();
            let bild = bild.ok_or("Nachschlagebild leer")?;
            let datei = format!("lut{i}-{}.rgba", t.meta.name);
            std::fs::write(ordner.join(&datei), bild.as_raw())?;
            luts.push(format!(
                "        Lut {{ name: {:?}, breite: {}, hoehe: {}, rgba: include_bytes!(\"../../wgsl/erzeugt/{kurz}/{datei}\"), linear: {}, rand: {}, mipmap: {} }},\n",
                t.meta.name.as_str(),
                bild.width(),
                bild.height(),
                t.meta.filter_mode == FilterMode::Linear,
                rand(t.meta.wrap_mode),
                t.meta.mipmap
            ));
            if t.meta.mipmap {
                return Err(format!("{kurz}: Nachschlagebild {} mit Mipmaps kennt der Ausführer nicht", t.meta.name).into());
            }
        }
        regler.sort_by(|a, b| a.0.cmp(&b.0));
        writeln!(rs, "/// Erzeugt aus `shader/{preset_pfad}`.")?;
        writeln!(rs, "pub const {rust}: Erzeugt = Erzeugt {{")?;
        writeln!(rs, "    name: {kurz:?},")?;
        writeln!(rs, "    preset: {preset_pfad:?},")?;
        writeln!(rs, "    lizenz: \"MIT\",")?;
        writeln!(rs, "    paesse: &[\n{}    ],", pass_texte.concat())?;
        writeln!(rs, "    luts: &[\n{}    ],", luts.concat())?;
        writeln!(
            rs,
            "    regler: &[\n{}    ],",
            regler.iter().map(|(n, s, k, g)| format!("        Regler {{ name: {n:?}, standard: {s:?}, kleinst: {k:?}, groesst: {g:?} }},\n")).collect::<String>()
        )?;
        writeln!(rs, "}};\n")?;
        eprintln!("{kurz}: {} Pässe, {} Nachschlagebilder, {} Regler", pass_texte.len(), luts.len(), regler.len());
    }
    std::fs::create_dir_all(ziel.join("src/erzeugt"))?;
    std::fs::write(ziel.join(format!("src/erzeugt/{datei}.rs")), rs)?;
    }
    Ok(())
}
