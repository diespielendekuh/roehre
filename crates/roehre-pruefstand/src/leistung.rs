//! Leistungsmessung (Etappe 8, Wunsch W-008): Zeit auf der Grafikkarte je Bild, getrennt für
//! „nur Filter“ und „3D-Raum“, bei 1080p und 4K, dazu die Kosten einzelner Regler.
//!
//! Gemessen wird mit Zeitstempeln der Grafikkarte am Anfang und Ende jedes Bilds (reine
//! GPU-Zeit). Kann der Adapter keine Zeitstempel, misst der Befehl die Wanduhr von der Übergabe
//! bis zum Ende der Arbeit (enthält dann etwas Treiberaufwand) und sagt das in der Ausgabe.
//! Jedes Bild wird einzeln abgewartet; das ist langsamer als im Spiel, misst aber genau.

use std::fmt::Write as _;
use std::path::Path;
use std::time::Instant;

use roehre_raum::{Ansicht, Raum};
use roehre_wgpu::{Ausgabe, Eingang, Einstellungen, FlaechenZiel, Roehre, Takt, Wert};

use crate::gpu::Gpu;
use crate::Ergebnis;

/// Kennzahlen einer Messreihe in Millisekunden.
#[derive(Clone, Copy, Debug)]
struct Kennzahl {
    median: f64,
    p95: f64,
    mittel: f64,
}

struct Messer {
    abfrage: Option<(wgpu::QuerySet, wgpu::Buffer, wgpu::Buffer, f64)>,
}

impl Messer {
    fn neu(gpu: &Gpu) -> Self {
        let f = gpu.device.features();
        let abfrage = f.contains(wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS).then(|| {
            let qs = gpu.device.create_query_set(&wgpu::QuerySetDescriptor { label: Some("Leistung"), ty: wgpu::QueryType::Timestamp, count: 2 });
            let aufl = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Leistung: Auflösen"),
                size: 16,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let lesen = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Leistung: Lesen"),
                size: 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            (qs, aufl, lesen, gpu.queue.get_timestamp_period() as f64)
        });
        Self { abfrage }
    }

    fn methode(&self) -> &'static str {
        if self.abfrage.is_some() { "Zeitstempel der Grafikkarte (reine GPU-Zeit)" } else { "Wanduhr je Bild (Adapter ohne Zeitstempel; enthält Treiberaufwand)" }
    }

    /// Misst `bilder` Bilder nach `bilder / 6` Bildern Anlauf.
    fn miss(&self, gpu: &Gpu, bilder: usize, mut bild: impl FnMut(&mut wgpu::CommandEncoder)) -> Kennzahl {
        let anlauf = (bilder / 6).max(2);
        let mut zeiten = Vec::with_capacity(bilder);
        for i in 0..anlauf + bilder {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            let t = Instant::now();
            if let Some((qs, _, _, _)) = &self.abfrage {
                enc.write_timestamp(qs, 0);
            }
            bild(&mut enc);
            if let Some((qs, aufl, lesen, _)) = &self.abfrage {
                enc.write_timestamp(qs, 1);
                enc.resolve_query_set(qs, 0..2, aufl, 0);
                enc.copy_buffer_to_buffer(aufl, 0, lesen, 0, 16);
            }
            gpu.queue.submit([enc.finish()]);
            let ms = if let Some((_, _, lesen, periode)) = &self.abfrage {
                lesen.slice(..).map_async(wgpu::MapMode::Read, |_| {});
                gpu.warte();
                let d = lesen.slice(..).get_mapped_range().expect("abgebildet");
                let a = u64::from_le_bytes(d[0..8].try_into().expect("8"));
                let b = u64::from_le_bytes(d[8..16].try_into().expect("8"));
                drop(d);
                lesen.unmap();
                b.saturating_sub(a) as f64 * periode / 1e6
            } else {
                gpu.warte();
                t.elapsed().as_secs_f64() * 1000.0
            };
            if i >= anlauf {
                zeiten.push(ms);
            }
        }
        zeiten.sort_by(|a, b| a.total_cmp(b));
        let q = |p: f64| zeiten[((zeiten.len() - 1) as f64 * p).round() as usize];
        Kennzahl { median: q(0.5), p95: q(0.95), mittel: zeiten.iter().sum::<f64>() / zeiten.len() as f64 }
    }
}

const GROESSEN: [(&str, (u32, u32)); 2] = [("1080p", (1920, 1080)), ("4K", (3840, 2160))];

fn eingang(gpu: &Gpu) -> (wgpu::Texture, Eingang) {
    // N64-typisch: 320×240, quadratische Pixel
    let mut bild = roehre_testbilder::pixelszene();
    if bild.breite != 320 {
        let alt = bild.clone();
        bild = roehre_testbilder::Bild::neu(320, 240, [0, 0, 0]);
        for y in 0..240 {
            for x in 0..320 {
                bild.setze(x as i32, y as i32, alt.hole(x * alt.breite / 320, y * alt.hoehe / 240));
            }
        }
    }
    (gpu.eingang(&bild), Eingang { breite: bild.breite, hoehe: bild.hoehe, pixel_seitenverhaeltnis: 1.0 })
}

fn mit(mut e: Einstellungen, aenderungen: &[(&str, Wert)]) -> Einstellungen {
    for (k, w) in aenderungen {
        e.setze(k, w.clone());
    }
    e
}

/// Misst die Röhre allein (flach) mit diesen Einstellungen.
fn filter(gpu: &Gpu, messer: &Messer, shader: &Path, e: &Einstellungen, ausgabe: Ausgabe, g: (u32, u32), bilder: usize, takt: Option<f32>) -> Result<Kennzahl, Box<dyn std::error::Error>> {
    let format = ausgabe.format;
    let mut r = Roehre::neu(&gpu.device, &gpu.queue, shader, e.clone(), ausgabe)?;
    let (t, ein) = eingang(gpu);
    let ziel = gpu.ziel_im_format(g.0, g.1, format);
    let ansicht = ziel.create_view(&Default::default());
    let mut nr = 0u32;
    let mut fehler = None;
    let k = messer.miss(gpu, bilder, |enc| {
        let takt = match takt {
            Some(n) => Takt { neues_spielbild: nr % n as u32 == 0, teilbilder_je_spielbild: n },
            None => Takt::EIN_BILD,
        };
        nr += 1;
        if let Err(f) = r.zeichne_im_takt(enc, &t, &ein, &ansicht, g, takt) {
            fehler = Some(f);
        }
    });
    match fehler {
        Some(f) => Err(f.into()),
        None => Ok(k),
    }
}

/// Misst den 3D-Raum (Röhrenbild als Fläche + Raum) mit diesen Einstellungen.
fn raum(gpu: &Gpu, messer: &Messer, shader: &Path, e: &Einstellungen, g: (u32, u32), bilder: usize) -> Result<Kennzahl, Box<dyn std::error::Error>> {
    let mut r = Roehre::neu(&gpu.device, &gpu.queue, shader, e.clone(), crate::gpu::ZIELFORMAT)?;
    let mut raum = Raum::neu(&gpu.device, &gpu.queue, shader, crate::gpu::ZIELFORMAT)?;
    let flaeche = FlaechenZiel::neu(&gpu.device, 1440, 1080);
    let (t, ein) = eingang(gpu);
    let ziel = gpu.ziel(g.0, g.1);
    let ansicht = ziel.create_view(&Default::default());
    let a = Ansicht::aus_einstellungen(e, 3.0, g.0 as f32 / g.1 as f32);
    let mut fehler = None;
    let k = messer.miss(gpu, bilder, |enc| match r.zeichne_flaeche(enc, &t, &ein, &flaeche) {
        Ok(werte) => raum.zeichne(&gpu.device, &gpu.queue, enc, &ansicht, g, &flaeche, &werte, &a),
        Err(f) => fehler = Some(f),
    });
    match fehler {
        Some(f) => Err(f.into()),
        None => Ok(k),
    }
}

fn zelle(k: Kennzahl) -> String {
    format!("{:.2} ({:.2})", k.median, k.p95)
}

fn anteil(ms: f64) -> String {
    format!("{:.0} %", ms / (1000.0 / 60.0) * 100.0)
}

pub fn messen(shader: &Path, pfad: &Path, bilder: usize, nur_1080p: bool) -> Ergebnis {
    let gpu = Gpu::neu();
    let messer = Messer::neu(&gpu);
    let groessen: Vec<_> = GROESSEN.iter().filter(|(n, _)| !nur_1080p || *n == "1080p").collect();
    let info = format!("{} · {}", gpu.adapter_name, messer.methode());
    eprintln!("Leistungsmessung auf {info}, {bilder} Bilder je Fall");
    let mut md = String::new();
    writeln!(md, "# Leistung von roehre\n")?;
    writeln!(md, "- Grafikkarte: {}", gpu.adapter_name)?;
    writeln!(md, "- Methode: {} · {bilder} Bilder je Fall, Median (95-%-Wert in Klammern), Millisekunden je Bild", messer.methode())?;
    writeln!(md, "- Spielbild 320×240; Budget bei 60 Hz: 16,7 ms je Bild für alles (Spiel + roehre)")?;
    writeln!(md, "- roehre-Version {}\n", env!("CARGO_PKG_VERSION"))?;

    // 1. Nur Filter: alle Voreinstellungen
    let kopf: Vec<&str> = groessen.iter().map(|(n, _)| *n).collect();
    writeln!(md, "## Nur Filter (flach), je Voreinstellung\n")?;
    writeln!(md, "| Voreinstellung | {} | Anteil am 60-Hz-Budget ({}) |", kopf.join(" | "), kopf.last().expect("eine Größe"))?;
    writeln!(md, "|---|{}---|", "---|".repeat(kopf.len()))?;
    for v in roehre_modell::voreinstellungen() {
        let e = Einstellungen::voreinstellung(v.schluessel)?;
        let mut zellen = Vec::new();
        let mut letzte = 0.0;
        for (_, g) in &groessen {
            let k = filter(&gpu, &messer, shader, &e, Ausgabe::sdr(crate::gpu::ZIELFORMAT), *g, bilder, None)?;
            letzte = k.median;
            zellen.push(zelle(k));
        }
        eprintln!("  {}: {}", v.schluessel, zellen.join(" / "));
        writeln!(md, "| `{}` | {} | {} |", v.schluessel, zellen.join(" | "), anteil(letzte))?;
    }

    // 2. 3D-Raum
    writeln!(md, "\n## 3D-Raum (Voreinstellung `wohnzimmer-90er-composite`, Kamera Standard)\n")?;
    writeln!(md, "| Fall | {} | Anteil am 60-Hz-Budget ({}) |", kopf.join(" | "), kopf.last().expect("eine Größe"))?;
    writeln!(md, "|---|{}---|", "---|".repeat(kopf.len()))?;
    let basis = Einstellungen::voreinstellung("wohnzimmer-90er-composite")?;
    let raumfaelle: Vec<(&str, Einstellungen)> = vec![
        ("Standard (Unschärfe 0,3, Glühen 0,5)", basis.clone()),
        ("ohne Tiefenunschärfe", mit(basis.clone(), &[("raum.unschaerfe", Wert::Zahl(0.0))])),
        ("ohne Glühen", mit(basis.clone(), &[("raum.gluehen", Wert::Zahl(0.0))])),
        ("ohne beides", mit(basis.clone(), &[("raum.unschaerfe", Wert::Zahl(0.0)), ("raum.gluehen", Wert::Zahl(0.0))])),
        ("Nähe weit (mehr Raum im Bild)", mit(basis.clone(), &[("raum.naehe", Wert::Zahl(0.0))])),
    ];
    for (name, e) in &raumfaelle {
        let mut zellen = Vec::new();
        let mut letzte = 0.0;
        for (_, g) in &groessen {
            let k = raum(&gpu, &messer, shader, e, *g, bilder)?;
            letzte = k.median;
            zellen.push(zelle(k));
        }
        eprintln!("  Raum, {name}: {}", zellen.join(" / "));
        writeln!(md, "| {name} | {} | {} |", zellen.join(" | "), anteil(letzte))?;
    }

    // 3. Kosten je Regler (Röhre flach, größte gemessene Auflösung)
    let (gname, g) = **groessen.last().expect("eine Größe");
    writeln!(md, "\n## Kosten je Regler (flach, {gname}, ausgehend von `scart-rgb`)\n")?;
    writeln!(md, "| Änderung | ms je Bild | Unterschied |")?;
    writeln!(md, "|---|---|---|")?;
    let scart = Einstellungen::voreinstellung("scart-rgb")?;
    let grund = filter(&gpu, &messer, shader, &scart, Ausgabe::sdr(crate::gpu::ZIELFORMAT), g, bilder, None)?;
    writeln!(md, "| Grundlage `scart-rgb` | {} | – |", zelle(grund))?;
    let auswahl = |s: &str| Wert::Auswahl(s.into());
    let regler: Vec<(&str, Einstellungen, Ausgabe, Option<f32>)> = vec![
        ("Röhre aus (`allgemein.roehre`)", mit(scart.clone(), &[("allgemein.roehre", Wert::Schalter(false))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), None),
        ("Leistungsstufe leicht", mit(scart.clone(), &[("allgemein.leistung", auswahl("leicht"))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), None),
        ("Composite statt RGB", mit(scart.clone(), &[("signal.art", auswahl("composite"))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), None),
        ("S-Video statt RGB", mit(scart.clone(), &[("signal.art", auswahl("svideo"))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), None),
        ("Maske aus", mit(scart.clone(), &[("maske.typ", auswahl("aus"))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), None),
        (
            "Licht im Glas aus (Glow, Bloom, Halation 0)",
            mit(scart.clone(), &[("licht.glow", Wert::Zahl(0.0)), ("licht.bloom", Wert::Zahl(0.0)), ("licht.halation", Wert::Zahl(0.0))]),
            Ausgabe::sdr(crate::gpu::ZIELFORMAT),
            None,
        ),
        ("ohne Wölbung", mit(scart.clone(), &[("geometrie.woelbung_x", Wert::Zahl(0.0)), ("geometrie.woelbung_y", Wert::Zahl(0.0))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), None),
        ("Gehäuse Fernseher", mit(scart.clone(), &[("gehaeuse.typ", auswahl("fernseher"))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), None),
        ("HDR (scRGB)", scart.clone(), Ausgabe::scrgb(), None),
        ("Strahlsimulation 240 Hz (je Auffrischung)", mit(scart.clone(), &[("bewegung.modus", auswahl("strahl"))]), Ausgabe::sdr(crate::gpu::ZIELFORMAT), Some(4.0)),
    ];
    for (name, e, ausgabe, takt) in regler {
        // Im Takt wechseln sich Auffrischungen mit und ohne neues Spielbild ab: dort zählt das
        // Mittel über einen ganzen Zyklus, nicht der Median.
        let bilder_takt = takt.map(|n| (bilder / n as usize).max(1) * n as usize).unwrap_or(bilder);
        let k = filter(&gpu, &messer, shader, &e, ausgabe, g, bilder_takt, takt)?;
        let wert = if takt.is_some() { k.mittel } else { k.median };
        eprintln!("  {name}: {wert:.2} ms");
        let z = if takt.is_some() { format!("{:.2} (Mittel je Auffrischung)", k.mittel) } else { zelle(k) };
        writeln!(md, "| {name} | {z} | {:+.2} ms |", wert - grund.median)?;
    }
    writeln!(
        md,
        "\nBei der Strahlsimulation zählt jede Bildschirmauffrischung: bei 240 Hz fällt der Wert viermal je Spielbild an, das Budget je Auffrischung ist 4,2 ms. Ein Gehäuse kann billiger sein als keins, weil das Röhrenbild darin kleiner ist."
    )?;
    std::fs::write(pfad, &md)?;
    eprintln!("{}", pfad.display());
    println!("{md}");
    Ok(())
}
