//! Decoder (MFT da Microsoft: H.264, HEVC, AV1) em thread própria; publica o quadro mais novo.
//!
//! Hardware: o MFT recebe um `IMFDXGIDeviceManager`, entrega NV12 em textura D3D11 e o
//! Video Processor converte para RGBA na GPU. Com `target` (adaptador do egui em DX12) o RGBA
//! vai para texturas compartilhadas (`present`) e nada passa pela CPU; senão, ou se a UI não
//! consegue importar, só o RGBA final é lido de volta. Se a GPU não serve (ou falha 3 vezes
//! seguidas), a thread troca por um MFT novo de software.
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use anyhow::{Result, anyhow, bail};
use eframe::egui::Color32;
use tracing::{info, warn};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_NV12, DXGI_SAMPLE_DESC,
};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::core::Interface;

use crate::protocol::{Codec, EncodedFrame};
use crate::video::convert::Converter;
use crate::video::{mf, present};

pub struct RgbaFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color32>,
}

/// Quadro pronto para a UI: pixels na CPU, ou textura compartilhada na GPU.
pub enum VideoFrame {
    Cpu(RgbaFrame),
    Gpu(present::GpuFrame),
}

impl VideoFrame {
    pub fn size(&self) -> (u32, u32) {
        match self {
            VideoFrame::Cpu(f) => (f.width, f.height),
            VideoFrame::Gpu(g) => g.size(),
        }
    }
}

// Color32 é [u8; 4] RGBA; com alfa 255 premultiplicado == não premultiplicado
const _: () = assert!(std::mem::size_of::<Color32>() == 4);
fn as_bytes(px: &mut [Color32]) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(px.as_mut_ptr().cast(), px.len() * 4) }
}

/// Contadores lidos pelo `stats_loop` do viewer.
pub struct DecodeStats {
    /// quadros publicados
    pub frames: AtomicU64,
    /// push + drain (decodificação + conversão), em µs
    pub busy_us: AtomicU64,
    /// modo atual do decoder
    pub hardware: AtomicBool,
    /// codec atualmente decodificado; u8::MAX = nenhum
    pub codec: AtomicU8,
    /// o quadro vai à UI por textura compartilhada, sem readback
    pub zero_copy: AtomicBool,
}

impl Default for DecodeStats {
    fn default() -> Self {
        Self {
            frames: AtomicU64::new(0),
            busy_us: AtomicU64::new(0),
            hardware: AtomicBool::new(false),
            codec: AtomicU8::new(u8::MAX),
            zero_copy: AtomicBool::new(false),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn(
    input: Receiver<EncodedFrame>,
    out: Arc<Mutex<Option<VideoFrame>>>,
    ctx: eframe::egui::Context,
    need_key: Arc<AtomicBool>,
    stats: Arc<DecodeStats>,
    hardware: bool,
    target: Option<present::Target>,
    readback_only: Arc<AtomicBool>,
) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("decoder".into())
        .spawn(move || {
            let _mmcss = crate::gpu::Mmcss::new(windows::core::w!("Playback"));
            if let Err(e) = run(input, out, ctx, need_key, stats, hardware, target, readback_only) {
                warn!("decoder: {e:#}");
            }
        })
        .expect("thread do decoder")
}

enum GpuOut {
    Readback {
        staging: Option<(ID3D11Texture2D, u32, u32, DXGI_FORMAT)>,
    },
    Shared(present::Producer),
}

struct Gpu {
    _mgr: IMFDXGIDeviceManager,
    _device: ID3D11Device,
    context: ID3D11DeviceContext,
    conv: Converter,
    out: GpuOut,
}

pub fn supported_codecs(hardware: bool) -> Vec<Codec> {
    if !hardware {
        return vec![Codec::H264];
    }
    mf::startup();
    let Ok((device, _)) = crate::gpu::create_device(None) else {
        return vec![Codec::H264];
    };
    let Ok(vdev) = device.cast::<ID3D11VideoDevice>() else {
        return vec![Codec::H264];
    };
    let mut codecs = vec![Codec::H264];
    for (codec, profile) in [
        (Codec::Hevc, D3D11_DECODER_PROFILE_HEVC_VLD_MAIN),
        (Codec::Av1, D3D11_DECODER_PROFILE_AV1_VLD_PROFILE0),
    ] {
        let fmt_ok = unsafe { vdev.CheckVideoDecoderFormat(&profile, DXGI_FORMAT_NV12) }
            .map(|b| b.as_bool())
            .unwrap_or(false);
        if !fmt_ok {
            continue;
        }
        let info = MFT_REGISTER_TYPE_INFO {
            guidMajorType: MFMediaType_Video,
            guidSubtype: mf::subtype(codec),
        };
        let flags = MFT_ENUM_FLAG(
            MFT_ENUM_FLAG_SYNCMFT.0
                | MFT_ENUM_FLAG_ASYNCMFT.0
                | MFT_ENUM_FLAG_HARDWARE.0
                | MFT_ENUM_FLAG_SORTANDFILTER.0,
        );
        let acts = mf::enumerate_mfts(MFT_CATEGORY_VIDEO_DECODER, flags, Some(info), None);
        if acts.is_empty() {
            continue;
        }
        let d3d11_aware = acts.iter().any(|a| {
            if let Ok(t) = unsafe { a.ActivateObject::<IMFTransform>() } {
                unsafe {
                    t.GetAttributes()
                        .ok()
                        .and_then(|attrs| attrs.GetUINT32(&MF_SA_D3D11_AWARE).ok())
                }
                .unwrap_or(0)
                    != 0
            } else {
                false
            }
        });
        if d3d11_aware {
            codecs.push(codec);
        }
    }
    codecs
}

fn create(hardware: bool, codec: Codec, target: Option<present::Target>) -> Result<(IMFTransform, Option<Gpu>)> {
    if hardware {
        match create_mft(true, codec, target) {
            Ok(r) => {
                info!("decoder de vídeo: hardware ({})", codec.label());
                return Ok(r);
            }
            Err(e) => warn!(
                "decoder de hardware indisponível para {}: {e:#}; usando software",
                codec.label()
            ),
        }
    }
    let r = create_mft(false, codec, None)?;
    info!("decoder de vídeo: software ({})", codec.label());
    Ok(r)
}

fn create_mft(hardware: bool, codec: Codec, target: Option<present::Target>) -> Result<(IMFTransform, Option<Gpu>)> {
    let t: IMFTransform = match codec {
        Codec::H264 => {
            unsafe { CoCreateInstance(&CLSID_MSH264DecoderMFT, None, CLSCTX_INPROC_SERVER)? }
        }
        Codec::Hevc | Codec::Av1 => {
            let info = MFT_REGISTER_TYPE_INFO {
                guidMajorType: MFMediaType_Video,
                guidSubtype: mf::subtype(codec),
            };
            let flags = if hardware {
                MFT_ENUM_FLAG(
                    MFT_ENUM_FLAG_SYNCMFT.0
                        | MFT_ENUM_FLAG_ASYNCMFT.0
                        | MFT_ENUM_FLAG_HARDWARE.0
                        | MFT_ENUM_FLAG_SORTANDFILTER.0,
                )
            } else {
                MFT_ENUM_FLAG(
                    MFT_ENUM_FLAG_SYNCMFT.0
                        | MFT_ENUM_FLAG_ASYNCMFT.0
                        | MFT_ENUM_FLAG_SORTANDFILTER.0,
                )
            };
            let act = mf::enumerate_mfts(MFT_CATEGORY_VIDEO_DECODER, flags, Some(info), None)
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("decoder não encontrado para {}", codec.label()))?;
            unsafe { act.ActivateObject()? }
        }
    };
    if mf::is_async(&t) {
        unsafe {
            if let Ok(attrs) = t.GetAttributes() {
                let _ = attrs.SetUINT32(&MF_TRANSFORM_ASYNC_UNLOCK, 1);
            }
        }
    }
    let gpu = if hardware {
        Some(attach_gpu(&t, target)?)
    } else {
        None
    };
    if let Ok(api) = t.cast::<ICodecAPI>() {
        mf::set_codec_value(&api, &CODECAPI_AVLowLatencyMode, mf::variant_u32(1));
    }
    unsafe {
        let mt = MFCreateMediaType()?;
        mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        mt.SetGUID(&MF_MT_SUBTYPE, &mf::subtype(codec))?;
        mt.SetUINT64(&MF_MT_FRAME_SIZE, mf::pack_pair(1920, 1080))?;
        t.SetInputType(0, &mt, 0)?;
    }
    set_nv12_output(&t)?;
    Ok((t, gpu))
}

fn attach_gpu(t: &IMFTransform, target: Option<present::Target>) -> Result<Gpu> {
    let aware = unsafe { t.GetAttributes()?.GetUINT32(&MF_SA_D3D11_AWARE) }.unwrap_or(0);
    if aware == 0 {
        bail!("decoder sem suporte a D3D11");
    }
    let (device, context) = crate::gpu::create_device(target.map(|t| t.luid))?;
    let mgr = unsafe {
        let mut token = 0u32;
        let mut mgr: Option<IMFDXGIDeviceManager> = None;
        MFCreateDXGIDeviceManager(&mut token, &mut mgr)?;
        let mgr = mgr.ok_or_else(|| anyhow!("MFCreateDXGIDeviceManager sem manager"))?;
        mgr.ResetDevice(&device, token)?;
        t.ProcessMessage(MFT_MESSAGE_SET_D3D_MANAGER, mgr.as_raw() as usize)?;
        mgr
    };
    let shared = target.and_then(|_| {
        let r = Converter::new_rgb_shared(&device, &context, present::SLOTS)
            .and_then(|c| present::Producer::new(&device, &context).map(|p| (c, p)));
        r.map_err(|e| warn!("zero-copy indisponível: {e:#}")).ok()
    });
    let (conv, out) = match shared {
        Some((conv, p)) => (conv, GpuOut::Shared(p)),
        None => (
            Converter::new_rgb(&device, &context)?,
            GpuOut::Readback { staging: None },
        ),
    };
    Ok(Gpu {
        _mgr: mgr,
        _device: device,
        context,
        conv,
        out,
    })
}

impl Gpu {
    fn zero_copy(&self) -> bool {
        matches!(self.out, GpuOut::Shared(_))
    }

    /// `None`: modo compartilhado sem slot livre (quadro descartado).
    fn output(&mut self, t: &IMFTransform, sample: &IMFSample, readback_only: bool) -> Result<Option<VideoFrame>> {
        if readback_only && self.zero_copy() {
            // a UI não conseguiu importar a textura: volta ao caminho com readback
            self.conv = Converter::new_rgb(&self._device, &self.context)?;
            self.out = GpuOut::Readback { staging: None };
        }
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
            let cache = match &mut self.out {
                GpuOut::Shared(p) => {
                    return Ok(p.publish(&mut self.conv, &tex, slice, (w, h))?.map(VideoFrame::Gpu));
                }
                GpuOut::Readback { staging } => staging,
            };
            let rgb = self.conv.convert_slice(&tex, slice, (w, h))?;

            let mut desc = D3D11_TEXTURE2D_DESC::default();
            rgb.GetDesc(&mut desc);
            let (fw, fh, format) = (desc.Width, desc.Height, desc.Format);
            let staging = match &*cache {
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
                    *cache = Some((s.clone(), fw, fh, format));
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
            Ok(Some(VideoFrame::Cpu(RgbaFrame {
                width: fw,
                height: fh,
                pixels,
            })))
        }
    }
}

fn run(
    input: Receiver<EncodedFrame>,
    out: Arc<Mutex<Option<VideoFrame>>>,
    ctx: eframe::egui::Context,
    need_key: Arc<AtomicBool>,
    stats: Arc<DecodeStats>,
    hardware: bool,
    target: Option<present::Target>,
    readback_only: Arc<AtomicBool>,
) -> Result<()> {
    mf::startup();
    let mut active: Option<(Codec, IMFTransform, Option<Gpu>)> = None;
    stats.codec.store(u8::MAX, Ordering::Relaxed);
    stats.hardware.store(false, Ordering::Relaxed);

    let mut discarding = true;
    let mut spare = mf::OutSample::default();
    let mut hw_failures = 0u32;
    loop {
        let frame = match input.recv() {
            Ok(f) => f,
            Err(_) => break,
        };
        let need_recreate = match &active {
            Some((c, _, _)) => *c != frame.codec,
            None => true,
        };
        if need_recreate {
            if !frame.keyframe {
                continue;
            }
            match create(hardware, frame.codec, target) {
                Ok((t, gpu)) => {
                    stats.hardware.store(gpu.is_some(), Ordering::Relaxed);
                    stats.codec.store(frame.codec.wire(), Ordering::Relaxed);
                    active = Some((frame.codec, t, gpu));
                    hw_failures = 0;
                    spare = mf::OutSample::default();
                    discarding = false;
                }
                Err(e) => {
                    warn!(
                        "decoder: falha ao criar para {}: {e:#}",
                        frame.codec.label()
                    );
                    need_key.store(true, Ordering::Relaxed);
                    active = None;
                    stats.codec.store(u8::MAX, Ordering::Relaxed);
                    stats.hardware.store(false, Ordering::Relaxed);
                    continue;
                }
            }
        }
        if discarding && !frame.keyframe {
            continue;
        }
        let Some((codec, t, gpu)) = active.as_mut() else {
            continue;
        };
        let t0 = Instant::now();
        let mut ok = true;
        if let Err(e) = push(t, &frame) {
            warn!("decoder: entrada: {e:#}");
            ok = false;
        } else if let Err(e) = drain(t, &out, &ctx, &mut spare, gpu.as_mut(), readback_only.load(Ordering::Relaxed), &stats) {
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
                    match create(false, *codec, target) {
                        Ok((new_t, new_gpu)) => {
                            *t = new_t;
                            *gpu = new_gpu;
                            stats.hardware.store(false, Ordering::Relaxed);
                            stats.zero_copy.store(false, Ordering::Relaxed);
                            spare = mf::OutSample::default();
                            hw_failures = 0;
                            continue;
                        }
                        Err(e) => {
                            warn!("decoder: falha ao trocar para software: {e:#}");
                            active = None;
                            stats.codec.store(u8::MAX, Ordering::Relaxed);
                            stats.hardware.store(false, Ordering::Relaxed);
                            continue;
                        }
                    }
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
    if let Some((_, t, _)) = active.as_ref() {
        unsafe {
            let _ = t.ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0);
        }
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
    out: &Arc<Mutex<Option<VideoFrame>>>,
    ctx: &eframe::egui::Context,
    spare: &mut mf::OutSample,
    mut gpu: Option<&mut Gpu>,
    readback_only: bool,
    stats: &DecodeStats,
) -> Result<()> {
    loop {
        let r = mf::process_output(t, spare)?;
        match r {
            mf::OutputResult::Sample(s) => {
                let frame = match gpu.as_deref_mut() {
                    Some(g) => {
                        let f = g.output(t, &s, readback_only)?;
                        stats.zero_copy.store(g.zero_copy(), Ordering::Relaxed);
                        f
                    }
                    None => Some(VideoFrame::Cpu(to_rgba(t, &s)?)),
                };
                // sem quadro: a UI está atrasada e não há slot livre; o quadro é só descartado
                if let Some(frame) = frame {
                    *out.lock().unwrap() = Some(frame);
                    ctx.request_repaint();
                }
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
    unsafe {
        if let Ok(len) = ty.GetBlobSize(&MF_MT_MINIMUM_DISPLAY_APERTURE) {
            if len as usize >= std::mem::size_of::<MFVideoArea>() {
                let mut area = MFVideoArea::default();
                let mut got = 0u32;
                let slice = std::slice::from_raw_parts_mut(
                    &mut area as *mut _ as *mut u8,
                    std::mem::size_of::<MFVideoArea>(),
                );
                if ty
                    .GetBlob(&MF_MT_MINIMUM_DISPLAY_APERTURE, slice, Some(&mut got))
                    .is_ok()
                    && area.Area.cx > 0
                    && area.Area.cy > 0
                {
                    return Ok((area.Area.cx as u32, area.Area.cy as u32));
                }
            }
        }
    }
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
