use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow, bail};
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use windows::Win32::Graphics::Dxgi::{
    DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_FRAME_INFO, IDXGIDevice,
    IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource,
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
    // O primeiro quadro sempre vai (mesmo que o timestamp venha zerado), para a
    // transmissão começar com uma imagem e não só depois da próxima mudança.
    let mut delivered = false;
    while !stop.load(Ordering::Relaxed) {
        let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        match unsafe { duplication.AcquireNextFrame(100, &mut info, &mut resource) } {
            Ok(()) => {
                // `LastPresentTime == 0`: só o cursor se moveu; a imagem duplicada não
                // mudou (e o cursor não faz parte dela), então não há o que codificar.
                if (info.LastPresentTime != 0 || !delivered)
                    && let Some(resource) = &resource
                    && let Ok(texture) = resource.cast::<ID3D11Texture2D>()
                {
                    delivered = true;
                    let mut desc =
                        windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE2D_DESC::default();
                    unsafe { texture.GetDesc(&mut desc) };
                    sink.on_frame(&texture, desc.Width, desc.Height);
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
