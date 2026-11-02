//! # roehre-pruefstand
//!
//! Rendert ohne Fenster (auch auf der CPU mit lavapipe) und erzeugt Fotos, Kontaktbögen und
//! Messwerte. Befehle:
//!
//! ```text
//! roehre-pruefstand foto -v scart-rgb -b pixelszene -g 1920x1080 -o foto.png
//! roehre-pruefstand bogen -g 1920x1080 -o docs/bilder/etappe-1
//! roehre-pruefstand testbilder -o testbilder/
//! roehre-pruefstand umschalten
//! ```
//!
//! `-v` nimmt den Schlüssel einer Voreinstellung oder den Pfad einer TOML-Datei,
//! `-b` den Namen eines Testbilds oder den Pfad eines PNG.

mod gpu;

use std::path::{Path, PathBuf};
use std::time::Instant;

use gpu::Gpu;
use roehre_testbilder::Bild;
use roehre_wgpu::{Eingang, Einstellungen, Roehre};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let befehl = args.first().map(String::as_str).unwrap_or("hilfe");
    let opt = |kurz: &str| -> Option<String> {
        args.iter().position(|a| a == kurz).and_then(|i| args.get(i + 1)).cloned()
    };
    let ergebnis = match befehl {
        "foto" => foto(
            &opt("-v").unwrap_or("scart-rgb".into()),
            &opt("-b").unwrap_or("pixelszene".into()),
            groesse(&opt("-g").unwrap_or("1920x1080".into())),
            Path::new(&opt("-o").unwrap_or("foto.png".into())),
            opt("--bilder").and_then(|s| s.parse().ok()).unwrap_or(3),
        ),
        "bogen" => bogen(
            groesse(&opt("-g").unwrap_or("1920x1080".into())),
            Path::new(&opt("-o").unwrap_or("bogen".into())),
            opt("-b").unwrap_or("pixelszene,farbbalken,schrift,szene-3d".into()),
        ),
        "testbilder" => testbilder(Path::new(&opt("-o").unwrap_or("testbilder".into()))),
        "umschalten" => umschalten(),
        _ => {
            eprintln!("Befehle: foto, bogen, testbilder, umschalten (Einzelheiten in der Quelltext-Doku)");
            Ok(())
        }
    };
    if let Err(e) = ergebnis {
        eprintln!("Fehler: {e}");
        std::process::exit(1);
    }
}

type Ergebnis = Result<(), Box<dyn std::error::Error>>;

fn groesse(s: &str) -> (u32, u32) {
    let (b, h) = s.split_once('x').expect("Größe als BREITExHÖHE");
    (b.parse().expect("Breite"), h.parse().expect("Höhe"))
}

fn shader_ordner() -> PathBuf {
    std::env::var_os("ROEHRE_SHADER")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shader"))
}

fn lade_einstellungen(v: &str) -> Result<Einstellungen, Box<dyn std::error::Error>> {
    if v.ends_with(".toml") {
        let (e, hinweise) = Einstellungen::aus_toml(&std::fs::read_to_string(v)?)?;
        for h in hinweise {
            eprintln!("Hinweis: {h}");
        }
        Ok(e)
    } else {
        Ok(Einstellungen::voreinstellung(v)?)
    }
}

fn lade_bild(b: &str) -> Result<Bild, Box<dyn std::error::Error>> {
    if b.ends_with(".png") {
        return Ok(Bild::lade(b)?);
    }
    roehre_testbilder::alle()
        .into_iter()
        .find(|(n, _)| *n == b)
        .map(|(_, bild)| bild)
        .ok_or_else(|| format!("unbekanntes Testbild „{b}“").into())
}

fn eingang_von(bild: &Bild) -> Eingang {
    Eingang { breite: bild.breite, hoehe: bild.hoehe, pixel_seitenverhaeltnis: bild.pixel_seitenverhaeltnis }
}

/// Rendert `bilder` Bilder hintereinander (Nachglühen und Verlauf brauchen Vorgänger) und
/// liefert das letzte.
fn rendere(gpu: &Gpu, roehre: &mut Roehre, bild: &Bild, g: (u32, u32), bilder: usize) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    let eingang = gpu.eingang(bild);
    let ziel = gpu.ziel(g.0, g.1);
    let ansicht = ziel.create_view(&Default::default());
    for _ in 0..bilder.max(1) {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        roehre.zeichne(&mut enc, &eingang, &eingang_von(bild), &ansicht, g)?;
        gpu.queue.submit([enc.finish()]);
    }
    Ok(gpu.lies(&ziel))
}

fn foto(v: &str, b: &str, g: (u32, u32), aus: &Path, bilder: usize) -> Ergebnis {
    let gpu = Gpu::neu();
    let bild = lade_bild(b)?;
    let mut roehre = Roehre::neu(&gpu.device, &gpu.queue, shader_ordner(), lade_einstellungen(v)?, gpu::ZIELFORMAT)?;
    let t = Instant::now();
    let ergebnis = rendere(&gpu, &mut roehre, &bild, g, bilder)?;
    ergebnis.save(aus)?;
    eprintln!("{} → {} ({}×{}, {:.1} s auf {})", v, aus.display(), g.0, g.1, t.elapsed().as_secs_f64(), gpu.adapter_name);
    if let Some(plan) = roehre.letzter_plan() {
        eprintln!("Kette: {:?}, Maske: {:?}", plan.kette, plan.maske);
    }
    Ok(())
}

fn testbilder(ordner: &Path) -> Ergebnis {
    std::fs::create_dir_all(ordner)?;
    for (name, bild) in roehre_testbilder::alle() {
        bild.speichere(ordner.join(format!("{name}.png")))?;
    }
    Ok(())
}

/// Kontaktbogen: je Voreinstellung und Testbild eine verkleinerte Gesamtansicht und ein
/// Ausschnitt in Originalgröße (1:1, zeigt Maske und Zeilen), dazu ein Übersichtsbogen.
fn bogen(g: (u32, u32), ordner: &Path, bilder: String) -> Ergebnis {
    std::fs::create_dir_all(ordner)?;
    let gpu = Gpu::neu();
    eprintln!("Adapter: {}", gpu.adapter_name);
    let bilder: Vec<(String, Bild)> = bilder.split(',').map(|b| Ok((b.to_string(), lade_bild(b)?))).collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let voreinstellungen = roehre_modell::voreinstellungen();
    let mut roehre = Roehre::neu(&gpu.device, &gpu.queue, shader_ordner(), Einstellungen::standard(), gpu::ZIELFORMAT)?;

    let (ab, ah) = (480u32, 270u32); // Ausschnitt 1:1
    let (gb, gh) = (480u32, 270u32); // Gesamtansicht verkleinert
    let kopf = 24u32;
    let mut uebersicht = image::RgbaImage::from_pixel(
        bilder.len() as u32 * (gb + ab + 8),
        voreinstellungen.len() as u32 * (gh + kopf),
        image::Rgba([24, 24, 28, 255]),
    );
    for (zeile, v) in voreinstellungen.iter().enumerate() {
        roehre.uebernehme(Einstellungen::voreinstellung(v.schluessel)?);
        roehre.warte_auf_kette()?;
        for (spalte, (name, bild)) in bilder.iter().enumerate() {
            let t = Instant::now();
            let voll = rendere(&gpu, &mut roehre, bild, g, 3)?;
            let r = roehre.flaeche(&eingang_von(bild), g);
            // Ausschnitt um einen bildtypischen Punkt der Röhrenfläche
            let (fx, fy) = match name.as_str() {
                "pixelszene" => (0.62, 0.72),
                "schrift" => (0.30, 0.55),
                _ => (0.5, 0.5),
            };
            let ax = (r.x as f64 + r.breite as f64 * fx - ab as f64 / 2.0).clamp(0.0, (g.0 - ab) as f64) as u32;
            let ay = (r.y as f64 + r.hoehe as f64 * fy - ah as f64 / 2.0).clamp(0.0, (g.1 - ah) as f64) as u32;
            let ausschnitt = image::imageops::crop_imm(&voll, ax, ay, ab, ah).to_image();
            let gesamt = image::imageops::resize(&voll, gb, gh, image::imageops::FilterType::Lanczos3);
            let basis = format!("{}_{}_{}x{}", v.schluessel, name, g.0, g.1);
            gesamt.save(ordner.join(format!("{basis}_gesamt.png")))?;
            ausschnitt.save(ordner.join(format!("{basis}_ausschnitt.png")))?;
            let x0 = spalte as u32 * (gb + ab + 8);
            let y0 = zeile as u32 * (gh + kopf);
            image::imageops::overlay(&mut uebersicht, &gesamt, x0 as i64, (y0 + kopf) as i64);
            image::imageops::overlay(&mut uebersicht, &ausschnitt, (x0 + gb) as i64, (y0 + kopf) as i64);
            beschrifte(&mut uebersicht, x0 + 4, y0 + 6, &format!("{} · {}", v.name(), name));
            eprintln!("{basis}: {:.1} s", t.elapsed().as_secs_f64());
        }
    }
    uebersicht.save(ordner.join(format!("uebersicht_{}x{}.png", g.0, g.1)))?;
    Ok(())
}

fn beschrifte(ziel: &mut image::RgbaImage, x: u32, y: u32, text: &str) {
    let breite = roehre_testbilder::textbreite(text, 2).max(1) as u32 + 2;
    let mut b = Bild::neu(breite, 14, [24, 24, 28]);
    roehre_testbilder::text(&mut b, 0, 0, 2, [230, 230, 230], &text.to_uppercase());
    let img = image::RgbaImage::from_raw(b.breite, b.hoehe, b.rgba).expect("Größe");
    image::imageops::overlay(ziel, &img, x as i64, y as i64);
}

/// Misst das Umschalten: Wie lange dauert der Bau einer neuen Kette im Hintergrund, und
/// werden in der Zeit weiter Bilder gezeichnet?
fn umschalten() -> Ergebnis {
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::pixelszene();
    let eingang = gpu.eingang(&bild);
    let ziel = gpu.ziel(1280, 720);
    let ansicht = ziel.create_view(&Default::default());
    let mut roehre = Roehre::neu(&gpu.device, &gpu.queue, shader_ordner(), Einstellungen::voreinstellung("scart-rgb")?, gpu::ZIELFORMAT)?;
    for ziel_v in ["wohnzimmer-90er-composite", "aus", "scart-rgb"] {
        let t = Instant::now();
        roehre.uebernehme(Einstellungen::voreinstellung(ziel_v)?);
        let mut bilder_waehrenddessen = 0;
        while !roehre.ist_bereit() {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            roehre.zeichne(&mut enc, &eingang, &eingang_von(&bild), &ansicht, (1280, 720))?;
            gpu.queue.submit([enc.finish()]);
            gpu.warte();
            bilder_waehrenddessen += 1;
        }
        eprintln!(
            "→ {ziel_v}: Kette nach {:.2} s bereit, währenddessen {bilder_waehrenddessen} Bilder mit der alten Kette gezeichnet",
            t.elapsed().as_secs_f64()
        );
    }
    Ok(())
}
