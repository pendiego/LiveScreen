use std::sync::Mutex;

use anyhow::{Context as _, Result};
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
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess;
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::core::factory;
use windows::core::{HSTRING, IInspectable, Interface};

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

const PIXEL_FORMAT: DirectXPixelFormat = DirectXPixelFormat::B8G8R8A8UIntNormalized;
const SESSION_TYPE: &str = "Windows.Graphics.Capture.GraphicsCaptureSession";

fn property_present(name: &str) -> bool {
    ApiInformation::IsPropertyPresent(&HSTRING::from(SESSION_TYPE), &HSTRING::from(name))
        .unwrap_or(false)
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
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(&winrt, PIXEL_FORMAT, 2, size)?;
    let session = pool.CreateCaptureSession(&item)?;
    if property_present("IsBorderRequired") {
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
    let sendable_device = crate::gpu::Sendable(winrt);
    let last_size = Mutex::new(size);
    let frame_token = pool.FrameArrived(&TypedEventHandler::<
        Direct3D11CaptureFramePool,
        IInspectable,
    >::new(move |sender, _| {
        if let Some(pool) = sender.as_ref() {
            handle_frame(pool, &sendable_device, &last_size, &frame_sink);
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
    device: &crate::gpu::Sendable<IDirect3DDevice>,
    last_size: &Mutex<SizeInt32>,
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
    if resized {
        // Se o Recreate falhar, zera o tamanho conhecido para tentar de novo no próximo
        // quadro em vez de usar uma superfície com tamanho diferente do `content`.
        if pool.Recreate(&device.0, PIXEL_FORMAT, 2, content).is_err()
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
    sink.on_frame(&texture, content.Width as u32, content.Height as u32);
}
