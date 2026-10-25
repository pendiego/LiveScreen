pub mod dxgi;
pub mod sources;
pub mod wgc;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Result, bail};
use tracing::{info, warn};
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D};

use crate::config::Overrides;

pub use sources::{SourceId, SourceInfo, enumerate, refresh_rate, source_size, thumbnail};

pub trait FrameSink: Send {
    fn on_frame(&mut self, tex: &ID3D11Texture2D, w: u32, h: u32);
    fn on_closed(&mut self);
    fn on_error(&mut self, e: anyhow::Error);
}

/// Sink compartilhável: o mesmo destino serve ao WGC, ao DXGI e ao fallback entre eles.
#[derive(Clone)]
pub struct SinkHandle(Arc<Mutex<Box<dyn FrameSink>>>);

impl SinkHandle {
    pub fn new(sink: Box<dyn FrameSink>) -> Self {
        Self(Arc::new(Mutex::new(sink)))
    }

    /// Um panic num quadro não pode matar todos os quadros seguintes: o mutex envenenado
    /// é recuperado em vez de descartar o sink.
    fn lock(&self) -> std::sync::MutexGuard<'_, Box<dyn FrameSink>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn on_frame(&self, tex: &ID3D11Texture2D, w: u32, h: u32) {
        let mut sink = self.lock();
        sink.on_frame(tex, w, h);
    }

    pub fn on_closed(&self) {
        let mut sink = self.lock();
        sink.on_closed();
    }

    pub fn on_error(&self, e: anyhow::Error) {
        let mut sink = self.lock();
        sink.on_error(e);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    GraphicsCapture,
    Dxgi,
}

impl Backend {
    pub fn label(self) -> &'static str {
        match self {
            Backend::GraphicsCapture => "WGC",
            Backend::Dxgi => "DXGI",
        }
    }
}

pub enum CaptureHandle {
    Wgc(wgc::WgcCapture),
    Dxgi(dxgi::DxgiCapture),
}

impl CaptureHandle {
    pub fn stop(self) {
        match self {
            CaptureHandle::Wgc(c) => c.stop(),
            CaptureHandle::Dxgi(c) => c.stop(),
        }
    }
}

/// Primeiro quadro sinaliza o canal para o watchdog do `start`.
struct FirstFrameSink {
    inner: SinkHandle,
    tx: Option<std::sync::mpsc::Sender<()>>,
}

impl FrameSink for FirstFrameSink {
    fn on_frame(&mut self, tex: &ID3D11Texture2D, w: u32, h: u32) {
        self.inner.on_frame(tex, w, h);
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        }
    }

    fn on_closed(&mut self) {
        self.inner.on_closed();
    }

    fn on_error(&mut self, e: anyhow::Error) {
        self.inner.on_error(e);
    }
}

pub fn start(
    source: &SourceId,
    overrides: &Overrides,
    device: &ID3D11Device,
    sink: Box<dyn FrameSink>,
) -> Result<(CaptureHandle, Backend)> {
    let shared = SinkHandle::new(sink);
    match source {
        SourceId::Window { .. } => {
            if overrides.deny_graphics_capture {
                bail!("captura de janela exige WGC, vetado pela config");
            }
            let capture = wgc::start(source, device, shared)?;
            Ok((CaptureHandle::Wgc(capture), Backend::GraphicsCapture))
        }
        SourceId::Monitor { .. } => {
            if overrides.deny_graphics_capture && overrides.deny_dxgi {
                bail!("nenhum backend de captura permitido");
            }
            // no Windows 10 o WGC sempre desenha a borda amarela; a duplicação DXGI não tem borda
            if !overrides.deny_dxgi && !wgc::border_removable() {
                match dxgi::start(source, device, shared.clone()) {
                    Ok(capture) => return Ok((CaptureHandle::Dxgi(capture), Backend::Dxgi)),
                    Err(e) if overrides.deny_graphics_capture => return Err(e),
                    Err(e) => {
                        info!("DXGI indisponível ({e:#}); WGC com borda amarela");
                        let capture = wgc::start(source, device, shared)?;
                        return Ok((CaptureHandle::Wgc(capture), Backend::GraphicsCapture));
                    }
                }
            }
            if !overrides.deny_graphics_capture {
                let (tx, rx) = std::sync::mpsc::channel();
                let probe = SinkHandle::new(Box::new(FirstFrameSink {
                    inner: shared.clone(),
                    tx: Some(tx),
                }));
                let (motivo, started) = match wgc::start(source, device, probe) {
                    Ok(capture) => match rx.recv_timeout(Duration::from_secs(2)) {
                        Ok(()) => {
                            return Ok((CaptureHandle::Wgc(capture), Backend::GraphicsCapture));
                        }
                        Err(_) => {
                            capture.stop();
                            ("sem quadros em 2 s".to_string(), true)
                        }
                    },
                    Err(e) => (format!("{e:#}"), false),
                };
                if overrides.deny_dxgi {
                    bail!("nenhum backend de captura permitido");
                }
                info!("fallback WGC→DXGI: {motivo}");
                // o WGC leva um instante para soltar a duplicação interna do monitor
                if started {
                    std::thread::sleep(Duration::from_millis(300));
                }
            }
            let capture = dxgi::start(source, device, shared)?;
            Ok((CaptureHandle::Dxgi(capture), Backend::Dxgi))
        }
    }
}

/// Roda cada backend permitido por 3 s direto (sem fallback) e conta quadros.
pub fn selftest() -> Result<()> {
    let overrides = crate::config::overrides();
    if overrides.deny_graphics_capture && overrides.deny_dxgi {
        bail!("nenhum backend de captura permitido");
    }
    let Some(source) = sources::primary_monitor() else {
        bail!("monitor primário não encontrado");
    };
    let (device, context) = crate::gpu::create_device()?;
    crate::gpu::com_init();

    let mut failed = false;
    if !overrides.deny_graphics_capture {
        let n = run_backend(&source, &device, &context, Backend::GraphicsCapture)?;
        println!("backend=WGC frames={n}");
        failed |= n == 0;
    }
    if !overrides.deny_dxgi {
        let n = run_backend(&source, &device, &context, Backend::Dxgi)?;
        println!("backend=DXGI frames={n}");
        failed |= n == 0;
    }
    if failed {
        bail!("backend de captura não entregou quadros");
    }
    Ok(())
}

fn run_backend(
    source: &SourceId,
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    backend: Backend,
) -> Result<u32> {
    let frames = Arc::new(AtomicU32::new(0));
    let converter = crate::video::convert::Converter::new(device, context, 1280, 720, 30)?;
    let sink = Box::new(CountSink {
        converter,
        frames: frames.clone(),
    });
    let handle = match backend {
        Backend::GraphicsCapture => {
            CaptureHandle::Wgc(wgc::start(source, device, SinkHandle::new(sink))?)
        }
        Backend::Dxgi => CaptureHandle::Dxgi(dxgi::start(source, device, SinkHandle::new(sink))?),
    };
    std::thread::sleep(Duration::from_secs(3));
    handle.stop();
    std::thread::sleep(Duration::from_millis(300));
    Ok(frames.load(Ordering::Relaxed))
}

struct CountSink {
    converter: crate::video::convert::Converter,
    frames: Arc<AtomicU32>,
}

impl FrameSink for CountSink {
    fn on_frame(&mut self, tex: &ID3D11Texture2D, _w: u32, _h: u32) {
        if self.converter.convert(tex).is_ok() {
            self.frames.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_closed(&mut self) {}

    fn on_error(&mut self, e: anyhow::Error) {
        warn!("captura: {e:#}");
    }
}
