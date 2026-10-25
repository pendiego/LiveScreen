//! Regression smoke (manual): compartilha uma janela tkinter que toca áudio em loop,
//! redimensiona, minimiza, restaura e fecha. Rodar:
//! `cargo test --release e2e_window -- --ignored --nocapture`
use std::process::{Command, Stdio};
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::capture::{SourceId, enumerate};
use livescreen_core::preset::Preset;
use crate::share::ShareSession;
use crate::video::encoder::EncoderKind;
use livescreen_core::protocol::Codec;
use livescreen_core::viewer::Phase;
use crate::watch::WatchSession;

const PY: &str = r#"
import tkinter as tk, winsound, wave, struct, math, tempfile, os

tf = os.path.join(tempfile.gettempdir(), 'lstest_tone.wav')
if not os.path.exists(tf):
    with wave.open(tf, 'w') as wf:
        wf.setnchannels(2); wf.setsampwidth(2); wf.setframerate(48000)
        samples = []
        for i in range(48000):
            val = int(32767 * 0.3 * math.sin(2 * math.pi * 440 * i / 48000))
            samples.append(struct.pack('<hh', val, val))
        wf.writeframes(b''.join(samples))
try:
    winsound.PlaySound(tf, winsound.SND_FILENAME | winsound.SND_ASYNC | winsound.SND_LOOP)
except Exception as e:
    print('winsound error:', e)

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
    let kinds = crate::video::probe::hardware_kinds();
    let share = ShareSession::start(
        &h,
        info,
        Preset::P720p30,
        false,
        None,
        kinds.clone(),
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
    let w = WatchSession::start(&h, &ticket, "", 1.0, true, None, ctx).unwrap();
    std::thread::sleep(Duration::from_secs(4));
    let snap = |tag: &str| {
        let s = w.status.lock().unwrap().clone();
        let f = w
            .frame
            .lock()
            .unwrap()
            .as_ref()
            .map(|f| f.size());
        let hs = share.status.lock().unwrap().clone();
        println!(
            "{tag}: phase={:?} fps={} frame={:?} welcome={}x{} audio={:?} ended={:?} decoder_hw={} decode={:.0}fps {:.2}ms",
            s.phase, s.fps, f, s.width, s.height, hs.audio, hs.ended, s.decoder_hw, s.decode_fps, s.decode_ms
        );
    };
    let _ = share.apply_codec();
    std::thread::sleep(Duration::from_secs(2));
    let expect_hevc = kinds.iter().any(|k| k.codec() == Codec::Hevc)
        && crate::video::decoder::supported_codecs(true).contains(&Codec::Hevc);
    if expect_hevc {
        assert_eq!(w.status.lock().unwrap().codec, "HEVC");
    }
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
    let wanted = livescreen_core::preset::encode_params(Preset::P720p30, sw, sh, hz);
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
    let (sw, sh, hz) = share.source_shape();
    // a janela externa é 1600 px; a captura é só a área cliente (sem bordas) e o teto
    // temporário 1080p60 vale para a saída, então o esperado vem de encode_params
    let expected = livescreen_core::preset::encode_params(Preset::Native60, sw, sh, hz).width;
    let width = || w.frame.lock().unwrap().as_ref().map(|f| f.size().0);
    let t = std::time::Instant::now();
    while t.elapsed() < Duration::from_secs(8) && !width().is_some_and(|x| x >= expected) {
        std::thread::sleep(Duration::from_millis(100));
    }
    let fw = width().expect("sem quadro");
    assert!(
        fw >= expected && fw < expected + 16,
        "largura do quadro {fw} fora de [{expected}, {})",
        expected + 16
    );
    assert!(sw >= 1500, "janela menor que o esperado: {sw}");
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
    let late = WatchSession::start(&h, &ticket, "", 1.0, true, None, egui::Context::default()).unwrap();
    std::thread::sleep(Duration::from_secs(4));
    println!(
        "late viewer while minimized: {:?} frame={:?}",
        late.status.lock().unwrap().phase,
        late.frame
            .lock()
            .unwrap()
            .as_ref()
            .map(|f| f.size())
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

#[test]
#[ignore]
fn e2e_android() {
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
    let kinds = vec![EncoderKind::Hardware {
        name: "NVIDIA H.264 Encoder MFT".into(),
        codec: Codec::H264,
    }];
    let share = ShareSession::start(
        &h,
        info,
        Preset::P720p30,
        true,
        None,
        kinds,
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
    println!("HOST TICKET: {ticket}");

    let adb = r"C:\Users\diego\AppData\Local\Android\Sdk\platform-tools\adb.exe";
    let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "logcat", "-c"]).status();
    let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "am", "force-stop", "io.github.pendiego.livescreen"]).status();
    let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "am", "start", "-n", "io.github.pendiego.livescreen/.MainActivity", "--es", "ticket", &ticket]).status();

    std::thread::sleep(Duration::from_secs(2));
    let home = Command::new(adb).args(["-s", "127.0.0.1:16384", "exec-out", "screencap", "-p"]).output().unwrap();
    std::fs::write(std::env::temp_dir().join("mumu_home_valid.png"), &home.stdout).unwrap();
    // Tap "Assistir" button
    if !tap_text(adb, "Assistir") {
        let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "input", "tap", "575", "880"]).status();
    }

    // Wait for initial streaming and decoding at 720p (12s)
    std::thread::sleep(Duration::from_secs(12));

    let temp = std::env::temp_dir();

    // Evidence A: Audio inspection
    println!("=== EVIDENCE A: AUDIO CHECK ===");
    let af_out = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "dumpsys", "media.audio_flinger"]).output().unwrap();
    let af_text = String::from_utf8_lossy(&af_out.stdout);
    println!("dumpsys media.audio_flinger:\n{af_text}");

    let aud_out = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "dumpsys", "audio"]).output().unwrap();
    let aud_text = String::from_utf8_lossy(&aud_out.stdout);
    for line in aud_text.lines() {
        if line.contains("AudioPlaybackConfiguration") || line.contains("LiveScreen") || line.contains("pendiego") {
            println!("dumpsys audio config: {line}");
        }
    }

    // Capture 720p screenshot
    let out720 = Command::new(adb).args(["-s", "127.0.0.1:16384", "exec-out", "screencap", "-p"]).output().unwrap();
    let p720 = temp.join("mumu_stream_720p.png");
    std::fs::write(&p720, &out720.stdout).unwrap();
    println!("Captured 720p screenshot: {} bytes -> {}", out720.stdout.len(), p720.display());

    // Overlay: reabre com um toque e captura 1 s e 3 s depois (precisa sumir em ~2,5 s).
    let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "input", "tap", "360", "640"]).status();
    std::thread::sleep(Duration::from_millis(700));
    let ov1 = Command::new(adb).args(["-s", "127.0.0.1:16384", "exec-out", "screencap", "-p"]).output().unwrap();
    std::fs::write(temp.join("mumu_overlay_1s.png"), &ov1.stdout).unwrap();
    std::thread::sleep(Duration::from_millis(2200));
    let ov3 = Command::new(adb).args(["-s", "127.0.0.1:16384", "exec-out", "screencap", "-p"]).output().unwrap();
    std::fs::write(temp.join("mumu_overlay_3s.png"), &ov3.stdout).unwrap();

    // Evidence B: Trigger Quality -> 480p 30 fps via UI
    println!("=== EVIDENCE B: QUALITY CHANGE ===");
    // O overlay some em 2,5 s e o `uiautomator dump` leva ~2 s: acha o botão com o overlay
    // visível, espera ele esconder e então reabre e toca na posição já conhecida.
    let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "input", "tap", "360", "640"]).status();
    std::thread::sleep(Duration::from_millis(300));
    let q_xy = find_text(adb, "Qualidade");
    assert!(q_xy.is_some(), "Deveria encontrar 'Qualidade'");
    std::thread::sleep(Duration::from_millis(3000));
    let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "input", "tap", "360", "640"]).status();
    std::thread::sleep(Duration::from_millis(300));
    tap_xy(adb, "Qualidade", q_xy.unwrap());
    std::thread::sleep(Duration::from_millis(600));

    let tapped_480 = tap_text(adb, "480p 30 fps");
    assert!(tapped_480, "Deveria encontrar e tocar em '480p 30 fps'");

    // Host applies the quality request
    let mut applied_quality = None;
    for _ in 0..20 {
        if let Some((who, preset, res)) = share.apply_quality_request() {
            println!("HOST APPLIED QUALITY: who='{who}', preset={preset:?}, res={res:?}");
            assert_eq!(preset, Preset::P480p30);
            res.unwrap();
            applied_quality = Some((who, preset));
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(applied_quality.is_some(), "Pedido de qualidade não chegou ao host");

    // Wait 12s for 480p frames and decoder stats
    std::thread::sleep(Duration::from_secs(12));

    // Capture 480p screenshot
    let out480 = Command::new(adb).args(["-s", "127.0.0.1:16384", "exec-out", "screencap", "-p"]).output().unwrap();
    let p480 = temp.join("mumu_stream_480p.png");
    std::fs::write(&p480, &out480.stdout).unwrap();
    println!("Captured 480p screenshot: {} bytes -> {}", out480.stdout.len(), p480.display());

    // Dump logcat
    let log_out = Command::new(adb).args(["-s", "127.0.0.1:16384", "logcat", "-d", "-s", "LiveScreen"]).output().unwrap();
    let logcat = String::from_utf8_lossy(&log_out.stdout);
    println!("--- LOGCAT LiveScreen --- \n{logcat}\n-------------------------");

    share.stop();
    std::thread::sleep(Duration::from_secs(2));
}

fn find_text(adb: &str, target_text: &str) -> Option<(i32, i32)> {
    for _ in 0..3 {
        let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "uiautomator", "dump", "/sdcard/dump.xml"]).output();
        if let Ok(out) = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "cat", "/sdcard/dump.xml"]).output() {
            let xml = String::from_utf8_lossy(&out.stdout);
            if let Some(pos) = xml.find(&format!("text=\"{target_text}\"")) {
                let rest = &xml[pos..];
                if let Some(b_start) = rest.find("bounds=\"[") {
                    let b_str = &rest[b_start + 8..];
                    if let Some(b_end) = b_str.find('"') {
                        let coords = &b_str[..b_end];
                        if let Some((p1, p2)) = coords.split_once("][") {
                            let p1 = p1.trim_matches(|c| c == '[' || c == ']');
                            let p2 = p2.trim_matches(|c| c == '[' || c == ']');
                            if let (Some((x1, y1)), Some((x2, y2))) = (p1.split_once(','), p2.split_once(',')) {
                                if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (x1.parse::<i32>(), y1.parse::<i32>(), x2.parse::<i32>(), y2.parse::<i32>()) {
                                    return Some(((x1 + x2) / 2, (y1 + y2) / 2));
                                }
                            }
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    println!("Could not find node with text='{target_text}'");
    None
}

fn tap_xy(adb: &str, label: &str, (cx, cy): (i32, i32)) {
    let _ = Command::new(adb).args(["-s", "127.0.0.1:16384", "shell", "input", "tap", &cx.to_string(), &cy.to_string()]).status();
    println!("Tapped '{label}' at {cx}, {cy}");
}

fn tap_text(adb: &str, target_text: &str) -> bool {
    match find_text(adb, target_text) {
        Some(xy) => {
            tap_xy(adb, target_text, xy);
            true
        }
        None => false,
    }
}
