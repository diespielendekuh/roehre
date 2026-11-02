//! # roehre-vorschau
//!
//! Vorschau-Programm für roehre: zeigt jede Voreinstellung und jede Option auf Testbildern,
//! eigenen Bildern oder Bildfolgen, mit Vergleich per Schieber, Lupe, Bildschirmfotos und dem
//! 3D-Vorführraum.
//!
//! ```text
//! cargo run --release -p roehre-vorschau [-- bild.png | ordner_mit_pngs]
//! ```
//!
//! Die Oberfläche (egui) gehört nur zu diesem Programm, nicht zur Bibliothek: Spiele bauen
//! ihr Menü selbst aus dem Katalog (`roehre_modell::katalog`). Genau das tut auch die
//! Seitenleiste hier, sie ist damit zugleich eine Probe, ob der Katalog dafür genügt.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use eframe::egui;
use roehre_modell::{festwerte, katalog, Gruppe, OptionsArt, Wert};
use roehre_testbilder::Bild;
use roehre_wgpu::{Eingang, Einstellungen, FlaechenZiel, Roehre};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn shader_ordner() -> PathBuf {
    std::env::var_os("ROEHRE_SHADER").map(PathBuf::from).unwrap_or_else(|| {
        let neben = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("shader")));
        match neben {
            Some(p) if p.is_dir() => p,
            _ => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shader"),
        }
    })
}

fn main() -> eframe::Result {
    let mut setup = egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    setup.device_descriptor = Arc::new(|adapter: &wgpu::Adapter| wgpu::DeviceDescriptor {
        label: Some("roehre-vorschau"),
        required_features: adapter.features() & roehre_wgpu::empfohlene_features(),
        required_limits: adapter.limits(),
        ..Default::default()
    });
    let mut optionen = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default().with_title("roehre – Vorschau").with_inner_size([1600.0, 950.0]),
        ..Default::default()
    };
    optionen.wgpu_options.wgpu_setup = egui_wgpu::WgpuSetup::CreateNew(setup);
    let start_pfad = std::env::args().nth(1).map(PathBuf::from);
    eframe::run_native(
        "roehre-vorschau",
        optionen,
        Box::new(move |cc| Ok(Box::new(Vorschau::neu(cc, start_pfad).map_err(|e| e.to_string())?))),
    )
}

/// Woher das Bild kommt.
enum Quelle {
    Testbild(&'static str),
    Datei(PathBuf),
    Folge(PathBuf, Vec<Bild>),
}

#[derive(PartialEq, Clone, Copy)]
enum Ansicht {
    Flach,
    Raum,
}

/// Eine Zieltextur samt egui-Kennung.
struct Ziel {
    textur: wgpu::Texture,
    ansicht: wgpu::TextureView,
    id: egui::TextureId,
    groesse: (u32, u32),
}

struct Vorschau {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>,
    roehre_a: Roehre,
    roehre_b: Roehre,
    e_a: Einstellungen,
    e_b: Einstellungen,
    vorein_a: String,
    vorein_b: String,
    quelle: Quelle,
    bild: Bild,
    eingang: Option<(wgpu::Texture, (u32, u32))>,
    bild_neu: bool,
    schwenk_test: bool,
    bildnummer: usize,
    ziel_a: Option<Ziel>,
    ziel_b: Option<Ziel>,
    vergleich: bool,
    schieber: f32,
    lupe: bool,
    lupe_faktor: f32,
    ansicht: Ansicht,
    raum: roehre_raum::Raum,
    flaeche: FlaechenZiel,
    foto_groesse: usize,
    meldung: String,
    start: Instant,
    zeitmessung: (f64, u32),
}

impl Vorschau {
    fn neu(cc: &eframe::CreationContext, start_pfad: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let rs = cc.wgpu_render_state.as_ref().ok_or("eframe ohne wgpu")?;
        let (device, queue) = (rs.device.clone(), rs.queue.clone());
        let e_a = Einstellungen::voreinstellung("scart-rgb")?;
        let e_b = Einstellungen::voreinstellung("aus")?;
        let roehre_a = Roehre::neu(&device, &queue, shader_ordner(), e_a.clone(), FORMAT)?;
        let roehre_b = Roehre::neu(&device, &queue, shader_ordner(), e_b.clone(), FORMAT)?;
        let raum = roehre_raum::Raum::neu(&device, &queue, &shader_ordner(), FORMAT)?;
        // Eigenes Wandbild: Umgebungsvariable ROEHRE_WANDBILD oder lokal/wandbild.png (nicht im
        // Repository, siehe .gitignore)
        let wandbild = std::env::var_os("ROEHRE_WANDBILD").map(std::path::PathBuf::from).unwrap_or_else(|| "lokal/wandbild.png".into());
        if let Ok(b) = image::open(&wandbild) {
            let b = b.to_rgba8();
            raum.wandbild_setzen(&queue, b.as_raw(), b.width(), b.height());
        }
        let flaeche = FlaechenZiel::neu(&device, 1440, 1080);
        let mut v = Self {
            device,
            queue,
            renderer: rs.renderer.clone(),
            roehre_a,
            roehre_b,
            e_a,
            e_b,
            vorein_a: "scart-rgb".into(),
            vorein_b: "aus".into(),
            quelle: Quelle::Testbild("pixelszene"),
            bild: roehre_testbilder::pixelszene(),
            eingang: None,
            bild_neu: true,
            schwenk_test: false,
            bildnummer: 0,
            ziel_a: None,
            ziel_b: None,
            vergleich: false,
            schieber: 0.5,
            lupe: false,
            lupe_faktor: 6.0,
            ansicht: Ansicht::Flach,
            raum,
            flaeche,
            foto_groesse: 0,
            meldung: String::new(),
            start: Instant::now(),
            zeitmessung: (0.0, 0),
        };
        if let Some(p) = start_pfad {
            v.lade_pfad(&p);
        }
        Ok(v)
    }

    fn lade_pfad(&mut self, p: &Path) {
        if p.is_dir() {
            let mut dateien: Vec<PathBuf> = std::fs::read_dir(p)
                .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "png")).collect())
                .unwrap_or_default();
            dateien.sort();
            let bilder: Vec<Bild> = dateien.iter().take(900).filter_map(|d| Bild::lade(d).ok()).collect();
            if bilder.is_empty() {
                self.meldung = format!("Keine PNG-Dateien in {}", p.display());
                return;
            }
            self.meldung = format!("Bildfolge: {} Bilder", bilder.len());
            self.bild = bilder[0].clone();
            self.quelle = Quelle::Folge(p.into(), bilder);
        } else if p.extension().is_some_and(|x| x == "png") {
            match Bild::lade(p) {
                Ok(b) => {
                    self.bild = b;
                    self.quelle = Quelle::Datei(p.into());
                    self.meldung = format!("Geladen: {}", p.display());
                }
                Err(e) => self.meldung = format!("Nicht lesbar: {e}"),
            }
        } else {
            self.meldung = "Nur PNG-Bilder oder Ordner mit PNG-Bildern. Videos vorher zerlegen, z. B. \
                            ffmpeg -i video.mp4 bilder/%04d.png"
                .into();
        }
        self.bild_neu = true;
    }

    /// Lädt das aktuelle Bild (bei Bildfolge und Schwenk-Test je Bild) auf die GPU.
    fn eingang_aktualisieren(&mut self) {
        let mut b = match &self.quelle {
            Quelle::Folge(_, bilder) => {
                self.bild_neu = true;
                bilder[self.bildnummer % bilder.len()].clone()
            }
            _ => self.bild.clone(),
        };
        if self.schwenk_test {
            // Waagerechter Schwenk um 2 Pixel je Bild: zeigt Bewegungsunschärfe und Nachglühen
            let versatz = (self.bildnummer * 2) as u32 % b.breite;
            let alt = b.rgba.clone();
            for y in 0..b.hoehe {
                for x in 0..b.breite {
                    let q = (((x + versatz) % b.breite + y * b.breite) * 4) as usize;
                    let z = ((x + y * b.breite) * 4) as usize;
                    b.rgba[z..z + 4].copy_from_slice(&alt[q..q + 4]);
                }
            }
            self.bild_neu = true;
        }
        if !self.bild_neu && self.eingang.is_some() {
            return;
        }
        let groesse = (b.breite, b.hoehe);
        if self.eingang.as_ref().map(|e| e.1) != Some(groesse) {
            let t = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Spielbild"),
                size: wgpu::Extent3d { width: b.breite, height: b.hoehe, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.eingang = Some((t, groesse));
        }
        let t = &self.eingang.as_ref().expect("angelegt").0;
        self.queue.write_texture(
            t.as_image_copy(),
            &b.rgba,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * b.breite), rows_per_image: None },
            t.size(),
        );
        self.bild = b;
        self.bild_neu = false;
    }

    fn ziel_sichern(platz: &mut Option<Ziel>, device: &wgpu::Device, renderer: &Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>, groesse: (u32, u32)) {
        if platz.as_ref().map(|z| z.groesse) == Some(groesse) {
            return;
        }
        let textur = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Vorschau-Ziel"),
            size: wgpu::Extent3d { width: groesse.0, height: groesse.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let ansicht = textur.create_view(&Default::default());
        let mut r = renderer.write();
        let id = match platz {
            Some(alt) => {
                r.update_egui_texture_from_wgpu_texture(device, &ansicht, wgpu::FilterMode::Nearest, alt.id);
                alt.id
            }
            None => r.register_native_texture(device, &ansicht, wgpu::FilterMode::Nearest),
        };
        *platz = Some(Ziel { textur, ansicht, id, groesse });
    }

    fn eingang_info(&self) -> Eingang {
        Eingang { breite: self.bild.breite, hoehe: self.bild.hoehe, pixel_seitenverhaeltnis: self.bild.pixel_seitenverhaeltnis }
    }

    /// Zeichnet A (und B beim Vergleich) bzw. den Raum in die Zieltexturen.
    fn zeichnen(&mut self, groesse: (u32, u32)) {
        self.eingang_aktualisieren();
        Self::ziel_sichern(&mut self.ziel_a, &self.device, &self.renderer, groesse);
        let eingang = self.eingang_info();
        let t0 = Instant::now();
        let mut enc = self.device.create_command_encoder(&Default::default());
        let quelle = &self.eingang.as_ref().expect("hochgeladen").0;
        let ergebnis = match self.ansicht {
            Ansicht::Flach => {
                let a = self.ziel_a.as_ref().expect("gesichert");
                let r = self.roehre_a.zeichne(&mut enc, quelle, &eingang, &a.ansicht, groesse).map(|_| ());
                if self.vergleich {
                    Self::ziel_sichern(&mut self.ziel_b, &self.device, &self.renderer, groesse);
                    let b = self.ziel_b.as_ref().expect("gesichert");
                    r.and(self.roehre_b.zeichne(&mut enc, quelle, &eingang, &b.ansicht, groesse).map(|_| ()))
                } else {
                    r
                }
            }
            Ansicht::Raum => self.roehre_a.zeichne_flaeche(&mut enc, quelle, &eingang, &self.flaeche).map(|werte| {
                let a = roehre_raum::Ansicht::aus_einstellungen(&self.e_a, self.start.elapsed().as_secs_f32(), groesse.0 as f32 / groesse.1.max(1) as f32);
                let z = self.ziel_a.as_ref().expect("gesichert");
                self.raum.zeichne(&self.device, &self.queue, &mut enc, &z.ansicht, groesse, &self.flaeche, &werte, &a);
            }),
        };
        if let Err(e) = ergebnis {
            self.meldung = format!("Fehler beim Zeichnen: {e}");
        }
        self.queue.submit([enc.finish()]);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        self.zeitmessung = (self.zeitmessung.0 * 0.95 + ms * 0.05, self.zeitmessung.1 + 1);
        self.bildnummer += 1;
    }

    /// Bildschirmfoto der Ansicht A in der gewählten Größe als PNG.
    fn foto(&mut self, fenster: (u32, u32)) {
        let groesse = [fenster, (1920, 1080), (2560, 1440), (3840, 2160)][self.foto_groesse];
        let mut ziel = None;
        Self::ziel_sichern(&mut ziel, &self.device, &self.renderer, groesse);
        let ziel = ziel.expect("angelegt");
        let eingang = self.eingang_info();
        let mut enc = self.device.create_command_encoder(&Default::default());
        let quelle = &self.eingang.as_ref().expect("hochgeladen").0;
        let ok = match self.ansicht {
            Ansicht::Flach => self.roehre_a.zeichne(&mut enc, quelle, &eingang, &ziel.ansicht, groesse).map(|_| ()),
            Ansicht::Raum => self.roehre_a.zeichne_flaeche(&mut enc, quelle, &eingang, &self.flaeche).map(|werte| {
                let a = roehre_raum::Ansicht::aus_einstellungen(&self.e_a, self.start.elapsed().as_secs_f32(), groesse.0 as f32 / groesse.1.max(1) as f32);
                self.raum.zeichne(&self.device, &self.queue, &mut enc, &ziel.ansicht, groesse, &self.flaeche, &werte, &a);
            }),
        };
        self.queue.submit([enc.finish()]);
        if let Err(e) = ok {
            self.meldung = format!("Foto fehlgeschlagen: {e}");
            return;
        }
        let bild = lies(&self.device, &self.queue, &ziel.textur);
        self.renderer.write().free_texture(&ziel.id);
        let _ = std::fs::create_dir_all("bildschirmfotos");
        let pfad = PathBuf::from(format!(
            "bildschirmfotos/roehre_{}_{}x{}_{}.png",
            self.vorein_a,
            groesse.0,
            groesse.1,
            self.start.elapsed().as_millis()
        ));
        self.meldung = match bild.save(&pfad) {
            Ok(()) => format!("Foto gespeichert: {}", pfad.display()),
            Err(e) => format!("Foto nicht gespeichert: {e}"),
        };
    }
}

/// Liest eine Textur (Rgba8) zurück.
fn lies(device: &wgpu::Device, queue: &wgpu::Queue, t: &wgpu::Texture) -> image::RgbaImage {
    let (b, h) = (t.width(), t.height());
    let zeile = (4 * b).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let puffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Foto"),
        size: (zeile * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &puffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(zeile), rows_per_image: None },
        },
        t.size(),
    );
    queue.submit([enc.finish()]);
    puffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let daten = puffer.slice(..).get_mapped_range().expect("abgebildet");
    let mut px = Vec::with_capacity((4 * b * h) as usize);
    for y in 0..h {
        let a = (y * zeile) as usize;
        px.extend_from_slice(&daten[a..a + (4 * b) as usize]);
    }
    drop(daten);
    puffer.unmap();
    image::RgbaImage::from_raw(b, h, px).expect("Größe")
}

/// Regler für alle Optionen einer Einstellung, nach Gruppen. Liefert true bei Änderung.
fn optionen_ui(ui: &mut egui::Ui, e: &mut Einstellungen, kennung: &str, gruppen: &[Gruppe]) -> bool {
    let mut geaendert = false;
    for &gruppe in gruppen {
        egui::CollapsingHeader::new(gruppe.name().de).id_salt((kennung, gruppe.schluessel())).show(ui, |ui| {
            for o in katalog().iter().filter(|o| o.gruppe == gruppe) {
                let wirkt = o.wirkt_wenn.is_none_or(|(s, werte)| werte.contains(&e.auswahl(s)));
                ui.add_enabled_ui(wirkt, |ui| {
                    let antwort = match o.art {
                        OptionsArt::Zahl { min, max, schritt, einheit, .. } => {
                            let mut z = e.zahl(o.schluessel);
                            let mut s = egui::Slider::new(&mut z, min..=max).step_by(schritt).text(o.name.de);
                            if let Some(einheit) = einheit {
                                s = s.suffix(format!(" {einheit}"));
                            }
                            let mut a = ui.add(s);
                            let stufen = festwerte(o.schluessel);
                            if !stufen.is_empty() {
                                ui.horizontal(|ui| {
                                    for (_, name, wert) in stufen {
                                        if ui.small_button(name.de).clicked() {
                                            z = *wert;
                                            a.mark_changed();
                                        }
                                    }
                                });
                            }
                            if a.changed() {
                                e.setze(o.schluessel, Wert::Zahl(z));
                            }
                            a
                        }
                        OptionsArt::Ganzzahl { min, max, .. } => {
                            let mut z = e.zahl(o.schluessel) as i64;
                            let a = ui.add(egui::Slider::new(&mut z, min..=max).text(o.name.de));
                            if a.changed() {
                                e.setze(o.schluessel, Wert::Ganzzahl(z));
                            }
                            a
                        }
                        OptionsArt::Schalter { .. } => {
                            let mut b = e.schalter(o.schluessel);
                            let a = ui.checkbox(&mut b, o.name.de);
                            if a.changed() {
                                e.setze(o.schluessel, Wert::Schalter(b));
                            }
                            a
                        }
                        OptionsArt::Auswahl { werte, .. } => {
                            let mut gewaehlt = e.auswahl(o.schluessel).to_string();
                            let anzeige = werte.iter().find(|(w, _)| *w == gewaehlt).map_or(gewaehlt.clone(), |(_, t)| t.de.to_string());
                            let a = egui::ComboBox::new((kennung, o.schluessel), o.name.de)
                                .selected_text(anzeige)
                                .show_ui(ui, |ui| {
                                    for (w, t) in werte {
                                        ui.selectable_value(&mut gewaehlt, w.to_string(), t.de);
                                    }
                                })
                                .response;
                            if gewaehlt != e.auswahl(o.schluessel) {
                                e.setze(o.schluessel, Wert::Auswahl(gewaehlt));
                                geaendert = true;
                            }
                            a
                        }
                    };
                    if antwort.changed() {
                        geaendert = true;
                    }
                    antwort.on_hover_text(o.beschreibung.de);
                });
            }
        });
    }
    geaendert
}

fn voreinstellung_ui(ui: &mut egui::Ui, kennung: &str, gewaehlt: &mut String) -> bool {
    let vorher = gewaehlt.clone();
    let name = roehre_modell::voreinstellung(gewaehlt).map_or(gewaehlt.clone(), |v| v.name());
    egui::ComboBox::new(kennung, "Voreinstellung").selected_text(name).show_ui(ui, |ui| {
        for v in roehre_modell::voreinstellungen() {
            ui.selectable_value(gewaehlt, v.schluessel.to_string(), v.name()).on_hover_text(v.beschreibung());
        }
    });
    *gewaehlt != vorher
}

impl eframe::App for Vorschau {
    fn ui(&mut self, wurzel: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = wurzel.ctx().clone();
        // Dateien per Ziehen und Ablegen
        let abgelegt: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        if let Some(p) = abgelegt.first() {
            self.lade_pfad(p);
        }

        egui::Panel::left("einstellungen").resizable(true).default_size(380.0).show(wurzel, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("roehre");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.ansicht, Ansicht::Flach, "Bildschirm");
                    ui.selectable_value(&mut self.ansicht, Ansicht::Raum, "3D-Raum");
                });
                ui.separator();
                ui.label("Bild");
                let name = match &self.quelle {
                    Quelle::Testbild(n) => n.to_string(),
                    Quelle::Datei(p) | Quelle::Folge(p, _) => p.file_name().map_or("Datei".into(), |n| n.to_string_lossy().to_string()),
                };
                egui::ComboBox::new("bild", "Testbild").selected_text(name).show_ui(ui, |ui| {
                    for (n, b) in roehre_testbilder::alle() {
                        if ui.selectable_label(matches!(self.quelle, Quelle::Testbild(x) if x == n), n).clicked() {
                            self.quelle = Quelle::Testbild(n);
                            self.bild = b;
                            self.bild_neu = true;
                        }
                    }
                });
                ui.small("Eigenes PNG oder Ordner mit PNG-Bildfolge ins Fenster ziehen.");
                ui.checkbox(&mut self.schwenk_test, "Bewegungstest: Schwenk (2 px je Bild)");
                ui.separator();

                ui.label("A");
                if voreinstellung_ui(ui, "a", &mut self.vorein_a) {
                    if let Ok(mut e) = Einstellungen::voreinstellung(&self.vorein_a) {
                        // Kamera und Stimmung des Raums gehören nicht zur Röhre: behalten
                        for (k, w) in self.e_a.alle().filter(|(k, _)| k.starts_with("raum.")) {
                            e.setze(k, w.clone());
                        }
                        self.e_a = e;
                        self.roehre_a.uebernehme(self.e_a.clone());
                    }
                }
                if self.ansicht == Ansicht::Flach {
                    ui.checkbox(&mut self.vergleich, "Vergleich mit B (Schieber im Bild ziehen)");
                    if self.vergleich {
                        ui.label("B");
                        if voreinstellung_ui(ui, "b", &mut self.vorein_b) {
                            if let Ok(e) = Einstellungen::voreinstellung(&self.vorein_b) {
                                self.e_b = e;
                                self.roehre_b.uebernehme(self.e_b.clone());
                            }
                        }
                    }
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.lupe, "Lupe");
                        ui.add_enabled(self.lupe, egui::Slider::new(&mut self.lupe_faktor, 2.0..=16.0).text("fach"));
                    });
                } else {
                    // Kamera und Stimmung sind gewöhnliche Optionen der Gruppe „3D-Raum“
                    optionen_ui(ui, &mut self.e_a, "raum", &[Gruppe::Raum]);
                }
                ui.separator();
                ui.horizontal(|ui| {
                    egui::ComboBox::new("fotogroesse", "")
                        .selected_text(["Fenstergröße", "1080p", "1440p", "4K"][self.foto_groesse])
                        .show_ui(ui, |ui| {
                            for (i, t) in ["Fenstergröße", "1080p", "1440p", "4K"].iter().enumerate() {
                                ui.selectable_value(&mut self.foto_groesse, i, *t);
                            }
                        });
                    if ui.button("Bildschirmfoto").clicked() {
                        let g = self.ziel_a.as_ref().map_or((1920, 1080), |z| z.groesse);
                        self.foto(g);
                    }
                });
                if let Some(plan) = self.roehre_a.letzter_plan() {
                    ui.small(format!(
                        "Kette: {:?} · Maske: {} · {:.1} ms je Bild (CPU-Zeit bis zum Absenden)",
                        plan.kette,
                        plan.maske.map_or("aus".into(), |m| format!("{:.0} Tripel, {} px", m.triaden, m.pixel_je_triade)),
                        self.zeitmessung.0
                    ));
                }
                if !self.meldung.is_empty() {
                    ui.small(&self.meldung);
                }
                ui.separator();
                ui.label("Optionen A");
                if optionen_ui(ui, &mut self.e_a, "oa", &Gruppe::ALLE) {
                    self.roehre_a.uebernehme(self.e_a.clone());
                }
                if self.vergleich && self.ansicht == Ansicht::Flach {
                    ui.separator();
                    ui.label("Optionen B");
                    if optionen_ui(ui, &mut self.e_b, "ob", &Gruppe::ALLE) {
                        self.roehre_b.uebernehme(self.e_b.clone());
                    }
                }
            });
        });

        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(egui::Color32::BLACK)).show(wurzel, |ui| {
            let rect = ui.max_rect();
            let ppp = ctx.pixels_per_point();
            let groesse = (((rect.width() * ppp).round() as u32).max(16), ((rect.height() * ppp).round() as u32).max(16));
            self.zeichnen(groesse);
            let antwort = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            let maler = ui.painter_at(rect);
            let uv_ganz = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            let a = self.ziel_a.as_ref().expect("gezeichnet");
            maler.image(a.id, rect, uv_ganz, egui::Color32::WHITE);
            if self.vergleich && self.ansicht == Ansicht::Flach {
                if let Some(p) = antwort.interact_pointer_pos() {
                    self.schieber = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                }
                let b = self.ziel_b.as_ref().expect("gezeichnet");
                let x = rect.left() + rect.width() * self.schieber;
                let rechts = egui::Rect::from_min_max(egui::pos2(x, rect.top()), rect.max);
                let uv = egui::Rect::from_min_max(egui::pos2(self.schieber, 0.0), egui::pos2(1.0, 1.0));
                maler.image(b.id, rechts, uv, egui::Color32::WHITE);
                maler.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], (2.0, egui::Color32::from_gray(200)));
                maler.text(rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, "A", egui::FontId::proportional(20.0), egui::Color32::WHITE);
                maler.text(rect.right_top() + egui::vec2(-8.0, 8.0), egui::Align2::RIGHT_TOP, "B", egui::FontId::proportional(20.0), egui::Color32::WHITE);
            }
            if self.lupe && self.ansicht == Ansicht::Flach {
                if let Some(p) = antwort.hover_pos() {
                    let seite = 320.0;
                    let ausschnitt = seite / self.lupe_faktor / rect.width().max(1.0);
                    let ausschnitt_h = seite / self.lupe_faktor / rect.height().max(1.0);
                    let mitte = egui::pos2((p.x - rect.left()) / rect.width(), (p.y - rect.top()) / rect.height());
                    let uv = egui::Rect::from_center_size(mitte, egui::vec2(ausschnitt, ausschnitt_h));
                    let ort = egui::Rect::from_min_size(p + egui::vec2(24.0, 24.0), egui::vec2(seite, seite));
                    let id = if self.vergleich && mitte.x > self.schieber { self.ziel_b.as_ref().map(|z| z.id) } else { None };
                    maler.image(id.unwrap_or(a.id), ort, uv, egui::Color32::WHITE);
                    maler.rect_stroke(ort, 0.0, (1.0, egui::Color32::from_gray(180)), egui::StrokeKind::Outside);
                }
            }
        });
        ctx.request_repaint();
    }
}
