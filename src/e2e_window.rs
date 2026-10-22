//! Regression smoke (manual): compartilha uma janela tkinter que toca áudio em loop,
//! redimensiona, minimiza, restaura e fecha. Rodar:
//! `cargo test --release e2e_window -- --ignored --nocapture`
use std::process::{Command, Stdio};
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::capture::{SourceId, enumerate};
use crate::preset::Preset;
use crate::share::ShareSession;
use crate::video::encoder::EncoderKind;
use crate::watch::{Phase, WatchSession};

const PY: &str = r#"
import tkinter as tk
r = tk.Tk(); r.title('LSTest'); r.geometry('640x400')
c = tk.Canvas(r, bg='red'); c.pack(fill='both', expand=True)
def tick(n=[0]):
    c.configure(bg='#%02x80%02x' % (n[0] % 256, (n[0]*3) % 256)); n[0] += 7; r.after(50, tick)
tick(); r.mainloop()
"#;

/// Mata e colhe o processo no fim, inclusive quando um assert falha (sem zumbi nem Python órfão).
struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn find() -> Option<(isize, u32)> {
    enumerate().into_iter().find_map(|s| match s.id {
        SourceId::Window { hwnd, pid } if s.title == "LSTest" => Some((hwnd, pid)),
        _ => None,
    })
}

#[test]
#[ignore]
fn e2e_window() {
    crate::video::mf::startup();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let h = rt.handle().clone();
    let _child = ChildGuard(
        Command::new("python")
            .args(["-c", PY])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    std::thread::sleep(Duration::from_secs(3));
    let (hwnd, _pid) = find().expect("janela LSTest");
    let info = enumerate()
        .into_iter()
        .find(|s| matches!(&s.id, SourceId::Window { hwnd: x, .. } if *x == hwnd))
        .unwrap();
    let kind = EncoderKind::Hardware {
        name: "NVIDIA H.264 Encoder MFT".into(),
    };
    let share =
        ShareSession::start(&h, info, Preset::P720p30, false, kind, Default::default()).unwrap();
    let ticket = loop {
        if let Some(t) = share.status.lock().unwrap().ticket.clone() {
            break t;
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let ctx = egui::Context::default();
    let w = WatchSession::start(&h, &ticket, "", 1.0, ctx).unwrap();
    std::thread::sleep(Duration::from_secs(4));
    let snap = |tag: &str| {
        let s = w.status.lock().unwrap().clone();
        let f = w
            .frame
            .lock()
            .unwrap()
            .as_ref()
            .map(|f| (f.width, f.height));
        let hs = share.status.lock().unwrap().clone();
        println!(
            "{tag}: phase={:?} fps={} frame={:?} welcome={}x{} audio={:?} ended={:?}",
            s.phase, s.fps, f, s.width, s.height, hs.audio, hs.ended
        );
    };
    snap("start");
    let hw = HWND(hwnd as *mut _);
    unsafe {
        let _ = MoveWindow(hw, 100, 100, 900, 300, true);
    }
    std::thread::sleep(Duration::from_secs(3));
    snap("resized");
    unsafe {
        let _ = ShowWindow(hw, SW_MINIMIZE);
    }
    std::thread::sleep(Duration::from_secs(3));
    snap("minimized");
    let late = WatchSession::start(&h, &ticket, "", 1.0, egui::Context::default()).unwrap();
    std::thread::sleep(Duration::from_secs(4));
    println!(
        "late viewer while minimized: {:?} frame={:?}",
        late.status.lock().unwrap().phase,
        late.frame
            .lock()
            .unwrap()
            .as_ref()
            .map(|f| (f.width, f.height))
    );
    unsafe {
        let _ = ShowWindow(hw, SW_RESTORE);
    }
    std::thread::sleep(Duration::from_secs(2));
    snap("restored");
    unsafe {
        let _ = PostMessageW(Some(hw), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
    std::thread::sleep(Duration::from_secs(3));
    println!("window still exists: {:?}", find());
    snap("closed");
    let ended = share.status.lock().unwrap().ended.clone();
    assert_eq!(ended.as_deref(), Some("A janela foi fechada"));
    share.stop(); // a UI faz isso ao ver `ended`
    std::thread::sleep(Duration::from_secs(2));
    let ph = w.status.lock().unwrap().phase.clone();
    println!("viewer phase after close: {ph:?}");
    assert_eq!(ph, Phase::Ended("Transmissão encerrada".into()));
}
