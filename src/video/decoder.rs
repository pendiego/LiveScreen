//! Decoder H.264 (MFT da Microsoft) em thread própria; publica o RGBA mais novo.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use anyhow::{Result, anyhow, bail};
use tracing::warn;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::core::Interface;

use crate::protocol::EncodedFrame;
use crate::video::mf;

pub struct RgbaFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub fn spawn(
    input: Receiver<EncodedFrame>,
    out: Arc<Mutex<Option<RgbaFrame>>>,
    ctx: eframe::egui::Context,
    need_key: Arc<AtomicBool>,
) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("decoder".into())
        .spawn(move || {
            if let Err(e) = run(input, out, ctx, need_key) {
                warn!("decoder: {e:#}");
            }
        })
        .expect("thread do decoder")
}

fn run(
    input: Receiver<EncodedFrame>,
    out: Arc<Mutex<Option<RgbaFrame>>>,
    ctx: eframe::egui::Context,
    need_key: Arc<AtomicBool>,
) -> Result<()> {
    mf::startup();
    let t: IMFTransform =
        unsafe { CoCreateInstance(&CLSID_MSH264DecoderMFT, None, CLSCTX_INPROC_SERVER)? };
    // o decoder da Microsoft é assíncrono; desbloqueia para uso síncrono
    if mf::is_async(&t) {
        unsafe {
            if let Ok(attrs) = t.GetAttributes() {
                let _ = attrs.SetUINT32(&MF_TRANSFORM_ASYNC_UNLOCK, 1);
            }
        }
    }
    if let Ok(api) = t.cast::<ICodecAPI>() {
        mf::set_codec_value(&api, &CODECAPI_AVLowLatencyMode, mf::variant_u32(1));
    }
    unsafe {
        let mt = MFCreateMediaType()?;
        mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        mt.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
        t.SetInputType(0, &mt, 0)?;
    }
    set_nv12_output(&t)?;

    let mut discarding = true;
    let mut spare = mf::OutSample::default();
    loop {
        let frame = match input.recv() {
            Ok(f) => f,
            Err(_) => break,
        };
        if discarding && !frame.keyframe {
            continue;
        }
        let mut ok = true;
        if let Err(e) = push(&t, &frame) {
            warn!("decoder: entrada: {e:#}");
            ok = false;
        } else if let Err(e) = drain(&t, &out, &ctx, &mut spare) {
            warn!("decoder: saída: {e:#}");
            ok = false;
        }
        if !ok {
            discarding = true;
            need_key.store(true, Ordering::Relaxed);
            unsafe {
                let _ = t.ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0);
            }
            continue;
        }
        discarding = false;
    }
    unsafe {
        let _ = t.ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0);
    }
    Ok(())
}

fn set_nv12_output(t: &IMFTransform) -> Result<()> {
    unsafe {
        for i in 0..32 {
            let Ok(mt) = t.GetOutputAvailableType(0, i) else {
                break;
            };
            if mt.GetGUID(&MF_MT_SUBTYPE).unwrap_or_default() == MFVideoFormat_NV12
                && t.SetOutputType(0, &mt, 0).is_ok()
            {
                return Ok(());
            }
        }
        // antes de parsear o SPS o decoder pode não oferecer um tipo completo
        let mt = MFCreateMediaType()?;
        mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        mt.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)?;
        t.SetOutputType(0, &mt, 0)?;
    }
    Ok(())
}

fn push(t: &IMFTransform, frame: &EncodedFrame) -> Result<()> {
    let len = frame.data.len() as u32;
    unsafe {
        let mb = MFCreateMemoryBuffer(len.max(1))?;
        let mut ptr: *mut u8 = std::ptr::null_mut();
        let mut max = 0u32;
        let mut cur = 0u32;
        mb.Lock(&mut ptr, Some(&mut max), Some(&mut cur))?;
        std::ptr::copy_nonoverlapping(frame.data.as_ptr(), ptr, len as usize);
        mb.Unlock()?;
        mb.SetCurrentLength(len)?;
        let sample = MFCreateSample()?;
        sample.AddBuffer(&mb)?;
        sample.SetSampleTime((frame.pts_us * 10) as i64)?;
        t.ProcessInput(0, &sample, 0)?;
    }
    Ok(())
}

fn drain(
    t: &IMFTransform,
    out: &Arc<Mutex<Option<RgbaFrame>>>,
    ctx: &eframe::egui::Context,
    spare: &mut mf::OutSample,
) -> Result<()> {
    loop {
        let r = mf::process_output(t, spare)?;
        match r {
            mf::OutputResult::Sample(s) => {
                let rgba = to_rgba(t, &s)?;
                *out.lock().unwrap() = Some(rgba);
                ctx.request_repaint();
            }
            mf::OutputResult::NeedMoreInput => return Ok(()),
            mf::OutputResult::StreamChange => {
                unsafe {
                    let _ = t.SetOutputType(0, None::<&IMFMediaType>, 0);
                }
                set_nv12_output(t)?;
            }
        }
    }
}

fn to_rgba(t: &IMFTransform, sample: &IMFSample) -> Result<RgbaFrame> {
    unsafe {
        let ty = t.GetOutputCurrentType(0)?;
        let (w, h) = mf::unpack_pair(ty.GetUINT64(&MF_MT_FRAME_SIZE)?);
        if w == 0 || h == 0 {
            bail!("dimensões de saída desconhecidas");
        }
        let buf = sample.ConvertToContiguousBuffer()?;
        let mut rgba = vec![0u8; (w * h * 4) as usize];

        let mut converted = false;
        if let Ok(i2d) = buf.cast::<IMF2DBuffer>() {
            let mut ptr: *mut u8 = std::ptr::null_mut();
            let mut pitch = 0i32;
            if i2d.Lock2D(&mut ptr, &mut pitch).is_ok() {
                if !ptr.is_null() && pitch > 0 {
                    let len = i2d
                        .GetContiguousLength()
                        .unwrap_or_else(|_| buf.GetCurrentLength().unwrap_or(0));
                    let aligned_h = (len / pitch as u32) * 2 / 3;
                    converted =
                        convert_planes(ptr, pitch as u32, aligned_h, w, h, &mut rgba).is_ok();
                }
                let _ = i2d.Unlock2D();
            }
        }
        if !converted {
            let mut ptr: *mut u8 = std::ptr::null_mut();
            let mut max = 0u32;
            let mut cur = 0u32;
            buf.Lock(&mut ptr, Some(&mut max), Some(&mut cur))?;
            let r = if ptr.is_null() {
                bail!("buffer de saída nulo")
            } else {
                convert_planes(ptr, w, h, w, h, &mut rgba)
            };
            let _ = buf.Unlock();
            r?;
        }
        Ok(RgbaFrame {
            width: w,
            height: h,
            data: rgba,
        })
    }
}

/// NV12 → RGBA (BT.709 limitado); `aligned_h` é a altura real do plano Y.
fn convert_planes(
    ptr: *const u8,
    pitch: u32,
    aligned_h: u32,
    w: u32,
    h: u32,
    rgba: &mut [u8],
) -> Result<()> {
    if aligned_h < h {
        bail!("NV12: altura alinhada ({aligned_h}) menor que a visível ({h})");
    }
    let metric = |a: u32, b: u32| (a * b) as usize;
    let y_len = metric(pitch, aligned_h);
    let uv_len = metric(pitch, aligned_h / 2);
    unsafe {
        let y = std::slice::from_raw_parts(ptr, y_len);
        let uv = std::slice::from_raw_parts(ptr.add(y_len), uv_len);
        let img = yuv::YuvBiPlanarImage {
            y_plane: y,
            y_stride: pitch,
            uv_plane: uv,
            uv_stride: pitch,
            width: w,
            height: h,
        };
        yuv::yuv_nv12_to_rgba(
            &img,
            rgba,
            w * 4,
            yuv::YuvRange::Limited,
            yuv::YuvStandardMatrix::Bt709,
            yuv::YuvConversionMode::Balanced,
        )
        .map_err(|e| anyhow!("nv12→rgba: {e:?}"))?;
    }
    Ok(())
}
