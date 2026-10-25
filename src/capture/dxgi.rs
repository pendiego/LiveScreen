use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow, bail};
use tracing::warn;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_RESOURCE_MISC_GDI_COMPATIBLE,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, ID3D11Device, ID3D11DeviceContext,
    ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_FRAME_INFO, IDXGIDevice,
    IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource, IDXGISurface1,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CURSOR_SHOWING, CURSORINFO, DI_NORMAL, DrawIconEx, GetCursorInfo, HICON,
};
use windows::core::Interface;

use super::{SinkHandle, SourceId};

pub struct DxgiCapture {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl DxgiCapture {
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for DxgiCapture {
    /// Mesmo destino de `stop`: um `CaptureHandle` descartado sem parar não pode deixar a
    /// thread de captura viva enchendo o encoder de quadros.
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn start(source: &SourceId, device: &ID3D11Device, sink: SinkHandle) -> Result<DxgiCapture> {
    let SourceId::Monitor { hmonitor, .. } = source else {
        bail!("DXGI só captura monitores");
    };
    let hmonitor = *hmonitor;
    let device = device.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let (tx, rx) = mpsc::channel::<Result<(), String>>();
    let thread = std::thread::spawn(move || {
        let duplication = match create_duplication(&device, hmonitor) {
            Ok(duplication) => {
                let _ = tx.send(Ok(()));
                duplication
            }
            Err(e) => {
                let _ = tx.send(Err(format!("{e:#}")));
                return;
            }
        };
        run(duplication, &device, hmonitor, &sink, &thread_stop);
    });

    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(())) => Ok(DxgiCapture {
            stop,
            thread: Some(thread),
        }),
        Ok(Err(e)) => {
            let _ = thread.join();
            Err(anyhow!("{e}"))
        }
        Err(_) => {
            stop.store(true, Ordering::Relaxed);
            Err(anyhow!("tempo esgotado ao duplicar o monitor"))
        }
    }
}

fn create_duplication(device: &ID3D11Device, hmonitor: isize) -> Result<IDXGIOutputDuplication> {
    let dxgi_device: IDXGIDevice = device.cast().context("IDXGIDevice")?;
    let adapter = unsafe { dxgi_device.GetAdapter() }.context("IDXGIAdapter")?;
    let mut index = 0;
    loop {
        let Ok(output) = (unsafe { adapter.EnumOutputs(index) }) else {
            break;
        };
        index += 1;
        let Ok(desc) = (unsafe { output.GetDesc() }) else {
            continue;
        };
        if desc.Monitor.0 as isize != hmonitor {
            continue;
        }
        let output: IDXGIOutput1 = output.cast().context("IDXGIOutput1")?;
        return unsafe { output.DuplicateOutput(device) }.context("DuplicateOutput");
    }
    bail!("monitor não pertence ao adaptador do encoder")
}

fn run(
    mut duplication: IDXGIOutputDuplication,
    device: &ID3D11Device,
    hmonitor: isize,
    sink: &SinkHandle,
    stop: &AtomicBool,
) {
    let context = match unsafe { device.GetImmediateContext() } {
        Ok(c) => c,
        Err(e) => {
            sink.on_error(anyhow!("contexto D3D11: {e}"));
            return;
        }
    };
    let mut comp = Composer::default();
    // O primeiro quadro sempre vai (mesmo que o timestamp venha zerado), para a
    // transmissão começar com uma imagem e não só depois da próxima mudança.
    let mut delivered = false;
    while !stop.load(Ordering::Relaxed) {
        let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        match unsafe { duplication.AcquireNextFrame(100, &mut info, &mut resource) } {
            Ok(()) => {
                // `LastPresentTime == 0`: só o ponteiro mudou; a imagem duplicada não
                // mudou, então o cursor é redesenhado sobre a última imagem.
                let image = info.LastPresentTime != 0 || !delivered;
                let mut moved = false;
                if info.LastMouseUpdateTime != 0 {
                    let p = info.PointerPosition;
                    let now = p.Visible.as_bool().then_some(p.Position);
                    moved = now != comp.pointer;
                    comp.pointer = now;
                }
                if image
                    && let Some(resource) = &resource
                    && let Ok(texture) = resource.cast::<ID3D11Texture2D>()
                {
                    let mut desc = D3D11_TEXTURE2D_DESC::default();
                    unsafe { texture.GetDesc(&mut desc) };
                    if let Err(e) = comp.store(device, &context, &texture, desc.Width, desc.Height)
                    {
                        sink.on_error(anyhow!("cópia do quadro DXGI: {e}"));
                        let _ = unsafe { duplication.ReleaseFrame() };
                        return;
                    }
                    delivered = true;
                }
                if delivered
                    && (image || moved)
                    && let Some(tex) = comp.frame(device, &context)
                {
                    sink.on_frame(&tex, comp.size.0, comp.size.1);
                }
                let _ = unsafe { duplication.ReleaseFrame() };
            }
            Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => {}
            Err(e) if e.code() == DXGI_ERROR_ACCESS_LOST => {
                // Só existe uma duplicação por saída: soltar a morta antes de recriar.
                drop(duplication);
                let mut recovered = None;
                for _ in 0..5 {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(500));
                    if let Ok(new) = create_duplication(device, hmonitor) {
                        recovered = Some(new);
                        break;
                    }
                }
                match recovered {
                    Some(new) => duplication = new,
                    None => {
                        sink.on_error(anyhow!("duplicação do monitor perdida"));
                        return;
                    }
                }
            }
            Err(e) => {
                sink.on_error(anyhow!("AcquireNextFrame: {e}"));
                return;
            }
        }
    }
}

/// Imagem do desktop + cursor: a duplicação DXGI não inclui o ponteiro.
#[derive(Default)]
struct Composer {
    /// última imagem do desktop (cópia: o recurso da duplicação é devolvido em ReleaseFrame)
    base: Option<ID3D11Texture2D>,
    /// base + cursor; GDI_COMPATIBLE para o DrawIconEx
    out: Option<ID3D11Texture2D>,
    size: (u32, u32),
    /// canto sup. esq. do ponteiro quando visível neste monitor
    pointer: Option<POINT>,
    /// GetDC/DrawIconEx falhou uma vez: segue sem cursor em vez de travar o vídeo
    cursor_broken: bool,
}

fn texture(device: &ID3D11Device, w: u32, h: u32, gdi: bool) -> Result<ID3D11Texture2D> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: w,
        Height: h,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        CPUAccessFlags: 0,
        MiscFlags: if gdi {
            D3D11_RESOURCE_MISC_GDI_COMPATIBLE.0 as u32
        } else {
            0
        },
    };
    let mut tex = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut tex))? };
    tex.ok_or_else(|| anyhow!("textura nula"))
}

impl Composer {
    fn store(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        src: &ID3D11Texture2D,
        w: u32,
        h: u32,
    ) -> Result<()> {
        if self.size != (w, h) || self.base.is_none() {
            self.base = Some(texture(device, w, h, false)?);
            self.out = None;
            self.size = (w, h);
        }
        if let Some(base) = &self.base {
            unsafe { context.CopyResource(base, src) };
        }
        Ok(())
    }

    fn frame(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
    ) -> Option<ID3D11Texture2D> {
        let base = self.base.clone()?;
        let p = match self.pointer {
            Some(p) if !self.cursor_broken => p,
            _ => return Some(base),
        };
        let mut ci = CURSORINFO {
            cbSize: size_of::<CURSORINFO>() as u32,
            ..Default::default()
        };
        if unsafe { GetCursorInfo(&mut ci) }.is_err()
            || ci.flags.0 & CURSOR_SHOWING.0 == 0
            || ci.hCursor.is_invalid()
        {
            return Some(base);
        }
        match self.draw(device, context, &base, p, ci) {
            Ok(out) => Some(out),
            Err(e) => {
                warn!("cursor no DXGI: {e:#}");
                self.cursor_broken = true;
                Some(base)
            }
        }
    }

    fn draw(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        base: &ID3D11Texture2D,
        p: POINT,
        ci: CURSORINFO,
    ) -> Result<ID3D11Texture2D> {
        if self.out.is_none() {
            self.out = Some(texture(device, self.size.0, self.size.1, true)?);
        }
        let out = self.out.clone().context("saída nula")?;
        unsafe { context.CopyResource(&out, base) };
        let surface: IDXGISurface1 = out.cast()?;
        let hdc = unsafe { surface.GetDC(false)? };
        let drawn = unsafe {
            DrawIconEx(hdc, p.x, p.y, HICON(ci.hCursor.0), 0, 0, 0, None, DI_NORMAL)
        };
        // sempre soltar o DC, senão a textura fica presa ao GDI
        let released = unsafe { surface.ReleaseDC(None) };
        drawn?;
        released?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_USAGE_STAGING,
    };
    use windows::Win32::UI::HiDpi::{
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
    };
    use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

    use super::*;
    use crate::capture::FrameSink;

    type Grab = Arc<Mutex<Option<(Vec<u8>, u32, u32, u32)>>>;

    struct GrabSink {
        device: ID3D11Device,
        context: ID3D11DeviceContext,
        last: Grab,
    }

    impl FrameSink for GrabSink {
        fn on_frame(&mut self, tex: &ID3D11Texture2D, w: u32, h: u32) {
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            unsafe { tex.GetDesc(&mut desc) };
            desc.Usage = D3D11_USAGE_STAGING;
            desc.BindFlags = 0;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
            desc.MiscFlags = 0;
            let mut staging = None;
            unsafe { self.device.CreateTexture2D(&desc, None, Some(&mut staging)) }.unwrap();
            let staging = staging.unwrap();
            unsafe {
                self.context.CopyResource(&staging, tex);
                let mut m = D3D11_MAPPED_SUBRESOURCE::default();
                self.context
                    .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut m))
                    .unwrap();
                let len = m.RowPitch as usize * h as usize;
                let bytes = std::slice::from_raw_parts(m.pData as *const u8, len).to_vec();
                self.context.Unmap(&staging, 0);
                *self.last.lock().unwrap() = Some((bytes, w, h, m.RowPitch));
            }
        }
        fn on_closed(&mut self) {
            panic!("fechado");
        }
        fn on_error(&mut self, e: anyhow::Error) {
            panic!("{e:#}");
        }
    }

    fn block(f: &(Vec<u8>, u32, u32, u32), x: usize, y: usize) -> Vec<u8> {
        (y..y + 16)
            .flat_map(|r| {
                let o = r * f.3 as usize + x * 4;
                f.0[o..o + 64].to_vec()
            })
            .collect()
    }

    #[test]
    #[ignore = "usa o monitor e o cursor reais: cargo test --release -- --ignored dxgi_draws_cursor"]
    fn dxgi_draws_cursor() {
        let _ = unsafe {
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
        };
        let (device, context) = crate::gpu::create_device(None).unwrap();
        let source = crate::capture::sources::primary_monitor().unwrap();
        let last: Grab = Arc::new(Mutex::new(None));
        let sink = GrabSink {
            device: device.clone(),
            context,
            last: last.clone(),
        };
        unsafe { SetCursorPos(200, 200).unwrap() };
        let handle = start(&source, &device, SinkHandle::new(Box::new(sink))).unwrap();
        std::thread::sleep(Duration::from_secs(1));
        let f1 = last.lock().unwrap().clone().expect("sem quadro");
        unsafe { SetCursorPos(700, 500).unwrap() };
        std::thread::sleep(Duration::from_secs(1));
        let f2 = last.lock().unwrap().clone().expect("sem quadro");
        handle.stop();
        for (x, y) in [(200, 200), (700, 500)] {
            assert_ne!(
                block(&f1, x, y),
                block(&f2, x, y),
                "cursor não desenhado em ({x},{y}): deixe essa área da tela parada durante o teste"
            );
        }
    }
}
