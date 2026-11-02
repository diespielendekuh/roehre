//! # roehre-raum
//!
//! Ein 3D-Vorführraum: ein Wohnzimmer der späten 80er bei Nacht, ein Röhrenfernseher mit
//! Holzfurnier auf einem Tisch, darauf das Spielbild mit Röhrensimulation. Er dient als
//! Prüfstein für den Flächen-Modus von roehre und als **Vorlage** für Spiele, die einen
//! Fernseher in einer 3D-Szene zeigen wollen (z. B. das „TV-Erlebnis“ im Mission:-Impossible-
//! Port).
//!
//! Stil: späte PS2-Zeit (Richtung Silent Hill 2): wenige Polygone, dichte, selbst erzeugte
//! Materialien, Nebel, Filmkorn, warmes Lampenlicht, das Bild als Lichtquelle im Raum. Alles
//! wird beim Start erzeugt (`szene.rs`, `texturen.rs`), es gibt keine Modell- oder
//! Bilddateien. Shader: `shader/eigen/raum/raum3d.wgsl` (mit naga nach HLSL/SPIR-V übertragbar).
//!
//! Ablauf je Bild:
//! 1. `Roehre::zeichne_flaeche` zeichnet das Röhrenbild ohne Maske in ein `FlaechenZiel`.
//! 2. [`Raum::zeichne`]: Schattenkarte, Szene (4-fach kantengeglättet, HDR), Nachbearbeitung.

pub mod geometrie;
mod rauschen;
pub mod szene;
pub mod texturen;

use roehre_wgpu::{FlaechenZiel, MaterialWerte};

/// Kamera und Stimmung.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ansicht {
    pub auge: [f32; 3],
    pub ziel: [f32; 3],
    /// Senkrechter Blickwinkel in Grad
    pub blickwinkel: f32,
    /// Verschiebung des Bildausschnitts in normierten Bildkoordinaten (außermittige
    /// Projektion; 1 = halbe Fensterbreite bzw. -höhe)
    pub verschiebung: [f32; 2],
    /// Zeit in Sekunden (Vorhang, Flackern, Korn, Schwenk)
    pub zeit: f32,
    /// Helligkeit des Bildschirms (1 = Röhrenbild unverändert)
    pub bildschirm: f32,
    /// Helligkeit der Lampe
    pub lampe: f32,
    /// Flackern der Lampe 0 … 1
    pub flackern: f32,
    /// Nebeldichte je Meter
    pub nebel: f32,
    /// Stärke des Filmkorns 0 … 1
    pub korn: f32,
    /// Belichtung
    pub belichtung: f32,
    /// Tiefenunschärfe 0 … 1 (scharf ist der Fernseher)
    pub unschaerfe: f32,
    /// Glühen heller Stellen 0 … 1
    pub gluehen: f32,
}

/// Kamera des Raums, wie sie die Optionen der Gruppe `raum` beschreiben (W-005).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kamera {
    /// 0 = weit (viel Raum), 0,5 = mittel (etwas Tisch), 1 = nah (Gerät füllt das Fenster)
    pub naehe: f32,
    /// −1 … 1: von links (−30°) über vorn bis von rechts (30°)
    pub blickwinkel: f32,
    /// Leichter, langsamer Schwenk an
    pub schwenk: bool,
    /// Stärke des Schwenks 0 … 1 (1 ≈ 6° zu jeder Seite)
    pub schwenk_staerke: f32,
    /// Lage des Geräts im Fenster: −1 am linken Rand, 0 Mitte, 1 am rechten Rand
    pub lage: f32,
}

impl Default for Kamera {
    fn default() -> Self {
        Self { naehe: 0.75, blickwinkel: 0.0, schwenk: true, schwenk_staerke: 0.35, lage: 0.0 }
    }
}

impl Kamera {
    /// Liest die Optionen `raum.*` (fehlende Werte haben im Katalog ihre Standards).
    pub fn aus_einstellungen(e: &roehre_wgpu::Einstellungen) -> Self {
        Self {
            naehe: e.zahl("raum.naehe") as f32,
            blickwinkel: e.zahl("raum.blickwinkel") as f32,
            schwenk: e.auswahl("raum.schwenk") == "an",
            schwenk_staerke: e.zahl("raum.schwenk_staerke") as f32,
            lage: e.zahl("raum.lage") as f32,
        }
    }
}

/// Senkrechter Blickwinkel der Raumkamera in Grad.
const BLICKWINKEL: f32 = 38.0;
/// Rand links/rechts, der bei voller Verschiebung zum Fensterrand bleibt (Anteil der halben Breite).
const RAND_SEITE: f32 = 0.015;

/// Hüllquader des Fernsehers samt Füßen (Mitte, halbe Größe) – daran richtet sich die Kamera.
fn geraet() -> ([f32; 3], [f32; 3]) {
    let h = 0.2625;
    ([0.0, szene::TISCH_HOEHE + h, 0.0], [0.33, h, 0.205])
}

/// Hüllquader der Bildröhre mit einem schmalen Streifen Rahmen – Bezug der Stufe „ganz nah“.
fn bildkasten() -> ([f32; 3], [f32; 3]) {
    (szene::BILD_MITTE, [szene::BILD_HALB[0] + 0.02, szene::BILD_HALB[1] + 0.02, 0.03])
}

/// Ab dieser Nähe wechselt der Bezug der Kamera vom ganzen Gerät zum Bildschirm.
const NAH: f32 = 0.75;

/// Bezugskasten der Kamera und sein Anteil an der Fensterhöhe für eine Nähe 0 … 1:
/// weit (0) 24 %, mittel (0,5) 45 %, nah (0,75) 80 % des Geräts; darüber gleitet der Bezug
/// zum Bildschirm, bis bei „ganz nah“ (1) Bildschirm samt schmalem Rahmen 84 % der Höhe
/// einnimmt – oben und unten ist dann nur noch Gehäuse zu sehen.
fn bezug(naehe: f32) -> ([f32; 3], [f32; 3], f32) {
    let (gm, gh) = geraet();
    if naehe <= NAH {
        let anteil = if naehe < 0.5 { 0.24 + (0.45 - 0.24) * naehe * 2.0 } else { 0.45 + (0.80 - 0.45) * (naehe - 0.5) / (NAH - 0.5) };
        return (gm, gh, anteil);
    }
    let t = (naehe - NAH) / (1.0 - NAH);
    let (bm, bh) = bildkasten();
    let l = |a: [f32; 3], b: [f32; 3]| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
    (l(gm, bm), l(gh, bh), 0.80 + (0.84 - 0.80) * t)
}

/// Projiziert einen Punkt (Spaltenmatrix wie im Shader) in normierte Bildkoordinaten.
fn projiziere(m: &Mat, p: [f32; 3]) -> [f32; 2] {
    let c = |r: usize| m[0][r] * p[0] + m[1][r] * p[1] + m[2][r] * p[2] + m[3][r];
    let w = c(3).max(1e-4);
    [c(0) / w, c(1) / w]
}

/// Grenzen eines Kastens im Bild (xmin, xmax, ymin, ymax) ohne Verschiebung.
fn grenzen(auge: [f32; 3], ziel: [f32; 3], seite: f32, kasten: ([f32; 3], [f32; 3])) -> [f32; 4] {
    let m = mal(&perspektive(BLICKWINKEL, seite, 0.03, 20.0), &blick(auge, ziel));
    let (mitte, halb) = kasten;
    let mut g = [f32::MAX, f32::MIN, f32::MAX, f32::MIN];
    for i in 0..8 {
        let p = [
            mitte[0] + if i & 1 == 0 { -halb[0] } else { halb[0] },
            mitte[1] + if i & 2 == 0 { -halb[1] } else { halb[1] },
            mitte[2] + if i & 4 == 0 { -halb[2] } else { halb[2] },
        ];
        let q = projiziere(&m, p);
        g = [g[0].min(q[0]), g[1].max(q[0]), g[2].min(q[1]), g[3].max(q[1])];
    }
    g
}

impl Ansicht {
    /// Ansicht aus der Kamera für ein Fenster mit Seitenverhältnis `seite` (Breite/Höhe).
    ///
    /// Die Nähe legt fest, welchen Anteil der Fensterhöhe das Gerät einnimmt (weit 24 %,
    /// mittel 45 %, nah 80 %; „ganz nah“ 84 % für den Bildschirm samt schmalem Rahmen, das
    /// Gehäuse reicht dann über den Fensterrand); ist das Fenster dafür zu schmal, wird so weit zurückgenommen,
    /// dass das Gerät in der Breite passt. Die Lage im Fenster entsteht durch Verschieben des
    /// Bildausschnitts (außermittige Projektion), nicht durch Drehen der Kamera: Blickwinkel
    /// und Form des Geräts bleiben gleich. Der Schwenk kreist um das Gerät und blickt immer auf
    /// seine Mitte; Abstand und Verschiebung bleiben dabei fest, das Gerät steht still im Bild.
    pub fn aus_kamera(k: &Kamera, zeit: f32, seite: f32) -> Self {
        let seite = seite.max(0.1);
        let naehe = k.naehe.clamp(0.0, 1.0);
        let (mitte, halb, anteil) = bezug(naehe);
        let kasten = (mitte, halb);
        let gier = k.blickwinkel.clamp(-1.0, 1.0) * 30f32.to_radians();
        let hoehe = if naehe <= NAH { 14.0 + (5.0 - 14.0) * naehe / NAH } else { 5.0 - 2.0 * (naehe - NAH) / (1.0 - NAH) }.to_radians();
        let lage_auge = |d: f32, gier: f32, hoehe: f32| {
            [mitte[0] + d * gier.sin() * hoehe.cos(), mitte[1] + d * hoehe.sin(), mitte[2] - d * gier.cos() * hoehe.cos()]
        };
        // Abstand so wählen, dass das Gerät den gewünschten Anteil hat und in die Breite passt
        let t = (BLICKWINKEL.to_radians() / 2.0).tan();
        let mut d = halb[1] / (anteil * t);
        for _ in 0..6 {
            let g = grenzen(lage_auge(d, gier, hoehe), mitte, seite, kasten);
            let (h, b) = ((g[3] - g[2]) / 2.0, (g[1] - g[0]) / 2.0);
            d *= (h / anteil).max(b / (1.0 - RAND_SEITE));
        }
        // im Zimmer bleiben (vordere Wand bei z = −4,6, Seitenwände bei ±2,4)
        let d_max = ((mitte[2] + 4.3) / (gier.cos() * hoehe.cos()).max(0.1)).min(2.1 / (gier.sin().abs() * hoehe.cos()).max(1e-3));
        d = d.min(d_max);
        let g = grenzen(lage_auge(d, gier, hoehe), mitte, seite, kasten);
        let b = (g[1] - g[0]) / 2.0;
        let spiel = (1.0 - RAND_SEITE - b).max(0.0);
        let verschiebung = [k.lage.clamp(-1.0, 1.0) * spiel - (g[0] + g[1]) / 2.0, -(g[2] + g[3]) / 2.0];
        // Schwenk: nur die Lage des Auges ändert sich, der Blick bleibt auf der Gerätemitte
        let (gs, hs) = if k.schwenk {
            let s = k.schwenk_staerke.clamp(0.0, 1.0);
            (gier + s * 0.105 * (zeit * std::f32::consts::TAU / 40.0).sin(), hoehe + s * 0.012 * (zeit * std::f32::consts::TAU / 57.0).sin())
        } else {
            (gier, hoehe)
        };
        Self {
            auge: lage_auge(d, gs, hs),
            ziel: mitte,
            blickwinkel: BLICKWINKEL,
            verschiebung,
            zeit,
            bildschirm: 1.0,
            lampe: 2.6,
            flackern: 0.4,
            nebel: 0.10,
            korn: 0.35,
            belichtung: 2.6,
            unschaerfe: 0.3,
            gluehen: 0.5,
        }
    }

    /// Ansicht aus allen Optionen der Gruppe `raum` (Kamera, Licht, Flackern, Korn).
    pub fn aus_einstellungen(e: &roehre_wgpu::Einstellungen, zeit: f32, seite: f32) -> Self {
        let mut a = Self::aus_kamera(&Kamera::aus_einstellungen(e), zeit, seite);
        a.lampe *= e.zahl("raum.licht") as f32;
        a.flackern = e.zahl("raum.flackern") as f32;
        a.korn = e.zahl("raum.korn") as f32;
        a.unschaerfe = e.zahl("raum.unschaerfe") as f32;
        a.gluehen = e.zahl("raum.gluehen") as f32;
        a
    }

    /// Kurzform: Kamera von vorn, mittig, für 16:9. `schwenk` 0 = still, sonst Stärke.
    pub fn sitzplatz(zeit: f32, naehe: f32, schwenk: f32) -> Self {
        let k = Kamera { naehe, schwenk: schwenk > 0.0, schwenk_staerke: schwenk.min(1.0), ..Kamera::default() };
        Self::aus_kamera(&k, zeit, 16.0 / 9.0)
    }
}

type Mat = [[f32; 4]; 4];

fn mal(a: &Mat, b: &Mat) -> Mat {
    let mut r = [[0.0; 4]; 4];
    for (c, spalte) in r.iter_mut().enumerate() {
        for (z, wert) in spalte.iter_mut().enumerate() {
            *wert = (0..4).map(|k| a[k][z] * b[c][k]).sum();
        }
    }
    r
}

fn blick(auge: [f32; 3], ziel: [f32; 3]) -> Mat {
    use geometrie::{cross, dot, norm, sub};
    // Die Szene ist so gebaut, dass +x beim Blick nach +z rechts liegt (linkshändig gedacht):
    // „rechts“ ist daher oben × vorn, nicht vorn × oben.
    let f = norm(sub(ziel, auge));
    let s = norm(cross([0.0, 1.0, 0.0], f));
    let u = cross(f, s);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot(s, auge), -dot(u, auge), dot(f, auge), 1.0],
    ]
}

fn perspektive(blickwinkel_grad: f32, seite: f32, nah: f32, fern: f32) -> Mat {
    let f = 1.0 / (blickwinkel_grad.to_radians() / 2.0).tan();
    [
        [f / seite, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, fern / (nah - fern), -1.0],
        [0.0, 0.0, nah * fern / (nah - fern), 0.0],
    ]
}

const SCHATTEN: u32 = 2048;
const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const TIEFE: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const PROBEN: u32 = 4;

struct Ziele {
    groesse: (u32, u32),
    farbe: wgpu::TextureView,
    aufgeloest: wgpu::TextureView,
    tiefe: wgpu::TextureView,
    /// helle Anteile auf 1/8 verkleinert (Glühen)
    glanz: wgpu::TextureView,
}

/// Der Raum.
pub struct Raum {
    ecken: wgpu::Buffer,
    indizes: wgpu::Buffer,
    anzahl: u32,
    uniform: wgpu::Buffer,
    material_puffer: wgpu::Buffer,
    farben_textur: wgpu::Texture,
    farben: wgpu::TextureView,
    eigenschaften: wgpu::TextureView,
    kachel: wgpu::Sampler,
    rand: wgpu::Sampler,
    vergleich: wgpu::Sampler,
    schatten: wgpu::TextureView,
    schatten_pipeline: wgpu::RenderPipeline,
    haupt_pipeline: wgpu::RenderPipeline,
    nach_pipeline: wgpu::RenderPipeline,
    glanz_pipeline: wgpu::RenderPipeline,
    glanz_layout: wgpu::BindGroupLayout,
    schatten_layout: wgpu::BindGroupLayout,
    haupt_layout: wgpu::BindGroupLayout,
    nach_layout: wgpu::BindGroupLayout,
    ziele: Option<Ziele>,
    srgb: bool,
}

/// Quelltext des Raum-Shaders samt Maskenfunktion (für eigene Backends übersetzbar).
pub fn shader_quelltext(shader_ordner: &std::path::Path) -> std::io::Result<String> {
    let maske = std::fs::read_to_string(shader_ordner.join("eigen/flaeche/roehre_maske.wgsl"))?;
    let raum = std::fs::read_to_string(shader_ordner.join("eigen/raum/raum3d.wgsl"))?;
    Ok(format!("{maske}\n{raum}"))
}

fn zu_linear(v: u8) -> f32 {
    let c = v as f32 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn zu_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0 + 0.5) as u8
}

/// Halbiert eine Ebene (2×2-Mittel); bei `srgb` in linearem Licht.
fn halbieren(daten: &[u8], g: u32, srgb: bool) -> Vec<u8> {
    let h = g / 2;
    let mut aus = Vec::with_capacity((h * h * 4) as usize);
    for y in 0..h {
        for x in 0..h {
            for k in 0..4 {
                let wert = |dx: u32, dy: u32| daten[(((y * 2 + dy) * g + x * 2 + dx) * 4 + k) as usize];
                if srgb && k < 3 {
                    let m = (zu_linear(wert(0, 0)) + zu_linear(wert(1, 0)) + zu_linear(wert(0, 1)) + zu_linear(wert(1, 1))) / 4.0;
                    aus.push(zu_srgb(m));
                } else {
                    aus.push(((wert(0, 0) as u32 + wert(1, 0) as u32 + wert(0, 1) as u32 + wert(1, 1) as u32 + 2) / 4) as u8);
                }
            }
        }
    }
    aus
}

fn texturfeld(device: &wgpu::Device, queue: &wgpu::Queue, ebenen: &[Vec<u8>], format: wgpu::TextureFormat, srgb: bool) -> (wgpu::Texture, wgpu::TextureView) {
    let g = texturen::GROESSE;
    let stufen = 32 - g.leading_zeros();
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("roehre-raum: Materialien"),
        size: wgpu::Extent3d { width: g, height: g, depth_or_array_layers: ebenen.len() as u32 },
        mip_level_count: stufen,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (ebene, daten) in ebenen.iter().enumerate() {
        ebene_schreiben(queue, &t, ebene as u32, daten, srgb);
    }
    let sicht = t.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default() });
    (t, sicht)
}

/// Schreibt eine Ebene samt aller Mip-Stufen.
fn ebene_schreiben(queue: &wgpu::Queue, t: &wgpu::Texture, ebene: u32, daten: &[u8], srgb: bool) {
    let mut stufe = daten.to_vec();
    let mut groesse = texturen::GROESSE;
    for s in 0..t.mip_level_count() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: t, mip_level: s, origin: wgpu::Origin3d { x: 0, y: 0, z: ebene }, aspect: wgpu::TextureAspect::All },
            &stufe,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * groesse), rows_per_image: None },
            wgpu::Extent3d { width: groesse, height: groesse, depth_or_array_layers: 1 },
        );
        if groesse > 1 {
            stufe = halbieren(&stufe, groesse, srgb);
            groesse /= 2;
        }
    }
}

fn eintrag(binding: u32, ty: wgpu::BindingType, sicht: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: sicht, ty, count: None }
}

const UNIFORM: wgpu::BindingType = wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None };

fn textur(dim: wgpu::TextureViewDimension, filterbar: bool) -> wgpu::BindingType {
    wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: filterbar }, view_dimension: dim, multisampled: false }
}

impl Raum {
    pub fn neu(device: &wgpu::Device, queue: &wgpu::Queue, shader_ordner: &std::path::Path, format: wgpu::TextureFormat) -> std::io::Result<Self> {
        // Materialien
        let ebenen = texturen::erzeugen();
        let farben: Vec<Vec<u8>> = ebenen.iter().map(|e| e.0.clone()).collect();
        let eig: Vec<Vec<u8>> = ebenen.into_iter().map(|e| e.1).collect();
        let (farben_textur, farben) = texturfeld(device, queue, &farben, wgpu::TextureFormat::Rgba8UnormSrgb, true);
        let (_, eigenschaften) = texturfeld(device, queue, &eig, wgpu::TextureFormat::Rgba8Unorm, false);

        // Geometrie
        let netz = szene::bauen();
        let bytes = |d: &[u8], usage| {
            let b = device.create_buffer(&wgpu::BufferDescriptor { label: Some("roehre-raum"), size: d.len() as u64, usage: usage | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            queue.write_buffer(&b, 0, d);
            b
        };
        let ecken_bytes: Vec<u8> = netz
            .ecken
            .iter()
            .flat_map(|e| {
                let mut v = Vec::with_capacity(40);
                for f in e.lage.iter().chain(e.normale.iter()).chain(e.uv.iter()) {
                    v.extend_from_slice(&f.to_le_bytes());
                }
                v.extend_from_slice(&e.material.to_le_bytes());
                v.extend_from_slice(&e.wind.to_le_bytes());
                v
            })
            .collect();
        let index_bytes: Vec<u8> = netz.indizes.iter().flat_map(|i| i.to_le_bytes()).collect();
        let ecken = bytes(&ecken_bytes, wgpu::BufferUsages::VERTEX);
        let indizes = bytes(&index_bytes, wgpu::BufferUsages::INDEX);
        let uniform = bytes(&[0u8; 224], wgpu::BufferUsages::UNIFORM);
        let material_puffer = bytes(&[0u8; 32], wgpu::BufferUsages::UNIFORM);

        // Abtaster
        let kachel = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("roehre-raum: Kachel"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let rand = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("roehre-raum: Rand"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let vergleich = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("roehre-raum: Schatten"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let schatten = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("roehre-raum: Schattenkarte"),
                size: wgpu::Extent3d { width: SCHATTEN, height: SCHATTEN, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TIEFE,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());

        // Pipelines
        let modul = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("roehre-raum"),
            source: wgpu::ShaderSource::Wgsl(shader_quelltext(shader_ordner)?.into()),
        });
        let v = wgpu::ShaderStages::VERTEX;
        let f = wgpu::ShaderStages::FRAGMENT;
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let schatten_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("roehre-raum: Schatten"), entries: &[eintrag(0, UNIFORM, v)] });
        let haupt_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("roehre-raum: Szene"),
            entries: &[
                eintrag(0, UNIFORM, vf),
                eintrag(1, UNIFORM, f),
                eintrag(2, textur(wgpu::TextureViewDimension::D2Array, true), f),
                eintrag(3, textur(wgpu::TextureViewDimension::D2Array, true), f),
                eintrag(4, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), f),
                eintrag(5, textur(wgpu::TextureViewDimension::D2, true), f),
                eintrag(6, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), f),
                eintrag(7, wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, f),
                eintrag(8, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), f),
            ],
        });
        let nach_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("roehre-raum: Nachbearbeitung"),
            entries: &[
                eintrag(0, UNIFORM, f),
                eintrag(1, textur(wgpu::TextureViewDimension::D2, false), f),
                eintrag(2, textur(wgpu::TextureViewDimension::D2, true), f),
                eintrag(3, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), f),
            ],
        });
        let glanz_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("roehre-raum: Glanz"),
            entries: &[
                eintrag(0, UNIFORM, f),
                eintrag(1, textur(wgpu::TextureViewDimension::D2, true), f),
                eintrag(2, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), f),
            ],
        });
        let ecken_layout = wgpu::VertexBufferLayout {
            array_stride: 40,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Uint32, 4 => Float32],
        };
        let pl = |layout: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("roehre-raum"), bind_group_layouts: &[Some(layout)], immediate_size: 0 })
        };
        let schatten_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre-raum: Schatten"),
            layout: Some(&pl(&schatten_layout)),
            vertex: wgpu::VertexState { module: &modul, entry_point: Some("schatten_ecke"), compilation_options: Default::default(), buffers: &[Some(ecken_layout.clone())] },
            fragment: None,
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TIEFE,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let haupt_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre-raum: Szene"),
            layout: Some(&pl(&haupt_layout)),
            vertex: wgpu::VertexState { module: &modul, entry_point: Some("haupt_ecke"), compilation_options: Default::default(), buffers: &[Some(ecken_layout)] },
            fragment: Some(wgpu::FragmentState { module: &modul, entry_point: Some("haupt_farbe"), compilation_options: Default::default(), targets: &[Some(HDR.into())] }),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TIEFE,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState { count: PROBEN, ..Default::default() },
            multiview_mask: None,
            cache: None,
        });
        let glanz_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre-raum: Glanz"),
            layout: Some(&pl(&glanz_layout)),
            vertex: wgpu::VertexState { module: &modul, entry_point: Some("nach_ecke"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &modul, entry_point: Some("glanz_farbe"), compilation_options: Default::default(), targets: &[Some(HDR.into())] }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let nach_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("roehre-raum: Nachbearbeitung"),
            layout: Some(&pl(&nach_layout)),
            vertex: wgpu::VertexState { module: &modul, entry_point: Some("nach_ecke"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &modul, entry_point: Some("nach_farbe"), compilation_options: Default::default(), targets: &[Some(format.into())] }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Ok(Self {
            ecken,
            indizes,
            anzahl: netz.indizes.len() as u32,
            uniform,
            material_puffer,
            farben_textur,
            farben,
            eigenschaften,
            kachel,
            rand,
            vergleich,
            schatten,
            schatten_pipeline,
            haupt_pipeline,
            nach_pipeline,
            glanz_pipeline,
            glanz_layout,
            schatten_layout,
            haupt_layout,
            nach_layout,
            ziele: None,
            srgb: format.is_srgb(),
        })
    }

    /// Zahl der Dreiecke der Szene.
    /// Ersetzt das Bild im Rahmen an der Wand durch ein eigenes (RGBA8, sRGB, Zeile für
    /// Zeile). Es wird mittig quadratisch zugeschnitten und auf die Texturgröße gebracht
    /// (Flächenmittel), danach wirkt es wie jede andere Raumtextur in Rohpixeln.
    pub fn wandbild_setzen(&self, queue: &wgpu::Queue, rgba: &[u8], breite: u32, hoehe: u32) {
        let g = texturen::GROESSE;
        let seite = breite.min(hoehe).max(1);
        let (x0, y0) = ((breite - seite) / 2, (hoehe - seite) / 2);
        let mut daten = Vec::with_capacity((g * g * 4) as usize);
        for y in 0..g {
            for x in 0..g {
                let (ax, ex) = (x0 + x * seite / g, x0 + ((x + 1) * seite / g).max(x * seite / g + 1));
                let (ay, ey) = (y0 + y * seite / g, y0 + ((y + 1) * seite / g).max(y * seite / g + 1));
                let mut summe = [0.0f32; 3];
                let mut zahl = 0.0;
                for sy in ay..ey.min(hoehe) {
                    for sx in ax..ex.min(breite) {
                        let i = ((sy * breite + sx) * 4) as usize;
                        for k in 0..3 {
                            summe[k] += zu_linear(rgba[i + k]);
                        }
                        zahl += 1.0;
                    }
                }
                for s in summe {
                    daten.push(zu_srgb(s / f32::max(zahl, 1.0)));
                }
                daten.push(255);
            }
        }
        ebene_schreiben(queue, &self.farben_textur, texturen::Material::Wandbild as u32, &daten, true);
    }

    pub fn dreiecke(&self) -> u32 {
        self.anzahl / 3
    }

    fn ziele_sichern(&mut self, device: &wgpu::Device, groesse: (u32, u32)) {
        if self.ziele.as_ref().map(|z| z.groesse) == Some(groesse) {
            return;
        }
        let glanz_groesse = (groesse.0.div_ceil(8).max(1), groesse.1.div_ceil(8).max(1));
        let textur_in = |g: (u32, u32), format, proben, usage| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("roehre-raum: Ziel"),
                    size: wgpu::Extent3d { width: g.0, height: g.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: proben,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let textur = |format, proben, usage| textur_in(groesse, format, proben, usage);
        self.ziele = Some(Ziele {
            groesse,
            farbe: textur(HDR, PROBEN, wgpu::TextureUsages::RENDER_ATTACHMENT),
            aufgeloest: textur(HDR, 1, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING),
            tiefe: textur(TIEFE, PROBEN, wgpu::TextureUsages::RENDER_ATTACHMENT),
            glanz: textur_in(glanz_groesse, HDR, 1, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING),
        });
    }

    /// Rendert den Raum in `ziel` (Größe `groesse`), mit dem Röhrenbild aus `flaeche`.
    #[allow(clippy::too_many_arguments)]
    pub fn zeichne(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        ziel: &wgpu::TextureView,
        groesse: (u32, u32),
        flaeche: &FlaechenZiel,
        material: &MaterialWerte,
        a: &Ansicht,
    ) {
        self.ziele_sichern(device, groesse);
        let ziele = self.ziele.as_ref().expect("gesichert");
        let seite = groesse.0 as f32 / groesse.1.max(1) as f32;
        let mut projektion = perspektive(a.blickwinkel, seite, 0.03, 20.0);
        // außermittig: x_ndc += verschiebung (w = −z_Ansicht)
        projektion[2][0] = -a.verschiebung[0];
        projektion[2][1] = -a.verschiebung[1];
        let ansicht = mal(&projektion, &blick(a.auge, a.ziel));
        let licht = mal(&perspektive(125.0, 1.0, 0.08, 8.0), &blick(szene::LAMPE, [-0.2, 0.75, -0.1]));
        let stufen = flaeche.textur.mip_level_count() as f32 - 1.0;
        let mut w: Vec<f32> = Vec::with_capacity(52);
        w.extend(ansicht.iter().flatten());
        w.extend(licht.iter().flatten());
        w.extend([a.auge[0], a.auge[1], a.auge[2], a.zeit]);
        w.extend([szene::LAMPE[0], szene::LAMPE[1], szene::LAMPE[2], a.lampe]);
        w.extend([szene::BILD_MITTE[0], szene::BILD_MITTE[1], szene::BILD_MITTE[2], a.bildschirm]);
        w.extend([szene::BILD_HALB[0], szene::BILD_HALB[1], stufen, a.flackern]);
        w.extend([a.nebel, a.korn, if self.srgb { 1.0 } else { 0.0 }, a.belichtung]);
        let fokus = geometrie::sub(szene::BILD_MITTE, a.auge);
        w.extend([geometrie::dot(fokus, fokus).sqrt(), a.unschaerfe, a.gluehen, 0.0]);
        let bytes: Vec<u8> = w.iter().flat_map(|f| f.to_le_bytes()).collect();
        queue.write_buffer(&self.uniform, 0, &bytes);
        let mut mw = *material;
        mw.seitenverhaeltnis = szene::BILD_HALB[0] / szene::BILD_HALB[1];
        queue.write_buffer(&self.material_puffer, 0, &mw.als_bytes());

        // 1. Schattenkarte
        let sg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre-raum: Schatten"),
            layout: &self.schatten_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() }],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("roehre-raum: Schatten"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.schatten,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.schatten_pipeline);
            pass.set_bind_group(0, &sg, &[]);
            pass.set_vertex_buffer(0, self.ecken.slice(..));
            pass.set_index_buffer(self.indizes.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.anzahl, 0, 0..1);
        }

        // 2. Szene
        let hg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre-raum: Szene"),
            layout: &self.haupt_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.material_puffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.farben) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&self.eigenschaften) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.kachel) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&flaeche.ansicht_srgb) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&self.rand) },
                wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(&self.schatten) },
                wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::Sampler(&self.vergleich) },
            ],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("roehre-raum: Szene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &ziele.farbe,
                    depth_slice: None,
                    resolve_target: Some(&ziele.aufgeloest),
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Discard },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &ziele.tiefe,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.haupt_pipeline);
            pass.set_bind_group(0, &hg, &[]);
            pass.set_vertex_buffer(0, self.ecken.slice(..));
            pass.set_index_buffer(self.indizes.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.anzahl, 0, 0..1);
        }

        // 3. Glanz: helle Anteile verkleinert (für das Glühen)
        let gg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre-raum: Glanz"),
            layout: &self.glanz_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&ziele.aufgeloest) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.rand) },
            ],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("roehre-raum: Glanz"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &ziele.glanz,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.glanz_pipeline);
            pass.set_bind_group(0, &gg, &[]);
            pass.draw(0..3, 0..1);
        }

        // 4. Nachbearbeitung
        let ng = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("roehre-raum: Nachbearbeitung"),
            layout: &self.nach_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&ziele.aufgeloest) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&ziele.glanz) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.rand) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("roehre-raum: Nachbearbeitung"),
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
        pass.set_pipeline(&self.nach_pipeline);
        pass.set_bind_group(0, &ng, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Grenzen des Geräts im fertigen Bild (mit Verschiebung).
    fn im_bild(a: &Ansicht, seite: f32) -> [f32; 4] {
        im_bild_von(a, seite, geraet())
    }

    fn im_bild_von(a: &Ansicht, seite: f32, kasten: ([f32; 3], [f32; 3])) -> [f32; 4] {
        let g = grenzen(a.auge, a.ziel, seite, kasten);
        [g[0] + a.verschiebung[0], g[1] + a.verschiebung[0], g[2] + a.verschiebung[1], g[3] + a.verschiebung[1]]
    }

    const FORMATE: [f32; 6] = [4.0 / 3.0, 16.0 / 10.0, 16.0 / 9.0, 21.0 / 9.0, 2.4, 32.0 / 9.0];

    #[test]
    fn geraet_wird_nie_abgeschnitten() {
        for seite in FORMATE {
            for naehe in [0.0, 0.25, 0.5, 0.75, 0.9, 1.0] {
                for blick in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                    for lage in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                        let k = Kamera { naehe, blickwinkel: blick, lage, schwenk: false, ..Kamera::default() };
                        // bis „nah“ das ganze Gerät, darüber mindestens der Bildschirm
                        let kasten = if naehe <= NAH { geraet() } else { bildkasten() };
                        let g = im_bild_von(&Ansicht::aus_kamera(&k, 0.0, seite), seite, kasten);
                        assert!(g[0] >= -1.0 + 1e-3 && g[1] <= 1.0 - 1e-3 && g[2] >= -1.0 && g[3] <= 1.0, "{seite} {naehe} {blick} {lage}: {g:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn naehe_bestimmt_den_anteil_der_hoehe() {
        // bei 16:9 von vorn: weit 24 %, mittel 45 %, nah 80 % der Fensterhöhe
        for (naehe, soll) in [(0.0, 0.24), (0.5, 0.45), (0.75, 0.80)] {
            let k = Kamera { naehe, schwenk: false, ..Kamera::default() };
            let g = im_bild(&Ansicht::aus_kamera(&k, 0.0, 16.0 / 9.0), 16.0 / 9.0);
            let anteil = (g[3] - g[2]) / 2.0;
            assert!((anteil - soll).abs() < 0.01, "{naehe}: {anteil}");
            assert!(((g[2] + g[3]) / 2.0).abs() < 1e-3, "senkrecht mittig");
        }
        // ganz nah: Bildschirm mit schmalem Rahmen 84 % der Höhe, das Gehäuse ragt hinaus
        let a = Ansicht::aus_kamera(&Kamera { naehe: 1.0, schwenk: false, ..Kamera::default() }, 0.0, 16.0 / 9.0);
        let b = im_bild_von(&a, 16.0 / 9.0, bildkasten());
        assert!(((b[3] - b[2]) / 2.0 - 0.84).abs() < 0.01, "{b:?}");
        let g = im_bild(&a, 16.0 / 9.0);
        assert!(g[3] > 1.0 && g[2] < -1.0, "Gehäuse oben und unten angeschnitten: {g:?}");
    }

    #[test]
    fn lage_schiebt_bis_an_den_rand_ohne_blickwinkel_zu_aendern() {
        for seite in [16.0 / 9.0, 32.0 / 9.0] {
            let mitte = Ansicht::aus_kamera(&Kamera { naehe: NAH, schwenk: false, ..Kamera::default() }, 0.0, seite);
            for lage in [-1.0f32, 1.0] {
                let a = Ansicht::aus_kamera(&Kamera { naehe: NAH, lage, schwenk: false, ..Kamera::default() }, 0.0, seite);
                assert_eq!(a.auge, mitte.auge, "Kamera bleibt, nur der Ausschnitt wandert");
                let g = im_bild(&a, seite);
                let rand = if lage < 0.0 { g[0] + 1.0 } else { 1.0 - g[1] };
                assert!((rand - RAND_SEITE).abs() < 1e-3, "{seite} {lage}: {g:?}");
            }
        }
    }

    #[test]
    fn schwenk_haelt_das_geraet_an_seiner_stelle() {
        let k = Kamera { naehe: 0.8, lage: 0.6, schwenk: true, schwenk_staerke: 1.0, ..Kamera::default() };
        let seite = 16.0 / 9.0;
        let m = |a: &Ansicht| {
            let p = mal(&perspektive(BLICKWINKEL, seite, 0.03, 20.0), &blick(a.auge, a.ziel));
            let q = projiziere(&p, a.ziel);
            [q[0] + a.verschiebung[0], q[1] + a.verschiebung[1]]
        };
        let ruhe = m(&Ansicht::aus_kamera(&k, 0.0, seite));
        let mut auge_bewegt = 0.0f32;
        for zeit in [5.0, 10.0, 20.0, 31.0] {
            let a = Ansicht::aus_kamera(&k, zeit, seite);
            let q = m(&a);
            assert!((q[0] - ruhe[0]).abs() < 1e-4 && (q[1] - ruhe[1]).abs() < 1e-4, "Mitte wandert: {q:?} {ruhe:?}");
            auge_bewegt = auge_bewegt.max((a.auge[0] - Ansicht::aus_kamera(&k, 0.0, seite).auge[0]).abs());
        }
        assert!(auge_bewegt > 0.05, "Schwenk wirkt");
        let still = Kamera { schwenk: false, ..k };
        assert_eq!(Ansicht::aus_kamera(&still, 0.0, seite).auge, Ansicht::aus_kamera(&still, 17.0, seite).auge);
    }
}
