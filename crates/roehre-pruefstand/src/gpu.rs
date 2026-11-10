//! Rendern ohne Fenster: Gerät anlegen, Spielbild hochladen, Ergebnis zurücklesen.

use roehre_testbilder::Bild;

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_name: String,
}

pub const ZIELFORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

impl Gpu {
    pub fn neu() -> Self {
        pollster::block_on(async {
            let instance = wgpu::Instance::default();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .expect("kein Grafikadapter gefunden (unter Linux ohne GPU: mesa-vulkan-drivers für lavapipe)");
            let info = adapter.get_info();
            // Zeitstempel für die Leistungsmessung, wenn der Adapter sie kann
            let features = adapter.features()
                & (roehre_wgpu::empfohlene_features()
                    | wgpu::Features::TIMESTAMP_QUERY
                    | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("roehre-pruefstand"),
                    required_features: features,
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                })
                .await
                .expect("Gerät konnte nicht angelegt werden");
            Self {
                device,
                queue,
                adapter_name: format!("{} ({:?})", info.name, info.backend),
            }
        })
    }

    /// Lädt ein Bild als Eingangstextur.
    pub fn eingang(&self, bild: &Bild) -> wgpu::Texture {
        let groesse = wgpu::Extent3d {
            width: bild.breite,
            height: bild.hoehe,
            depth_or_array_layers: 1,
        };
        let t = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spielbild"),
            size: groesse,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            t.as_image_copy(),
            &bild.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * bild.breite),
                rows_per_image: None,
            },
            groesse,
        );
        t
    }

    pub fn ziel(&self, breite: u32, hoehe: u32) -> wgpu::Texture {
        self.ziel_im_format(breite, hoehe, ZIELFORMAT)
    }

    pub fn ziel_im_format(
        &self,
        breite: u32,
        hoehe: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ziel"),
            size: wgpu::Extent3d {
                width: breite,
                height: hoehe,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    /// Liest eine Zieltextur als RGBA-Bild zurück.
    pub fn lies(&self, t: &wgpu::Texture) -> image::RgbaImage {
        let (b, h) = (t.width(), t.height());
        image::RgbaImage::from_raw(b, h, self.lies_roh(t, 4)).expect("Größe passt")
    }

    /// Liest eine Textur roh zurück (`bpp` Byte je Pixel), ohne Zeilenfüllung.
    pub fn lies_roh(&self, t: &wgpu::Texture, bpp: u32) -> Vec<u8> {
        let (b, h) = (t.width(), t.height());
        let zeile = (bpp * b).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let puffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rücklesen"),
            size: (zeile * h) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            t.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &puffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(zeile),
                    rows_per_image: None,
                },
            },
            t.size(),
        );
        self.queue.submit([enc.finish()]);
        puffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU-Warten");
        let daten = puffer
            .slice(..)
            .get_mapped_range()
            .expect("Puffer abgebildet");
        let mut px = Vec::with_capacity((bpp * b * h) as usize);
        for y in 0..h {
            let a = (y * zeile) as usize;
            px.extend_from_slice(&daten[a..a + (bpp * b) as usize]);
        }
        drop(daten);
        puffer.unmap();
        px
    }

    pub fn warte(&self) {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU-Warten");
    }
}
