use std::mem::size_of;

use anyhow::{Result, bail};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC, GetMonitorInfoW, HALFTONE, HBITMAP, HDC,
    HGDIOBJ, HMONITOR, MONITORINFO, MONITORINFOEXW, ReleaseDC, SRCCOPY, SelectObject,
    SetStretchBltMode, StretchBlt,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GW_OWNER, GWL_EXSTYLE, GetWindow, GetWindowLongW, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, MONITORINFOF_PRIMARY, WS_EX_TOOLWINDOW,
};
use windows::core::{BOOL, PWSTR};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceId {
    Monitor {
        hmonitor: isize,
        device_name: String,
    },
    Window {
        hwnd: isize,
        pid: u32,
    },
}

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub id: SourceId,
    pub title: String,
    pub subtitle: String,
    pub minimized: bool,
}

fn monitor_info(hmonitor: HMONITOR) -> Option<MONITORINFOEXW> {
    let mut mi = MONITORINFOEXW::default();
    mi.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    let ok = unsafe { GetMonitorInfoW(hmonitor, &mut mi.monitorInfo as *mut MONITORINFO) };
    ok.as_bool().then_some(mi)
}

fn device_name(mi: &MONITORINFOEXW) -> String {
    let len = mi
        .szDevice
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(mi.szDevice.len());
    String::from_utf16_lossy(&mi.szDevice[..len])
}

fn monitors() -> Vec<(isize, u32, u32, String)> {
    let mut list: Vec<(isize, u32, u32, String)> = Vec::new();
    unsafe {
        // `addr_of_mut!` evita criar um `&mut` que o callback também usa.
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_cb),
            LPARAM(std::ptr::addr_of_mut!(list) as isize),
        );
    }
    list
}

pub fn enumerate() -> Vec<SourceInfo> {
    let mut out: Vec<SourceInfo> = monitors()
        .into_iter()
        .enumerate()
        .map(|(i, (hmonitor, w, h, device_name))| SourceInfo {
            id: SourceId::Monitor {
                hmonitor,
                device_name,
            },
            title: format!("Tela {}", i + 1),
            subtitle: {
                let primary = monitor_info(HMONITOR(hmonitor as *mut _))
                    .is_some_and(|mi| mi.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0);
                if primary {
                    format!("{w}×{h} · Principal")
                } else {
                    format!("{w}×{h}")
                }
            },
            minimized: false,
        })
        .collect();

    let mut windows: Vec<SourceInfo> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(window_cb),
            LPARAM(std::ptr::addr_of_mut!(windows) as isize),
        );
    }
    out.extend(windows);
    out
}

pub fn primary_monitor() -> Option<SourceId> {
    let mut fallback = None;
    for (hmonitor, ..) in monitors() {
        let Some(mi) = monitor_info(HMONITOR(hmonitor as *mut _)) else {
            continue;
        };
        let id = SourceId::Monitor {
            hmonitor,
            device_name: device_name(&mi),
        };
        if mi.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0 {
            return Some(id);
        }
        let rc = mi.monitorInfo.rcMonitor;
        if fallback.is_none() && rc.left == 0 && rc.top == 0 {
            fallback = Some(id);
        }
    }
    fallback
}

unsafe extern "system" fn monitor_cb(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let list = unsafe { &mut *(data.0 as *mut Vec<(isize, u32, u32, String)>) };
    if let Some(mi) = monitor_info(hmonitor) {
        let rc = mi.monitorInfo.rcMonitor;
        list.push((
            hmonitor.0 as isize,
            (rc.right - rc.left).max(0) as u32,
            (rc.bottom - rc.top).max(0) as u32,
            device_name(&mi),
        ));
    }
    BOOL(1)
}

unsafe extern "system" fn window_cb(hwnd: HWND, data: LPARAM) -> BOOL {
    let list = unsafe { &mut *(data.0 as *mut Vec<SourceInfo>) };
    if let Some(info) = inspect_window(hwnd) {
        list.push(info);
    }
    BOOL(1)
}

/// Critérios iguais aos de robmikh/screenshot-rs.
fn inspect_window(hwnd: HWND) -> Option<SourceInfo> {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return None;
        }
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(hwnd, &mut buf);
        if n <= 0 {
            return None;
        }
        let title = String::from_utf16_lossy(&buf[..n as usize]);
        if GetWindow(hwnd, GW_OWNER).is_ok() {
            return None;
        }
        if GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0 {
            return None;
        }
        let mut cloaked = 0u32;
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            size_of::<u32>() as u32,
        )
        .is_ok()
            && cloaked != 0
        {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            return None;
        }
        Some(SourceInfo {
            id: SourceId::Window {
                hwnd: hwnd.0 as isize,
                pid,
            },
            title,
            subtitle: exe_name(pid),
            minimized: IsIconic(hwnd).as_bool(),
        })
    }
}

fn exe_name(pid: u32) -> String {
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return format!("pid {pid}");
        };
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let res = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        if res.is_err() {
            return format!("pid {pid}");
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string()
    }
}

pub fn source_size(id: &SourceId) -> Result<(u32, u32)> {
    match id {
        SourceId::Monitor { hmonitor, .. } => {
            let Some(mi) = monitor_info(HMONITOR(*hmonitor as *mut _)) else {
                bail!("monitor não encontrado");
            };
            let rc = mi.monitorInfo.rcMonitor;
            Ok((
                (rc.right - rc.left).max(1) as u32,
                (rc.bottom - rc.top).max(1) as u32,
            ))
        }
        SourceId::Window { hwnd, .. } => {
            let rect = window_rect(HWND(*hwnd as *mut _))?;
            // Janela quebrada pode ter coordenadas extremas; i64 evita overflow.
            Ok((
                (rect.right as i64 - rect.left as i64).clamp(1, u32::MAX as i64) as u32,
                (rect.bottom as i64 - rect.top as i64).clamp(1, u32::MAX as i64) as u32,
            ))
        }
    }
}

fn window_rect(hwnd: HWND) -> Result<RECT> {
    let mut rect = RECT::default();
    unsafe {
        // DWM pode não conhecer a janela (ou o processo ser elevado); o retângulo
        // clássico é o fallback.
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut _,
            size_of::<RECT>() as u32,
        )
        .is_err()
        {
            GetWindowRect(hwnd, &mut rect)?;
        }
    }
    Ok(rect)
}

pub fn thumbnail(id: &SourceId, max_w: u32) -> Option<(u32, u32, Vec<u8>)> {
    match id {
        SourceId::Monitor { hmonitor, .. } => {
            thumbnail_monitor(HMONITOR(*hmonitor as *mut _), max_w)
        }
        SourceId::Window { hwnd, .. } => thumbnail_window(HWND(*hwnd as *mut _), max_w),
    }
}

fn thumbnail_monitor(hmonitor: HMONITOR, max_w: u32) -> Option<(u32, u32, Vec<u8>)> {
    let mi = monitor_info(hmonitor)?;
    let rc = mi.monitorInfo.rcMonitor;
    let (sw, sh) = (rc.right - rc.left, rc.bottom - rc.top);
    if sw <= 0 || sh <= 0 {
        return None;
    }
    let (ow, oh) = fit(sw as u32, sh as u32, max_w);
    let dib = Dib::new(ow, oh)?;
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return None;
        }
        SetStretchBltMode(dib.dc, HALFTONE);
        let ok = StretchBlt(
            dib.dc,
            0,
            0,
            ow as i32,
            oh as i32,
            Some(screen),
            rc.left,
            rc.top,
            sw,
            sh,
            SRCCOPY,
        );
        ReleaseDC(None, screen);
        if !ok.as_bool() {
            return None;
        }
    }
    Some((ow, oh, dib.rgba()))
}

/// `PW_RENDERFULLCONTENT`: janelas compostas pelo DWM só aparecem com esta flag.
const PW_RENDERFULLCONTENT: PRINT_WINDOW_FLAGS = PRINT_WINDOW_FLAGS(2);

fn thumbnail_window(hwnd: HWND, max_w: u32) -> Option<(u32, u32, Vec<u8>)> {
    unsafe {
        // Minimizada ou escondida não tem o que imprimir (e oculta o último quadro).
        if IsIconic(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() {
            return None;
        }
    }
    let rc = window_rect(hwnd).ok()?;
    let (sw, sh) = (
        rc.right as i64 - rc.left as i64,
        rc.bottom as i64 - rc.top as i64,
    );
    if sw <= 0 || sh <= 0 {
        return None;
    }
    let (sw, sh) = (sw as u32, sh as u32);
    let full = Dib::new(sw, sh)?;
    let printed = unsafe { PrintWindow(hwnd, full.dc, PW_RENDERFULLCONTENT) };
    if !printed.as_bool() {
        return None;
    }
    let (ow, oh) = fit(sw, sh, max_w);
    if (ow, oh) == (sw, sh) {
        return Some((ow, oh, full.rgba()));
    }
    let scaled = Dib::new(ow, oh)?;
    unsafe {
        SetStretchBltMode(scaled.dc, HALFTONE);
        let ok = StretchBlt(
            scaled.dc,
            0,
            0,
            ow as i32,
            oh as i32,
            Some(full.dc),
            0,
            0,
            sw as i32,
            sh as i32,
            SRCCOPY,
        );
        if !ok.as_bool() {
            return None;
        }
    }
    Some((ow, oh, scaled.rgba()))
}

fn fit(w: u32, h: u32, max_w: u32) -> (u32, u32) {
    if w <= max_w {
        return (w, h);
    }
    (max_w, ((h as u64 * max_w as u64) / w as u64).max(1) as u32)
}

/// DIB section de 32 bpp top-down (BGRA) com DC próprio.
struct Dib {
    dc: HDC,
    bmp: HBITMAP,
    old: HGDIOBJ,
    bits: *mut u8,
    w: u32,
    h: u32,
}

impl Dib {
    fn new(w: u32, h: u32) -> Option<Self> {
        // Retângulo absurdo (janela quebrada) não pode virar alocação de centenas de MB.
        const MAX_PIXELS: u64 = 40_000_000;
        if w == 0 || h == 0 || w as u64 * h as u64 > MAX_PIXELS {
            return None;
        }
        let mut bi = BITMAPINFO::default();
        bi.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w as i32,
            biHeight: -(h as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        unsafe {
            let dc = CreateCompatibleDC(None);
            if dc.is_invalid() {
                return None;
            }
            let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
            let bmp = match CreateDIBSection(Some(dc), &bi, DIB_RGB_COLORS, &mut bits, None, 0) {
                Ok(bmp) if !bits.is_null() => bmp,
                _ => {
                    let _ = DeleteDC(dc);
                    return None;
                }
            };
            let old = SelectObject(dc, HGDIOBJ(bmp.0));
            Some(Dib {
                dc,
                bmp,
                old,
                bits: bits as *mut u8,
                w,
                h,
            })
        }
    }

    fn rgba(&self) -> Vec<u8> {
        let n = (self.w as usize) * (self.h as usize) * 4;
        let src = unsafe { std::slice::from_raw_parts(self.bits, n) };
        let mut out = vec![0u8; n];
        for (dst, px) in out.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            dst[0] = px[2];
            dst[1] = px[1];
            dst[2] = px[0];
            dst[3] = 255;
        }
        out
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(HGDIOBJ(self.bmp.0));
            let _ = DeleteDC(self.dc);
        }
    }
}
