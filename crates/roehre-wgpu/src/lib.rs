//! # roehre-wgpu
//!
//! Röhrensimulation für Spiele mit wgpu.
//!
//! ```no_run
//! # fn beispiel(device: &wgpu::Device, queue: &wgpu::Queue, spielbild: &wgpu::Texture,
//! #             ziel: &wgpu::TextureView) -> Result<(), roehre_wgpu::Fehler> {
//! use roehre_wgpu::{Roehre, Eingang, Einstellungen};
//! let einstellungen = Einstellungen::voreinstellung("scart-rgb")?;
//! let mut roehre = Roehre::neu(device, queue, "shader", einstellungen, wgpu::TextureFormat::Bgra8UnormSrgb)?;
//! let eingang = Eingang { breite: 256, hoehe: 224, pixel_seitenverhaeltnis: 8.0 / 7.0 };
//! let mut encoder = device.create_command_encoder(&Default::default());
//! roehre.zeichne(&mut encoder, spielbild, &eingang, ziel, (1920, 1080))?;
//! queue.submit([encoder.finish()]);
//! # Ok(()) }
//! ```
//!
//! ## Ablauf je Bild
//!
//! 1. Die Röhrenfläche wird aus Seitenverhältnis, Füllung und ganzzahliger Skalierung
//!    berechnet ([`flaeche::berechne`]); sie liegt immer auf ganzen Pixeln.
//! 2. Die Shaderkette (librashader) zeichnet in eine Zwischentextur genau dieser Größe.
//!    Damit fällt die Maske 1:1 auf Ausgabepixel, es gibt kein Moiré durch Nachskalieren.
//! 3. Ein eigener Pass setzt die Fläche pixelgenau ins Ziel und füllt den Rest mit dem
//!    Hintergrund (später: Gehäuse).
//!
//! ## Umschalten ohne Ruckeln
//!
//! [`Roehre::uebernehme`] wirkt bei Reglern sofort (nur Uniform-Werte). Ist eine andere
//! Shaderkette nötig (z. B. RGB → Composite), wird sie auf einem eigenen Thread gebaut; bis
//! sie fertig ist, läuft die bisherige Kette mit den neuen Werten weiter.
//!
//! ## Shader-Ordner
//!
//! Die Shader liegen als Dateien im Ordner `shader/` (GPL-Shader werden bewusst nicht ins
//! Programm eingebettet). Der Ordner wird mit dem Spiel ausgeliefert.

pub mod flaeche;

use std::path::PathBuf;
use std::sync::mpsc;

use librashader::runtime::wgpu::{FilterChain, FilterChainOptions, FrameOptions, WgpuOutputView};
use librashader::runtime::{FilterChainParameters, Viewport};
pub use roehre_kette::{Kette, MaskenWahl};
pub use roehre_modell::{self as modell, Einstellungen, Wert};

pub use flaeche::{Eingang, Rechteck};

/// Fehler der wgpu-Schnittstelle.
#[derive(Debug, thiserror::Error)]
pub enum Fehler {
    #[error(transparent)]
    Einstellungen(#[from] roehre_modell::Fehler),
    #[error(transparent)]
    Kette(#[from] roehre_kette::Fehler),
    #[error("Shaderkette konnte nicht gebaut werden: {0}")]
    Filter(#[from] librashader::runtime::wgpu::error::FilterChainError),
    #[error("Bauthread der Shaderkette ist abgebrochen")]
    Bauthread,
}

/// Fähigkeiten, die das Gerät für alle Shaderketten haben sollte. Das Spiel fordert sie beim
/// Anlegen des `wgpu::Device` an, soweit der Adapter sie bietet.
pub fn empfohlene_features() -> wgpu::Features {
    wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER
        | wgpu::Features::FLOAT32_FILTERABLE
        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::PIPELINE_CACHE
}

const ZWISCHENFORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Eine Zwischentextur mit Ansicht und Größe.
struct Zwischen {
    textur: wgpu::Texture,
    ansicht: wgpu::TextureView,
    groesse: (u32, u32),
}

impl Zwischen {
    /// Legt die Textur an, wenn sie fehlt oder die Größe nicht passt.
    fn sichern(platz: &mut Option<Zwischen>, device: &wgpu::Device, groesse: (u32, u32), name: &str) {
        if platz.as_ref().map(|z| z.groesse) == Some(groesse) {
            return;
        }
        let textur = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size: wgpu::Extent3d { width: groesse.0, height: groesse.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ZWISCHENFORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let ansicht = textur.create_view(&Default::default());
        *platz = Some(Zwischen { textur, ansicht, groesse });
    }

    fn viewport(&self) -> Viewport<'_, WgpuOutputView<'_>> {
        let ausgabe = WgpuOutputView::new_from_raw(
            &self.ansicht,
            librashader::runtime::Size::new(self.groesse.0, self.groesse.1),
            ZWISCHENFORMAT,
        );
        Viewport::new_render_target_sized_origin(ausgabe, None).expect("Größe ist bekannt")
    }
}

struct Ausstehend {
    kette: Kette,
    empfaenger: mpsc::Receiver<Result<FilterChain, Fehler>>,
}

/// Die Röhre.
pub struct Roehre {
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader_ordner: PathBuf,
    einstellungen: Einstellungen,
    aktiv: (Kette, FilterChain),
    ausstehend: Option<Ausstehend>,
    zwischen: Option<Zwischen>,
    /// Vorstufe „virtuelle Zeilen“ (eigener Shader) und ihr Ziel.
    vorstufe: FilterChain,
    vorstufe_ziel: Option<Zwischen>,
    setzer: Zusammensetzer,
    bild: usize,
    /// Hintergrundfarbe außerhalb der Röhrenfläche (linear).
    pub hintergrund: [f32; 3],
    letzter_plan: Option<roehre_kette::Plan>,
}

impl Roehre {
    /// Legt die Röhre an und baut die erste Shaderkette (blockierend).
    ///
    /// `ziel_format` ist das Format der Ziele, in die [`Roehre::zeichne`] später schreibt.
    pub fn neu(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_ordner: impl Into<PathBuf>,
        einstellungen: Einstellungen,
        ziel_format: wgpu::TextureFormat,
    ) -> Result<Self, Fehler> {
        let shader_ordner = shader_ordner.into();
        let kette = kette_fuer(&einstellungen);
        let filter = baue(device, queue, &shader_ordner, kette)?;
        let vorstufe = FilterChain::load_from_preset(
            roehre_kette::lade_vorstufe(&shader_ordner)?,
            device,
            queue,
            Some(&FilterChainOptions { force_no_mipmaps: true, enable_cache: true, adapter_info: None }),
        )?;
        Ok(Self {
            vorstufe,
            vorstufe_ziel: None,
            device: device.clone(),
            queue: queue.clone(),
            shader_ordner,
            einstellungen,
            aktiv: (kette, filter),
            ausstehend: None,
            zwischen: None,
            setzer: Zusammensetzer::neu(device, ziel_format),
            bild: 0,
            hintergrund: [0.0; 3],
            letzter_plan: None,
        })
    }

    /// Aktuelle Einstellungen.
    pub fn einstellungen(&self) -> &Einstellungen {
        &self.einstellungen
    }

    /// Übernimmt neue Einstellungen. Regler wirken ab dem nächsten Bild; braucht es eine
    /// andere Shaderkette, wird sie im Hintergrund gebaut.
    pub fn uebernehme(&mut self, einstellungen: Einstellungen) {
        self.einstellungen = einstellungen;
        let gewuenscht = kette_fuer(&self.einstellungen);
        let schon_unterwegs = self.ausstehend.as_ref().map(|a| a.kette);
        if gewuenscht == self.aktiv.0 {
            self.ausstehend = None;
        } else if schon_unterwegs != Some(gewuenscht) {
            let (sender, empfaenger) = mpsc::channel();
            let (device, queue, ordner) = (self.device.clone(), self.queue.clone(), self.shader_ordner.clone());
            std::thread::Builder::new()
                .name("roehre-kettenbau".into())
                .spawn(move || {
                    let _ = sender.send(baue(&device, &queue, &ordner, gewuenscht));
                })
                .expect("Thread für den Kettenbau");
            self.ausstehend = Some(Ausstehend { kette: gewuenscht, empfaenger });
        }
    }

    /// Ist die Kette zu den aktuellen Einstellungen fertig gebaut?
    pub fn ist_bereit(&mut self) -> bool {
        let _ = self.tausche_wenn_fertig(false);
        self.ausstehend.is_none()
    }

    /// Wartet, bis eine im Hintergrund gebaute Kette fertig ist (für Werkzeuge und Tests).
    pub fn warte_auf_kette(&mut self) -> Result<(), Fehler> {
        self.tausche_wenn_fertig(true)
    }

    fn tausche_wenn_fertig(&mut self, warten: bool) -> Result<(), Fehler> {
        let Some(a) = &self.ausstehend else { return Ok(()) };
        let ergebnis = if warten {
            a.empfaenger.recv().map_err(|_| Fehler::Bauthread)?
        } else {
            match a.empfaenger.try_recv() {
                Ok(e) => e,
                Err(mpsc::TryRecvError::Empty) => return Ok(()),
                Err(mpsc::TryRecvError::Disconnected) => Err(Fehler::Bauthread),
            }
        };
        let kette = a.kette;
        self.ausstehend = None;
        self.aktiv = (kette, ergebnis?);
        Ok(())
    }

    /// Röhrenfläche für ein Ziel der angegebenen Größe (berücksichtigt die virtuelle
    /// Zeilenzahl, die bei ganzzahliger Skalierung zählt).
    pub fn flaeche(&self, eingang: &Eingang, ziel_groesse: (u32, u32)) -> Rechteck {
        let seite = eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64;
        let wirksam = match roehre_kette::virtuelle_groesse(&self.einstellungen, (eingang.breite, eingang.hoehe), seite, ziel_groesse.1) {
            None => *eingang,
            Some((vb, vh)) => Eingang { breite: vb, hoehe: vh, pixel_seitenverhaeltnis: seite * vh as f64 / vb as f64 },
        };
        flaeche::berechne(&self.einstellungen, &wirksam, ziel_groesse)
    }

    /// Plan des zuletzt gezeichneten Bildes (Kette, Parameter, gewählte Maske).
    pub fn letzter_plan(&self) -> Option<&roehre_kette::Plan> {
        self.letzter_plan.as_ref()
    }

    /// Zeichnet ein Bild. `eingang_textur` ist das Spielbild (mit `TEXTURE_BINDING`), `ziel`
    /// eine Ansicht der Zieltextur im bei [`Roehre::neu`] genannten Format.
    pub fn zeichne(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        eingang_textur: &wgpu::Texture,
        eingang: &Eingang,
        ziel: &wgpu::TextureView,
        ziel_groesse: (u32, u32),
    ) -> Result<Rechteck, Fehler> {
        self.tausche_wenn_fertig(false)?;

        // Vorstufe: hochaufgelöste Bilder auf die Zeilenzahl der Röhre bringen
        let seite = eingang.breite as f64 * eingang.pixel_seitenverhaeltnis / eingang.hoehe.max(1) as f64;
        let virtuell = roehre_kette::virtuelle_groesse(&self.einstellungen, (eingang.breite, eingang.hoehe), seite, ziel_groesse.1);
        let (roehren_eingang, wirksam) = match virtuell {
            None => (eingang_textur, *eingang),
            Some((vb, vh)) => {
                Zwischen::sichern(&mut self.vorstufe_ziel, &self.device, (vb, vh), "roehre: virtuelle Zeilen");
                let z = self.vorstufe_ziel.as_ref().expect("eben gesichert");
                self.vorstufe.frame(eingang_textur, &z.viewport(), encoder, self.bild, None)?;
                let par = seite * vh as f64 / vb as f64;
                (&z.textur, Eingang { breite: vb, hoehe: vh, pixel_seitenverhaeltnis: par })
            }
        };

        let r = self.flaeche(&wirksam, ziel_groesse);
        Zwischen::sichern(&mut self.zwischen, &self.device, (r.breite, r.hoehe), "roehre: Röhrenfläche");
        let zwischen = self.zwischen.as_ref().expect("eben gesichert");

        // Parameter setzen (Regler wirken sofort, auch während eine neue Kette gebaut wird)
        let kontext = roehre_kette::Kontext { eingang: (wirksam.breite, wirksam.hoehe), flaeche: (r.breite, r.hoehe) };
        let plan = roehre_kette::plane(&self.einstellungen, &kontext);
        {
            let parameter = self.aktiv.1.parameters();
            for (name, wert) in &plan.parameter {
                parameter.set_parameter_value(name, *wert);
            }
        }
        self.letzter_plan = Some(plan);

        let optionen = FrameOptions { aspect_ratio: seite as f32, ..Default::default() };
        self.aktiv.1.frame(roehren_eingang, &zwischen.viewport(), encoder, self.bild, Some(&optionen))?;
        self.bild = self.bild.wrapping_add(1);

        self.setzer.setze(&self.device, &self.queue, encoder, &zwischen.ansicht, ziel, r, self.hintergrund);
        Ok(r)
    }
}

fn kette_fuer(e: &Einstellungen) -> Kette {
    // Die Kettenwahl hängt nicht von den Größen ab.
    let k = roehre_kette::Kontext { eingang: (1, 1), flaeche: (1, 1) };
    roehre_kette::plane(e, &k).kette
}

fn baue(device: &wgpu::Device, queue: &wgpu::Queue, ordner: &std::path::Path, kette: Kette) -> Result<FilterChain, Fehler> {
    let preset = kette.lade(ordner)?;
    let optionen = FilterChainOptions { force_no_mipmaps: false, enable_cache: true, adapter_info: None };
    Ok(FilterChain::load_from_preset(preset, device, queue, Some(&optionen))?)
}

/// Eigener Pass: Röhrenfläche ins Ziel setzen, Rest mit Hintergrund füllen.
struct Zusammensetzer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    puffer: Option<wgpu::Buffer>,
    srgb: bool,
}

impl Zusammensetzer {
    fn neu(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let modul = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("roehre: zusammensetzen"),
            source: wgpu::ShaderSource::Wgsl(include_str!("zusammensetzen.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("roehre: zusammensetzen"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("roehre: zusammensetzen"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre: zusammensetzen"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &modul,
                entry_point: Some("ecke"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &modul,
                entry_point: Some("farbe"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Self { pipeline, layout, puffer: None, srgb: format.is_srgb() }
    }

    #[allow(clippy::too_many_arguments)]
    fn setze(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        quelle: &wgpu::TextureView,
        ziel: &wgpu::TextureView,
        r: Rechteck,
        hintergrund: [f32; 3],
    ) {
        let daten: [f32; 8] = [
            r.x as f32,
            r.y as f32,
            r.breite as f32,
            r.hoehe as f32,
            hintergrund[0],
            hintergrund[1],
            hintergrund[2],
            if self.srgb { 1.0 } else { 0.0 },
        ];
        let bytes: Vec<u8> = daten.iter().flat_map(|f| f.to_le_bytes()).collect();
        let puffer = self.puffer.get_or_insert_with(|| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("roehre: zusammensetzen"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        queue.write_buffer(puffer, 0, &bytes);
        let gruppe = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre: zusammensetzen"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: puffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(quelle) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("roehre: zusammensetzen"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ziel,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &gruppe, &[]);
        pass.draw(0..3, 0..1);
    }
}
