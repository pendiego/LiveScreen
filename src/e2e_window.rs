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
r.after(1500, lambda: r.geometry('1600x900'))
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

/// Trava vira falha: sem isso um `join` preso deixaria o teste parado para sempre.
fn within(tag: &str, f: impl FnOnce()) {
    let (tx, rx) = std::sync::mpsc::channel();
    let tag2 = tag.to_string();
    std::thread::spawn(move || {
        if rx.recv_timeout(Duration::from_secs(10)).is_err() {
            eprintln!("TRAVOU: {tag2}");
            std::process::exit(101);
        }
    });
    let t = std::time::Instant::now();
    f();
    let _ = tx.send(());
    println!("{tag}: {:?}", t.elapsed());
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
    let share = ShareSession::start(
        &h,
        info,
        Preset::P720p30,
        false,
        kind,
        Default::default(),
        4,
    )
    .unwrap();
    let ticket = loop {
        if let Some(t) = share.status.lock().unwrap().ticket.clone() {
            break t;
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let ctx = egui::Context::default();
    let w = WatchSession::start(&h, &ticket, "", 1.0, true, ctx).unwrap();
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
            "{tag}: phase={:?} fps={} frame={:?} welcome={}x{} audio={:?} ended={:?} decoder_hw={} decode={:.0}fps {:.2}ms",
            s.phase, s.fps, f, s.width, s.height, hs.audio, hs.ended, s.decoder_hw, s.decode_fps, s.decode_ms
        );
    };
    snap("start");
    w.request_quality(Preset::P480p30);
    std::thread::sleep(Duration::from_secs(1));
    let (_who, asked, res) = share
        .apply_quality_request()
        .expect("pedido de qualidade não chegou ao host");
    assert_eq!(asked, Preset::P480p30);
    res.unwrap();
    std::thread::sleep(Duration::from_secs(2));
    snap("após pedido do viewer (480p)");
    // mais pesado que o atual (480p) é recusado; o intervalo mínimo entre pedidos é 5 s
    std::thread::sleep(Duration::from_secs(5));
    w.request_quality(Preset::P1080p60);
    std::thread::sleep(Duration::from_secs(1));
    let (_, heavier, res) = share.apply_quality_request().expect("pedido não chegou");
    assert_eq!(heavier, Preset::P1080p60);
    assert!(res.is_err(), "pedido mais pesado deveria ser recusado");
    assert_eq!(share.current().0, Preset::P480p30);
    let hw = HWND(hwnd as *mut _);
    unsafe {
        let _ = MoveWindow(hw, 100, 100, 900, 300, true);
    }
    std::thread::sleep(Duration::from_secs(3));
    snap("resized");
    unsafe {
        let _ = MoveWindow(hw, 100, 100, 1600, 900, true);
    }
    std::thread::sleep(Duration::from_secs(2));
    snap("big (720p)");
    // redimensionar não reconfigura sozinho: o encoder segue no tamanho antigo, e é ele que a
    // UI usa para capacidade; escolher a mesma opção de novo aplica o tamanho novo
    let (_, before, _) = share.current();
    let (sw, sh, hz) = share.source_shape();
    let wanted = crate::preset::encode_params(Preset::P720p30, sw, sh, hz);
    assert_ne!(before, wanted, "o encoder não deveria ter mudado sozinho");
    share.set_preset(Preset::P720p30).unwrap();
    assert_eq!(share.current().1, wanted);
    std::thread::sleep(Duration::from_secs(2));
    snap("mesma opção, tamanho novo");
    share.set_preset(Preset::Native60).unwrap();
    std::thread::sleep(Duration::from_secs(3));
    snap("live Native60");
    // a janela é movida para 1600x900 px lógicos; com escala de DPI o tamanho físico é maior,
    // então o esperado é a largura real da fonte (o decoder devolve múltiplo de 16). Espera o
    // quadro novo chegar em vez de confiar num sleep fixo.
    let (sw, _, _) = share.source_shape();
    let width = || w.frame.lock().unwrap().as_ref().map(|f| f.width);
    let t = std::time::Instant::now();
    while t.elapsed() < Duration::from_secs(8) && !width().is_some_and(|x| x >= sw) {
        std::thread::sleep(Duration::from_millis(100));
    }
    let fw = width().expect("sem quadro");
    assert!(
        fw >= sw && fw < sw + 16,
        "largura do quadro {fw} fora de [{sw}, {})",
        sw + 16
    );
    assert!(sw >= 1600, "janela menor que 1600 px: {sw}");
    share.set_preset(Preset::P720p30).unwrap();
    std::thread::sleep(Duration::from_secs(3));
    snap("live 720p");
    unsafe {
        let _ = ShowWindow(hw, SW_MINIMIZE);
    }
    std::thread::sleep(Duration::from_secs(3));
    snap("minimized");
    // troca → troca sem nenhum quadro capturado no meio (a janela minimizada não entrega quadros)
    within("troca 1 minimizada", || {
        share.set_preset(Preset::Native60).unwrap()
    });
    within("troca 2 minimizada", || {
        share.set_preset(Preset::P720p30).unwrap()
    });
    let late = WatchSession::start(&h, &ticket, "", 1.0, true, egui::Context::default()).unwrap();
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
    // troca → parar sem nenhum quadro capturado no meio
    unsafe {
        let _ = ShowWindow(hw, SW_MINIMIZE);
    }
    std::thread::sleep(Duration::from_secs(2));
    within("troca antes de parar", || {
        share.set_preset(Preset::P1080p60).unwrap()
    });
    unsafe {
        let _ = PostMessageW(Some(hw), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
    std::thread::sleep(Duration::from_secs(3));
    println!("window still exists: {:?}", find());
    snap("closed");
    let ended = share.status.lock().unwrap().ended.clone();
    assert_eq!(ended.as_deref(), Some("A janela foi fechada"));
    within("stop com troca pendente", || share.stop()); // a UI faz isso ao ver `ended`
    std::thread::sleep(Duration::from_secs(2));
    let ph = w.status.lock().unwrap().phase.clone();
    println!("viewer phase after close: {ph:?}");
    assert_eq!(ph, Phase::Ended("Transmissão encerrada".into()));
}
