use anyhow::{Context as _, Result};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Win32::Foundation::{HMODULE, LUID};
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_11_0,
    D3D_FEATURE_LEVEL_11_1,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_SDK_VERSION,
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Multithread,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter, IDXGIDevice, IDXGIFactory4};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice;
use windows::core::Interface;

/// O mesmo dispositivo serve à captura, ao conversor e ao encoder.
pub fn create_device(adapter: Option<LUID>) -> Result<(ID3D11Device, ID3D11DeviceContext)> {
    let adapter = match adapter {
        Some(luid) => {
            let factory = unsafe { CreateDXGIFactory1::<IDXGIFactory4>() }.context("CreateDXGIFactory1")?;
            Some(unsafe { factory.EnumAdapterByLuid::<IDXGIAdapter>(luid) }.context("EnumAdapterByLuid")?)
        }
        None => None,
    };
    let mut device = None;
    let mut context = None;
    let levels: [D3D_FEATURE_LEVEL; 2] = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];
    unsafe {
        D3D11CreateDevice(
            adapter.as_ref(),
            if adapter.is_some() { D3D_DRIVER_TYPE_UNKNOWN } else { D3D_DRIVER_TYPE_HARDWARE },
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
        .context("D3D11CreateDevice")?;
    }
    let device = device.context("D3D11CreateDevice sem dispositivo")?;
    let context = context.context("D3D11CreateDevice sem contexto")?;
    let mt: ID3D11Multithread = device.cast().context("ID3D11Multithread")?;
    unsafe {
        let _ = mt.SetMultithreadProtected(true);
    }
    // prioridade máxima de GPU (faixa -7..7): encode/decode não engasgam com um jogo saturando a GPU
    if let Ok(dxgi) = device.cast::<IDXGIDevice>()
        && let Err(e) = unsafe { dxgi.SetGPUThreadPriority(7) }
    {
        tracing::debug!("SetGPUThreadPriority: {e}");
    }
    Ok((device, context))
}

/// Thread registrada no MMCSS (fatias regulares de prioridade alta) enquanto o guard viver.
pub struct Mmcss(windows::Win32::Foundation::HANDLE);
impl Mmcss {
    pub fn new(task: windows::core::PCWSTR) -> Option<Self> {
        let mut index = 0u32;
        match unsafe { windows::Win32::System::Threading::AvSetMmThreadCharacteristicsW(task, &mut index) } {
            Ok(h) => Some(Self(h)),
            Err(e) => {
                tracing::debug!("MMCSS indisponível: {e}");
                None
            }
        }
    }
}
impl Drop for Mmcss {
    fn drop(&mut self) {
        let _ = unsafe { windows::Win32::System::Threading::AvRevertMmThreadCharacteristics(self.0) };
    }
}

pub fn winrt_device(device: &ID3D11Device) -> Result<IDirect3DDevice> {
    let dxgi: IDXGIDevice = device.cast().context("IDXGIDevice")?;
    let inspected = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
        .context("CreateDirect3D11DeviceFromDXGIDevice")?;
    inspected.cast().context("IDirect3DDevice")
}

/// WinRT/COM exige apartamento inicializado na thread; tolera quem já inicializou.
pub fn com_init() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

/// WinRT (`IDirect3DDevice`) não é `Send` no crate `windows`; o objeto é ágil e o pool de captura
/// é free-threaded, então carregá-lo para o handler é seguro aqui.
pub struct Sendable<T>(pub T);
unsafe impl Send for Sendable<IDirect3DDevice> {}
