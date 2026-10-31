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
use livescreen_core::preset::EncodeParams;
use livescreen_core::protocol::Codec;
use crate::video::encoder::{self, EncoderKind, Nv12Frame};
use livescreen_core::host::HostStats;
use crate::video::mf;

/// Escolha do encoder: hardware só se o probe fora do processo aprovar.
pub fn detect(overrides: &Overrides) -> Vec<EncoderKind> {
    if overrides.deny_hardware_encoder {
        return vec![EncoderKind::Software];
    }
    match probe_child() {
        Ok(kinds) if !kinds.is_empty() => {
            let mut all = kinds;
            all.push(EncoderKind::Software);
            all
        }
        Ok(_) => {
            warn!("nenhum encoder de hardware funcionou; usando software");
            vec![EncoderKind::Software]
        }
        Err(e) => {
            warn!("probe do encoder falhou: {e:#}; usando software");
            vec![EncoderKind::Software]
        }
    }
}

fn probe_child() -> Result<Vec<EncoderKind>> {
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
        if start.elapsed() > Duration::from_secs(30) {
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
        let mut kinds = Vec::new();
        if let Some(arr) = v["encoders"].as_array() {
            for item in arr {
                let codec: Codec = serde_json::from_value(item["codec"].clone())?;
                let name = item["name"].as_str().unwrap_or_default().to_string();
                kinds.push(EncoderKind::Hardware { name, codec });
            }
        }
        Ok(kinds)
    } else {
        Ok(Vec::new())
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

fn try_encoder(device: &ID3D11Device, params: EncodeParams, kind: &EncoderKind) -> Result<()> {
    let tex = black_nv12(device, params.width, params.height)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(2);
    let (btx, mut brx) = broadcast::channel(8);
    let handle = encoder::spawn(
        kind.clone(),
        device,
        params,
        false,
        rx,
        btx,
        Arc::new(AtomicBool::new(false)),
        Arc::new(HostStats::default()),
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

fn candidates(codec: Codec) -> Vec<String> {
    let info = MFT_REGISTER_TYPE_INFO {
        guidMajorType: MFMediaType_Video,
        guidSubtype: mf::subtype(codec),
    };
    let flags = MFT_ENUM_FLAG(
        MFT_ENUM_FLAG_HARDWARE.0 | MFT_ENUM_FLAG_TRANSCODE_ONLY.0 | MFT_ENUM_FLAG_SORTANDFILTER.0,
    );
    mf::enumerate_mfts(MFT_CATEGORY_VIDEO_ENCODER, flags, None, Some(info))
        .into_iter()
        .map(|a| mf::friendly_name(&a))
        .collect()
}

/// `--probe-encoder`: testa cada MFT de hardware e imprime uma linha JSON.
pub fn run_child() -> Result<()> {
    mf::startup();
    let (device, _ctx) = crate::gpu::create_device(None)?;
    let params = EncodeParams {
        width: 1280,
        height: 720,
        fps: 30,
        bitrate_bps: 1_000_000,
    };
    let mut encoders = Vec::new();
    let mut errors = Vec::new();
    for codec in [Codec::Av1, Codec::Hevc, Codec::H264] {
        for name in candidates(codec) {
            let kind = EncoderKind::Hardware {
                name: name.clone(),
                codec,
            };
            match try_encoder(&device, params, &kind) {
                Ok(()) => {
                    encoders.push(serde_json::json!({
                        "codec": codec,
                        "name": name,
                    }));
                    break;
                }
                Err(e) => errors.push(format!("{name} ({}): {e:#}", codec.label())),
            }
        }
    }
    if !encoders.is_empty() {
        println!("{}", serde_json::json!({ "ok": true, "encoders": encoders }));
    } else {
        let error = if errors.is_empty() {
            "nenhum encoder de hardware encontrado".to_string()
        } else {
            errors.join("; ")
        };
        println!("{}", serde_json::json!({ "ok": false, "error": error }));
    }
    Ok(())
}

#[cfg(test)]
pub fn hardware_kinds() -> Vec<EncoderKind> {
    mf::startup();
    let mut kinds = Vec::new();
    for codec in [Codec::Av1, Codec::Hevc, Codec::H264] {
        if let Some(name) = candidates(codec).into_iter().next() {
            kinds.push(EncoderKind::Hardware { name, codec });
        }
    }
    kinds.push(EncoderKind::Software);
    kinds
}
