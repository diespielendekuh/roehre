//! Vulkan-Weg des Austauschs: eigenes Gerät auf derselben Grafikkarte (über die deviceUUID
//! gefunden), Bilder in exportierbarem Speicher (Linux: OPAQUE_FD, Windows: OPAQUE_WIN32),
//! zwei binäre Semaphoren „bereit“ (Spiel → roehre) und „fertig“ (roehre → Spiel).

use std::ffi::CStr;
use std::time::Instant;

use ash::vk;
use wgpu::hal::api::Vulkan;
use wgpu::hal::vulkan as hv;

use crate::arbeitsplatz::Arbeitsplatz;
use crate::austausch::{lies_parameter, RoehreAustausch, RoehreAustauschParameter, Weg, BILD_NUTZUNG, EINGANG_FORMAT};
use crate::fehler::geschuetzt;

#[cfg(not(windows))]
mod plattform {
    use super::*;
    pub const SPEICHER_ART: vk::ExternalMemoryHandleTypeFlags = vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD;
    pub const SEMAPHOREN_ART: vk::ExternalSemaphoreHandleTypeFlags = vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD;
    pub const SPEICHER_ERW: &CStr = ash::khr::external_memory_fd::NAME;
    pub const SEMAPHOREN_ERW: &CStr = ash::khr::external_semaphore_fd::NAME;

    pub unsafe fn speicher_handle(i: &ash::Instance, d: &ash::Device, m: vk::DeviceMemory) -> Result<i64, vk::Result> {
        let e = ash::khr::external_memory_fd::Device::new(i, d);
        unsafe { e.get_memory_fd(&vk::MemoryGetFdInfoKHR::default().memory(m).handle_type(SPEICHER_ART)) }.map(|fd| fd as i64)
    }

    pub unsafe fn semaphoren_handle(i: &ash::Instance, d: &ash::Device, s: vk::Semaphore) -> Result<i64, vk::Result> {
        let e = ash::khr::external_semaphore_fd::Device::new(i, d);
        unsafe { e.get_semaphore_fd(&vk::SemaphoreGetFdInfoKHR::default().semaphore(s).handle_type(SEMAPHOREN_ART)) }.map(|fd| fd as i64)
    }
}

#[cfg(windows)]
mod plattform {
    use super::*;
    pub const SPEICHER_ART: vk::ExternalMemoryHandleTypeFlags = vk::ExternalMemoryHandleTypeFlags::OPAQUE_WIN32;
    pub const SEMAPHOREN_ART: vk::ExternalSemaphoreHandleTypeFlags = vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32;
    pub const SPEICHER_ERW: &CStr = ash::khr::external_memory_win32::NAME;
    pub const SEMAPHOREN_ERW: &CStr = ash::khr::external_semaphore_win32::NAME;

    pub unsafe fn speicher_handle(i: &ash::Instance, d: &ash::Device, m: vk::DeviceMemory) -> Result<i64, vk::Result> {
        let e = ash::khr::external_memory_win32::Device::new(i, d);
        unsafe { e.get_memory_win32_handle(&vk::MemoryGetWin32HandleInfoKHR::default().memory(m).handle_type(SPEICHER_ART)) }.map(|h| h as i64)
    }

    pub unsafe fn semaphoren_handle(i: &ash::Instance, d: &ash::Device, s: vk::Semaphore) -> Result<i64, vk::Result> {
        let e = ash::khr::external_semaphore_win32::Device::new(i, d);
        unsafe { e.get_semaphore_win32_handle(&vk::SemaphoreGetWin32HandleInfoKHR::default().semaphore(s).handle_type(SEMAPHOREN_ART)) }
            .map(|h| h as i64)
    }
}

use plattform::*;

/// Beschreibung eines geteilten Bilds für das Spiel (siehe roehre.h).
#[repr(C)]
#[derive(Default)]
pub struct RoehreVulkanBild {
    pub handle: i64,
    pub handle_art: u32,
    pub vk_format: u32,
    pub vk_nutzung: u32,
    pub breite: u32,
    pub hoehe: u32,
    pub speicher_groesse: u64,
    pub speicher_typ: u32,
    pub dediziert: u32,
}

const VK_NUTZUNG: vk::ImageUsageFlags = vk::ImageUsageFlags::from_raw(
    vk::ImageUsageFlags::SAMPLED.as_raw()
        | vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw()
        | vk::ImageUsageFlags::TRANSFER_SRC.as_raw()
        | vk::ImageUsageFlags::TRANSFER_DST.as_raw(),
);

fn vk_format(f: wgpu::TextureFormat) -> vk::Format {
    match f {
        wgpu::TextureFormat::Rgba8Unorm => vk::Format::R8G8B8A8_UNORM,
        wgpu::TextureFormat::Rgba8UnormSrgb => vk::Format::R8G8B8A8_SRGB,
        wgpu::TextureFormat::Rgba16Float => vk::Format::R16G16B16A16_SFLOAT,
        wgpu::TextureFormat::Rgb10a2Unorm => vk::Format::A2B10G10R10_UNORM_PACK32,
        _ => unreachable!("nur die Formate des Austauschs"),
    }
}

struct Geteilt {
    speicher: vk::DeviceMemory,
    groesse: u64,
    typ: u32,
    format: wgpu::TextureFormat,
    breite: u32,
    hoehe: u32,
}

/// Plattformteil: was das Spiel abholen kann, und die Semaphoren.
struct VulkanTeile {
    instanz: ash::Instance,
    geraet: ash::Device,
    bilder: [Geteilt; 2],
    semaphoren: [vk::Semaphore; 2],
}

impl Drop for VulkanTeile {
    fn drop(&mut self) {
        unsafe {
            let _ = self.geraet.device_wait_idle();
            for s in self.semaphoren.into_iter().filter(|s| *s != vk::Semaphore::null()) {
                self.geraet.destroy_semaphore(s, None);
            }
        }
    }
}

struct VulkanWeg {
    bereit: vk::Semaphore,
    fertig: vk::Semaphore,
}

impl Weg for VulkanWeg {
    fn vor_submit(&mut self, queue: &wgpu::Queue) -> Result<(), String> {
        if self.bereit == vk::Semaphore::null() {
            return Ok(()); // Notweg: das Spiel ordnet über die CPU
        }
        let q = unsafe { queue.as_hal::<Vulkan>() }.ok_or("Warteschlange ist nicht Vulkan")?;
        q.add_wait_semaphore(self.bereit, None, vk::PipelineStageFlags::ALL_COMMANDS);
        q.add_signal_semaphore(self.fertig, None);
        Ok(())
    }

    fn nach_submit(&mut self, _queue: &wgpu::Queue) -> Result<(), String> {
        Ok(())
    }
}

fn vk_fehler(was: &str) -> impl Fn(vk::Result) -> String + '_ {
    move |e| format!("{was}: {e}")
}

/// Sucht den Adapter mit der deviceUUID des Spiels (alles 0: der erste Vulkan-Adapter).
fn adapter_suchen(kennung: &[u8; 16]) -> Result<wgpu::Adapter, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::VULKAN, ..wgpu::InstanceDescriptor::new_without_display_handle_from_env() });
    let adapter = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
    if adapter.is_empty() {
        return Err("kein Vulkan-Adapter gefunden".into());
    }
    if kennung.iter().all(|b| *b == 0) {
        return Ok(adapter.into_iter().next().expect("nicht leer"));
    }
    let mut gesehen = Vec::new();
    for a in adapter {
        let uuid = {
            let hal = unsafe { a.as_hal::<Vulkan>() }.ok_or("Adapter ist nicht Vulkan")?;
            let inst = hal.shared_instance().raw_instance();
            let mut id = vk::PhysicalDeviceIDProperties::default();
            let mut p2 = vk::PhysicalDeviceProperties2::default().push_next(&mut id);
            unsafe { inst.get_physical_device_properties2(hal.raw_physical_device(), &mut p2) };
            id.device_uuid
        };
        if uuid == *kennung {
            return Ok(a);
        }
        gesehen.push(a.get_info().name);
    }
    Err(format!("keine Grafikkarte mit dieser deviceUUID (gefunden: {})", gesehen.join(", ")))
}

/// Legt das Gerät von roehre mit den Erweiterungen für geteilten Speicher an. Liefert dazu, ob
/// geteilte Semaphoren möglich sind (Mesa lavapipe kann sie z. B. nicht; dann Notweg über die
/// CPU, siehe roehre.h).
fn geraet_anlegen(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue, bool), String> {
    let features = adapter.features() & roehre_wgpu::empfohlene_features();
    let limits = wgpu::Limits::default().using_resolution(adapter.limits());
    let offen = {
        let hal = unsafe { adapter.as_hal::<Vulkan>() }.ok_or("Adapter ist nicht Vulkan")?;
        let inst = hal.shared_instance().raw_instance();
        let vorhanden = unsafe { inst.enumerate_device_extension_properties(hal.raw_physical_device()) }.map_err(vk_fehler("Erweiterungen"))?;
        let hat = |erw: &CStr| vorhanden.iter().any(|e| e.extension_name_as_c_str() == Ok(erw));
        if !hat(SPEICHER_ERW) {
            return Err(format!("Grafiktreiber bietet {SPEICHER_ERW:?} nicht an"));
        }
        let mit_semaphoren = hat(SEMAPHOREN_ERW);
        let erweiterungen: Vec<&'static CStr> = if mit_semaphoren { vec![SPEICHER_ERW, SEMAPHOREN_ERW] } else { vec![SPEICHER_ERW] };
        let offen = unsafe {
            hal.open_with_callback(
                features,
                &limits,
                &wgpu::MemoryHints::Performance,
                Some(Box::new(move |args: hv::CreateDeviceCallbackArgs| {
                    for erw in erweiterungen {
                        if !args.extensions.contains(&erw) {
                            args.extensions.push(erw);
                        }
                    }
                })),
            )
        }
        .map_err(|e| format!("Gerät nicht angelegt: {e}"))?;
        (offen, mit_semaphoren)
    };
    let (offen, mit_semaphoren) = offen;
    let (d, q) = unsafe {
        adapter.create_device_from_hal::<Vulkan>(
            offen,
            &wgpu::DeviceDescriptor { label: Some("roehre-austausch"), required_features: features, required_limits: limits, ..Default::default() },
        )
    }
    .map_err(|e| format!("Gerät nicht angelegt: {e}"))?;
    Ok((d, q, mit_semaphoren))
}

/// Ein Bild in exportierbarem Speicher, als wgpu-Textur.
fn geteiltes_bild(device: &wgpu::Device, format: wgpu::TextureFormat, b: u32, h: u32, anfang: wgpu::TextureUses) -> Result<(wgpu::Texture, Geteilt), String> {
    let (hal_textur, geteilt) = {
        let hal = unsafe { device.as_hal::<Vulkan>() }.ok_or("Gerät ist nicht Vulkan")?;
        let d = hal.raw_device().clone();
        let inst = hal.shared_instance().raw_instance();
        let mut extern_info = vk::ExternalMemoryImageCreateInfo::default().handle_types(SPEICHER_ART);
        let info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk_format(format))
            .extent(vk::Extent3D { width: b, height: h, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(VK_NUTZUNG)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut extern_info);
        let bild = unsafe { d.create_image(&info, None) }.map_err(vk_fehler("Bild anlegen"))?;
        let bedarf = unsafe { d.get_image_memory_requirements(bild) };
        let eigenschaften = unsafe { inst.get_physical_device_memory_properties(hal.raw_physical_device()) };
        let typ = (0..eigenschaften.memory_type_count)
            .find(|&i| {
                bedarf.memory_type_bits & (1 << i) != 0
                    && eigenschaften.memory_types[i as usize].property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .ok_or("kein passender Grafikspeicher")?;
        let mut dediziert = vk::MemoryDedicatedAllocateInfo::default().image(bild);
        let mut export = vk::ExportMemoryAllocateInfo::default().handle_types(SPEICHER_ART);
        let zuteilung = vk::MemoryAllocateInfo::default()
            .allocation_size(bedarf.size)
            .memory_type_index(typ)
            .push_next(&mut export)
            .push_next(&mut dediziert);
        let speicher = unsafe { d.allocate_memory(&zuteilung, None) }.map_err(vk_fehler("Speicher anlegen"))?;
        unsafe { d.bind_image_memory(bild, speicher, 0) }.map_err(vk_fehler("Speicher binden"))?;
        let d2 = d.clone();
        let hal_textur = unsafe {
            hal.texture_from_raw(
                bild,
                &wgpu::hal::TextureDescriptor {
                    label: Some("roehre-austausch"),
                    size: wgpu::Extent3d { width: b, height: h, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUses::RESOURCE | wgpu::TextureUses::COLOR_TARGET | wgpu::TextureUses::COPY_SRC | wgpu::TextureUses::COPY_DST,
                    memory_flags: wgpu::hal::MemoryFlags::empty(),
                    view_formats: vec![],
                },
                Some(Box::new(move || {
                    d2.destroy_image(bild, None);
                    d2.free_memory(speicher, None);
                })),
                hv::TextureMemory::External,
            )
        };
        (hal_textur, Geteilt { speicher, groesse: bedarf.size, typ, format, breite: b, hoehe: h })
    };
    let textur = unsafe {
        device.create_texture_from_hal::<Vulkan>(
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
    Ok((textur, geteilt))
}

/// Legt den Austausch für ein Vulkan-Spiel an.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_vulkan_erstellen(p: *const RoehreAustauschParameter) -> *mut RoehreAustausch {
    geschuetzt(std::ptr::null_mut(), || {
        let p = unsafe { lies_parameter(p) }?;
        let adapter = adapter_suchen(&p.kennung)?;
        let (device, queue, mit_semaphoren) = geraet_anlegen(&adapter)?;
        let ziel_format = p.ausgabe.format;
        // Eingang: das Spiel übergibt ihn im Layout SHADER_READ_ONLY_OPTIMAL
        let (eingang, g_ein) = geteiltes_bild(&device, EINGANG_FORMAT, p.eingang.breite, p.eingang.hoehe, wgpu::TextureUses::RESOURCE)?;
        let (ziel, g_ziel) = geteiltes_bild(&device, ziel_format, p.ziel.0, p.ziel.1, wgpu::TextureUses::UNINITIALIZED)?;
        let (instanz, geraet) = {
            let hal = unsafe { device.as_hal::<Vulkan>() }.ok_or("Gerät ist nicht Vulkan")?;
            (hal.shared_instance().raw_instance().clone(), hal.raw_device().clone())
        };
        let mut semaphoren = [vk::Semaphore::null(); 2];
        for s in semaphoren.iter_mut().filter(|_| mit_semaphoren) {
            let mut export = vk::ExportSemaphoreCreateInfo::default().handle_types(SEMAPHOREN_ART);
            *s = unsafe { geraet.create_semaphore(&vk::SemaphoreCreateInfo::default().push_next(&mut export), None) }.map_err(vk_fehler("Semaphore"))?;
        }
        let teile = VulkanTeile { instanz, geraet, bilder: [g_ein, g_ziel], semaphoren };
        let platz = Arbeitsplatz::neu(device, queue, p.shader, p.ausgabe, p.einstellungen.clone())?;
        let mut a = RoehreAustausch {
            platz,
            eingang,
            ziel,
            eingang_info: p.eingang,
            weg: Box::new(VulkanWeg { bereit: semaphoren[0], fertig: semaphoren[1] }),
            start: Instant::now(),
            teile: Box::new(teile),
        };
        a.platz.einstellungen(&p.einstellungen, true)?;
        Ok(Box::into_raw(Box::new(a)))
    })
}

fn teile(a: &RoehreAustausch) -> Result<&VulkanTeile, String> {
    a.teile.downcast_ref::<VulkanTeile>().ok_or_else(|| "kein Vulkan-Austausch".to_string())
}

/// Liefert ein geteiltes Bild (0 = Eingang, 1 = Ziel) mit einem **neuen** Handle; das Spiel
/// übernimmt es (Linux: der fd gehört nach dem Import dem Treiber; Windows: Handle schließen).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_vulkan_bild(a: *const RoehreAustausch, welches: i32, aus: *mut RoehreVulkanBild) -> i32 {
    geschuetzt(-1, || {
        let a = unsafe { a.as_ref() }.ok_or("Austausch fehlt (NULL)")?;
        let aus = unsafe { aus.as_mut() }.ok_or("Ausgabe fehlt (NULL)")?;
        let t = teile(a)?;
        let g = t.bilder.get(welches as usize).ok_or("welches: 0 = Eingang, 1 = Ziel")?;
        *aus = RoehreVulkanBild {
            handle: unsafe { speicher_handle(&t.instanz, &t.geraet, g.speicher) }.map_err(vk_fehler("Handle"))?,
            handle_art: SPEICHER_ART.as_raw(),
            vk_format: vk_format(g.format).as_raw() as u32,
            vk_nutzung: VK_NUTZUNG.as_raw(),
            breite: g.breite,
            hoehe: g.hoehe,
            speicher_groesse: g.groesse,
            speicher_typ: g.typ,
            dediziert: 1,
        };
        Ok(0)
    })
}

/// Liefert eine Semaphore (0 = „bereit“: das Spiel meldet, 1 = „fertig“: roehre meldet) mit
/// einem neuen Handle (binäre Semaphore, dauerhaft zu importieren).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roehre_vulkan_semaphore(a: *const RoehreAustausch, welches: i32, handle: *mut i64, handle_art: *mut u32) -> i32 {
    geschuetzt(-1, || {
        let a = unsafe { a.as_ref() }.ok_or("Austausch fehlt (NULL)")?;
        let t = teile(a)?;
        let s = *t.semaphoren.get(welches as usize).ok_or("welches: 0 = bereit, 1 = fertig")?;
        if s == vk::Semaphore::null() {
            return Err(format!("Grafiktreiber bietet {SEMAPHOREN_ERW:?} nicht an: Notweg ohne Semaphoren benutzen (roehre.h)"));
        }
        let h = unsafe { semaphoren_handle(&t.instanz, &t.geraet, s) }.map_err(vk_fehler("Handle"))?;
        unsafe {
            *handle.as_mut().ok_or("handle fehlt (NULL)")? = h;
            if let Some(art) = handle_art.as_mut() {
                *art = SEMAPHOREN_ART.as_raw();
            }
        }
        Ok(0)
    })
}
