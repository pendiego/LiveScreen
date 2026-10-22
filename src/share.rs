use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use bytes::Bytes;
use iroh::Endpoint;
use iroh::endpoint::{Connection, presets};
use iroh_tickets::endpoint::EndpointTicket;
use tokio::runtime::Handle;
use tokio::sync::{Semaphore, broadcast, watch};
use tracing::{info, warn};

use crate::audio::capture::{AudioHandle, AudioTarget};
use crate::capture::{self, Backend, CaptureHandle, FrameSink, SourceId, SourceInfo};
use crate::config::Overrides;
use crate::preset::{Preset, encode_params};
use crate::protocol::*;
use crate::video::convert::Converter;
use crate::video::encoder::{self, EncoderHandle, EncoderKind, EncoderStats, Nv12Frame};

/// Handshakes em andamento aceitos ao mesmo tempo; o excedente é recusado. Só limita a
/// enxurrada de conexões: quem já está assistindo não ocupa vaga.
const MAX_HANDSHAKES: usize = 16;
/// Teto de espera por um espectador que não lê o "welcome"/não abre o stream.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub struct ViewerInfo {
    pub id_short: String,
    pub name: String,
    pub path: String,
    pub rtt_ms: u32,
}

impl ViewerInfo {
    pub fn label(&self) -> String {
        if self.name.is_empty() {
            format!("Anônimo #{}", &self.id_short[..4.min(self.id_short.len())])
        } else {
            self.name.clone()
        }
    }
}

/// Remove caracteres de controle e limita o texto a `max_chars` caracteres.
pub fn sanitize(raw: &str, max_chars: usize) -> String {
    raw.chars()
        .filter(|c| !c.is_control())
        .take(max_chars)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Convite curto: só o ID e os relays; o iroh tenta a conexão direta depois.
/// Sem relay (offline), mantém os IPs para a conexão direta funcionar.
fn invite_addr(addr: iroh::EndpointAddr) -> iroh::EndpointAddr {
    let relays: Vec<_> = addr.relay_urls().cloned().collect();
    if relays.is_empty() {
        return addr;
    }
    relays
        .into_iter()
        .fold(iroh::EndpointAddr::new(addr.id), |a, r| a.with_relay_url(r))
}

#[derive(Clone, Debug)]
pub enum AudioState {
    Off,
    On,
    Unavailable(String),
}

#[derive(Clone, Debug)]
pub struct ShareStatus {
    pub ticket: Option<String>,
    pub online_warning: bool,
    pub backend: Backend,
    pub encoder: String,
    pub viewers: Vec<ViewerInfo>,
    pub fps: f32,
    pub kbps: u32,
    pub audio: AudioState,
    pub ended: Option<String>,
    pub width: u32,
    pub height: u32,
    pub target_fps: u32,
    /// O encoder está perdendo quadros de forma contínua (PC sem fôlego para a qualidade).
    pub overloaded: bool,
}

/// Liga com 3 s seguidos perdendo ≥10% dos quadros e desliga com 5 s limpos, para não piscar.
#[derive(Default)]
struct Overload {
    bad: u32,
    good: u32,
    on: bool,
}

impl Overload {
    fn update(&mut self, dropped_last_sec: u64, target_fps: u32) -> bool {
        if dropped_last_sec >= (target_fps as u64 / 10).max(2) {
            (self.bad, self.good) = (self.bad + 1, 0);
        } else {
            (self.bad, self.good) = (0, self.good + 1);
        }
        if self.bad >= 3 {
            self.on = true;
        } else if self.good >= 5 {
            self.on = false;
        }
        self.on
    }
}

type SinkTx = Arc<Mutex<Option<std::sync::mpsc::SyncSender<Nv12Frame>>>>;

struct Resources {
    capture: CaptureHandle,
    audio: Option<AudioHandle>,
    encoder: Option<EncoderHandle>,
    sink_tx: SinkTx,
    pending: PendingReconf,
    device: windows::Win32::Graphics::Direct3D11::ID3D11Device,
    context: windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
    kind: EncoderKind,
    src_size: Arc<AtomicU64>,
    /// Hz do monitor da fonte, lido no início (a "Nativa (monitor)" segue ele).
    hz: u32,
    stats: Arc<EncoderStats>,
    preset: Preset,
    params: crate::preset::EncodeParams,
    /// Tamanho da fonte quando `params` foi calculado.
    applied_src: (u32, u32),
    ep: Endpoint,
    shutdown: watch::Sender<bool>,
}

pub struct ShareSession {
    rt: Handle,
    pub status: Arc<Mutex<ShareStatus>>,
    shared: Arc<Shared>,
    res: Mutex<Option<Resources>>,
}

struct Shared {
    status: Arc<Mutex<ShareStatus>>,
    viewers: AtomicUsize,
    conns: Mutex<HashMap<String, (Connection, String)>>,
    /// Teto de handshakes simultâneos; sem ele, uma enxurrada de conexões vira enxurrada de tasks.
    pending: Arc<Semaphore>,
    /// Desempata ids de espectadores que compartilham o mesmo prefixo.
    next_id: AtomicUsize,
    /// Limite da sala; ajustável durante a transmissão.
    max_viewers: AtomicUsize,
    video_tx: broadcast::Sender<Arc<EncodedFrame>>,
    audio_tx: broadcast::Sender<Bytes>,
    force_key: Arc<AtomicBool>,
    last_key: Mutex<Instant>,
    params: watch::Sender<(u32, u32, u32, bool)>,
    title: String,
    shutdown: watch::Receiver<bool>,
}

impl Shared {
    fn request_keyframe(&self) {
        let mut l = self.last_key.lock().unwrap();
        if l.elapsed() >= Duration::from_secs(1) {
            *l = Instant::now();
            self.force_key.store(true, Ordering::SeqCst);
        }
    }
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
    /// Quadros jogados fora com a fila do encoder cheia: o encoder não está dando conta.
    dropped: Arc<AtomicU64>,
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
            self.dropped.fetch_add(1, Ordering::Relaxed);
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
    params: crate::preset::EncodeParams,
    out: broadcast::Sender<Arc<EncodedFrame>>,
    force_key: Arc<AtomicBool>,
    stats: Arc<EncoderStats>,
) -> Result<(
    EncoderHandle,
    std::sync::mpsc::SyncSender<Nv12Frame>,
    String,
)> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<Nv12Frame>(2);
    match encoder::spawn(
        kind.clone(),
        device,
        params,
        rx,
        out.clone(),
        force_key.clone(),
        stats.clone(),
    ) {
        Ok(h) => {
            let name = match &kind {
                EncoderKind::Hardware { name } => name.clone(),
                EncoderKind::Software => "software (Microsoft H.264)".into(),
            };
            Ok((h, tx, name))
        }
        Err(e) if matches!(kind, EncoderKind::Hardware { .. }) => {
            warn!("encoder de hardware falhou: {e:#}; fallback para software");
            let (tx, rx) = std::sync::mpsc::sync_channel::<Nv12Frame>(2);
            let h = encoder::spawn(
                EncoderKind::Software,
                device,
                params,
                rx,
                out,
                force_key,
                stats,
            )?;
            Ok((h, tx, "software (fallback para encoder software)".into()))
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
        encoder_kind: EncoderKind,
        overrides: Overrides,
        max_viewers: usize,
    ) -> Result<ShareSession> {
        let (device, context) = crate::gpu::create_device()?;
        let (sw, sh) = capture::source_size(&source.id)?;
        let hz = capture::refresh_rate(&source.id);
        let params = encode_params(preset, sw, sh, hz);
        let conv = Converter::new(&device, &context, params.width, params.height, params.fps)?;

        let (video_tx, _) = broadcast::channel::<Arc<EncodedFrame>>(16);
        let (audio_tx, _) = broadcast::channel::<Bytes>(64);
        let force_key = Arc::new(AtomicBool::new(true));
        let stats = Arc::new(EncoderStats::default());

        let (enc, tx, enc_name) = start_encoder(
            encoder_kind.clone(),
            &device,
            params,
            video_tx.clone(),
            force_key.clone(),
            stats.clone(),
        )?;
        let sink_tx: SinkTx = Arc::new(Mutex::new(Some(tx)));

        let status = Arc::new(Mutex::new(ShareStatus {
            ticket: None,
            online_warning: false,
            backend: Backend::GraphicsCapture,
            encoder: enc_name,
            viewers: vec![],
            fps: 0.0,
            kbps: 0,
            audio: if audio {
                AudioState::Unavailable("iniciando".into())
            } else {
                AudioState::Off
            },
            ended: None,
            width: params.width,
            height: params.height,
            target_fps: params.fps,
            overloaded: false,
        }));

        let src_size = Arc::new(AtomicU64::new(pack_size(sw, sh)));
        let dropped = Arc::new(AtomicU64::new(0));
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
            dropped: dropped.clone(),
        };
        let (cap, backend) = match capture::start(&source.id, &overrides, &device, Box::new(sink)) {
            Ok(v) => v,
            Err(e) => {
                sink_tx.lock().unwrap().take();
                enc.stop();
                return Err(e);
            }
        };
        status.lock().unwrap().backend = backend;

        let audio_h = if audio {
            let target = match &source.id {
                SourceId::Window { pid, .. } => AudioTarget::Process(*pid),
                SourceId::Monitor { .. } => AudioTarget::System,
            };
            match crate::audio::capture::start(target, audio_tx.clone()) {
                Ok(h) => {
                    status.lock().unwrap().audio = AudioState::On;
                    Some(h)
                }
                Err(e) => {
                    warn!("áudio indisponível: {e:#}");
                    status.lock().unwrap().audio = AudioState::Unavailable(format!("{e}"));
                    None
                }
            }
        } else {
            None
        };

        let ep = rt.block_on(async {
            Endpoint::builder(presets::N0)
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await
        })?;
        let (sd_tx, sd_rx) = watch::channel(false);
        let shared = Arc::new(Shared {
            status: status.clone(),
            viewers: AtomicUsize::new(0),
            conns: Mutex::new(HashMap::new()),
            pending: Arc::new(Semaphore::new(MAX_HANDSHAKES)),
            next_id: AtomicUsize::new(0),
            max_viewers: AtomicUsize::new(max_viewers.max(1)),
            video_tx,
            audio_tx,
            force_key,
            last_key: Mutex::new(Instant::now() - Duration::from_secs(2)),
            params: watch::channel((params.width, params.height, params.fps, audio_h.is_some())).0,
            title: source.title.clone(),
            shutdown: sd_rx,
        });

        {
            let (ep, shared, status) = (ep.clone(), shared.clone(), status.clone());
            rt.spawn(async move {
                if tokio::time::timeout(Duration::from_secs(15), ep.online())
                    .await
                    .is_err()
                {
                    status.lock().unwrap().online_warning = true;
                }
                let ticket = EndpointTicket::new(invite_addr(ep.addr())).to_string();
                status.lock().unwrap().ticket = Some(ticket);
                while let Some(incoming) = ep.accept().await {
                    // Sem vaga de handshake, recusa na hora: sem task por conexão pendurada.
                    let Ok(permit) = shared.pending.clone().try_acquire_owned() else {
                        incoming.refuse();
                        continue;
                    };
                    let shared = shared.clone();
                    tokio::spawn(async move {
                        match incoming.await {
                            Ok(conn) => {
                                if let Err(e) = handle_viewer(conn, shared, permit).await {
                                    info!("espectador encerrado: {e:#}");
                                }
                            }
                            Err(e) => info!("conexão recusada: {e:#}"),
                        }
                    });
                }
            });
        }
        {
            let (shared, stats, dropped) = (shared.clone(), stats.clone(), dropped.clone());
            rt.spawn(async move {
                let (mut pf, mut pb, mut pd) = (0u64, 0u64, 0u64);
                let mut overload = Overload::default();
                let mut tick = tokio::time::interval(Duration::from_secs(1));
                loop {
                    tick.tick().await;
                    if *shared.shutdown.borrow() {
                        break;
                    }
                    let f = stats.frames.load(Ordering::Relaxed);
                    let b = stats.bytes.load(Ordering::Relaxed);
                    let viewers = shared
                        .conns
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|(id, (c, name))| {
                            let (mut path, mut rtt) = ("?".to_string(), 0u32);
                            for p in c.paths().iter() {
                                if p.is_selected() {
                                    path = if p.is_relay() {
                                        "Relay"
                                    } else if p.is_ip() {
                                        "Direto"
                                    } else {
                                        "?"
                                    }
                                    .into();
                                    rtt = p.rtt().as_millis() as u32;
                                }
                            }
                            ViewerInfo {
                                id_short: id.clone(),
                                name: name.clone(),
                                path,
                                rtt_ms: rtt,
                            }
                        })
                        .collect();
                    let d = dropped.load(Ordering::Relaxed);
                    let mut s = shared.status.lock().unwrap();
                    s.viewers = viewers;
                    s.fps = (f - pf) as f32;
                    s.kbps = ((b - pb) * 8 / 1000) as u32;
                    s.overloaded = overload.update(d - pd, s.target_fps);
                    pf = f;
                    pb = b;
                    pd = d;
                }
            });
        }

        if let SourceId::Window { hwnd, .. } = source.id {
            // WGC nem sempre dispara item.Closed; confirma por polling.
            let (status, mut sd) = (status.clone(), shared.shutdown.clone());
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
            rt: rt.clone(),
            status,
            shared,
            res: Mutex::new(Some(Resources {
                capture: cap,
                audio: audio_h,
                encoder: Some(enc),
                sink_tx,
                pending,
                device,
                context,
                kind: encoder_kind,
                src_size,
                hz,
                stats,
                preset,
                params,
                applied_src: (sw, sh),
                ep,
                shutdown: sd_tx,
            })),
        })
    }

    /// O que o encoder está usando agora (opção, parâmetros e tamanho da fonte quando foi
    /// configurado). A janela pode ter mudado de tamanho depois: capacidade e rótulo da
    /// transmissão atual vêm daqui, não das opções calculadas para o tamanho novo.
    pub fn current(&self) -> (Preset, crate::preset::EncodeParams, (u32, u32)) {
        let g = self.res.lock().unwrap();
        g.as_ref().map_or(
            (Preset::default(), encode_params(Preset::default(), 1920, 1080, 60), (1920, 1080)),
            |r| (r.preset, r.params, r.applied_src),
        )
    }

    /// Muda o limite da sala na hora; quem já está assistindo continua.
    pub fn set_max_viewers(&self, n: usize) {
        self.shared.max_viewers.store(n.max(1), Ordering::SeqCst);
    }

    /// Tamanho atual da fonte (último quadro capturado) e Hz do monitor dela: base das opções
    /// de qualidade e da recomendação de pessoas.
    pub fn source_shape(&self) -> (u32, u32, u32) {
        let g = self.res.lock().unwrap();
        g.as_ref().map_or((1920, 1080, 60), |r| {
            let (w, h) = unpack_size(r.src_size.load(Ordering::Relaxed));
            (w, h, r.hz)
        })
    }

    /// Troca a qualidade com a transmissão no ar. Bloqueia a thread chamadora (nunca chamar de
    /// dentro do runtime). Se o encoder novo falhar, volta ao anterior e devolve o erro.
    pub fn set_preset(&self, preset: Preset) -> Result<()> {
        let mut g = self.res.lock().unwrap();
        let r = g.as_mut().ok_or_else(|| anyhow!("transmissão encerrada"))?;
        // O retângulo da janela minimizada é minúsculo; o último quadro capturado não.
        let (sw, sh) = unpack_size(r.src_size.load(Ordering::Relaxed));
        let params = encode_params(preset, sw, sh, r.hz);
        // mesma opção vale de novo se a fonte mudou de tamanho desde a última configuração
        if r.preset == preset && r.params == params {
            return Ok(());
        }
        let conv = Converter::new(&r.device, &r.context, params.width, params.height, params.fps)?;
        // Corta a alimentação e espera o encoder antigo esvaziar: o novo só publica depois,
        // então nenhum quadro da resolução antiga aparece após o novo keyframe.
        cut_feed(&r.pending, &r.sink_tx);
        if let Some(e) = r.encoder.take() {
            e.stop();
        }
        let start = |p| {
            start_encoder(
                r.kind.clone(),
                &r.device,
                p,
                self.shared.video_tx.clone(),
                self.shared.force_key.clone(),
                r.stats.clone(),
            )
        };
        let (enc, tx, name, applied, applied_preset, failed) = match start(params) {
            Ok((h, tx, n)) => (h, tx, n, params, preset, None),
            Err(e) => match start(r.params) {
                Ok((h, tx, n)) => (h, tx, n, r.params, r.preset, Some(e)),
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
        // conversor e fila de entrada trocam juntos, na thread de captura
        let new_conv = failed
            .is_none()
            .then(|| (conv, Duration::from_secs_f64(1.0 / applied.fps as f64)));
        *r.pending.lock().unwrap() = Some(Reconf { conv: new_conv, tx });
        r.encoder = Some(enc);
        r.params = applied;
        r.preset = applied_preset;
        if failed.is_none() {
            r.applied_src = (sw, sh);
        }
        self.shared.force_key.store(true, Ordering::SeqCst);
        {
            let mut s = self.status.lock().unwrap();
            s.encoder = name;
            s.width = applied.width;
            s.height = applied.height;
            s.target_fps = applied.fps;
        }
        self.shared.params.send_replace((
            applied.width,
            applied.height,
            applied.fps,
            r.audio.is_some(),
        ));
        failed.map_or(Ok(()), Err)
    }

    pub fn kick(&self, id_short: &str) {
        // Tira da lista de status na hora: sem isso o tick ainda listaria o espectador
        // por até 1 s e a UI repetiria o aviso "entrou". A entrada do mapa fica até a
        // task do espectador sair, para o id não ser reaproveitado nesse meio-tempo.
        let conn = self
            .shared
            .conns
            .lock()
            .unwrap()
            .get(id_short)
            .map(|(c, _)| c.clone());
        let Some(c) = conn else { return };
        self.shared
            .status
            .lock()
            .unwrap()
            .viewers
            .retain(|v| v.id_short != id_short);
        c.close(CLOSE_KICKED.into(), b"kicked");
    }

    /// Idempotente. Bloqueia a thread chamadora (nunca chamar de dentro do runtime).
    pub fn stop(&self) {
        let Some(r) = self.res.lock().unwrap().take() else {
            return;
        };
        let _ = r.shutdown.send(true);
        let shared = self.shared.clone();
        self.rt.block_on(async {
            let deadline = Instant::now() + Duration::from_millis(1500);
            while shared.viewers.load(Ordering::SeqCst) > 0 && Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            let _ = tokio::time::timeout(Duration::from_secs(2), r.ep.close()).await;
        });
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

struct CountGuard<'a>(&'a Shared, Option<String>);
impl Drop for CountGuard<'_> {
    fn drop(&mut self) {
        self.0.viewers.fetch_sub(1, Ordering::SeqCst);
        if let Some(id) = &self.1 {
            self.0.conns.lock().unwrap().remove(id);
            self.0
                .status
                .lock()
                .unwrap()
                .viewers
                .retain(|v| &v.id_short != id);
        }
    }
}

async fn handle_viewer(
    conn: Connection,
    sh: Arc<Shared>,
    handshake: tokio::sync::OwnedSemaphorePermit,
) -> Result<()> {
    let (mut send, mut recv) = tokio::time::timeout(Duration::from_secs(10), conn.accept_bi())
        .await
        .map_err(|_| anyhow!("timeout aguardando controle"))??;
    let hello: ViewerMsg = tokio::time::timeout(Duration::from_secs(10), read_msg(&mut recv))
        .await
        .map_err(|_| anyhow!("timeout hello"))??;
    let ViewerMsg::Hello { protocol, name, .. } = hello else {
        conn.close(CLOSE_PROTOCOL.into(), b"protocol");
        return Ok(());
    };
    if protocol != PROTOCOL_VERSION {
        let _ = write_msg(
            &mut send,
            &HostMsg::Rejected {
                reason: "versão incompatível".into(),
            },
        )
        .await;
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_millis(500), conn.closed()).await;
        conn.close(CLOSE_PROTOCOL.into(), b"protocol");
        return Ok(());
    }
    // limite da sala definido por quem transmite; baixar o limite não derruba quem já está
    if sh.viewers.fetch_add(1, Ordering::SeqCst) >= sh.max_viewers.load(Ordering::SeqCst) {
        sh.viewers.fetch_sub(1, Ordering::SeqCst);
        let _ = write_msg(
            &mut send,
            &HostMsg::Rejected {
                reason: "sala cheia".into(),
            },
        )
        .await;
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_millis(500), conn.closed()).await;
        conn.close(CLOSE_FULL.into(), b"full");
        return Ok(());
    }
    let id = {
        // Prefixo curto é exibido na UI, mas precisa ser único no mapa: estende e,
        // se o mesmo peer aparecer duas vezes, desempata com um sufixo.
        let full = conn.remote_id().to_string();
        let total = full.chars().count();
        let conns = sh.conns.lock().unwrap();
        let mut n = 8usize.min(total.max(1));
        loop {
            let cand: String = full.chars().take(n).collect();
            if !conns.contains_key(&cand) {
                break cand;
            }
            if n >= total {
                break format!("{cand}{}", sh.next_id.fetch_add(1, Ordering::Relaxed));
            }
            n = (n + 4).min(total);
        }
    };
    let _guard = CountGuard(&sh, Some(id.clone()));
    sh.conns
        .lock()
        .unwrap()
        .insert(id.clone(), (conn.clone(), sanitize(&name, 32)));

    let mut prx = sh.params.subscribe();
    let (w, h, fps, audio) = *prx.borrow_and_update();
    tokio::time::timeout(
        HANDSHAKE_TIMEOUT,
        write_msg(
            &mut send,
            &HostMsg::Welcome {
                width: w,
                height: h,
                fps,
                audio,
                title: sh.title.clone(),
            },
        ),
    )
    .await
    .map_err(|_| anyhow!("timeout enviando welcome"))??;
    let mut vsend = tokio::time::timeout(HANDSHAKE_TIMEOUT, conn.open_uni())
        .await
        .map_err(|_| anyhow!("timeout abrindo vídeo"))??;
    // handshake concluído: a vaga volta para a próxima conexão; espectadores não têm teto
    drop(handshake);
    let mut vrx = sh.video_tx.subscribe();
    let mut arx = sh.audio_tx.subscribe();
    let mut shutdown = sh.shutdown.clone();

    // controle: pedidos de keyframe
    let sh2 = sh.clone();
    let ctl = tokio::spawn(async move {
        while let Ok(m) = read_msg::<ViewerMsg>(&mut recv).await {
            if matches!(m, ViewerMsg::RequestKeyframe) {
                sh2.request_keyframe();
            }
        }
    });

    let mut waiting_key = true;
    // Um espectador lento não deve pedir keyframe a cada quadro perdido (inflaria o
    // bitrate de todos); no máximo um pedido por segundo, por espectador.
    let mut last_kf = Instant::now() - Duration::from_secs(2);
    sh.request_keyframe();
    let res: Result<()> = loop {
        tokio::select! {
            r = vrx.recv() => match r {
                Ok(f) => {
                    if waiting_key && !f.keyframe {
                        if last_kf.elapsed() >= Duration::from_secs(1) {
                            last_kf = Instant::now();
                            sh.request_keyframe();
                        }
                        continue;
                    }
                    waiting_key = false;
                    if let Err(e) = vsend.write_all(&encode_video_frame(&f)).await { break Err(e.into()); }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => { waiting_key = true; }
                Err(broadcast::error::RecvError::Closed) => break Ok(()),
            },
            r = arx.recv() => if let Ok(pkt) = r { let _ = conn.send_datagram(pkt); },
            _ = prx.changed() => {
                let (width, height, fps, _) = *prx.borrow_and_update();
                if let Err(e) = write_msg(&mut send, &HostMsg::StreamParams { width, height, fps }).await { break Err(e); }
            }
            _ = conn.closed() => break Ok(()),
            _ = shutdown.changed() => {
                let _ = tokio::time::timeout(Duration::from_millis(500), write_msg(&mut send, &HostMsg::StreamEnded)).await;
                conn.close(CLOSE_STOPPED.into(), b"stopped");
                break Ok(());
            }
        }
    };
    ctl.abort();
    res
}

#[cfg(test)]
mod tests {
    use super::{Overload, invite_addr, sanitize};

    #[test]
    fn overload_needs_sustained_drops_and_clears_slowly() {
        let mut o = Overload::default();
        // 60 fps: limiar de 6 quadros perdidos por segundo
        assert!(!o.update(6, 60) && !o.update(6, 60));
        assert!(!o.update(5, 60), "rajada curta não acende");
        assert!(!o.update(9, 60) && !o.update(9, 60));
        assert!(o.update(9, 60), "3 s seguidos acendem");
        for _ in 0..4 {
            assert!(o.update(0, 60), "segue aceso até 5 s limpos");
        }
        assert!(!o.update(0, 60));
        // fps baixo ainda exige pelo menos 2 perdidos por segundo
        let mut o = Overload::default();
        for _ in 0..5 {
            assert!(!o.update(1, 15));
        }
    }

    #[test]
    fn sanitize_strips_controls_and_trims() {
        assert_eq!(sanitize("  Ana\u{7}Maria\n ", 32), "AnaMaria");
    }

    #[test]
    fn sanitize_limits_length() {
        assert_eq!(sanitize(&"x".repeat(40), 32).chars().count(), 32);
    }

    fn ids() -> (iroh::EndpointId, iroh::RelayUrl) {
        (
            iroh::SecretKey::from_bytes(&[7u8; 32]).public(),
            "https://relay.example./".parse().unwrap(),
        )
    }

    #[test]
    fn invite_addr_keeps_only_relay() {
        let (id, relay) = ids();
        let full = iroh::EndpointAddr::new(id)
            .with_relay_url(relay.clone())
            .with_ip_addr("1.2.3.4:5".parse().unwrap());
        assert_eq!(
            invite_addr(full),
            iroh::EndpointAddr::new(id).with_relay_url(relay)
        );
    }

    #[test]
    fn invite_addr_without_relay_keeps_ips() {
        let (id, _) = ids();
        let a = iroh::EndpointAddr::new(id).with_ip_addr("1.2.3.4:5".parse().unwrap());
        assert_eq!(invite_addr(a.clone()), a);
    }
}
