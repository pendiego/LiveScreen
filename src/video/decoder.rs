//! Decoder H.264 (MFT da Microsoft) em thread própria; publica o quadro mais novo.
//!
//! Hardware: o MFT recebe um `IMFDXGIDeviceManager`, entrega NV12 em textura D3D11 e o
//! Video Processor converte para RGB na GPU; só o RGB final é lido de volta. Se a GPU não
//! serve (ou falha 3 vezes seguidas), a thread troca por um MFT novo de software.
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use anyhow::{Result, anyhow, bail};
use eframe::egui::Color32;
use tracing::{info, warn};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC,
};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::core::Interface;

use crate::protocol::EncodedFrame;
use crate::video::convert::Converter;
use crate::video::mf;

pub struct RgbaFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color32>,
}

// Color32 é [u8; 4] RGBA; com alfa 255 premultiplicado == não premultiplicado
const _: () = assert!(std::mem::size_of::<Color32>() == 4);
fn as_bytes(px: &mut [Color32]) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(px.as_mut_ptr().cast(), px.len() * 4) }
}

/// Contadores lidos pelo `stats_loop` do viewer.
#[derive(Default)]
pub struct DecodeStats {
    /// quadros publicados
    pub frames: AtomicU64,
    /// push + drain (decodificação + conversão), em µs
    pub busy_us: AtomicU64,
    /// modo atual do decoder
    pub hardware: AtomicBool,
}

pub fn spawn(
    input: Receiver<EncodedFrame>,
    out: Arc<Mutex<Option<RgbaFrame>>>,
    ctx: eframe::egui::Context,
    need_key: Arc<AtomicBool>,
    stats: Arc<DecodeStats>,
    hardware: bool,
) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("decoder".into())
        .spawn(move || {
            let _mmcss = crate::gpu::Mmcss::new(windows::core::w!("Playback"));
            if let Err(e) = run(input, out, ctx, need_key, stats, hardware) {
                warn!("decoder: {e:#}");
            }
        })
        .expect("thread do decoder")
}

struct Gpu {
    _mgr: IMFDXGIDeviceManager,
    _device: ID3D11Device,
    context: ID3D11DeviceContext,
    conv: Converter,
    staging: Option<(ID3D11Texture2D, u32, u32, DXGI_FORMAT)>,
}

fn create(hardware: bool) -> Result<(IMFTransform, Option<Gpu>)> {
    if hardware {
        match create_mft(true) {
            Ok(r) => {
                info!("decoder de vídeo: hardware");
                return Ok(r);
            }
            Err(e) => warn!("decoder de hardware indisponível: {e:#}; usando software"),
        }
    }
    let r = create_mft(false)?;
    info!("decoder de vídeo: software");
    Ok(r)
}

fn create_mft(hardware: bool) -> Result<(IMFTransform, Option<Gpu>)> {
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
    // o manager precisa vir antes de qualquer tipo
    let gpu = if hardware {
        Some(attach_gpu(&t)?)
    } else {
        None
    };
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
    Ok((t, gpu))
}

fn attach_gpu(t: &IMFTransform) -> Result<Gpu> {
    let aware = unsafe { t.GetAttributes()?.GetUINT32(&MF_SA_D3D11_AWARE) }.unwrap_or(0);
    if aware == 0 {
        bail!("decoder H.264 sem suporte a D3D11");
    }
    let (device, context) = crate::gpu::create_device()?;
    let mgr = unsafe {
        let mut token = 0u32;
        let mut mgr: Option<IMFDXGIDeviceManager> = None;
        MFCreateDXGIDeviceManager(&mut token, &mut mgr)?;
        let mgr = mgr.ok_or_else(|| anyhow!("MFCreateDXGIDeviceManager sem manager"))?;
        mgr.ResetDevice(&device, token)?;
        t.ProcessMessage(MFT_MESSAGE_SET_D3D_MANAGER, mgr.as_raw() as usize)?;
        mgr
    };
    let conv = Converter::new_rgb(&device, &context)?;
    Ok(Gpu {
        _mgr: mgr,
        _device: device,
        context,
        conv,
        staging: None,
    })
}

impl Gpu {
    fn read(&mut self, t: &IMFTransform, sample: &IMFSample) -> Result<RgbaFrame> {
        let (w, h) = frame_size(t)?;
        unsafe {
            let buf: IMFDXGIBuffer = sample.GetBufferByIndex(0)?.cast()?;
            let mut raw = std::ptr::null_mut();
            buf.GetResource(&ID3D11Texture2D::IID, &mut raw)?;
            if raw.is_null() {
                bail!("textura do decoder nula");
            }
            let tex = ID3D11Texture2D::from_raw(raw);
            let slice = buf.GetSubresourceIndex()?;
            let rgb = self.conv.convert_slice(&tex, slice, (w, h))?;

            let mut desc = D3D11_TEXTURE2D_DESC::default();
            rgb.GetDesc(&mut desc);
            let (fw, fh, format) = (desc.Width, desc.Height, desc.Format);
            let staging = match &self.staging {
                Some((s, sw, sh, sf)) if (*sw, *sh, *sf) == (fw, fh, format) => s.clone(),
                _ => {
                    let sd = D3D11_TEXTURE2D_DESC {
                        Width: fw,
                        Height: fh,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: format,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Usage: D3D11_USAGE_STAGING,
                        BindFlags: 0,
                        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                        MiscFlags: 0,
                    };
                    let mut s = None;
                    self._device.CreateTexture2D(&sd, None, Some(&mut s))?;
                    let s = s.ok_or_else(|| anyhow!("staging nula"))?;
                    self.staging = Some((s.clone(), fw, fh, format));
                    s
                }
            };

            let mut pixels = vec![Color32::TRANSPARENT; fw as usize * fh as usize];
            self.context.CopyResource(&staging, &rgb);
            let mut m = D3D11_MAPPED_SUBRESOURCE::default();
            self.context
                .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut m))?;
            let r: Result<()> = if m.pData.is_null() {
                Err(anyhow!("mapa RGB nulo"))
            } else {
                let row = fw as usize * 4;
                let dst = as_bytes(&mut pixels);
                for y in 0..fh as usize {
                    let p = (m.pData as *const u8).add(y * m.RowPitch as usize);
                    std::ptr::copy_nonoverlapping(p, dst.as_mut_ptr().add(y * row), row);
                }
                Ok(())
            };
            self.context.Unmap(&staging, 0);
            r?;
            if format == DXGI_FORMAT_B8G8R8A8_UNORM {
                for px in as_bytes(&mut pixels).chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }
            Ok(RgbaFrame {
                width: fw,
                height: fh,
                pixels,
            })
        }
    }
}

fn run(
    input: Receiver<EncodedFrame>,
    out: Arc<Mutex<Option<RgbaFrame>>>,
    ctx: eframe::egui::Context,
    need_key: Arc<AtomicBool>,
    stats: Arc<DecodeStats>,
    hardware: bool,
) -> Result<()> {
    mf::startup();
    let (mut t, mut gpu) = create(hardware)?;
    stats.hardware.store(gpu.is_some(), Ordering::Relaxed);

    let mut discarding = true;
    let mut spare = mf::OutSample::default();
    let mut hw_failures = 0u32;
    loop {
        let frame = match input.recv() {
            Ok(f) => f,
            Err(_) => break,
        };
        if discarding && !frame.keyframe {
            continue;
        }
        let t0 = Instant::now();
        let mut ok = true;
        if let Err(e) = push(&t, &frame) {
            warn!("decoder: entrada: {e:#}");
            ok = false;
        } else if let Err(e) = drain(&t, &out, &ctx, &mut spare, gpu.as_mut()) {
            warn!("decoder: saída: {e:#}");
            ok = false;
        }
        if !ok {
            discarding = true;
            need_key.store(true, Ordering::Relaxed);
            if gpu.is_some() {
                hw_failures += 1;
                if hw_failures >= 3 {
                    warn!("decoder de hardware falhou 3 vezes seguidas; trocando para software");
                    (t, gpu) = create(false)?;
                    stats.hardware.store(false, Ordering::Relaxed);
                    spare = mf::OutSample::default();
                    hw_failures = 0;
                    continue;
                }
            }
            unsafe {
                let _ = t.ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0);
            }
            continue;
        }
        hw_failures = 0;
        discarding = false;
        stats.frames.fetch_add(1, Ordering::Relaxed);
        stats
            .busy_us
            .fetch_add(t0.elapsed().as_micros() as u64, Ordering::Relaxed);
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
    mut gpu: Option<&mut Gpu>,
) -> Result<()> {
    loop {
        let r = mf::process_output(t, spare)?;
        match r {
            mf::OutputResult::Sample(s) => {
                let frame = match gpu.as_deref_mut() {
                    Some(g) => g.read(t, &s)?,
                    None => to_rgba(t, &s)?,
                };
                *out.lock().unwrap() = Some(frame);
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

fn frame_size(t: &IMFTransform) -> Result<(u32, u32)> {
    let ty = unsafe { t.GetOutputCurrentType(0)? };
    let (w, h) = mf::unpack_pair(unsafe { ty.GetUINT64(&MF_MT_FRAME_SIZE)? });
    if w == 0 || h == 0 {
        bail!("dimensões de saída desconhecidas");
    }
    Ok((w, h))
}

fn to_rgba(t: &IMFTransform, sample: &IMFSample) -> Result<RgbaFrame> {
    unsafe {
        let (w, h) = frame_size(t)?;
        let buf = sample.ConvertToContiguousBuffer()?;
        let mut pixels = vec![Color32::TRANSPARENT; (w * h) as usize];

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
                        convert_planes(ptr, pitch as u32, aligned_h, w, h, as_bytes(&mut pixels))
                            .is_ok();
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
                convert_planes(ptr, w, h, w, h, as_bytes(&mut pixels))
            };
            let _ = buf.Unlock();
            r?;
        }
        Ok(RgbaFrame {
            width: w,
            height: h,
            pixels,
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
