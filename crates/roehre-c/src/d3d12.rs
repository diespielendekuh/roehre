//! D3D12-Weg des Austauschs: eigenes Gerät auf demselben Adapter (über die LUID gefunden),
//! Bilder als geteilte Ressourcen (D3D12_HEAP_FLAG_SHARED, NT-Handles), ein geteilter Zaun.
//! Am Gerät des Spiels muss dafür nichts geändert werden.
//!
//! Zaunwerte: beim k-ten Aufruf von roehre_austausch_zeichnen (k = 1, 2, …) wartet roehre auf
//! den Wert 2k−1 („bereit“, setzt das Spiel) und setzt danach 2k („fertig“).

use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use wgpu::hal::api::Dx12;
use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{GENERIC_ALL, HANDLE};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use crate::arbeitsplatz::Arbeitsplatz;
use crate::austausch::{lies_parameter, RoehreAustausch, RoehreAustauschParameter, Weg, BILD_NUTZUNG, EINGANG_FORMAT};
use crate::fehler::geschuetzt;

/// Beschreibung eines geteilten Bilds für das Spiel (siehe roehre.h).
#[repr(C)]
pub struct RoehreD3D12Bild {
    pub handle: *mut c_void,
    pub dxgi_format: u32,
    pub breite: u32,
    pub hoehe: u32,
}

fn dxgi_format(f: wgpu::TextureFormat) -> DXGI_FORMAT {
    match f {
        wgpu::TextureFormat::Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
        wgpu::TextureFormat::Rgba8UnormSrgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        wgpu::TextureFormat::Rgba16Float => DXGI_FORMAT_R16G16B16A16_FLOAT,
        wgpu::TextureFormat::Rgb10a2Unorm => DXGI_FORMAT_R10G10B10A2_UNORM,
        _ => unreachable!("nur die Formate des Austauschs"),
    }
}

struct D3d12Teile {
    geraet: ID3D12Device,
    bilder: [(ID3D12Resource, wgpu::TextureFormat, u32, u32); 2],
    zaun: ID3D12Fence,
    zaehler: Arc<AtomicU64>,
}

// Die COM-Zeiger werden nur unter dem Austausch-Objekt benutzt, das nicht von zwei Threads
// zugleich verwendet wird (roehre.h).
unsafe impl Send for D3d12Teile {}

struct D3d12Weg {
    zaun: ID3D12Fence,
    zaehler: Arc<AtomicU64>,
}

unsafe impl Send for D3d12Weg {}

impl Weg for D3d12Weg {
    fn vor_submit(&mut self, queue: &wgpu::Queue) -> Result<(), String> {
        let k = self.zaehler.fetch_add(1, Ordering::SeqCst) + 1;
        let q = unsafe { queue.as_hal::<Dx12>() }.ok_or("Warteschlange ist nicht D3D12")?;
        q.add_wait_fence(self.zaun.clone(), 2 * k - 1);
        q.add_signal_fence(self.zaun.clone(), 2 * k);
        Ok(())
    }

    fn nach_submit(&mut self, _queue: &wgpu::Queue) -> Result<(), String> {
        Ok(())
    }
}

fn win(was: &'static str) -> impl Fn(windows::core::Error) -> String {
    move |e| format!("{was}: {e}")
}

fn adapter_suchen(kennung: &[u8; 16]) -> Result<wgpu::Adapter, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::DX12, ..wgpu::InstanceDescriptor::new_without_display_handle_from_env() });
    let adapter = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::DX12));
    if adapter.is_empty() {
        return Err("kein D3D12-Adapter gefunden".into());
    }
    if kennung.iter().all(|b| *b == 0) {
        return Ok(adapter.into_iter().next().expect("nicht leer"));
    }
    let gesucht = (u32::from_le_bytes(kennung[0..4].try_into().expect("4")), i32::from_le_bytes(kennung[4..8].try_into().expect("4")));
    let mut gesehen = Vec::new();
    for a in adapter {
        let luid = {
            let hal = unsafe { a.as_hal::<Dx12>() }.ok_or("Adapter ist nicht D3D12")?;
            let desc = unsafe { hal.as_raw().GetDesc1() }.map_err(win("Adapterbeschreibung"))?;
            (desc.AdapterLuid.LowPart, desc.AdapterLuid.HighPart)
        };
        if luid == gesucht {
            return Ok(a);
        }
        gesehen.push(a.get_info().name);
    }
    Err(format!("kein Adapter mit dieser LUID (gefunden: {})", gesehen.join(", ")))
}

/// Eine geteilte Ressource, als wgpu-Textur.
fn geteiltes_bild(device: &wgpu::Device, format: wgpu::TextureFormat, b: u32, h: u32, anfang: wgpu::TextureUses) -> Result<(wgpu::Texture, ID3D12Resource), String> {
    let roh = {
        let hal = unsafe { device.as_hal::<Dx12>() }.ok_or("Gerät ist nicht D3D12")?;
        let d = hal.raw_device();
        let heap = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() };
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: b as u64,
            Height: h,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: dxgi_format(format),
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
        };
        let mut r: Option<ID3D12Resource> = None;
        unsafe { d.CreateCommittedResource(&heap, D3D12_HEAP_FLAG_SHARED, &desc, D3D12_RESOURCE_STATE_COMMON, None, &mut r) }
            .map_err(win("Ressource anlegen"))?;
        r.ok_or("Ressource fehlt")?
    };
    let hal_textur = unsafe {
        wgpu::hal::dx12::Device::texture_from_raw(roh.clone(), format, wgpu::TextureDimension::D2, wgpu::Extent3d { width: b, height: h, depth_or_array_layers: 1 }, 1, 1)
    };
    let textur = unsafe {
        device.create_texture_from_hal::<Dx12>(
            hal_textur,
            &wgpu::TextureDescriptor {
                label: Some("roehre-austausch"),
                size: wgpu::Extent3d { width: b, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: BILD_NUTZUNG,
                view_formats: &[],
            },
            anfang,
        )
    };
    Ok((textur, roh))
}

/// Legt den Austausch für ein D3D12-Spiel an.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_d3d12_erstellen(p: *const RoehreAustauschParameter) -> *mut RoehreAustausch {
    geschuetzt(std::ptr::null_mut(), || {
        let p = unsafe { lies_parameter(p) }?;
        let adapter = adapter_suchen(&p.kennung)?;
        let features = adapter.features() & roehre_wgpu::empfohlene_features();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("roehre-austausch"),
            required_features: features,
            ..Default::default()
        }))
        .map_err(|e| format!("Gerät nicht angelegt: {e}"))?;
        let ziel_format = p.ausgabe.format;
        // Eingang: das Spiel übergibt ihn im Zustand PIXEL_SHADER_RESOURCE | NON_PIXEL_SHADER_RESOURCE
        let (eingang, r_ein) = geteiltes_bild(&device, EINGANG_FORMAT, p.eingang.breite, p.eingang.hoehe, wgpu::TextureUses::RESOURCE)?;
        let (ziel, r_ziel) = geteiltes_bild(&device, ziel_format, p.ziel.0, p.ziel.1, wgpu::TextureUses::UNINITIALIZED)?;
        let (geraet, zaun) = {
            let hal = unsafe { device.as_hal::<Dx12>() }.ok_or("Gerät ist nicht D3D12")?;
            let d = hal.raw_device().clone();
            let zaun: ID3D12Fence = unsafe { d.CreateFence(0, D3D12_FENCE_FLAG_SHARED) }.map_err(win("Zaun anlegen"))?;
            (d, zaun)
        };
        let zaehler = Arc::new(AtomicU64::new(0));
        let teile = D3d12Teile {
            geraet,
            bilder: [(r_ein, EINGANG_FORMAT, p.eingang.breite, p.eingang.hoehe), (r_ziel, ziel_format, p.ziel.0, p.ziel.1)],
            zaun: zaun.clone(),
            zaehler: zaehler.clone(),
        };
        let platz = Arbeitsplatz::neu(device, queue, p.shader, p.ausgabe, p.einstellungen.clone())?;
        let mut a = RoehreAustausch {
            platz,
            eingang,
            ziel,
            eingang_info: p.eingang,
            weg: Box::new(D3d12Weg { zaun, zaehler }),
            start: Instant::now(),
            teile: Box::new(teile),
        };
        a.platz.einstellungen(&p.einstellungen, true)?;
        Ok(Box::into_raw(Box::new(a)))
    })
}

fn teile(a: &RoehreAustausch) -> Result<&D3d12Teile, String> {
    a.teile.downcast_ref::<D3d12Teile>().ok_or_else(|| "kein D3D12-Austausch".to_string())
}

fn neues_handle(d: &ID3D12Device, objekt: &ID3D12DeviceChild) -> Result<HANDLE, String> {
    unsafe { d.CreateSharedHandle(objekt, None, GENERIC_ALL.0, PCWSTR::null()) }.map_err(win("geteiltes Handle"))
}

/// Liefert ein geteiltes Bild (0 = Eingang, 1 = Ziel) mit einem **neuen** NT-Handle. Das Spiel
/// öffnet es mit ID3D12Device::OpenSharedHandle und schließt es danach mit CloseHandle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_d3d12_bild(a: *const RoehreAustausch, welches: i32, aus: *mut RoehreD3D12Bild) -> i32 {
    geschuetzt(-1, || {
        let a = unsafe { a.as_ref() }.ok_or("Austausch fehlt (NULL)")?;
        let aus = unsafe { aus.as_mut() }.ok_or("Ausgabe fehlt (NULL)")?;
        let t = teile(a)?;
        let (r, f, b, h) = t.bilder.get(welches as usize).ok_or("welches: 0 = Eingang, 1 = Ziel")?;
        let kind: ID3D12DeviceChild = r.cast().map_err(win("Schnittstelle"))?;
        let handle = neues_handle(&t.geraet, &kind)?;
        *aus = RoehreD3D12Bild { handle: handle.0, dxgi_format: dxgi_format(*f).0 as u32, breite: *b, hoehe: *h };
        Ok(0)
    })
}

/// Liefert den geteilten Zaun mit einem neuen NT-Handle (öffnen mit OpenSharedHandle, danach
/// CloseHandle).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_d3d12_zaun(a: *const RoehreAustausch, handle: *mut *mut c_void) -> i32 {
    geschuetzt(-1, || {
        let a = unsafe { a.as_ref() }.ok_or("Austausch fehlt (NULL)")?;
        let t = teile(a)?;
        let kind: ID3D12DeviceChild = t.zaun.cast().map_err(win("Schnittstelle"))?;
        let h = neues_handle(&t.geraet, &kind)?;
        unsafe { *handle.as_mut().ok_or("handle fehlt (NULL)")? = h.0 };
        Ok(0)
    })
}

/// Zaunwerte für den **nächsten** Aufruf von roehre_austausch_zeichnen: das Spiel setzt den Zaun
/// auf `bereit`, wenn EINGANG beschrieben ist, und wartet auf `fertig`, bevor es ZIEL liest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_d3d12_zaunwerte(a: *const RoehreAustausch, bereit: *mut u64, fertig: *mut u64) -> i32 {
    geschuetzt(-1, || {
        let a = unsafe { a.as_ref() }.ok_or("Austausch fehlt (NULL)")?;
        let k = teile(a)?.zaehler.load(Ordering::SeqCst) + 1;
        unsafe {
            if let Some(b) = bereit.as_mut() {
                *b = 2 * k - 1;
            }
            if let Some(f) = fertig.as_mut() {
                *f = 2 * k;
            }
        }
        Ok(0)
    })
}
