//! Teste ignorado de ida e volta encode→decode com PSNR.
//!
//! `cargo test --release -- --ignored codec_roundtrip --nocapture`
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use tokio::sync::broadcast;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use crate::preset::{EncodeParams, Preset};
use crate::protocol::EncodedFrame;
use crate::video::convert::Converter;
use crate::video::decoder::{self, RgbaFrame, VideoFrame};
use crate::video::encoder::{self, EncoderKind, EncoderStats, Nv12Frame};
use crate::video::mf;

const W: u32 = 1280;
const H: u32 = 720;
const N: usize = 60;
const TAU: f32 = std::f32::consts::TAU;

fn pattern(x: u32, i: usize) -> f32 {
    let phase = i as f32 / N as f32 * TAU;
    155.0 + 95.0 * (TAU * 3.0 * x as f32 / W as f32 + phase).sin()
}

fn patterns() -> Vec<Vec<f32>> {
    (0..N)
        .map(|i| (0..W).map(|x| pattern(x, i)).collect())
        .collect()
}

fn bgra(i: usize, pat: &[Vec<f32>]) -> Vec<u8> {
    let mut row = Vec::with_capacity((W * 4) as usize);
    for x in 0..W {
        let v = pat[i][x as usize].round().clamp(0.0, 255.0) as u8;
        row.extend_from_slice(&[v, v, v, 255]);
    }
    let mut data = Vec::with_capacity((W * H * 4) as usize);
    for _ in 0..H {
        data.extend_from_slice(&row);
    }
    data
}

fn create_bgra(device: &ID3D11Device) -> Result<ID3D11Texture2D> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: W,
        Height: H,
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
        MiscFlags: 0,
    };
    let mut t = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut t))? };
    t.ok_or_else(|| anyhow!("textura BGRA nula"))
}

#[derive(Default)]
struct Rec {
    frames: usize,
    decoded: usize,
    psnrs: Vec<f64>,
    bad_dims: bool,
    last_fp: u64,
}

fn fingerprint(f: &RgbaFrame) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for c in f.pixels.iter().step_by(997) {
        h = (h ^ c.r() as u64).wrapping_mul(0x100_0000_01b3);
    }
    h ^ f.pixels.len() as u64
}

fn luma(f: &RgbaFrame, x: u32, y: u32) -> f32 {
    let c = f.pixels[(y * W + x) as usize];
    0.2126 * c.r() as f32 + 0.7152 * c.g() as f32 + 0.0722 * c.b() as f32
}

fn match_index(f: &RgbaFrame, pat: &[Vec<f32>]) -> usize {
    let mut best = (f64::MAX, 0usize);
    for (i, row) in pat.iter().enumerate() {
        let mut err = 0.0f64;
        for k in 0..80 {
            let x = k * 16;
            let d = (luma(f, x, 20) - row[x as usize]) as f64;
            err += d * d;
        }
        if err < best.0 {
            best = (err, i);
        }
    }
    best.1
}

fn psnr(f: &RgbaFrame, i: usize, pat: &[Vec<f32>]) -> f64 {
    let mut err = 0.0f64;
    let mut n = 0.0f64;
    for y in (0..H).step_by(4) {
        for x in (0..W).step_by(4) {
            let d = (luma(f, x, y) - pat[i][x as usize]) as f64;
            err += d * d;
            n += 1.0;
        }
    }
    let mse = err / n;
    if mse <= 0.0 {
        return 99.0;
    }
    10.0 * (255.0 * 255.0 / mse).log10()
}

fn forward(
    brx: &mut broadcast::Receiver<Arc<EncodedFrame>>,
    dtx: &std::sync::mpsc::Sender<EncodedFrame>,
    rec: &mut Rec,
) {
    loop {
        match brx.try_recv() {
            Ok(f) => {
                rec.frames += 1;
                let _ = dtx.send((*f).clone());
            }
            Err(_) => return,
        }
    }
}

fn collect(slot: &Arc<Mutex<Option<VideoFrame>>>, rec: &mut Rec, pat: &[Vec<f32>]) {
    let guard = slot.lock().unwrap();
    let Some(VideoFrame::Cpu(frame)) = guard.as_ref() else {
        return;
    };
    let fp = fingerprint(frame);
    if fp == rec.last_fp {
        return;
    }
    rec.last_fp = fp;
    if frame.width != W || frame.height != H {
        rec.bad_dims = true;
        return;
    }
    let i = match_index(frame, pat);
    rec.psnrs.push(psnr(frame, i, pat));
    rec.decoded += 1;
}

fn roundtrip(
    device: &ID3D11Device,
    ctx: &ID3D11DeviceContext,
    params: EncodeParams,
    kind: EncoderKind,
    hardware_decode: bool,
    pat: &[Vec<f32>],
) -> Result<(usize, f64, f64)> {
    let mut conv = Converter::new(device, ctx, params.width, params.height, params.fps)?;
    let src = create_bgra(device)?;
    let (tx, rx) = std::sync::mpsc::sync_channel::<Nv12Frame>(2);
    let (btx, mut brx) = broadcast::channel(16);
    let enc = encoder::spawn(
        kind,
        device,
        params,
        rx,
        btx,
        Arc::new(AtomicBool::new(false)),
        Arc::new(EncoderStats::default()),
    )?;
    let slot: Arc<Mutex<Option<VideoFrame>>> = Arc::default();
    let (dtx, drx) = std::sync::mpsc::channel::<EncodedFrame>();
    let dstats = Arc::new(decoder::DecodeStats::default());
    let dec = decoder::spawn(
        drx,
        slot.clone(),
        eframe::egui::Context::default(),
        Arc::new(AtomicBool::new(false)),
        dstats.clone(),
        hardware_decode,
        None,
        Arc::new(AtomicBool::new(false)),
    );

    let mut rec = Rec::default();
    let start = Instant::now();
    for i in 0..N {
        let data = bgra(i, pat);
        unsafe {
            ctx.UpdateSubresource(&src, 0, None, data.as_ptr() as *const _, W * 4, W * H * 4);
        }
        let nv = conv.convert(&src)?;
        let _ = tx.send(Nv12Frame {
            tex: nv,
            pts: start.elapsed(),
        });
        let until = Instant::now() + Duration::from_millis(33);
        while Instant::now() < until {
            forward(&mut brx, &dtx, &mut rec);
            collect(&slot, &mut rec, pat);
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    drop(tx);
    enc.stop();
    let until = Instant::now() + Duration::from_millis(2000);
    while Instant::now() < until {
        forward(&mut brx, &dtx, &mut rec);
        collect(&slot, &mut rec, pat);
        if rec.decoded >= N {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    drop(dtx);
    while !dec.is_finished() {
        collect(&slot, &mut rec, pat);
        std::thread::sleep(Duration::from_millis(2));
    }
    collect(&slot, &mut rec, pat);
    let _ = dec.join();
    if hardware_decode && !dstats.hardware.load(Ordering::Relaxed) {
        return Err(anyhow!("decoder de hardware não ficou ativo"));
    }

    if rec.bad_dims {
        return Err(anyhow!("quadro decodificado não é {W}x{H}"));
    }
    let mean = rec.psnrs.iter().sum::<f64>() / rec.psnrs.len().max(1) as f64;
    let min = rec.psnrs.iter().cloned().fold(f64::MAX, f64::min);
    println!(
        "  stream={} quadros, decodificados={}",
        rec.frames, rec.decoded
    );
    if rec.frames < 55 {
        return Err(anyhow!("stream entregou {} quadros (< 55)", rec.frames));
    }
    if rec.decoded < 55 {
        return Err(anyhow!("decodificados {} quadros (< 55)", rec.decoded));
    }
    if mean <= 30.0 {
        return Err(anyhow!("PSNR médio {mean:.2} dB (<= 30)"));
    }
    Ok((rec.decoded, mean, min))
}

#[test]
#[ignore = "usa GPU/encoder real: cargo test --release -- --ignored codec_roundtrip"]
fn codec_roundtrip() {
    mf::startup();
    let (device, ctx) = crate::gpu::create_device(None).expect("dispositivo D3D11");
    let params = crate::preset::encode_params(Preset::P720p30, W, H, 60);
    assert_eq!((params.width, params.height, params.fps), (W, H, 30));
    let pat = patterns();

    let kinds = crate::video::probe::hardware_kinds();
    let mut failures = Vec::new();
    for (kind, hardware_decode) in kinds
        .into_iter()
        .flat_map(|k| {
            if k.codec() == crate::protocol::Codec::H264 {
                vec![(k.clone(), true), (k, false)]
            } else {
                vec![(k, true)]
            }
        })
    {
        let name = kind.label();
        let name = format!(
            "{name} / decoder {}",
            if hardware_decode {
                "hardware"
            } else {
                "software"
            }
        );
        match roundtrip(&device, &ctx, params, kind, hardware_decode, &pat) {
            Ok((decoded, mean, min)) => {
                println!(
                    "{name}: decodificados={decoded} PSNR médio={mean:.2} dB (min {min:.2} dB)"
                );
            }
            Err(e) => {
                println!("{name}: FALHOU: {e:#}");
                failures.push(format!("{name}: {e:#}"));
            }
        }
    }
    assert!(failures.is_empty(), "falhas:\n{}", failures.join("\n"));
}

