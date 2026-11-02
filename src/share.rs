use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tracing::warn;

use crate::audio::capture::{AudioHandle, AudioRequest, AudioTarget, Mic};
use crate::capture::{self, Backend, CaptureHandle, FrameSink, SourceId, SourceInfo};
use crate::config::Overrides;
use crate::video::convert::Converter;
use crate::video::encoder::{self, EncoderHandle, EncoderKind, Nv12Frame};
use livescreen_core::host::{AudioState, Feeds, Host, HostConfig, HostStats, ShareStatus, WebState};
use livescreen_core::preset::{EncodeParams, Preset, encode_params, is_reduction};
use livescreen_core::protocol::{Codec, EncodedFrame};

type SinkTx = Arc<Mutex<Option<std::sync::mpsc::SyncSender<Nv12Frame>>>>;

struct Resources {
    capture: CaptureHandle,
    audio: Option<AudioHandle>,
    encoder: Option<EncoderHandle>,
    sink_tx: SinkTx,
    pending: PendingReconf,
    device: windows::Win32::Graphics::Direct3D11::ID3D11Device,
    context: windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
    kinds: Vec<EncoderKind>,
    kind: EncoderKind,
    src_size: Arc<AtomicU64>,
    /// Hz do monitor da fonte, lido no início (a "Nativa (monitor)" segue ele).
    hz: u32,
    preset: Preset,
    params: EncodeParams,
    /// Tamanho da fonte quando `params` foi calculado.
    applied_src: (u32, u32),
    /// Encoder H.264 em perfil Baseline (algum navegador não decodifica High).
    baseline: bool,
}

pub struct ShareSession {
    pub status: Arc<Mutex<ShareStatus>>,
    pub host: Host,
    feeds: Feeds,
    res: Mutex<Option<Resources>>,
    mic_muted: Arc<AtomicBool>,
    /// Cópia de (preset, params, applied_src) de `Resources`, para a UI ler sem esperar uma troca de encoder.
    applied: Mutex<(Preset, EncodeParams, (u32, u32))>,
    /// Mesmo `Arc` de `Resources::src_size`.
    src_size: Arc<AtomicU64>,
    hz: u32,
}

fn pick_encoder(kinds: &[EncoderKind], viewers: &[Vec<Codec>]) -> Option<EncoderKind> {
    if viewers.is_empty() {
        return None;
    }
    for k in kinds {
        let codec = k.codec();
        let accepted_by_all = viewers
            .iter()
            .all(|v| codec == Codec::H264 || v.contains(&codec));
        if accepted_by_all {
            return Some(k.clone());
        }
    }
    None
}

struct Reconf {
    /// `None` ao restaurar o encoder anterior: o conversor atual continua valendo.
    conv: Option<(Converter, Duration)>,
    tx: std::sync::mpsc::SyncSender<Nv12Frame>,
}

type PendingReconf = Arc<Mutex<Option<Reconf>>>;

/// Solta todo sender do encoder (o instalado e o que ainda espera um quadro). Sem isso o
/// `join` do encoder trava quando a captura não entrega quadros (janela minimizada).
/// Ordem de locks igual à de `Sink::on_frame`: `pending` e depois `tx`.
fn cut_feed(pending: &PendingReconf, tx: &SinkTx) {
    let mut p = pending.lock().unwrap();
    p.take();
    tx.lock().unwrap().take();
}

struct Sink {
    conv: Converter,
    tx: SinkTx,
    pending: PendingReconf,
    /// Horário a partir do qual o próximo quadro vale.
    due: Instant,
    interval: Duration,
    start: Instant,
    status: Arc<Mutex<ShareStatus>>,
    /// Contadores do host: `dropped` conta quadros jogados fora com a fila do encoder cheia.
    stats: Arc<HostStats>,
    /// Tamanho do último quadro capturado; base de uma troca de qualidade.
    src_size: Arc<AtomicU64>,
}

fn pack_size(w: u32, h: u32) -> u64 {
    (w as u64) << 32 | h as u64
}

fn unpack_size(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, v as u32)
}

impl FrameSink for Sink {
    fn on_frame(
        &mut self,
        tex: &windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
        w: u32,
        h: u32,
    ) {
        if w > 0 && h > 0 {
            self.src_size.store(pack_size(w, h), Ordering::Relaxed);
        }
        {
            // Instala sob o lock de `pending`: `cut_feed` não pode correr no meio e deixar
            // um sender recolocado depois do corte.
            let mut p = self.pending.lock().unwrap();
            if let Some(r) = p.take() {
                if let Some((conv, interval)) = r.conv {
                    self.conv = conv;
                    self.interval = interval;
                }
                *self.tx.lock().unwrap() = Some(r.tx);
            }
        }
        let now = Instant::now();
        // agenda fixa: o próximo horário é o anterior + intervalo, não o quadro aceito +
        // intervalo; senão a captura a 144 Hz vira 48 fps num alvo de 60 (sempre o 3º quadro)
        if now + Duration::from_millis(2) < self.due {
            return;
        }
        self.due = if now.duration_since(self.due) > self.interval {
            now + self.interval // ficou para trás (tela parada): recomeça daqui
        } else {
            self.due + self.interval
        };
        let nv12 = match self.conv.convert(tex) {
            Ok(t) => t,
            Err(e) => {
                warn!("conversão falhou: {e}");
                return;
            }
        };
        let pts = now.duration_since(self.start);
        if let Some(tx) = self.tx.lock().unwrap().as_ref()
            && tx.try_send(Nv12Frame { tex: nv12, pts }).is_err()
        {
            self.stats.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn on_closed(&mut self) {
        self.status
            .lock()
            .unwrap()
            .ended
            .get_or_insert_with(|| "A janela foi fechada".into());
    }
    fn on_error(&mut self, e: anyhow::Error) {
        self.status
            .lock()
            .unwrap()
            .ended
            .get_or_insert_with(|| format!("Erro de captura: {e}"));
    }
}

fn start_encoder(
    kind: EncoderKind,
    device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
    params: EncodeParams,
    baseline: bool,
    out: broadcast::Sender<Arc<EncodedFrame>>,
    force_key: Arc<AtomicBool>,
    stats: Arc<HostStats>,
) -> Result<(
    EncoderHandle,
    std::sync::mpsc::SyncSender<Nv12Frame>,
    EncoderKind,
)> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<Nv12Frame>(2);
    match encoder::spawn(
        kind.clone(),
        device,
        params,
        baseline,
        rx,
        out.clone(),
        force_key.clone(),
        stats.clone(),
    ) {
        Ok(h) => Ok((h, tx, kind)),
        Err(e) if matches!(kind, EncoderKind::Hardware { .. }) => {
            warn!("encoder de hardware falhou: {e:#}; fallback para software");
            let (tx, rx) = std::sync::mpsc::sync_channel::<Nv12Frame>(2);
            let h = encoder::spawn(
                EncoderKind::Software,
                device,
                params,
                baseline,
                rx,
                out,
                force_key,
                stats,
            )?;
            Ok((h, tx, EncoderKind::Software))
        }
        Err(e) => Err(e),
    }
}

impl ShareSession {
    pub fn start(
        rt: &Handle,
        source: SourceInfo,
        preset: Preset,
        audio: bool,
        mic: Option<Mic>,
        encoders: Vec<EncoderKind>,
        overrides: Overrides,
        max_viewers: usize,
        identity: iroh::SecretKey,
        host_name: String,
        friends: HashSet<iroh::EndpointId>,
        donate: bool,
    ) -> Result<ShareSession> {
        let (device, context) = crate::gpu::create_device(None)?;
        let (sw, sh) = capture::source_size(&source.id)?;
        let hz = capture::refresh_rate(&source.id);
        let params = encode_params(preset, sw, sh, hz);
        let conv = Converter::new(&device, &context, params.width, params.height, params.fps)?;

        let feeds = Feeds::new();

        let initial_kind = encoders
            .iter()
            .find(|k| k.codec() == Codec::H264)
            .cloned()
            .unwrap_or(EncoderKind::Software);
        let (enc, tx, used_kind) = start_encoder(
            initial_kind,
            &device,
            params,
            false,
            feeds.video.clone(),
            feeds.force_key.clone(),
            feeds.stats.clone(),
        )?;
        let sink_tx: SinkTx = Arc::new(Mutex::new(Some(tx)));

        let status = Arc::new(Mutex::new(ShareStatus {
            ticket: None,
            online_warning: false,
            web: WebState::Off,
            web_vpn: None,
            backend: Backend::GraphicsCapture.label(),
            encoder: used_kind.label(),
            viewers: vec![],
            fps: 0.0,
            kbps: 0,
            audio: if audio {
                AudioState::Unavailable("iniciando".into())
            } else {
                AudioState::Off
            },
            mic: if mic.is_some() {
                AudioState::Unavailable("iniciando".into())
            } else {
                AudioState::Off
            },
            mic_muted: false,
            ended: None,
            width: params.width,
            height: params.height,
            target_fps: params.fps,
            overloaded: false,
        }));

        let src_size = Arc::new(AtomicU64::new(pack_size(sw, sh)));
        let pending: PendingReconf = Arc::new(Mutex::new(None));
        let sink = Sink {
            conv,
            tx: sink_tx.clone(),
            pending: pending.clone(),
            due: Instant::now(),
            interval: Duration::from_secs_f64(1.0 / params.fps as f64),
            start: Instant::now(),
            status: status.clone(),
            src_size: src_size.clone(),
            stats: feeds.stats.clone(),
        };
        let (cap, backend) = match capture::start(&source.id, &overrides, &device, Box::new(sink)) {
            Ok(v) => v,
            Err(e) => {
                sink_tx.lock().unwrap().take();
                enc.stop();
                return Err(e);
            }
        };
        status.lock().unwrap().backend = backend.label();

        let mic_muted = Arc::new(AtomicBool::new(false));
        let audio_h = if audio || mic.is_some() {
            let loopback = audio.then(|| match &source.id {
                SourceId::Window { pid, .. } => AudioTarget::Process(*pid),
                SourceId::Monitor { .. } => AudioTarget::System,
            });
            let req = AudioRequest {
                loopback,
                mic: mic.clone(),
                mic_muted: mic_muted.clone(),
            };
            let to_state = |r: Option<Result<(), String>>| match r {
                None => AudioState::Off,
                Some(Ok(())) => AudioState::On,
                Some(Err(m)) => AudioState::Unavailable(m),
            };
            match crate::audio::capture::start(req, feeds.audio.clone()) {
                Ok((h, opened)) => {
                    let mut s = status.lock().unwrap();
                    s.audio = to_state(opened.loopback);
                    s.mic = to_state(opened.mic);
                    Some(h)
                }
                Err(e) => {
                    warn!("áudio indisponível: {e:#}");
                    let mut s = status.lock().unwrap();
                    if audio {
                        s.audio = AudioState::Unavailable(format!("{e}"));
                    }
                    if mic.is_some() {
                        s.mic = AudioState::Unavailable(format!("{e}"));
                    }
                    None
                }
            }
        } else {
            None
        };

        let host = match Host::start(
            rt,
            HostConfig {
                identity,
                host_name,
                title: source.title.clone(),
                max_viewers,
                friends,
                width: params.width,
                height: params.height,
                fps: params.fps,
                audio: audio_h.is_some(),
                donate,
            },
            &feeds,
            status.clone(),
        ) {
            Ok(h) => h,
            Err(e) => {
                sink_tx.lock().unwrap().take();
                cap.stop();
                if let Some(a) = audio_h {
                    a.stop();
                }
                enc.stop();
                return Err(e);
            }
        };

        if let SourceId::Window { hwnd, .. } = source.id {
            // WGC nem sempre dispara item.Closed; confirma por polling.
            let (status, mut sd) = (status.clone(), host.shutdown_signal());
            rt.spawn(async move {
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                        _ = sd.changed() => return,
                    }
                    let alive = unsafe {
                        windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(
                            windows::Win32::Foundation::HWND(hwnd as *mut _),
                        ))
                    };
                    if !alive.as_bool() {
                        status
                            .lock()
                            .unwrap()
                            .ended
                            .get_or_insert_with(|| "A janela foi fechada".into());
                        return;
                    }
                }
            });
        }

        Ok(ShareSession {
            status,
            host,
            feeds,
            mic_muted,
            res: Mutex::new(Some(Resources {
                capture: cap,
                audio: audio_h,
                encoder: Some(enc),
                sink_tx,
                pending,
                device,
                context,
                kinds: encoders,
                kind: used_kind,
                src_size: src_size.clone(),
                hz,
                preset,
                params,
                applied_src: (sw, sh),
                baseline: false,
            })),
            applied: Mutex::new((preset, params, (sw, sh))),
            src_size,
            hz,
        })
    }

    /// O que o encoder está usando agora (opção, parâmetros e tamanho da fonte quando foi
    /// configurado). A janela pode ter mudado de tamanho depois: capacidade e rótulo da
    /// transmissão atual vêm daqui, não das opções calculadas para o tamanho novo.
    pub fn current(&self) -> (Preset, EncodeParams, (u32, u32)) {
        *self.applied.lock().unwrap()
    }

    pub fn set_mic_muted(&self, muted: bool) {
        self.mic_muted.store(muted, Ordering::Relaxed);
        self.status.lock().unwrap().mic_muted = muted;
    }

    /// Aplica um pedido de espectador: só aceita opções mais leves que a atual (um espectador
    /// não pode pesar a sala). Bloqueia como `set_preset`.
    pub fn apply_quality_request(&self, preset: Preset) -> Result<()> {
        let (sw, sh, hz) = self.source_shape();
        if is_reduction(self.current().1, encode_params(preset, sw, sh, hz)) {
            self.set_preset(preset)
        } else {
            Err(anyhow!("só é aceita uma qualidade mais leve que a atual"))
        }
    }

    /// Tamanho atual da fonte (último quadro capturado) e Hz do monitor dela: base das opções
    /// de qualidade e da recomendação de pessoas.
    pub fn source_shape(&self) -> (u32, u32, u32) {
        let (w, h) = unpack_size(self.src_size.load(Ordering::Relaxed));
        (w, h, self.hz)
    }

    /// Troca a qualidade com a transmissão no ar. Bloqueia a thread chamadora (nunca chamar de
    /// dentro do runtime). Se o encoder novo falhar, volta ao anterior e devolve o erro.
    fn reconfigure(&self, r: &mut Resources, preset: Preset, kind: EncoderKind, baseline: bool) -> Result<()> {
        let (sw, sh) = unpack_size(r.src_size.load(Ordering::Relaxed));
        let params = encode_params(preset, sw, sh, r.hz);
        let conv = Converter::new(
            &r.device,
            &r.context,
            params.width,
            params.height,
            params.fps,
        )?;
        cut_feed(&r.pending, &r.sink_tx);
        if let Some(e) = r.encoder.take() {
            e.stop();
        }
        let start = |k: &EncoderKind, p: livescreen_core::preset::EncodeParams, b: bool| {
            start_encoder(
                k.clone(),
                &r.device,
                p,
                b,
                self.feeds.video.clone(),
                self.feeds.force_key.clone(),
                self.feeds.stats.clone(),
            )
        };
        let (enc, tx, used_kind, applied, applied_preset, applied_baseline, failed) = match start(&kind, params, baseline) {
            Ok((h, tx, uk)) => (h, tx, uk, params, preset, baseline, None),
            Err(e) => match start(&r.kind, r.params, r.baseline) {
                Ok((h, tx, uk)) => (h, tx, uk, r.params, r.preset, r.baseline, Some(e)),
                Err(e2) => {
                    self.status
                        .lock()
                        .unwrap()
                        .ended
                        .get_or_insert_with(|| format!("Erro no encoder: {e2:#}"));
                    return Err(e2);
                }
            },
        };
        let new_conv = failed
            .is_none()
            .then(|| (conv, Duration::from_secs_f64(1.0 / applied.fps as f64)));
        *r.pending.lock().unwrap() = Some(Reconf { conv: new_conv, tx });
        r.encoder = Some(enc);
        r.kind = used_kind.clone();
        r.params = applied;
        r.preset = applied_preset;
        if failed.is_none() {
            r.applied_src = (sw, sh);
        }
        *self.applied.lock().unwrap() = (r.preset, r.params, r.applied_src);
        self.feeds.force_key.store(true, Ordering::SeqCst);
        r.baseline = applied_baseline;
        let mut label = used_kind.label();
        if applied_baseline && used_kind.codec() == Codec::H264 {
            label.push_str(" · H.264 Baseline (navegador)");
        }
        self.status.lock().unwrap().encoder = label;
        self.host
            .set_stream(applied.width, applied.height, applied.fps, r.audio.is_some());
        failed.map_or(Ok(()), Err)
    }

    pub fn set_preset(&self, preset: Preset) -> Result<()> {
        let mut g = self.res.lock().unwrap();
        let r = g.as_mut().ok_or_else(|| anyhow!("transmissão encerrada"))?;
        let (sw, sh) = unpack_size(r.src_size.load(Ordering::Relaxed));
        let params = encode_params(preset, sw, sh, r.hz);
        if r.preset == preset && r.params == params {
            return Ok(());
        }
        self.reconfigure(r, preset, r.kind.clone(), r.baseline)
    }

    pub fn apply_codec(&self) -> Option<Result<Codec>> {
        let viewers_decoders = self.host.take_viewer_decoders()?;
        let mut g = self.res.lock().unwrap();
        let r = g.as_mut()?;
        let want = pick_encoder(&r.kinds, &viewers_decoders)?;
        let baseline = self.host.needs_h264_baseline();
        if want == r.kind && baseline == r.baseline {
            return None;
        }
        let res = self.reconfigure(r, r.preset, want.clone(), baseline);
        if res.is_err() || r.kind != want {
            r.kinds.retain(|k| k != &want);
            let err = res.err().unwrap_or_else(|| anyhow!("encoder {} indisponível", want.label()));
            return Some(Err(err));
        }
        Some(Ok(want.codec()))
    }

    /// Idempotente. Bloqueia a thread chamadora (nunca chamar de dentro do runtime).
    pub fn stop(&self) {
        let Some(r) = self.res.lock().unwrap().take() else {
            return;
        };
        self.host.stop();
        r.capture.stop();
        if let Some(a) = r.audio {
            a.stop();
        }
        cut_feed(&r.pending, &r.sink_tx);
        if let Some(e) = r.encoder {
            e.stop();
        }
    }
}

impl Drop for ShareSession {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::encoder::EncoderKind;
    use livescreen_core::protocol::Codec;

    #[test]
    fn pick_encoder_common_denominator() {
        let hevc_hw = EncoderKind::Hardware {
            name: "HEVC HW".into(),
            codec: Codec::Hevc,
        };
        let h264_hw = EncoderKind::Hardware {
            name: "H264 HW".into(),
            codec: Codec::H264,
        };
        let av1 = EncoderKind::Hardware {
            name: "AV1 HW".into(),
            codec: Codec::Av1,
        };
        let sw = EncoderKind::Software;

        let kinds1 = [hevc_hw.clone(), h264_hw.clone(), sw.clone()];
        assert_eq!(
            pick_encoder(&kinds1, &[vec![Codec::H264, Codec::Hevc], vec![Codec::H264]]),
            Some(h264_hw.clone())
        );
        assert_eq!(
            pick_encoder(&kinds1, &[vec![Codec::Hevc]]),
            Some(hevc_hw.clone())
        );
        assert_eq!(
            pick_encoder(&kinds1, &[vec![Codec::Hevc], vec![]]),
            Some(h264_hw.clone())
        );
        assert_eq!(pick_encoder(&kinds1, &[]), None);

        let kinds2 = [av1.clone(), hevc_hw, sw];
        assert_eq!(
            pick_encoder(&kinds2, &[vec![Codec::H264, Codec::Av1, Codec::Hevc]]),
            Some(av1)
        );
    }

}
