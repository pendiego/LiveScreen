//! Viewer sem cópia para a CPU: o Video Processor do decoder (D3D11) escreve em texturas RGBA
//! compartilhadas por NT handle, e o wgpu (backend DX12 do egui) as abre como `ID3D12Resource`.
//!
//! Sincronização por duas fences compartilhadas, monotônicas:
//! - `produced = N`: o produtor terminou de escrever o quadro N (D3D11 `Signal`; a fila do wgpu espera);
//! - `consumed = N`: a UI terminou de copiar o quadro N para a textura de exibição (D3D12 `Signal`).
//!
//! Há 3 slots; o produtor só reusa um slot quando `consumed` já passou do último quadro nele e
//! nunca o slot publicado por último. Sem slot livre o quadro é decodificado mas não exibido.
//! A UI copia o slot para uma textura de exibição persistente (uma cópia GPU→GPU), então o egui
//! não precisa de bind group novo por quadro e o slot volta logo para o produtor.
use std::sync::Arc;

use anyhow::{Context as _, Result, anyhow};
use eframe::egui;
use eframe::egui_wgpu::{self, wgpu};
use windows::Win32::Foundation::{CloseHandle, HANDLE, LUID};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_FENCE_FLAG_SHARED, ID3D11Device, ID3D11Device5, ID3D11DeviceContext,
    ID3D11DeviceContext4, ID3D11Fence, ID3D11Texture2D,
};
use windows::Win32::Graphics::Direct3D12::{ID3D12CommandQueue, ID3D12Device, ID3D12Fence, ID3D12Resource};
use windows::Win32::Graphics::Dxgi::{DXGI_SHARED_RESOURCE_READ, DXGI_SHARED_RESOURCE_WRITE, IDXGIResource1};
use windows::core::{Interface, PCWSTR};

use crate::video::convert::Converter;

pub const SLOTS: usize = 3;

/// Adaptador da UI (wgpu); o decoder cria o device D3D11 nele para o compartilhamento valer.
#[derive(Clone, Copy)]
pub struct Target {
    pub luid: LUID,
}

/// Handles de uma geração do pool; fechados quando o último `GpuFrame` e o produtor os soltam.
pub struct Surfaces {
    generation: u64,
    width: u32,
    height: u32,
    slots: [HANDLE; SLOTS],
    produced: HANDLE,
    consumed: HANDLE,
}

// HANDLE é um ponteiro cru; handles NT são válidos em qualquer thread do processo.
unsafe impl Send for Surfaces {}
unsafe impl Sync for Surfaces {}

impl Drop for Surfaces {
    fn drop(&mut self) {
        for h in self.slots.iter().chain([&self.produced, &self.consumed]) {
            let _ = unsafe { CloseHandle(*h) };
        }
    }
}

pub struct GpuFrame {
    pub surfaces: Arc<Surfaces>,
    pub slot: usize,
    pub seq: u64,
}

impl GpuFrame {
    pub fn size(&self) -> (u32, u32) {
        (self.surfaces.width, self.surfaces.height)
    }
}

pub struct Producer {
    device5: ID3D11Device5,
    context4: ID3D11DeviceContext4,
    produced: Option<ID3D11Fence>,
    consumed: Option<ID3D11Fence>,
    slot_seq: [u64; SLOTS],
    seq: u64,
    last_slot: Option<usize>,
    surfaces: Option<Arc<Surfaces>>,
    generation: u64,
}

impl Producer {
    pub fn new(device: &ID3D11Device, context: &ID3D11DeviceContext) -> Result<Self> {
        Ok(Self {
            device5: device.cast().context("ID3D11Device5")?,
            context4: context.cast().context("ID3D11DeviceContext4")?,
            produced: None,
            consumed: None,
            slot_seq: [0; SLOTS],
            seq: 0,
            last_slot: None,
            surfaces: None,
            generation: 0,
        })
    }

    fn new_fence(&self) -> Result<(ID3D11Fence, HANDLE)> {
        let mut f: Option<ID3D11Fence> = None;
        unsafe { self.device5.CreateFence(0, D3D11_FENCE_FLAG_SHARED, &mut f) }.context("CreateFence")?;
        let f = f.context("fence nula")?;
        // 0x10000000 = GENERIC_ALL
        let h = unsafe { f.CreateSharedHandle(None, 0x1000_0000, PCWSTR::null()) }
            .context("CreateSharedHandle da fence")?;
        Ok((f, h))
    }

    /// Converte o quadro do decoder para um slot livre. `None`: a UI está atrasada ou parada.
    pub fn publish(
        &mut self,
        conv: &mut Converter,
        tex: &ID3D11Texture2D,
        slice: u32,
        size: (u32, u32),
    ) -> Result<Option<GpuFrame>> {
        conv.prepare(size.0, size.1)?;
        if conv.generation() != self.generation || self.surfaces.is_none() {
            let (width, height) = conv.output_size();
            // `Surfaces` fecha o que já foi criado se algo falhar no meio
            let mut surf = Surfaces {
                generation: conv.generation(),
                width,
                height,
                slots: [HANDLE::default(); SLOTS],
                produced: HANDLE::default(),
                consumed: HANDLE::default(),
            };
            for i in 0..SLOTS {
                let res: IDXGIResource1 = conv.pool_texture(i).cast().context("IDXGIResource1")?;
                surf.slots[i] = unsafe {
                    res.CreateSharedHandle(
                        None,
                        DXGI_SHARED_RESOURCE_READ.0 | DXGI_SHARED_RESOURCE_WRITE.0,
                        PCWSTR::null(),
                    )
                }
                .context("CreateSharedHandle da textura")?;
            }
            let (pf, ph) = self.new_fence()?;
            surf.produced = ph;
            let (cf, ch) = self.new_fence()?;
            surf.consumed = ch;
            self.produced = Some(pf);
            self.consumed = Some(cf);
            self.slot_seq = [0; SLOTS];
            self.last_slot = None;
            self.generation = surf.generation;
            self.surfaces = Some(Arc::new(surf));
        }
        let (Some(surfaces), Some(produced), Some(consumed)) =
            (&self.surfaces, &self.produced, &self.consumed)
        else {
            return Ok(None);
        };
        let done = unsafe { consumed.GetCompletedValue() };
        let Some(i) = (0..SLOTS).find(|&i| Some(i) != self.last_slot && done >= self.slot_seq[i]) else {
            return Ok(None);
        };
        self.seq += 1;
        conv.convert_into(tex, slice, size, i)?;
        unsafe {
            self.context4.Signal(produced, self.seq)?;
            self.context4.Flush();
        }
        self.slot_seq[i] = self.seq;
        self.last_slot = Some(i);
        Ok(Some(GpuFrame {
            surfaces: surfaces.clone(),
            slot: i,
            seq: self.seq,
        }))
    }
}

struct Imported {
    generation: u64,
    slots: Vec<wgpu::Texture>,
    produced: ID3D12Fence,
    consumed: ID3D12Fence,
    size: (u32, u32),
    display: wgpu::Texture,
}

pub struct Display {
    rs: egui_wgpu::RenderState,
    raw_device: ID3D12Device,
    raw_queue: ID3D12CommandQueue,
    imported: Option<Imported>,
    id: Option<egui::TextureId>,
    last_seq: u64,
}

impl Display {
    /// `None` se o egui não roda sobre DX12 (o viewer segue no caminho com readback).
    pub fn new(rs: &egui_wgpu::RenderState) -> Option<Display> {
        use wgpu::hal::api::Dx12;
        if rs.adapter.get_info().backend != wgpu::Backend::Dx12 {
            return None;
        }
        // guards soltos na hora: nada de segurar o hal entre chamadas do wgpu
        let raw_device = unsafe { rs.device.as_hal::<Dx12>() }?.raw_device().clone();
        let raw_queue = unsafe { rs.queue.as_hal::<Dx12>() }?.as_raw().clone();
        Some(Display {
            rs: rs.clone(),
            raw_device,
            raw_queue,
            imported: None,
            id: None,
            last_seq: 0,
        })
    }

    pub fn target(&self) -> Target {
        Target {
            luid: unsafe { self.raw_device.GetAdapterLuid() },
        }
    }

    pub fn reset(&mut self) {
        if let Some(id) = self.id.take() {
            self.rs.renderer.write().free_texture(&id);
        }
        self.imported = None;
        self.last_seq = 0;
    }

    fn import(&self, f: &GpuFrame) -> Result<Imported> {
        use wgpu::hal::api::Dx12;
        let s = &f.surfaces;
        let size = wgpu::Extent3d {
            width: s.width,
            height: s.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("livescreen-slot"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let mut slots = Vec::with_capacity(SLOTS);
        for h in s.slots {
            let mut res: Option<ID3D12Resource> = None;
            unsafe { self.raw_device.OpenSharedHandle(h, &mut res) }.context("OpenSharedHandle da textura")?;
            let res = res.ok_or_else(|| anyhow!("recurso D3D12 nulo"))?;
            // Recurso compartilhado nasce em COMMON e é promovido implicitamente para COPY_SOURCE;
            // como só serve de origem de cópia, o wgpu nunca emite barreira para ele.
            let hal = unsafe {
                wgpu::hal::dx12::Device::texture_from_raw(
                    res,
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::TextureDimension::D2,
                    size,
                    1,
                    1,
                )
            };
            slots.push(unsafe {
                self.rs
                    .device
                    .create_texture_from_hal::<Dx12>(hal, &desc, wgpu::TextureUses::COPY_SRC)
            });
        }
        let fence = |h: HANDLE| -> Result<ID3D12Fence> {
            let mut f: Option<ID3D12Fence> = None;
            unsafe { self.raw_device.OpenSharedHandle(h, &mut f) }.context("OpenSharedHandle da fence")?;
            f.ok_or_else(|| anyhow!("fence D3D12 nula"))
        };
        let display = self.rs.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("livescreen-display"),
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            ..desc
        });
        Ok(Imported {
            generation: s.generation,
            slots,
            produced: fence(s.produced)?,
            consumed: fence(s.consumed)?,
            size: (s.width, s.height),
            display,
        })
    }

    /// Copia o quadro para a textura de exibição (na fila do wgpu) e devolve o id do egui.
    pub fn show(&mut self, f: &GpuFrame) -> Result<(egui::TextureId, egui::Vec2)> {
        let fresh = self.imported.as_ref().is_none_or(|i| i.generation != f.surfaces.generation);
        if fresh {
            let imp = self.import(f)?;
            let view = imp.display.create_view(&wgpu::TextureViewDescriptor::default());
            let mut r = self.rs.renderer.write();
            match self.id {
                Some(id) => r.update_egui_texture_from_wgpu_texture(&self.rs.device, &view, wgpu::FilterMode::Linear, id),
                None => self.id = Some(r.register_native_texture(&self.rs.device, &view, wgpu::FilterMode::Linear)),
            }
            drop(r);
            self.imported = Some(imp);
        }
        let imp = self.imported.as_ref().context("sem textura importada")?;
        let id = self.id.context("sem id de textura")?;
        if f.seq > self.last_seq {
            let src = imp.slots.get(f.slot).context("slot inválido")?;
            let (w, h) = imp.size;
            unsafe { self.raw_queue.Wait(&imp.produced, f.seq) }?;
            let mut enc = self.rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("livescreen-copy"),
            });
            enc.copy_texture_to_texture(
                src.as_image_copy(),
                imp.display.as_image_copy(),
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
            self.rs.queue.submit([enc.finish()]);
            unsafe { self.raw_queue.Signal(&imp.consumed, f.seq) }?;
            self.last_seq = f.seq;
        }
        Ok((id, egui::vec2(imp.size.0 as f32, imp.size.1 as f32)))
    }
}
