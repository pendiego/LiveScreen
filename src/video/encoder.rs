//! Encoder H.264 por Media Foundation, uma thread por encoder.
//!
//! Hardware (MFT de hardware + `IMFDXGIDeviceManager`) recebe os samples
//! direto da textura NV12; o software (MFT da Microsoft) copia via CPU.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use bytes::Bytes;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::core::Interface;

use livescreen_core::host::HostStats;
use livescreen_core::preset::EncodeParams;
use livescreen_core::protocol::{Codec, EncodedFrame};
use crate::video::mf;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncoderKind {
    Hardware { name: String, codec: Codec },
    Software,
}

impl EncoderKind {
    pub fn codec(&self) -> Codec {
        match self {
            EncoderKind::Hardware { codec, .. } => *codec,
            EncoderKind::Software => Codec::H264,
        }
    }

    pub fn label(&self) -> String {
        match self {
            EncoderKind::Hardware { name, .. } => name.clone(),
            EncoderKind::Software => "software (Microsoft H.264)".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Nv12Frame {
    pub tex: ID3D11Texture2D,
    pub pts: Duration,
}

/// VBV curto: limita picos de quadro que enchem a fila da rede (250 ms de buffer, em bytes).
fn vbv_bytes(bps: u32) -> u32 {
    bps / 32
}

pub struct EncoderHandle {
    join: Option<std::thread::JoinHandle<()>>,
}

impl EncoderHandle {
    /// Só retorna depois que a thread terminou; chame **depois** de abandonar o
    /// `Sender` de entrada, senão a thread fica esperando quadros.
    pub fn stop(mut self) {
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

pub fn spawn(
    kind: EncoderKind,
    device: &ID3D11Device,
    params: EncodeParams,
    input: Receiver<Nv12Frame>,
    out: broadcast::Sender<Arc<EncodedFrame>>,
    force_key: Arc<AtomicBool>,
    stats: Arc<HostStats>,
) -> Result<EncoderHandle> {
    mf::startup();
    let device = device.clone();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<()>>();
    let join =
        std::thread::Builder::new()
            .name("encoder".into())
            .spawn(move || {
                let _mmcss = crate::gpu::Mmcss::new(windows::core::w!("Capture"));
                match Encoder::new(kind, &device, params, input, out, force_key, stats) {
                    Ok(mut enc) => {
                        let _ = ready_tx.send(Ok(()));
                        enc.run();
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                    }
                }
            })?;
    match ready_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(Ok(())) => Ok(EncoderHandle { join: Some(join) }),
        Ok(Err(e)) => {
            let _ = join.join();
            Err(e)
        }
        Err(_) => Err(anyhow!("inicialização do encoder não respondeu")),
    }
}

/// Cópia NV12 para a CPU, exigida pelo MFT de software.
struct CpuCopy {
    ctx: ID3D11DeviceContext,
    staging: ID3D11Texture2D,
    w: u32,
    h: u32,
}

impl CpuCopy {
    fn new(device: &ID3D11Device, w: u32, h: u32) -> Result<Self> {
        let ctx = unsafe { device.GetImmediateContext()? };
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
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut tex = None;
        unsafe { device.CreateTexture2D(&desc, None, Some(&mut tex))? };
        Ok(Self {
            ctx,
            staging: tex.ok_or_else(|| anyhow!("staging nula"))?,
            w,
            h,
        })
    }

    fn len(&self) -> usize {
        (self.w as usize) * (self.h as usize * 3 / 2)
    }

    fn read_into(&mut self, src: &ID3D11Texture2D, dst: *mut u8) -> Result<()> {
        unsafe {
            self.ctx.CopyResource(&self.staging, src);
            let mut m = D3D11_MAPPED_SUBRESOURCE::default();
            self.ctx
                .Map(&self.staging, 0, D3D11_MAP_READ, 0, Some(&mut m))?;
            let r: Result<()> = if m.pData.is_null() {
                Err(anyhow!("mapa NV12 nulo"))
            } else {
                let w = self.w as usize;
                let rows = self.h as usize * 3 / 2;
                for y in 0..rows {
                    let p = (m.pData as *const u8).add(y * m.RowPitch as usize);
                    std::ptr::copy_nonoverlapping(p, dst.add(y * w), w);
                }
                Ok(())
            };
            self.ctx.Unmap(&self.staging, 0);
            r
        }
    }
}

enum Feed {
    Frame(Nv12Frame),
    Stop,
    Idle,
}

struct Encoder {
    t: IMFTransform,
    events: Option<IMFMediaEventGenerator>,
    api: Option<ICodecAPI>,
    input_id: u32,
    output_id: u32,
    seq_header: Vec<u8>,
    fps: u32,
    seq: u64,
    last: Option<Nv12Frame>,
    start: Instant,
    input: Receiver<Nv12Frame>,
    out: broadcast::Sender<Arc<EncodedFrame>>,
    force_key: Arc<AtomicBool>,
    stats: Arc<HostStats>,
    ceiling: u32,
    bitrate: u32,
    cpu: Option<CpuCopy>,
    spare: mf::OutSample,
    codec: Codec,
}

impl Encoder {
    fn new(
        kind: EncoderKind,
        device: &ID3D11Device,
        params: EncodeParams,
        input: Receiver<Nv12Frame>,
        out: broadcast::Sender<Arc<EncodedFrame>>,
        force_key: Arc<AtomicBool>,
        stats: Arc<HostStats>,
    ) -> Result<Self> {
        stats.ceiling_bps.store(params.bitrate_bps, Ordering::Relaxed);
        stats.target_bps.store(0, Ordering::Relaxed);
        mf::startup();
        let (t, is_async) = create_mft(&kind, device)?;
        let (input_id, output_id) = stream_ids(&t);
        let codec = kind.codec();
        let api = configure(&t, &params, codec)?;
        let seq_header = read_seq_header(&t);
        let events_gen = if is_async {
            t.cast::<IMFMediaEventGenerator>().ok()
        } else {
            None
        };
        let cpu = match kind {
            EncoderKind::Software => {
                info!("encoder software: cópia via CPU");
                Some(CpuCopy::new(device, params.width, params.height)?)
            }
            EncoderKind::Hardware { .. } => None,
        };
        Ok(Self {
            t,
            events: events_gen,
            api,
            input_id,
            output_id,
            seq_header,
            fps: params.fps,
            seq: 0,
            last: None,
            start: Instant::now(),
            input,
            out,
            force_key,
            stats,
            ceiling: params.bitrate_bps,
            bitrate: params.bitrate_bps,
            cpu,
            spare: mf::OutSample::default(),
            codec,
        })
    }

    fn run(&mut self) {
        if self.events.is_some() {
            self.run_async();
        } else {
            self.run_sync();
        }
        // encerra o MFT
        unsafe {
            let _ = self.t.ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0);
            if let Ok(s) = self.t.cast::<IMFShutdown>() {
                let _ = s.Shutdown();
            }
        }
    }

    fn run_async(&mut self) {
        let events = self.events.clone().expect("generator");
        let mut need_input = false;
        loop {
            // drena eventos pendentes sem bloquear
            loop {
                match unsafe { events.GetEvent(MF_EVENT_FLAG_NO_WAIT) } {
                    Ok(ev) => match unsafe { ev.GetType() }.unwrap_or(0) {
                        t if t == METransformNeedInput.0 as u32 => need_input = true,
                        t if t == METransformHaveOutput.0 as u32 => self.drain(true),
                        _ => {}
                    },
                    Err(_) => break,
                }
            }
            if need_input {
                match self.feed(self.timeout()) {
                    Feed::Frame(f) => {
                        need_input = false;
                        if let Err(e) = self.process_input(&f) {
                            warn!("encoder: {e:#}");
                            break;
                        }
                    }
                    Feed::Stop => break,
                    Feed::Idle => continue,
                }
                continue;
            }
            // sem pedido pendente: espera o próximo evento
            match unsafe { events.GetEvent(MF_EVENT_FLAG_NONE) } {
                Ok(ev) => match unsafe { ev.GetType() }.unwrap_or(0) {
                    t if t == METransformNeedInput.0 as u32 => need_input = true,
                    t if t == METransformHaveOutput.0 as u32 => self.drain(true),
                    _ => {}
                },
                Err(e) => {
                    debug!("encoder: fim dos eventos: {e}");
                    break;
                }
            }
        }
    }

    fn run_sync(&mut self) {
        loop {
            match self.feed(self.timeout()) {
                Feed::Frame(f) => {
                    if let Err(e) = self.process_input(&f) {
                        warn!("encoder: {e:#}");
                        break;
                    }
                    self.drain(false);
                }
                Feed::Stop => break,
                Feed::Idle => continue,
            }
        }
    }

    fn timeout(&self) -> Duration {
        if self.force_key.load(Ordering::Relaxed) {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(1)
        }
    }

    /// Quadro novo, ou o último reenviado com pts novo para manter o stream vivo
    /// (janela minimizada / tela parada / keyframe para quem acabou de entrar).
    fn feed(&mut self, t: Duration) -> Feed {
        match self.input.recv_timeout(t) {
            Ok(f) => {
                self.last = Some(f.clone());
                Feed::Frame(f)
            }
            Err(RecvTimeoutError::Timeout) => match &self.last {
                Some(last) => {
                    let mut f = last.clone();
                    f.pts = self.start.elapsed();
                    Feed::Frame(f)
                }
                None => Feed::Idle,
            },
            Err(RecvTimeoutError::Disconnected) => Feed::Stop,
        }
    }

    fn make_sample(&mut self, f: &Nv12Frame) -> Result<IMFSample> {
        let sample = unsafe { MFCreateSample()? };
        unsafe {
            let buf = if let Some(cpu) = &mut self.cpu {
                let len = cpu.len();
                let mb = MFCreateMemoryBuffer(len as u32)?;
                let mut ptr: *mut u8 = std::ptr::null_mut();
                let mut max = 0u32;
                let mut cur = 0u32;
                mb.Lock(&mut ptr, Some(&mut max), Some(&mut cur))?;
                let copied = cpu.read_into(&f.tex, ptr);
                let _ = mb.Unlock();
                copied?;
                mb.SetCurrentLength(len as u32)?;
                mb
            } else {
                MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, &f.tex, 0, false)?
            };
            sample.AddBuffer(&buf)?;
            sample.SetSampleTime((f.pts.as_nanos() / 100) as i64)?;
            sample.SetSampleDuration((10_000_000f64 / self.fps as f64) as i64)?;
        }
        Ok(sample)
    }

    fn process_input(&mut self, f: &Nv12Frame) -> Result<()> {
        let want = match self.stats.target_bps.load(Ordering::Relaxed) {
            0 => self.ceiling,
            t => t.min(self.ceiling),
        };
        if want != self.bitrate {
            if let Some(api) = &self.api {
                mf::set_codec_value(api, &CODECAPI_AVEncCommonMeanBitRate, mf::variant_u32(want));
                mf::set_codec_value(api, &CODECAPI_AVEncCommonBufferSize, mf::variant_u32(vbv_bytes(want)));
            }
            self.bitrate = want;
        }
        if self.force_key.swap(false, Ordering::Relaxed) {
            if let Some(api) = &self.api {
                mf::set_codec_value(api, &CODECAPI_AVEncVideoForceKeyFrame, mf::variant_u32(1));
            }
        }
        let sample = self.make_sample(f)?;
        match unsafe { self.t.ProcessInput(self.input_id, &sample, 0) } {
            Ok(()) => Ok(()),
            Err(e) if e.code() == MF_E_NOTACCEPTING => {
                self.drain(false);
                unsafe { self.t.ProcessInput(self.input_id, &sample, 0) }.map_err(Into::into)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// `one`: MFT assíncrono (todo encoder de GPU) aceita exatamente um `ProcessOutput` por
    /// `METransformHaveOutput`; chamar de novo devolve E_UNEXPECTED (0x8000FFFF) a cada quadro.
    fn drain(&mut self, one: bool) {
        loop {
            let out = mf::process_output(&self.t, &mut self.spare);
            match out {
                Ok(mf::OutputResult::Sample(s)) => {
                    if let Err(e) = self.emit(&s) {
                        warn!("encoder: {e:#}");
                    }
                    if one {
                        break;
                    }
                }
                Ok(mf::OutputResult::NeedMoreInput) => break,
                Ok(mf::OutputResult::StreamChange) => {
                    debug!("encoder: mudança de formato na saída ignorada");
                    break;
                }
                Err(e) => {
                    warn!("encoder: process_output: {e:#}");
                    break;
                }
            }
        }
    }

    fn emit(&mut self, sample: &IMFSample) -> Result<()> {
        let key = unsafe { sample.GetUINT32(&MFSampleExtension_CleanPoint) }
            .map(|v| v == 1)
            .unwrap_or(false);
        let time = unsafe { sample.GetSampleTime() }.unwrap_or(0);
        let mut data = mf::sample_bytes(sample)?;
        if key && !mf::has_param_sets(self.codec, &data) && !self.seq_header.is_empty() {
            let should_prepend = match self.codec {
                Codec::Av1 => {
                    if (self.seq_header[0] >> 3) & 0xF == 1 {
                        true
                    } else {
                        static WARNED: AtomicBool = AtomicBool::new(false);
                        if !WARNED.swap(true, Ordering::Relaxed) {
                            warn!("AV1 seq_header sem OBU de sequência");
                        }
                        false
                    }
                }
                _ => true,
            };
            if should_prepend {
                data = mf::prepend_sequence_header(&self.seq_header, &data);
            }
        }
        let frame = EncodedFrame {
            seq: self.seq,
            keyframe: key,
            pts_us: (time / 10).max(0) as u64,
            codec: self.codec,
            data: Bytes::from(data),
        };
        self.seq += 1;
        self.stats.frames.fetch_add(1, Ordering::Relaxed);
        self.stats
            .bytes
            .fetch_add(frame.data.len() as u64, Ordering::Relaxed);
        let _ = self.out.send(Arc::new(frame));
        Ok(())
    }
}

fn create_mft(kind: &EncoderKind, device: &ID3D11Device) -> Result<(IMFTransform, bool)> {
    match kind {
        EncoderKind::Hardware { name, codec } => {
            let info = MFT_REGISTER_TYPE_INFO {
                guidMajorType: MFMediaType_Video,
                guidSubtype: mf::subtype(*codec),
            };
            let flags = MFT_ENUM_FLAG(
                MFT_ENUM_FLAG_HARDWARE.0
                    | MFT_ENUM_FLAG_TRANSCODE_ONLY.0
                    | MFT_ENUM_FLAG_SORTANDFILTER.0,
            );
            let act = mf::enumerate_mfts(MFT_CATEGORY_VIDEO_ENCODER, flags, None, Some(info))
                .into_iter()
                .find(|a| mf::friendly_name(a) == *name)
                .ok_or_else(|| anyhow!("encoder de hardware não encontrado: {name}"))?;
            let t: IMFTransform = unsafe { act.ActivateObject() }?;
            let is_async = mf::is_async(&t);
            unsafe {
                if is_async {
                    if let Ok(attrs) = t.GetAttributes() {
                        let _ = attrs.SetUINT32(&MF_TRANSFORM_ASYNC_UNLOCK, 1);
                    }
                }
                let mut token = 0u32;
                let mut mgr: Option<IMFDXGIDeviceManager> = None;
                MFCreateDXGIDeviceManager(&mut token, &mut mgr)?;
                let mgr = mgr.ok_or_else(|| anyhow!("MFCreateDXGIDeviceManager sem manager"))?;
                mgr.ResetDevice(device, token)?;
                t.ProcessMessage(MFT_MESSAGE_SET_D3D_MANAGER, mgr.as_raw() as usize)?;
            }
            info!("encoder de hardware: {name}");
            Ok((t, is_async))
        }
        EncoderKind::Software => {
            let t: IMFTransform =
                unsafe { CoCreateInstance(&CLSID_MSH264EncoderMFT, None, CLSCTX_INPROC_SERVER)? };
            let is_async = mf::is_async(&t);
            if is_async {
                unsafe {
                    if let Ok(attrs) = t.GetAttributes() {
                        let _ = attrs.SetUINT32(&MF_TRANSFORM_ASYNC_UNLOCK, 1);
                    }
                }
            }
            Ok((t, is_async))
        }
    }
}

fn stream_ids(t: &IMFTransform) -> (u32, u32) {
    let mut i = [0u32; 1];
    let mut o = [0u32; 1];
    // E_NOTIMPL (MFT com ids fixos) → 0/0
    match unsafe { t.GetStreamIDs(&mut i, &mut o) } {
        Ok(()) => (i[0], o[0]),
        Err(_) => (0, 0),
    }
}

fn configure(t: &IMFTransform, params: &EncodeParams, codec: Codec) -> Result<Option<ICodecAPI>> {
    unsafe {
        let api = t.cast::<ICodecAPI>().ok();
        if let Some(api) = &api {
            mf::set_codec_value(
                api,
                &CODECAPI_AVEncMPVDefaultBPictureCount,
                mf::variant_u32(0),
            );
            mf::set_codec_value(api, &CODECAPI_AVLowLatencyMode, mf::variant_bool(true));
        }

        let out = MFCreateMediaType()?;
        out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        out.SetGUID(&MF_MT_SUBTYPE, &mf::subtype(codec))?;
        out.SetUINT32(&MF_MT_AVG_BITRATE, params.bitrate_bps)?;
        out.SetUINT64(
            &MF_MT_FRAME_SIZE,
            mf::pack_pair(params.width, params.height),
        )?;
        out.SetUINT64(&MF_MT_FRAME_RATE, mf::pack_pair(params.fps, 1))?;
        out.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, mf::pack_pair(1, 1))?;
        out.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
        let profile = match codec {
            Codec::H264 => eAVEncH264VProfile_High.0 as u32,
            Codec::Hevc => eAVEncH265VProfile_Main_420_8.0 as u32,
            Codec::Av1 => eAVEncAV1VProfile_Main_420_8.0 as u32,
        };
        out.SetUINT32(&MF_MT_MPEG2_PROFILE, profile)?;
        t.SetOutputType(0, &out, 0)?;

        let mut i = 0;
        loop {
            let mt = t
                .GetInputAvailableType(0, i)
                .map_err(|_| anyhow!("sem tipo de entrada NV12"))?;
            let _ = mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video);
            let _ = mt.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12);
            let _ = mt.SetUINT64(
                &MF_MT_FRAME_SIZE,
                mf::pack_pair(params.width, params.height),
            );
            let _ = mt.SetUINT64(&MF_MT_FRAME_RATE, mf::pack_pair(params.fps, 1));
            if t.SetInputType(0, &mt, 0).is_ok() {
                break;
            }
            i += 1;
            if i > 64 {
                bail!("nenhum tipo de entrada NV12 aceito");
            }
        }

        if let Some(api) = &api {
            let vbr = mf::set_codec_value(
                api,
                &CODECAPI_AVEncCommonRateControlMode,
                mf::variant_u32(eAVEncCommonRateControlMode_LowDelayVBR.0 as u32),
            );
            if !vbr {
                mf::set_codec_value(
                    api,
                    &CODECAPI_AVEncCommonRateControlMode,
                    mf::variant_u32(eAVEncCommonRateControlMode_CBR.0 as u32),
                );
            }
            info!("encoder: controle de taxa {}", if vbr { "LowDelayVBR" } else { "CBR" });
            mf::set_codec_value(
                api,
                &CODECAPI_AVEncCommonMeanBitRate,
                mf::variant_u32(params.bitrate_bps),
            );
            mf::set_codec_value(
                api,
                &CODECAPI_AVEncCommonBufferSize,
                mf::variant_u32(vbv_bytes(params.bitrate_bps)),
            );
            mf::set_codec_value(
                api,
                &CODECAPI_AVEncMPVGOPSize,
                mf::variant_u32(params.fps * 30),
            );
            mf::set_codec_value(api, &CODECAPI_AVLowLatencyMode, mf::variant_bool(true));
        }

        let _ = t.ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0);
        let _ = t.ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0);
        let _ = t.ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0);
        Ok(api)
    }
}

fn read_seq_header(t: &IMFTransform) -> Vec<u8> {
    unsafe {
        let Ok(ty) = t.GetOutputCurrentType(0) else {
            return Vec::new();
        };
        let Ok(n) = ty.GetBlobSize(&MF_MT_MPEG_SEQUENCE_HEADER) else {
            return Vec::new();
        };
        if n == 0 {
            return Vec::new();
        }
        let mut v = vec![0u8; n as usize];
        let mut got = 0u32;
        if ty
            .GetBlob(&MF_MT_MPEG_SEQUENCE_HEADER, &mut v, Some(&mut got))
            .is_ok()
        {
            v.truncate((got as usize).min(v.len()));
            v
        } else {
            Vec::new()
        }
    }
}
