use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use tracing::{info, warn};
use windows::Foundation::Metadata::ApiInformation;
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{
    Direct3D11CaptureFramePool, GraphicsCaptureAccess, GraphicsCaptureAccessKind,
    GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::SizeInt32;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D11::{D3D11_TEXTURE2D_DESC, ID3D11Device, ID3D11Texture2D};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R16G16B16A16_FLOAT;
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess;
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::core::factory;
use windows::core::{HSTRING, IInspectable, Interface};

use super::hdr::{self, HdrConverter};
use super::{SinkHandle, SourceId};

pub struct WgcCapture {
    session: GraphicsCaptureSession,
    pool: Direct3D11CaptureFramePool,
    item: GraphicsCaptureItem,
    frame_token: i64,
    closed_token: i64,
}

impl WgcCapture {
    pub fn stop(self) {
        let _ = self.pool.RemoveFrameArrived(self.frame_token);
        let _ = self.item.RemoveClosed(self.closed_token);
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}

const DEFAULT_PIXEL_FORMAT: DirectXPixelFormat = DirectXPixelFormat::B8G8R8A8UIntNormalized;
const SESSION_TYPE: &str = "Windows.Graphics.Capture.GraphicsCaptureSession";

fn property_present(name: &str) -> bool {
    ApiInformation::IsPropertyPresent(&HSTRING::from(SESSION_TYPE), &HSTRING::from(name))
        .unwrap_or(false)
}

/// `false` no Windows 10 (antes do build 20348): lá o WGC sempre desenha a borda amarela.
pub fn border_removable() -> bool {
    property_present("IsBorderRequired")
}

pub fn start(source: &SourceId, device: &ID3D11Device, sink: SinkHandle) -> Result<WgcCapture> {
    crate::gpu::com_init();
    let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
        .context("IGraphicsCaptureItemInterop")?;
    let item: GraphicsCaptureItem = unsafe {
        match source {
            SourceId::Monitor { hmonitor, .. } => {
                interop.CreateForMonitor(HMONITOR(*hmonitor as *mut _))
            }
            SourceId::Window { hwnd, .. } => interop.CreateForWindow(HWND(*hwnd as *mut _)),
        }
    }
    .context("criar item WGC")?;

    if let Ok(op) = GraphicsCaptureAccess::RequestAccessAsync(GraphicsCaptureAccessKind::Borderless)
    {
        let _ = op.join();
    }

    let winrt = crate::gpu::winrt_device(device)?;
    let size = item.Size().context("tamanho do item WGC")?;

    let hdr_info = hdr::detect_hdr(device, source);
    let mut converter = None;
    let initial_format = if let Some(info) = hdr_info {
        match HdrConverter::new(device, info.sdr_white_nits) {
            Ok(conv) => {
                info!("captura HDR -> SDR (branco {} nits)", info.sdr_white_nits);
                converter = Some(conv);
                DirectXPixelFormat::R16G16B16A16Float
            }
            Err(e) => {
                warn!("falha ao inicializar conversor HDR -> SDR, usando BGRA8: {e:#}");
                DEFAULT_PIXEL_FORMAT
            }
        }
    } else {
        DEFAULT_PIXEL_FORMAT
    };

    let pool = match Direct3D11CaptureFramePool::CreateFreeThreaded(&winrt, initial_format, 2, size) {
        Ok(p) => p,
        Err(e) if initial_format == DirectXPixelFormat::R16G16B16A16Float => {
            warn!("falha ao criar pool WGC em FP16, revertendo para B8G8R8A8: {e:#}");
            converter = None;
            Direct3D11CaptureFramePool::CreateFreeThreaded(&winrt, DEFAULT_PIXEL_FORMAT, 2, size)?
        }
        Err(e) => return Err(e.into()),
    };
    let session = pool.CreateCaptureSession(&item)?;
    if border_removable() {
        let _ = session.SetIsBorderRequired(false);
    }
    if property_present("IsCursorCaptureEnabled") {
        let _ = session.SetIsCursorCaptureEnabled(true);
    }
    // sem isto o WGC limita a entrega de quadros e a "Nativa (monitor)" não passa disso
    if property_present("MinUpdateInterval") {
        let _ = session.SetMinUpdateInterval(windows::Foundation::TimeSpan { Duration: 10_000 });
    }

    let frame_sink = sink.clone();
    let sendable_winrt = crate::gpu::Sendable(winrt);
    let d3d_device = device.clone();
    let last_size = Mutex::new(size);
    let format_atomic = Arc::new(AtomicI32::new(initial_format.0));
    let converter_mutex = Arc::new(Mutex::new(converter));
    let frame_count = Arc::new(AtomicU64::new(0));
    let source_clone = source.clone();

    let frame_token = pool.FrameArrived(&TypedEventHandler::<
        Direct3D11CaptureFramePool,
        IInspectable,
    >::new(move |sender, _| {
        if let Some(pool) = sender.as_ref() {
            handle_frame(
                pool,
                &sendable_winrt,
                &d3d_device,
                &last_size,
                &format_atomic,
                &converter_mutex,
                &frame_count,
                &source_clone,
                &frame_sink,
            );
        }
        Ok(())
    }))?;

    let closed_sink = sink;
    let closed_token = item.Closed(
        &TypedEventHandler::<GraphicsCaptureItem, IInspectable>::new(move |_, _| {
            closed_sink.on_closed();
            Ok(())
        }),
    )?;

    session.StartCapture()?;
    Ok(WgcCapture {
        session,
        pool,
        item,
        frame_token,
        closed_token,
    })
}

fn handle_frame(
    pool: &Direct3D11CaptureFramePool,
    winrt_device: &crate::gpu::Sendable<IDirect3DDevice>,
    d3d_device: &ID3D11Device,
    last_size: &Mutex<SizeInt32>,
    format_atomic: &AtomicI32,
    converter_mutex: &Mutex<Option<HdrConverter>>,
    frame_count: &AtomicU64,
    source: &SourceId,
    sink: &SinkHandle,
) {
    let Ok(frame) = pool.TryGetNextFrame() else {
        return;
    };
    let Ok(content) = frame.ContentSize() else {
        return;
    };
    if content.Width <= 0 || content.Height <= 0 {
        return;
    }

    let current_format = DirectXPixelFormat(format_atomic.load(Ordering::Relaxed));

    // A cada ~60 quadros, verifica se o estado de HDR do monitor ou janela mudou
    let count = frame_count.fetch_add(1, Ordering::Relaxed);
    let mut target_format = current_format;
    if count % 60 == 0 {
        let current_hdr = hdr::detect_hdr(d3d_device, source);
        if let Some(info) = current_hdr {
            if current_format != DirectXPixelFormat::R16G16B16A16Float {
                if let Ok(mut conv_guard) = converter_mutex.lock() {
                    match HdrConverter::new(d3d_device, info.sdr_white_nits) {
                        Ok(c) => {
                            info!(
                                "HDR ativado na fonte; alternando captura WGC para FP16 (branco {} nits)",
                                info.sdr_white_nits
                            );
                            *conv_guard = Some(c);
                            target_format = DirectXPixelFormat::R16G16B16A16Float;
                        }
                        Err(e) => {
                            warn!("falha ao criar conversor HDR ao alternar: {e:#}");
                        }
                    }
                }
            } else if let Ok(mut conv_guard) = converter_mutex.lock()
                && let Some(c) = conv_guard.as_mut()
            {
                c.update_sdr_white_nits(info.sdr_white_nits);
            }
        } else if current_format == DirectXPixelFormat::R16G16B16A16Float {
            info!("HDR desativado na fonte; alternando captura WGC para B8G8R8A8");
            target_format = DEFAULT_PIXEL_FORMAT;
            if let Ok(mut conv_guard) = converter_mutex.lock() {
                *conv_guard = None;
            }
        }
    }

    let format_changed = target_format != current_format;
    if format_changed {
        format_atomic.store(target_format.0, Ordering::Relaxed);
    }

    let resized = match last_size.lock() {
        Ok(mut last) => {
            let changed = last.Width != content.Width || last.Height != content.Height;
            if changed {
                *last = content;
            }
            changed
        }
        Err(_) => false,
    };

    if resized || format_changed {
        let effective_format = DirectXPixelFormat(format_atomic.load(Ordering::Relaxed));
        if pool.Recreate(&winrt_device.0, effective_format, 2, content).is_err()
            && let Ok(mut last) = last_size.lock()
        {
            *last = SizeInt32 {
                Width: 0,
                Height: 0,
            };
        }
        return;
    }

    let Ok(surface) = frame.Surface() else { return };
    let Ok(access) = surface.cast::<IDirect3DDxgiInterfaceAccess>() else {
        return;
    };
    let Ok(texture) = (unsafe { access.GetInterface::<ID3D11Texture2D>() }) else {
        return;
    };

    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe { texture.GetDesc(&mut desc) };

    if desc.Format == DXGI_FORMAT_R16G16B16A16_FLOAT {
        if let Ok(mut conv_guard) = converter_mutex.lock()
            && let Some(conv) = conv_guard.as_mut()
            && let Ok(context) = unsafe { d3d_device.GetImmediateContext() }
        {
            match conv.convert(
                d3d_device,
                &context,
                &texture,
                content.Width as u32,
                content.Height as u32,
            ) {
                Ok(sdr_tex) => {
                    sink.on_frame(&sdr_tex, content.Width as u32, content.Height as u32);
                    return;
                }
                Err(e) => {
                    warn!("conversão HDR -> SDR WGC falhou: {e:#}");
                }
            }
        }
    }

    sink.on_frame(&texture, content.Width as u32, content.Height as u32);
}
