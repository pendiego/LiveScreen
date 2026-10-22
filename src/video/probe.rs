//! Detecção do encoder de hardware fora do processo (`--probe-encoder`).
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use tokio::sync::broadcast;
use tracing::warn;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Media::MediaFoundation::*;

use crate::config::Overrides;
use crate::preset::EncodeParams;
use crate::video::encoder::{self, EncoderKind, EncoderStats, Nv12Frame};
use crate::video::mf;

/// Escolha do encoder: hardware só se o probe fora do processo aprovar.
pub fn detect(overrides: &Overrides) -> EncoderKind {
    if overrides.deny_hardware_encoder {
        return EncoderKind::Software;
    }
    match probe_child() {
        Ok(Some(name)) => EncoderKind::Hardware { name },
        Ok(None) => {
            warn!("nenhum encoder H.264 de hardware funcionou; usando software");
            EncoderKind::Software
        }
        Err(e) => {
            warn!("probe do encoder falhou: {e:#}; usando software");
            EncoderKind::Software
        }
    }
}

fn probe_child() -> Result<Option<String>> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let exe = std::env::current_exe()?;
    let mut child = Command::new(exe)
        .arg("--probe-encoder")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    let start = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if start.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            bail!("probe expirou");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let out = child.wait_with_output()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .next_back()
        .ok_or_else(|| anyhow!("probe sem saída"))?;
    let v: serde_json::Value = serde_json::from_str(line)?;
    if v["ok"].as_bool() == Some(true) {
        Ok(Some(v["name"].as_str().unwrap_or_default().to_string()))
    } else {
        Ok(None)
    }
}

fn black_nv12(device: &ID3D11Device, w: u32, h: u32) -> Result<ID3D11Texture2D> {
    let mut data = vec![16u8; (w * h) as usize];
    data.resize((w * h * 3 / 2) as usize, 128);
    let desc = D3D11_TEXTURE2D_DESC {
        Width: w,
        Height: h,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_NV12,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_VIDEO_ENCODER.0) as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };
    let init = D3D11_SUBRESOURCE_DATA {
        pSysMem: data.as_ptr() as *const _,
        SysMemPitch: w,
        SysMemSlicePitch: w * h * 3 / 2,
    };
    let mut tex = None;
    unsafe { device.CreateTexture2D(&desc, Some(&init), Some(&mut tex))? };
    tex.ok_or_else(|| anyhow!("textura NV12 nula"))
}

fn try_encoder(device: &ID3D11Device, params: EncodeParams, name: &str) -> Result<()> {
    let tex = black_nv12(device, params.width, params.height)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(2);
    let (btx, mut brx) = broadcast::channel(8);
    let handle = encoder::spawn(
        EncoderKind::Hardware {
            name: name.to_string(),
        },
        device,
        params,
        rx,
        btx,
        Arc::new(AtomicBool::new(false)),
        Arc::new(EncoderStats::default()),
    )?;
    let start = Instant::now();
    let mut ok = false;
    for _ in 0..10 {
        if tx
            .send(Nv12Frame {
                tex: tex.clone(),
                pts: start.elapsed(),
            })
            .is_err()
        {
            break;
        }
        let deadline = Instant::now() + Duration::from_millis(300);
        while Instant::now() < deadline {
            if brx.try_recv().is_ok() {
                ok = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if ok || start.elapsed() > Duration::from_secs(3) {
            break;
        }
    }
    drop(tx);
    handle.stop();
    if ok {
        Ok(())
    } else {
        bail!("nenhuma saída em 3 s")
    }
}

/// `--probe-encoder`: testa cada MFT de hardware e imprime uma linha JSON.
pub fn run_child() -> Result<()> {
    mf::startup();
    let (device, _ctx) = crate::gpu::create_device()?;
    let params = EncodeParams {
        width: 1280,
        height: 720,
        fps: 30,
        bitrate_bps: 1_000_000,
    };
    let info = MFT_REGISTER_TYPE_INFO {
        guidMajorType: MFMediaType_Video,
        guidSubtype: MFVideoFormat_H264,
    };
    let flags = MFT_ENUM_FLAG(
        MFT_ENUM_FLAG_HARDWARE.0 | MFT_ENUM_FLAG_TRANSCODE_ONLY.0 | MFT_ENUM_FLAG_SORTANDFILTER.0,
    );
    let acts = mf::enumerate_mfts(MFT_CATEGORY_VIDEO_ENCODER, flags, None, Some(info));
    let mut errors = Vec::new();
    for a in acts {
        let name = mf::friendly_name(&a);
        drop(a);
        match try_encoder(&device, params, &name) {
            Ok(()) => {
                println!("{}", serde_json::json!({ "ok": true, "name": name }));
                return Ok(());
            }
            Err(e) => errors.push(format!("{name}: {e:#}")),
        }
    }
    let error = if errors.is_empty() {
        "nenhum encoder H.264 de hardware encontrado".to_string()
    } else {
        errors.join("; ")
    };
    println!("{}", serde_json::json!({ "ok": false, "error": error }));
    Ok(())
}
