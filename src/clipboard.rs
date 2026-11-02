use std::time::Duration;
use windows::core::w;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, IsWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
};

struct ClipboardGuard {
    opened: bool,
    temp_hwnd: Option<HWND>,
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        if self.opened {
            unsafe {
                let _ = CloseClipboard();
            }
        }
        if let Some(hwnd) = self.temp_hwnd {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }
    }
}

fn create_message_window() -> anyhow::Result<(HWND, Option<HWND>)> {
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("LiveScreenClip"),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )?
    };
    Ok((hwnd, Some(hwnd)))
}

/// Define o conteúdo da área de transferência com sinalizadores para histórico e nuvem.
pub(crate) fn set_sensitive_clipboard(text: &str) -> anyhow::Result<()> {
    let (owner, temp_hwnd) = if let Some(h) = crate::ui::own_hwnd().map(|h| HWND(h as *mut _)) {
        if unsafe { IsWindow(Some(h)).as_bool() } {
            (h, None)
        } else {
            create_message_window()?
        }
    } else {
        create_message_window()?
    };

    let mut guard = ClipboardGuard {
        opened: false,
        temp_hwnd,
    };

    let mut opened = false;
    for _ in 0..5 {
        if unsafe { OpenClipboard(Some(owner)).is_ok() } {
            opened = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    if !opened {
        anyhow::bail!("OpenClipboard falhou após tentativas");
    }
    guard.opened = true;

    let res = unsafe { publish(text) };
    if res.is_err() {
        // falha fechada: nunca deixa o convite na área de transferência sem as proteções
        let _ = unsafe { EmptyClipboard() };
    }
    res
}

/// Publica uma alocação global no formato dado; em falha libera a alocação.
unsafe fn set_data(fmt: u32, h: HGLOBAL) -> anyhow::Result<()> {
    unsafe {
        if SetClipboardData(fmt, Some(HANDLE(h.0))).is_err() {
            let _ = GlobalFree(Some(h));
            anyhow::bail!("SetClipboardData falhou para o formato {fmt}");
        }
    }
    Ok(())
}

/// Com a área de transferência aberta: texto + formatos que excluem histórico e nuvem.
/// Qualquer formato de privacidade que falhe é erro.
unsafe fn publish(text: &str) -> anyhow::Result<()> {
    unsafe {
        EmptyClipboard()?;

        let u16_chars: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = u16_chars.len() * std::mem::size_of::<u16>();
        let h_text = GlobalAlloc(GMEM_MOVEABLE, bytes)?;
        let ptr = GlobalLock(h_text);
        if ptr.is_null() {
            let _ = GlobalFree(Some(h_text));
            anyhow::bail!("GlobalLock falhou para texto Unicode");
        }
        std::ptr::copy_nonoverlapping(u16_chars.as_ptr() as *const u8, ptr as *mut u8, bytes);
        let _ = GlobalUnlock(h_text);
        set_data(13, h_text)?; // CF_UNICODETEXT

        // (nome, tamanho): o primeiro é sinalizador sem dados, os outros dois são DWORD 0
        for (fmt, size) in [
            (w!("ExcludeClipboardContentFromMonitorProcessing"), 1),
            (w!("CanIncludeInClipboardHistory"), 4),
            (w!("CanUploadToCloudClipboard"), 4),
        ] {
            let id = RegisterClipboardFormatW(fmt);
            if id == 0 {
                anyhow::bail!("RegisterClipboardFormatW falhou");
            }
            set_data(id, GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, size)?)?;
        }
    }
    Ok(())
}

/// Copia texto sensível para a área de transferência protegida; sem fallback sem proteção.
pub fn copy_sensitive(text: &str) -> anyhow::Result<()> {
    set_sensitive_clipboard(text).inspect_err(|e| {
        tracing::warn!("Falha ao copiar para área de transferência protegida: {e}");
    })
}
