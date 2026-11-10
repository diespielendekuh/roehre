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
mod leistung;
mod moire;
mod szene3d;
mod wgsl;
mod wgsl_erzeugen;
mod wgsl_kette;
mod wgsl_leistung;

use std::path::{Path, PathBuf};
use std::time::Instant;

use gpu::Gpu;
use roehre_testbilder::Bild;
use roehre_wgpu::{Eingang, Einstellungen, Roehre};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let befehl = args.first().map(String::as_str).unwrap_or("hilfe");
    let opt = |kurz: &str| -> Option<String> {
        args.iter()
            .position(|a| a == kurz)
            .and_then(|i| args.get(i + 1))
            .cloned()
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
        "moire" => moire_messen(opt("-v")).map(|_| ()),
        "flaeche" => flaeche_vorfuehren(
            &opt("-v").unwrap_or("scart-rgb".into()),
            Path::new(&opt("-o").unwrap_or("flaeche.png".into())),
        ),
        "helligkeit" => helligkeit_messen().map(|_| ()),
        "kamera" => kamera_bogen(
            &opt("-v").unwrap_or("wohnzimmer-90er-composite".into()),
            Path::new(&opt("-o").unwrap_or("kamera.png".into())),
        ),
        "formate" => raum_formate(
            &opt("-v").unwrap_or("scart-rgb".into()),
            groesse(&opt("-g").unwrap_or("1920x1080".into())),
            Path::new(&opt("-o").unwrap_or("formate".into())),
        ),
        "raum" => raum_fotos(
            &opt("-v").unwrap_or("wohnzimmer-90er-composite".into()),
            groesse(&opt("-g").unwrap_or("1920x1080".into())),
            Path::new(&opt("-o").unwrap_or("raum".into())),
            opt("--wandbild").map(std::path::PathBuf::from),
            args.iter().any(|x| x == "--ohne-rohpixel"),
            args.iter().any(|x| x == "--ohne-korn") || args.iter().any(|x| x == "--nur-unschaerfe"),
            args.iter().any(|x| x == "--nur-unschaerfe"),
            opt("--zeit").and_then(|z| z.parse().ok()),
        ),
        "bewegung" => bewegung_messen(
            opt("-n").and_then(|n| n.parse().ok()).unwrap_or(4.0),
            Path::new(&opt("-o").unwrap_or("bewegung.png".into())),
        )
        .map(|_| ()),
        "leistung" => leistung::messen(
            &shader_ordner(),
            Path::new(&opt("-o").unwrap_or("leistung.md".into())),
            opt("-n").and_then(|n| n.parse().ok()).unwrap_or(120),
            args.iter().any(|a| a == "--nur-1080p"),
        ),
        "pruefe" => pruefe(args.iter().any(|a| a == "--neu")),
        "wgsl" => wgsl::pruefen(&shader_ordner()),
        "wgsl-leistung" => wgsl_leistung::messen(
            Path::new(&opt("-o").unwrap_or("wgsl-leistung.md".into())),
            opt("-n").and_then(|n| n.parse().ok()).unwrap_or(60),
        ),
        "wgsl-erzeugen" => wgsl_erzeugen::erzeugen(
            &shader_ordner(),
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../roehre-wgsl"),
        ),
        "hdr" => hdr_messen(groesse(&opt("-g").unwrap_or("1920x1080".into()))),
        _ => {
            eprintln!(
                "roehre-pruefstand <befehl>\n\n\
                 leistung [-o leistung.md] [-n 120] [--nur-1080p]   Leistung messen (Filter, 3D-Raum, Kosten je Regler)\n\
                 pruefe [--neu]                                    Referenzbilder, Moiré, Helligkeit, Strahl\n\
                 wgsl                                              versionsfreie Stufen (roehre-wgsl) gegen librashader und CPU\n\
                 wgsl-leistung [-o wgsl-leistung.md] [-n 60]       Zeit je Bild jeder versionsfreien Kette (1080p, 4K)\n\
                 foto, bogen, raum, formate, kamera, testbilder   Fotos\n\
                 moire, helligkeit, hdr, bewegung, umschalten      Einzelmessungen"
            );
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

/// Shader-Ordner: `ROEHRE_SHADER`, sonst `shader/` neben dem Programm (ausgelieferte Fassung),
/// sonst der Ordner im Quellbaum.
fn shader_ordner() -> PathBuf {
    if let Some(p) = std::env::var_os("ROEHRE_SHADER") {
        return PathBuf::from(p);
    }
    if let Some(neben) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("shader")))
    {
        if neben.join("eigen").is_dir() {
            return neben;
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shader")
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
    Eingang {
        breite: bild.breite,
        hoehe: bild.hoehe,
        pixel_seitenverhaeltnis: bild.pixel_seitenverhaeltnis,
    }
}

/// Rendert `bilder` Bilder hintereinander (Nachglühen und Verlauf brauchen Vorgänger) und
/// liefert das letzte.
fn rendere(
    gpu: &Gpu,
    roehre: &mut Roehre,
    bild: &Bild,
    g: (u32, u32),
    bilder: usize,
) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
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
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        lade_einstellungen(v)?,
        gpu::ZIELFORMAT,
    )?;
    let t = Instant::now();
    let ergebnis = rendere(&gpu, &mut roehre, &bild, g, bilder)?;
    ergebnis.save(aus)?;
    eprintln!(
        "{} → {} ({}×{}, {:.1} s auf {})",
        v,
        aus.display(),
        g.0,
        g.1,
        t.elapsed().as_secs_f64(),
        gpu.adapter_name
    );
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
    let bilder: Vec<(String, Bild)> = bilder
        .split(',')
        .map(|b| Ok((b.to_string(), lade_bild(b)?)))
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let voreinstellungen = roehre_modell::voreinstellungen();
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        Einstellungen::standard(),
        gpu::ZIELFORMAT,
    )?;

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
            let ax = (r.x as f64 + r.breite as f64 * fx - ab as f64 / 2.0)
                .clamp(0.0, (g.0 - ab) as f64) as u32;
            let ay = (r.y as f64 + r.hoehe as f64 * fy - ah as f64 / 2.0)
                .clamp(0.0, (g.1 - ah) as f64) as u32;
            let ausschnitt = image::imageops::crop_imm(&voll, ax, ay, ab, ah).to_image();
            let gesamt =
                image::imageops::resize(&voll, gb, gh, image::imageops::FilterType::Lanczos3);
            let basis = format!("{}_{}_{}x{}", v.schluessel, name, g.0, g.1);
            gesamt.save(ordner.join(format!("{basis}_gesamt.png")))?;
            ausschnitt.save(ordner.join(format!("{basis}_ausschnitt.png")))?;
            let x0 = spalte as u32 * (gb + ab + 8);
            let y0 = zeile as u32 * (gh + kopf);
            image::imageops::overlay(&mut uebersicht, &gesamt, x0 as i64, (y0 + kopf) as i64);
            image::imageops::overlay(
                &mut uebersicht,
                &ausschnitt,
                (x0 + gb) as i64,
                (y0 + kopf) as i64,
            );
            beschrifte(
                &mut uebersicht,
                x0 + 4,
                y0 + 6,
                &format!("{} · {}", v.name(), name),
            );
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
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        Einstellungen::voreinstellung("scart-rgb")?,
        gpu::ZIELFORMAT,
    )?;
    for ziel_v in ["wohnzimmer-90er-composite", "aus", "scart-rgb"] {
        let t = Instant::now();
        roehre.uebernehme(Einstellungen::voreinstellung(ziel_v)?);
        let mut bilder_waehrenddessen = 0;
        while !roehre.ist_bereit() {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            roehre.zeichne(
                &mut enc,
                &eingang,
                &eingang_von(&bild),
                &ansicht,
                (1280, 720),
            )?;
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

/// Moiré-Messung über Voreinstellungen, Ausgabegrößen und Eingänge (siehe `moire.rs`).
/// Grenzen der Moiré-Messung für den Prüflauf: ab 1080p höchstens 1,2 %, darunter 2,5 %;
/// Maske überall höchstens 0,5 %.
const MOIRE_GRENZE: (f64, f64, f64) = (1.2, 2.5, 0.5);

fn moire_messen(nur: Option<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut verstoesse = Vec::new();
    let gpu = Gpu::neu();
    let groessen = [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)];
    let eingaenge: [(&str, u32, u32, f64); 3] = [
        ("SNES 256×224", 256, 224, 8.0 / 7.0),
        ("N64 320×240", 320, 240, 1.0),
        ("PAL 256×239", 256, 239, 8.0 / 7.0),
    ];
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        Einstellungen::standard(),
        gpu::ZIELFORMAT,
    )?;
    println!(
        "| Voreinstellung | Eingang | Ausgabe | px je Zeile | Zeilenschwebung | px je Tripel | Maskenschwebung |"
    );
    println!("|---|---|---|---:|---:|---:|---:|");
    let liste: Vec<(String, Einstellungen)> = match nur.as_deref() {
        Some(pfad) if pfad.ends_with(".toml") => {
            vec![(pfad.to_string(), lade_einstellungen(pfad)?)]
        }
        _ => roehre_modell::voreinstellungen()
            .iter()
            .filter(|v| v.schluessel != "aus" && nur.as_deref().is_none_or(|n| n == v.schluessel))
            .map(|v| {
                Ok((
                    v.schluessel.to_string(),
                    Einstellungen::voreinstellung(v.schluessel)?,
                ))
            })
            .collect::<Result<_, roehre_modell::Fehler>>()?,
    };
    for (schluessel, e) in liste {
        roehre.uebernehme(e);
        roehre.warte_auf_kette()?;
        for (name, b, h, par) in eingaenge {
            let mut bild = Bild::neu(b, h, [128, 128, 128]);
            bild.pixel_seitenverhaeltnis = par;
            for g in groessen {
                let voll = rendere(&gpu, &mut roehre, &bild, g, 2)?;
                let r = roehre.flaeche(&eingang_von(&bild), g);
                let (zeilen, spalten) = moire::profile(&voll, r.x, r.y, r.breite, r.hoehe);
                // Sichtbare Zeilen: Overscan schneidet oben und unten ab
                let (_, osy) = roehre_kette::overscan_pixel(roehre.einstellungen(), (b, h));
                // Zeilen, die die Röhre zeichnet (Vorstufe: Zeilenverdopplung, virtuelle Zeilen)
                let roehrenzeilen = roehre_kette::virtuelle_groesse(
                    roehre.einstellungen(),
                    (b, h),
                    b as f64 * par / h as f64,
                    g.1,
                )
                .map_or(h, |v| v.1);
                let osy = osy as f64 * roehrenzeilen as f64 / h as f64;
                let pz = r.hoehe as f64 / (roehrenzeilen as f64 - osy);
                let pt = roehre
                    .letzter_plan()
                    .and_then(|p| p.maske)
                    .map(|m| m.pixel_je_triade)
                    .unwrap_or(3.0);
                let (sz, sm) = (
                    moire::schwebung(&zeilen, pz),
                    moire::schwebung(&spalten, pt),
                );
                println!(
                    "| {} | {} | {}×{} | {:.2} | {:.2} % | {:.0} | {:.2} % |",
                    schluessel, name, g.0, g.1, pz, sz, pt, sm
                );
                let grenze = if g.1 >= 1080 {
                    MOIRE_GRENZE.0
                } else {
                    MOIRE_GRENZE.1
                };
                // bewusst verneint: ein NaN-Messwert soll als Verstoß zählen
                #[allow(clippy::neg_cmp_op_on_partial_ord)]
                let verstoss = !(sz <= grenze) || !(sm <= MOIRE_GRENZE.2);
                if verstoss {
                    verstoesse.push(format!(
                        "Moiré {schluessel} {name} {}×{}: Zeilen {sz:.2} %, Maske {sm:.2} %",
                        g.0, g.1
                    ));
                }
            }
        }
    }
    Ok(verstoesse)
}

/// Halbgenaue Gleitkommazahl (IEEE 754 binary16) → f32.
fn f16(bits: u16) -> f32 {
    let vz = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = ((bits >> 10) & 0x1f) as i32;
    let man = (bits & 0x3ff) as f32;
    vz * match exp {
        0 => man * 2f32.powi(-24),
        31 => f32::INFINITY,
        e => (1.0 + man / 1024.0) * 2f32.powi(e - 15),
    }
}

/// PQ (SMPTE ST 2084) → nit.
fn pq_zu_nit(e: f64) -> f64 {
    let (m1, m2) = (2610.0 / 16384.0, 2523.0 / 4096.0 * 128.0);
    let (c1, c2, c3) = (
        3424.0 / 4096.0,
        2413.0 / 4096.0 * 32.0,
        2392.0 / 4096.0 * 32.0,
    );
    let p = e.powf(1.0 / m2);
    10000.0 * ((p - c1).max(0.0) / (c2 - c3 * p)).powf(1.0 / m1)
}

/// HDR-Messung: Leuchtdichten des Farbbalkens in scRGB und HDR10, je Voreinstellung.
fn hdr_messen(g: (u32, u32)) -> Ergebnis {
    use roehre_wgpu::Ausgabe;
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::farbbalken(320, 240);
    let eingang = gpu.eingang(&bild);
    println!(
        "| Voreinstellung | Transmission Weiß | 75-%-Weiß (scRGB) | 75-%-Weiß (HDR10) | 100-%-Weiß | hellster Punkt |"
    );
    println!("|---|---:|---:|---:|---:|---:|");
    for v in roehre_modell::voreinstellungen() {
        let e = Einstellungen::voreinstellung(v.schluessel)?;
        let mut werte = Vec::new();
        let mut transmission = 0.0;
        for ausgabe in [Ausgabe::scrgb(), Ausgabe::hdr10()] {
            let mut roehre =
                Roehre::neu(&gpu.device, &gpu.queue, shader_ordner(), e.clone(), ausgabe)?;
            let ziel = gpu.ziel_im_format(g.0, g.1, ausgabe.format);
            let ansicht = ziel.create_view(&Default::default());
            // Erstes Bild stößt die Eichung an, danach mit Messwert zeichnen
            for runde in 0..4 {
                let mut enc = gpu.device.create_command_encoder(&Default::default());
                roehre.zeichne(&mut enc, &eingang, &eingang_von(&bild), &ansicht, g)?;
                gpu.queue.submit([enc.finish()]);
                if runde == 0 {
                    gpu.warte();
                    roehre.warte_auf_eichung();
                }
            }
            transmission = roehre.transmission();
            if std::env::var_os("ROEHRE_PLAN").is_some() {
                eprintln!("{:?}", roehre.letzter_plan());
            }
            let r = roehre.flaeche(&eingang_von(&bild), g);
            let roh = gpu.lies_roh(
                &ziel,
                if ausgabe.format == wgpu::TextureFormat::Rgba16Float {
                    8
                } else {
                    4
                },
            );
            // Nit je Pixel (Leuchtdichte nach BT.709 bzw. BT.2020)
            let nit = |x: u32, y: u32| -> f64 {
                let i = (y * g.0 + x) as usize;
                if ausgabe.format == wgpu::TextureFormat::Rgba16Float {
                    let k = |j: usize| {
                        f16(u16::from_le_bytes([
                            roh[i * 8 + 2 * j],
                            roh[i * 8 + 2 * j + 1],
                        ])) as f64
                            * 80.0
                    };
                    0.2126 * k(0) + 0.7152 * k(1) + 0.0722 * k(2)
                } else {
                    let w = u32::from_le_bytes([
                        roh[i * 4],
                        roh[i * 4 + 1],
                        roh[i * 4 + 2],
                        roh[i * 4 + 3],
                    ]);
                    let k = |j: u32| pq_zu_nit(((w >> (10 * j)) & 0x3ff) as f64 / 1023.0);
                    0.2627 * k(0) + 0.6780 * k(1) + 0.0593 * k(2)
                }
            };
            let mittel = |fx0: f64, fx1: f64, fy0: f64, fy1: f64| -> f64 {
                let (x0, x1) = (
                    r.x + (r.breite as f64 * fx0) as u32,
                    r.x + (r.breite as f64 * fx1) as u32,
                );
                let (y0, y1) = (
                    r.y + (r.hoehe as f64 * fy0) as u32,
                    r.y + (r.hoehe as f64 * fy1) as u32,
                );
                let mut s = 0.0;
                for y in y0..y1 {
                    for x in x0..x1 {
                        s += nit(x, y);
                    }
                }
                s / ((x1 - x0) * (y1 - y0)) as f64
            };
            let weiss75 = mittel(0.25 / 7.0, 0.75 / 7.0, 0.2, 0.45);
            let weiss100 = mittel(0.02, 0.1, 0.78, 0.95);
            let mut hellster: f64 = 0.0;
            for y in r.y..r.y + r.hoehe {
                for x in r.x..r.x + r.breite {
                    hellster = hellster.max(nit(x, y));
                }
            }
            werte.push((weiss75, weiss100, hellster));
        }
        println!(
            "| {} | {:.3} | {:.0} nit | {:.0} nit | {:.0} nit | {:.0} nit |",
            v.schluessel, transmission, werte[0].0, werte[1].0, werte[0].1, werte[0].2
        );
    }
    Ok(())
}

/// Vorführung des Flächen-Modus: Bildschirm als Viereck in einer 3D-Szene, nah, schräg und
/// fern; obere Reihe naive Maske, untere Reihe bandbegrenzte Maske (roehre_maske).
fn flaeche_vorfuehren(v: &str, aus: &Path) -> Ergebnis {
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::pixelszene();
    let eingang = gpu.eingang(&bild);
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        lade_einstellungen(v)?,
        gpu::ZIELFORMAT,
    )?;
    let flaeche = roehre_wgpu::FlaechenZiel::neu(&gpu.device, 1440, 1080);
    let mut werte = None;
    for _ in 0..3 {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        werte = Some(roehre.zeichne_flaeche(&mut enc, &eingang, &eingang_von(&bild), &flaeche)?);
        gpu.queue.submit([enc.finish()]);
    }
    let werte = werte.expect("gezeichnet");
    eprintln!("Materialwerte: {werte:?}");
    let szene = szene3d::Szene::neu(&gpu.device, &shader_ordner(), gpu::ZIELFORMAT);
    let (b, h) = (640u32, 360u32);
    let seite = werte.seitenverhaeltnis;
    let ansichten = [
        (
            "nah",
            szene3d::kamera(b as f32 / h as f32, 0.22, 0.0, [0.12, -0.2]),
        ),
        (
            "schräg",
            szene3d::kamera(b as f32 / h as f32, 1.9, 62.0, [0.0, 0.0]),
        ),
        (
            "fern",
            szene3d::kamera(b as f32 / h as f32, 5.5, 0.0, [0.0, 0.0]),
        ),
    ];
    let mut bogen =
        image::RgbaImage::from_pixel(3 * b, 2 * (h + 24), image::Rgba([24, 24, 28, 255]));
    for (zeile, naiv) in [true, false].into_iter().enumerate() {
        for (spalte, (name, mvp)) in ansichten.iter().enumerate() {
            let ziel = gpu.ziel(b, h);
            szene.zeichne(
                &gpu.device,
                &gpu.queue,
                &ziel.create_view(&Default::default()),
                &flaeche,
                &werte,
                mvp,
                naiv,
            );
            let foto = gpu.lies(&ziel);
            let (x0, y0) = (spalte as u32 * b, zeile as u32 * (h + 24));
            image::imageops::overlay(&mut bogen, &foto, x0 as i64, (y0 + 24) as i64);
            let titel = format!(
                "{} · {}",
                if naiv { "naive Maske" } else { "bandbegrenzt" },
                name
            );
            beschrifte(&mut bogen, x0 + 4, y0 + 6, &titel);
        }
    }
    let _ = seite;
    bogen.save(aus)?;
    eprintln!("→ {}", aus.display());
    Ok(())
}

/// Lineare Leuchtdichte (BT.709) eines Bereichs der Röhrenfläche, Anteile 0…1.
fn leuchtdichte(
    bild: &image::RgbaImage,
    r: &roehre_wgpu::Rechteck,
    fx: (f64, f64),
    fy: (f64, f64),
) -> f64 {
    let lin = |v: u8| {
        let c = v as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (x0, x1) = (
        r.x + (r.breite as f64 * fx.0) as u32,
        r.x + (r.breite as f64 * fx.1) as u32,
    );
    let (y0, y1) = (
        r.y + (r.hoehe as f64 * fy.0) as u32,
        r.y + (r.hoehe as f64 * fy.1) as u32,
    );
    let mut s = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = bild.get_pixel(x, y);
            s += 0.2126 * lin(p[0]) + 0.7152 * lin(p[1]) + 0.0722 * lin(p[2]);
        }
    }
    s / ((x1 - x0) * (y1 - y0)) as f64
}

/// Helligkeit je Voreinstellung (SDR, 1080p): 75-%-Weißbalken und 40-%-Graustufe des
/// Farbbalkens, bezogen auf „Sauber/aus“. Ziel: 90–110 % beim Weiß.
fn helligkeit_messen() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut verstoesse = Vec::new();
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::farbbalken(320, 240);
    let g = (1920, 1080);
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        Einstellungen::standard(),
        gpu::ZIELFORMAT,
    )?;
    let mut bezug: Option<(f64, f64)> = None;
    println!("| Voreinstellung | 75-%-Weiß | Anteil | 40-%-Grau | Anteil |");
    println!("|---|---:|---:|---:|---:|");
    for v in roehre_modell::voreinstellungen() {
        roehre.uebernehme(Einstellungen::voreinstellung(v.schluessel)?);
        roehre.warte_auf_kette()?;
        let voll = rendere(&gpu, &mut roehre, &bild, g, 3)?;
        let r = roehre.flaeche(&eingang_von(&bild), g);
        // 75-%-Weiß: erster Balken; 40-%-Grau: vierte Stufe der Grautreppe (unten)
        let weiss = leuchtdichte(&voll, &r, (0.25 / 7.0, 0.75 / 7.0), (0.25, 0.5));
        let grau = leuchtdichte(&voll, &r, (0.40, 0.46), (0.80, 0.92));
        let (bw, bg) = *bezug.get_or_insert((weiss, grau));
        let (aw, ag) = (100.0 * weiss / bw, 100.0 * grau / bg);
        println!(
            "| {} | {:.3} | {:.0} % | {:.3} | {:.0} % |",
            v.schluessel, weiss, aw, grau, ag
        );
        if !(95.0..=105.0).contains(&aw) || ag < 85.0 {
            verstoesse.push(format!(
                "Helligkeit {}: Weiß {aw:.0} % (Soll 95–105), Grau {ag:.0} % (Soll ≥ 85)",
                v.schluessel
            ));
        }
    }
    Ok(verstoesse)
}

/// Prüflauf: Referenzbilder, Moiré- und Helligkeitsgrenzen. Mit `--neu` werden die
/// Referenzbilder neu geschrieben (nur nach bewusster, begutachteter Änderung!).
fn pruefe(neu: bool) -> Ergebnis {
    let ordner = Path::new(env!("CARGO_MANIFEST_DIR")).join("referenz");
    let diff_ordner = std::env::temp_dir().join("roehre-pruefung");
    std::fs::create_dir_all(&ordner)?;
    std::fs::create_dir_all(&diff_ordner)?;
    let mut verstoesse = Vec::new();

    // 1. Referenzbilder: Pixelszene je Voreinstellung, gerendert bei 960×540 (Fläche 720×540)
    let gpu = Gpu::neu();
    eprintln!("Adapter: {}", gpu.adapter_name);
    let bild = roehre_testbilder::pixelszene();
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        Einstellungen::standard(),
        gpu::ZIELFORMAT,
    )?;
    for v in roehre_modell::voreinstellungen() {
        roehre.uebernehme(Einstellungen::voreinstellung(v.schluessel)?);
        roehre.warte_auf_kette()?;
        let voll = rendere(&gpu, &mut roehre, &bild, (960, 540), 3)?;
        // Gespeichert wird nebeneinander: 1:1-Ausschnitt der Mitte (Maske, Zeilen, Signal)
        // und die verkleinerte Gesamtansicht (Geometrie, Ränder, Helligkeit).
        let mut ist = image::RgbaImage::new(320 + 320, 180);
        image::imageops::overlay(
            &mut ist,
            &image::imageops::crop_imm(&voll, 400, 230, 320, 180).to_image(),
            0,
            0,
        );
        image::imageops::overlay(
            &mut ist,
            &image::imageops::resize(&voll, 320, 180, image::imageops::FilterType::Triangle),
            320,
            0,
        );
        let pfad = ordner.join(format!("{}.png", v.schluessel));
        if neu || !pfad.exists() {
            ist.save(&pfad)?;
            eprintln!("Referenz geschrieben: {}", pfad.display());
            continue;
        }
        let soll = image::open(&pfad)?.to_rgba8();
        if soll.dimensions() != ist.dimensions() {
            verstoesse.push(format!("Referenz {}: andere Größe", v.schluessel));
            continue;
        }
        let mut abw: Vec<u8> = soll
            .as_raw()
            .iter()
            .zip(ist.as_raw())
            .map(|(a, b)| a.abs_diff(*b))
            .collect();
        let mittel = abw.iter().map(|&d| d as f64).sum::<f64>() / abw.len() as f64;
        let diff = image::RgbaImage::from_fn(ist.width(), ist.height(), |x, y| {
            let i = ((y * ist.width() + x) * 4) as usize;
            let d = abw[i].max(abw[i + 1]).max(abw[i + 2]).saturating_mul(8);
            image::Rgba([d, d, d, 255])
        });
        abw.sort_unstable();
        let p999 = abw[abw.len() * 999 / 1000];
        eprintln!(
            "Referenz {}: mittlere Abweichung {mittel:.3}, 99,9-%-Wert {p999}",
            v.schluessel
        );
        // Grenzen: lässt Rundungsunterschiede verschiedener Treiber zu, keine sichtbare Änderung
        if mittel > 1.0 || p999 > 24 {
            diff.save(diff_ordner.join(format!("{}_diff.png", v.schluessel)))?;
            ist.save(diff_ordner.join(format!("{}_ist.png", v.schluessel)))?;
            verstoesse.push(format!(
                "Referenz {}: mittlere Abweichung {mittel:.2}, 99,9 % {p999}",
                v.schluessel
            ));
        }
    }
    drop(roehre);

    // 2. Moiré und 3. Helligkeit
    verstoesse.extend(moire_messen(None)?);
    verstoesse.extend(helligkeit_messen()?);

    // 4. Strahlsimulation: Lichtsumme der Teilbilder = Original × Verstärkung
    for n in [2.0f32, 2.4, 4.0] {
        let abw = bewegung_messen(n, &diff_ordner.join(format!("bewegung_{n}.png")))?;
        if abw > 0.01 {
            verstoesse.push(format!(
                "Strahlsimulation n = {n}: Lichtsumme weicht um {abw:.3} ab (Soll ≤ 0,01)"
            ));
        }
    }

    if verstoesse.is_empty() {
        eprintln!("Prüflauf bestanden.");
        Ok(())
    } else {
        for v in &verstoesse {
            eprintln!("VERSTOSS: {v}");
        }
        eprintln!("Differenzbilder: {}", diff_ordner.display());
        Err(format!("{} Verstöße", verstoesse.len()).into())
    }
}

/// Strahlsimulation: Lichtsumme der Teilbilder gegen das Bild ohne Simulation, dazu ein Foto
/// der Teilbilder nebeneinander.
fn bewegung_messen(n: f32, aus: &Path) -> Result<f64, Box<dyn std::error::Error>> {
    use roehre_wgpu::Takt;
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::graukeil(320, 240);
    let eingang = gpu.eingang(&bild);
    let g = (1280u32, 720u32);
    let lin = |v: u8| {
        let c = v as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let mut e = Einstellungen::voreinstellung("scart-rgb")?;
    // Bezug: ohne Simulation
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        e.clone(),
        gpu::ZIELFORMAT,
    )?;
    let bezug = rendere(&gpu, &mut roehre, &bild, g, 3)?;
    let r = roehre.flaeche(&eingang_von(&bild), g);
    e.setze(
        "bewegung.modus",
        roehre_modell::Wert::Auswahl("strahl".into()),
    );
    e.setze("bewegung.lcd_schutz", roehre_modell::Wert::Schalter(false));
    let gewinn = e.zahl("bewegung.helligkeit");
    let mut roehre = Roehre::neu(&gpu.device, &gpu.queue, shader_ordner(), e, gpu::ZIELFORMAT)?;
    let ziel = gpu.ziel(g.0, g.1);
    let ansicht = ziel.create_view(&Default::default());
    // Echter Takt: Auffrischung r zeigt Spielbild floor(r / n). Bei nicht ganzzahligem n
    // wechseln sich lange und kurze Spielbilder ab (2,4: 3, 2, 3, 2, 2 …). Ausgewertet werden
    // so viele Auffrischungen, dass eine ganze Zahl von Spielbildern hineinpasst.
    let mut periode = 1usize;
    while ((periode as f64 * n as f64).round() - periode as f64 * n as f64).abs() > 1e-6
        && periode < 100
    {
        periode += 1;
    }
    let auswertung = (periode as f64 * n as f64).round() as usize;
    let vorlauf = 3 * auswertung;
    let mut fotos = Vec::new();
    for r in 0..vorlauf + auswertung {
        let neu = r == 0 || (r as f64 / n as f64).floor() != ((r - 1) as f64 / n as f64).floor();
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        let takt = Takt {
            neues_spielbild: neu,
            teilbilder_je_spielbild: n,
        };
        roehre.zeichne_im_takt(&mut enc, &eingang, &eingang_von(&bild), &ansicht, g, takt)?;
        gpu.queue.submit([enc.finish()]);
        if r >= vorlauf {
            fotos.push(gpu.lies(&ziel));
        }
    }
    eprintln!(
        "{} Auffrischungen ausgewertet ({} Spielbilder)",
        auswertung, periode
    );
    // Lichtsumme je Graustufe (oberer Streifen: 16 Stufen), Mitte jeder Stufe
    println!("| Stufe | ohne Simulation | Mittel der Teilbilder | Verhältnis | Soll |");
    println!("|---:|---:|---:|---:|---:|");
    let y0 = r.y + r.hoehe / 10;
    let y1 = r.y + r.hoehe * 3 / 10;
    let mut groesste_abweichung: f64 = 0.0;
    for stufe in 0..16u32 {
        let x0 = r.x + r.breite * (4 * stufe + 1) / 64;
        let x1 = r.x + r.breite * (4 * stufe + 3) / 64;
        let mittel = |b: &image::RgbaImage| {
            let mut s = 0.0;
            for y in y0..y1 {
                for x in x0..x1 {
                    s += lin(b.get_pixel(x, y)[1]);
                }
            }
            s / ((x1 - x0) * (y1 - y0)) as f64
        };
        let ohne = mittel(&bezug);
        let mit = fotos.iter().map(mittel).sum::<f64>() / fotos.len() as f64;
        let verh = if ohne > 1e-4 { mit / ohne } else { f64::NAN };
        if ohne > 0.01 {
            groesste_abweichung = groesste_abweichung.max((verh - gewinn).abs());
        }
        println!(
            "| {} | {:.4} | {:.4} | {:.3} | {:.3} |",
            stufe * 17,
            ohne,
            mit,
            verh,
            gewinn
        );
    }
    eprintln!("größte Abweichung vom Soll (ab 1 % Helligkeit): {groesste_abweichung:.3}");
    // Foto: Teilbilder nebeneinander, verkleinert
    let (fb, fh) = (g.0 / 2, g.1 / 2);
    let zeigen = fotos.len().min(8);
    let mut bogen = image::RgbaImage::new(fb * zeigen as u32, fh);
    for (i, f) in fotos.iter().take(zeigen).enumerate() {
        let k = image::imageops::resize(f, fb, fh, image::imageops::FilterType::Triangle);
        image::imageops::overlay(&mut bogen, &k, (i as u32 * fb) as i64, 0);
    }
    bogen.save(aus)?;
    Ok(groesste_abweichung)
}

/// Kontaktbogen der Raumkamera: Spalten 4:3, 16:9, 32:9; Zeilen weit, mittel, nah, nah links,
/// nah rechts, mittel leicht von links mit Gerät rechts.
fn kamera_bogen(v: &str, pfad: &Path) -> Ergebnis {
    use roehre_raum::{Ansicht, Kamera, Raum};
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::pixelszene();
    let eingang = gpu.eingang(&bild);
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        lade_einstellungen(v)?,
        gpu::ZIELFORMAT,
    )?;
    let flaeche = roehre_wgpu::FlaechenZiel::neu(&gpu.device, 1440, 1080);
    let mut raum = Raum::neu(&gpu.device, &gpu.queue, &shader_ordner(), gpu::ZIELFORMAT)?;
    let mut werte = None;
    for _ in 0..3 {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        werte = Some(roehre.zeichne_flaeche(&mut enc, &eingang, &eingang_von(&bild), &flaeche)?);
        gpu.queue.submit([enc.finish()]);
    }
    let werte = werte.expect("gezeichnet");
    let h = 270u32;
    let formate = [(4u32, 3u32), (16, 9), (32, 9)];
    let zeilen = [
        (0.0, 0.0, 0.0),
        (0.5, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (1.0, 0.0, -1.0),
        (1.0, 0.0, 1.0),
        (0.5, -0.5, 1.0),
    ];
    let breiten: Vec<u32> = formate.iter().map(|(a, b)| h * a / b).collect();
    let rand = 12u32;
    let gesamt_b = breiten.iter().sum::<u32>() + rand * (formate.len() as u32 + 1);
    let gesamt_h = zeilen.len() as u32 * (h + rand) + rand;
    let mut bogen =
        image::RgbaImage::from_pixel(gesamt_b, gesamt_h, image::Rgba([40, 40, 44, 255]));
    for (zi, (naehe, blick, lage)) in zeilen.iter().enumerate() {
        let mut x = rand;
        for (fi, b) in breiten.iter().enumerate() {
            let k = Kamera {
                naehe: *naehe,
                blickwinkel: *blick,
                lage: *lage,
                schwenk: false,
                ..Kamera::default()
            };
            let a = Ansicht::aus_kamera(&k, 0.0, *b as f32 / h as f32);
            let ziel = gpu.ziel(*b, h);
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            raum.zeichne(
                &gpu.device,
                &gpu.queue,
                &mut enc,
                &ziel.create_view(&Default::default()),
                (*b, h),
                &flaeche,
                &werte,
                &a,
            );
            gpu.queue.submit([enc.finish()]);
            let foto = gpu.lies(&ziel);
            image::imageops::overlay(
                &mut bogen,
                &foto,
                x as i64,
                (rand + zi as u32 * (h + rand)) as i64,
            );
            x += b + rand;
            let _ = fi;
        }
    }
    bogen.save(pfad)?;
    eprintln!("{}", pfad.display());
    Ok(())
}

/// 3D-Vorführraum: Fotos aus mehreren Blickwinkeln und Abständen.
#[allow(clippy::too_many_arguments)]
/// Der Raum mit dem Fernseher in den drei üblichen Bildformaten (8:7 für SNES pixelgenau, 4:3,
/// 16:9), je einmal mittel und nah: zum Ansehen, ob Gerät, Tisch und Gegenstände zueinander passen.
fn raum_formate(v: &str, g: (u32, u32), ordner: &Path) -> Ergebnis {
    std::fs::create_dir_all(ordner)?;
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::pixelszene();
    let eingang = gpu.eingang(&bild);
    let mut roehre = Roehre::neu(&gpu.device, &gpu.queue, shader_ordner(), lade_einstellungen(v)?, gpu::ZIELFORMAT)?;
    for (name, format) in [("8-7", 8.0f32 / 7.0), ("4-3", 4.0 / 3.0), ("16-9", 16.0 / 9.0)] {
        let breite = ((1080.0 * format / 2.0).round() as u32) * 2;
        let flaeche = roehre_wgpu::FlaechenZiel::neu(&gpu.device, breite, 1080);
        // ROEHRE_KONSOLE=n64 stellt die Konsole dazu (zum Ansehen)
        let konsole = roehre_raum::Konsole::aus(&std::env::var("ROEHRE_KONSOLE").unwrap_or_default());
        let mut raum = roehre_raum::Raum::neu_mit(&gpu.device, &gpu.queue, &shader_ordner(), gpu::ZIELFORMAT, format, konsole)?;
        let mut werte = None;
        for _ in 0..3 {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            werte = Some(roehre.zeichne_flaeche(&mut enc, &eingang, &eingang_von(&bild), &flaeche)?);
            gpu.queue.submit([enc.finish()]);
        }
        let werte = werte.expect("gezeichnet");
        for (stufe, naehe) in [("weit", 0.0f32), ("mittel", 0.5), ("nah", 0.85)] {
            let k = roehre_raum::Kamera { naehe, schwenk: false, bildformat: format, ..Default::default() };
            let a = roehre_raum::Ansicht::aus_kamera(&k, 2.0, g.0 as f32 / g.1 as f32);
            let ziel = gpu.ziel(g.0, g.1);
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            raum.zeichne(&gpu.device, &gpu.queue, &mut enc, &ziel.create_view(&Default::default()), g, &flaeche, &werte, &a);
            gpu.queue.submit([enc.finish()]);
            let pfad = ordner.join(format!("raum_{name}_{stufe}.png"));
            gpu.lies(&ziel).save(&pfad)?;
            eprintln!("{}", pfad.display());
        }
        // Mit Konsole: Nahaufnahmen von Controller und Konsole (zum Vergleich mit dem Original)
        if konsole != roehre_raum::Konsole::Keine && name == "4-3" {
            let tisch = 0.70f32;
            for (stufe, auge, blick, winkel) in [
                ("controller", [-0.03f32, tisch + 0.30, -0.72], [-0.03f32, tisch + 0.01, -0.36], 24.0f32),
                ("controller_oben", [-0.03, tisch + 0.55, -0.42], [-0.03, tisch, -0.37], 26.0),
                ("konsole", [0.47, tisch + 0.33, -0.80], [0.47, tisch + 0.02, -0.30], 28.0),
                ("konsole_oben", [0.47, tisch + 0.70, -0.38], [0.47, tisch, -0.30], 28.0),
                ("beide", [0.20, tisch + 0.42, -1.05], [0.22, tisch + 0.02, -0.32], 30.0),
            ] {
                let mut a = roehre_raum::Ansicht::sitzplatz(2.0, 1.0, 0.0);
                a.auge = auge;
                a.ziel = blick;
                a.blickwinkel = winkel;
                a.verschiebung = [0.0, 0.0];
                a.unschaerfe = 0.0;
                a.korn = 0.0;
                a.vignette = 0.0;
                let ziel = gpu.ziel(g.0, g.1);
                let mut enc = gpu.device.create_command_encoder(&Default::default());
                raum.zeichne(&gpu.device, &gpu.queue, &mut enc, &ziel.create_view(&Default::default()), g, &flaeche, &werte, &a);
                gpu.queue.submit([enc.finish()]);
                let pfad = ordner.join(format!("nah_{stufe}.png"));
                gpu.lies(&ziel).save(&pfad)?;
                eprintln!("{}", pfad.display());
            }
        }
    }
    Ok(())
}

fn raum_fotos(v: &str, g: (u32, u32), ordner: &Path, wandbild: Option<std::path::PathBuf>, ohne_rohpixel: bool, ohne_korn: bool, nur_unschaerfe: bool, zeit: Option<f32>) -> Ergebnis {
    use roehre_raum::{Ansicht, Raum};
    std::fs::create_dir_all(ordner)?;
    let gpu = Gpu::neu();
    let bild = roehre_testbilder::pixelszene();
    let eingang = gpu.eingang(&bild);
    let mut roehre = Roehre::neu(
        &gpu.device,
        &gpu.queue,
        shader_ordner(),
        lade_einstellungen(v)?,
        gpu::ZIELFORMAT,
    )?;
    let flaeche = roehre_wgpu::FlaechenZiel::neu(&gpu.device, 1440, 1080);
    let t = Instant::now();
    let mut raum = Raum::neu(&gpu.device, &gpu.queue, &shader_ordner(), gpu::ZIELFORMAT)?;
    eprintln!(
        "Raum aufgebaut: {} Dreiecke, {:.1} s",
        raum.dreiecke(),
        t.elapsed().as_secs_f64()
    );
    if let Some(pfad) = wandbild {
        let b = image::open(&pfad)?.to_rgba8();
        raum.wandbild_setzen(&gpu.queue, b.as_raw(), b.width(), b.height());
    }
    let mut werte = None;
    for _ in 0..3 {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        werte = Some(roehre.zeichne_flaeche(&mut enc, &eingang, &eingang_von(&bild), &flaeche)?);
        gpu.queue.submit([enc.finish()]);
    }
    let werte = werte.expect("gezeichnet");
    let mut seitlich = Ansicht::sitzplatz(0.0, 0.45, 0.0);
    seitlich.auge = [-1.1, 1.35, -1.4];
    seitlich.ziel = [-0.1, 0.95, 0.0];
    let mut detail = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    detail.auge = [0.55, 1.15, -0.75];
    detail.ziel = [0.12, 0.92, -0.1];
    let mut pflanze = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    pflanze.auge = [0.6, 1.3, -1.0];
    pflanze.ziel = [1.15, 1.05, 0.1];
    let mut wand = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    wand.auge = [0.3, 1.45, -0.6];
    wand.ziel = [0.6, 1.5, 1.05];
    let mut konsole = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    konsole.auge = [-0.25, 0.98, -0.72];
    konsole.ziel = [-0.52, 0.74, -0.2];
    let mut tischkante = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    tischkante.auge = [0.30, 1.02, -0.70];
    tischkante.ziel = [0.55, 0.69, -0.30];
    let mut bedienfeld = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    bedienfeld.auge = [0.12, 1.02, -0.62];
    bedienfeld.ziel = [0.235, 0.98, -0.2];
    bedienfeld.blickwinkel = 30.0;
    let mut fenster = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    fenster.auge = [-1.0, 1.35, -1.2];
    fenster.ziel = [-1.4, 1.45, 1.05];
    let mut fernbedienung = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    fernbedienung.auge = [0.10, 0.93, -0.58];
    fernbedienung.ziel = [0.22, 0.71, -0.34];
    fernbedienung.blickwinkel = 28.0;
    let mut heizung = Ansicht::sitzplatz(0.0, 1.0, 0.0);
    heizung.auge = [-0.95, 0.95, -0.55];
    heizung.ziel = [-1.35, 0.45, 0.95];
    // mitten in der Fahrt des Autos (Scheinwerferlicht im Zimmer)
    let mut auto = Ansicht::sitzplatz(15.0, 0.0, 0.0);
    auto.zeit = 15.0;
    let ganznah = Ansicht::aus_kamera(
        &roehre_raum::Kamera {
            naehe: 1.0,
            schwenk: false,
            ..Default::default()
        },
        0.0,
        g.0 as f32 / g.1 as f32,
    );
    let ansichten = [
        ("nah", Ansicht::sitzplatz(4.0, 0.85, 1.0)),
        ("mittel", Ansicht::sitzplatz(2.0, 0.5, 1.0)),
        ("ueberblick", Ansicht::sitzplatz(1.0, 0.0, 0.0)),
        ("seitlich", seitlich),
        ("detail", detail),
        ("pflanze", pflanze),
        ("wand", wand),
        ("kassetten", konsole),
        ("tischkante", tischkante),
        ("ganznah", ganznah),
        ("bedienfeld", bedienfeld),
        ("fenster", fenster),
        ("fernbedienung", fernbedienung),
        ("heizung", heizung),
        ("auto", auto),
    ];
    for (name, mut a) in ansichten {
        if let Some(z) = zeit {
            a.zeit = z;
        }
        a.rohpixel = !ohne_rohpixel;
        if ohne_korn {
            a.korn = 0.0;
        }
        if nur_unschaerfe {
            // über dem Raum liegt dann nur noch die Tiefenunschärfe
            a.vignette = 0.0;
            a.gluehen = 0.0;
        }
        let t = Instant::now();
        let ziel = gpu.ziel(g.0, g.1);
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        raum.zeichne(
            &gpu.device,
            &gpu.queue,
            &mut enc,
            &ziel.create_view(&Default::default()),
            g,
            &flaeche,
            &werte,
            &a,
        );
        gpu.queue.submit([enc.finish()]);
        let foto = gpu.lies(&ziel);
        let pfad = ordner.join(format!("raum_{name}_{}x{}.png", g.0, g.1));
        foto.save(&pfad)?;
        eprintln!("{} ({:.1} s)", pfad.display(), t.elapsed().as_secs_f64());
    }
    Ok(())
}
