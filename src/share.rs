use std::collections::{HashMap, HashSet};
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

use crate::audio::capture::{AudioHandle, AudioRequest, AudioTarget, Mic};
use crate::capture::{self, Backend, CaptureHandle, FrameSink, SourceId, SourceInfo};
use crate::config::Overrides;
use livescreen_core::preset::{EncodeParams, Preset, encode_params, is_reduction};
use livescreen_core::protocol::*;
use crate::video::convert::Converter;
use crate::video::encoder::{self, EncoderHandle, EncoderKind, EncoderStats, Nv12Frame};

/// Handshakes em andamento aceitos ao mesmo tempo; o excedente é recusado. Só limita a
/// enxurrada de conexões: quem já está assistindo não ocupa vaga.
const MAX_HANDSHAKES: usize = 16;
/// Teto de espera por um espectador que não lê o "welcome"/não abre o stream.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
/// Vídeo mais velho que isso não vale a fila: o próximo keyframe substitui.
const STALE_VIDEO: Duration = Duration::from_millis(500);
/// Piso do bitrate adaptativo (bps).
const ABR_FLOOR: u32 = 500_000;

/// AIMD: congestionou → 70% na hora; 3 s calmos seguidos → +8% por segundo até o teto do preset.
#[derive(Default)]
struct Abr {
    calm: u32,
}
impl Abr {
    fn next(&mut self, current: u32, ceiling: u32, congested: bool) -> u32 {
        let t = if congested {
            self.calm = 0;
            current as u64 * 7 / 10
        } else {
            self.calm += 1;
            if self.calm >= 3 { current as u64 * 108 / 100 } else { current as u64 }
        };
        t.clamp(ABR_FLOOR.min(ceiling) as u64, ceiling as u64) as u32
    }
}

/// GOP antigo: termina; se o peer não confirmar em STALE_VIDEO, descarta (para de retransmitir).
fn retire(mut s: iroh::endpoint::SendStream) {
    let _ = s.finish();
    tokio::spawn(async move {
        if tokio::time::timeout(STALE_VIDEO, s.stopped()).await.is_err() {
            let _ = s.reset(0u32.into());
        }
    });
}

#[derive(Clone, Debug)]
pub struct ViewerInfo {
    pub id: iroh::EndpointId,
    pub id_short: String,
    pub name: String,
    pub friend: bool,
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
    pub mic: AudioState,
    pub mic_muted: bool,
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
struct Peer {
    conn: Connection,
    name: String,
    decoders: Vec<Codec>,
}

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
    stats: Arc<EncoderStats>,
    preset: Preset,
    params: EncodeParams,
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
    mic_muted: Arc<AtomicBool>,
    /// Cópia de (preset, params, applied_src) de `Resources`, para a UI ler sem esperar uma troca de encoder.
    applied: Mutex<(Preset, EncodeParams, (u32, u32))>,
    /// Mesmo `Arc` de `Resources::src_size`.
    src_size: Arc<AtomicU64>,
    hz: u32,
}

struct Shared {
    status: Arc<Mutex<ShareStatus>>,
    viewers: AtomicUsize,
    conns: Mutex<HashMap<String, Peer>>,
    codec_dirty: AtomicBool,
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
    quality_req: Mutex<Option<(Preset, String)>>,
    last_quality: Mutex<Instant>,
    params: watch::Sender<(u32, u32, u32, bool)>,
    title: String,
    shutdown: watch::Receiver<bool>,
    /// Algum espectador ficou para trás desde o último tick.
    congested: AtomicBool,
    room_key: Mutex<[u8; ROOM_KEY_LEN]>,
    ticket: Mutex<Option<EndpointTicket>>,
    host_id: iroh::EndpointId,
    friends: Mutex<HashSet<iroh::EndpointId>>,
    banned: Mutex<HashSet<iroh::EndpointId>>,
    host_name: String,
}

impl Shared {
    fn request_keyframe(&self) {
        let mut l = self.last_key.lock().unwrap();
        if l.elapsed() >= Duration::from_secs(1) {
            *l = Instant::now();
            self.force_key.store(true, Ordering::SeqCst);
        }
    }

    /// Um pedido a cada 5 s no total: um espectador não pode ficar trocando o encoder da sala.
    fn request_quality(&self, preset: Preset, who: &str) {
        let mut l = self.last_quality.lock().unwrap();
        if l.elapsed() >= Duration::from_secs(5) {
            *l = Instant::now();
            *self.quality_req.lock().unwrap() = Some((preset, who.to_string()));
        }
    }

    fn new_invite(&self) {
        let key = new_room_key();
        *self.room_key.lock().unwrap() = key;
        if let Some(t) = self.ticket.lock().unwrap().as_ref() {
            let invite = format_invite(t, &key);
            self.status.lock().unwrap().ticket = Some(invite);
        }
    }
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
    params: EncodeParams,
    out: broadcast::Sender<Arc<EncodedFrame>>,
    force_key: Arc<AtomicBool>,
    stats: Arc<EncoderStats>,
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
    ) -> Result<ShareSession> {
        let (device, context) = crate::gpu::create_device(None)?;
        let (sw, sh) = capture::source_size(&source.id)?;
        let hz = capture::refresh_rate(&source.id);
        let params = encode_params(preset, sw, sh, hz);
        let conv = Converter::new(&device, &context, params.width, params.height, params.fps)?;

        let (video_tx, _) = broadcast::channel::<Arc<EncodedFrame>>(16);
        let (audio_tx, _) = broadcast::channel::<Bytes>(64);
        let force_key = Arc::new(AtomicBool::new(true));
        let stats = Arc::new(EncoderStats::default());

        let initial_kind = encoders
            .iter()
            .find(|k| k.codec() == Codec::H264)
            .cloned()
            .unwrap_or(EncoderKind::Software);
        let (enc, tx, used_kind) = start_encoder(
            initial_kind,
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
            match crate::audio::capture::start(req, audio_tx.clone()) {
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

        let ep = rt.block_on(async {
            // BBR: fila curta e tolerância a perda aleatória (Wi-Fi); o CUBIC padrão enche o buffer e aumenta a latência
            let transport = iroh::endpoint::QuicTransportConfig::builder()
                .congestion_controller_factory(Arc::new(noq_proto::congestion::Bbr3Config::default()))
                .max_concurrent_uni_streams(noq_proto::VarInt::from_u32(0))
                .max_concurrent_bidi_streams(noq_proto::VarInt::from_u32(1))
                .datagram_receive_buffer_size(None)
                .build();
            Endpoint::builder(presets::N0)
                .secret_key(identity)
                .alpns(vec![ALPN.to_vec(), livescreen_core::friends::PRESENCE_ALPN.to_vec()])
                .transport_config(transport)
                .bind()
                .await
        })?;
        let (sd_tx, sd_rx) = watch::channel(false);
        let shared = Arc::new(Shared {
            status: status.clone(),
            viewers: AtomicUsize::new(0),
            conns: Mutex::new(HashMap::new()),
            codec_dirty: AtomicBool::new(false),
            pending: Arc::new(Semaphore::new(MAX_HANDSHAKES)),
            next_id: AtomicUsize::new(0),
            max_viewers: AtomicUsize::new(max_viewers.max(1)),
            video_tx,
            audio_tx,
            force_key,
            last_key: Mutex::new(Instant::now() - Duration::from_secs(2)),
            quality_req: Mutex::new(None),
            last_quality: Mutex::new(Instant::now() - Duration::from_secs(10)),
            params: watch::channel((params.width, params.height, params.fps, audio_h.is_some())).0,
            title: source.title.clone(),
            shutdown: sd_rx,
            congested: AtomicBool::new(false),
            room_key: Mutex::new(new_room_key()),
            ticket: Mutex::new(None),
            host_id: ep.id(),
            friends: Mutex::new(friends),
            banned: Mutex::new(HashSet::new()),
            host_name,
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
                let ticket = EndpointTicket::new(invite_addr(ep.addr()));
                let key = *shared.room_key.lock().unwrap();
                *shared.ticket.lock().unwrap() = Some(ticket.clone());
                status.lock().unwrap().ticket = Some(format_invite(&ticket, &key));
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
                                if conn.alpn() == livescreen_core::friends::PRESENCE_ALPN {
                                    answer_presence(conn, shared).await;
                                } else if let Err(e) = handle_viewer(conn, shared, permit).await {
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
                let mut abr = Abr::default();
                let mut min_rtt: HashMap<String, (iroh::endpoint::PathId, Duration)> = HashMap::new();
                let mut tick = tokio::time::interval(Duration::from_secs(1));
                loop {
                    tick.tick().await;
                    if *shared.shutdown.borrow() {
                        break;
                    }
                    let f = stats.frames.load(Ordering::Relaxed);
                    let b = stats.bytes.load(Ordering::Relaxed);
                    let mut samples: Vec<(String, iroh::endpoint::PathId, Duration)> = Vec::new();
                    let friends_snap = shared.friends.lock().unwrap().clone();
                    let viewers = shared
                        .conns
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|(id, peer)| {
                            let (c, name) = (&peer.conn, &peer.name);
                            let peer_id = c.remote_id();
                            let friend = friends_snap.contains(&peer_id);
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
                                    samples.push((id.clone(), p.id(), p.rtt()));
                                }
                            }
                            ViewerInfo {
                                id: peer_id,
                                id_short: id.clone(),
                                name: name.clone(),
                                friend,
                                path,
                                rtt_ms: rtt,
                            }
                        })
                        .collect();
                    let mut congested = shared.congested.swap(false, Ordering::Relaxed);
                    for (id, pid, rtt) in &samples {
                        let e = min_rtt.entry(id.clone()).or_insert((*pid, *rtt));
                        // troca de caminho (relay ↔ direto) muda a base de RTT
                        if e.0 != *pid {
                            *e = (*pid, *rtt);
                        } else if *rtt < e.1 {
                            e.1 = *rtt;
                        }
                        // fila crescendo: RTT acima de 2× o mínimo e mais de 80 ms acima dele
                        if *rtt > e.1 + e.1.max(Duration::from_millis(80)) {
                            congested = true;
                        }
                    }
                    min_rtt.retain(|id, _| samples.iter().any(|(s, _, _)| s == id));
                    let ceiling = stats.ceiling_bps.load(Ordering::Relaxed);
                    if samples.is_empty() {
                        stats.target_bps.store(0, Ordering::Relaxed);
                        abr = Abr::default();
                    } else {
                        let cur = match stats.target_bps.load(Ordering::Relaxed) {
                            0 => ceiling,
                            t => t.min(ceiling),
                        };
                        let next = abr.next(cur, ceiling, congested);
                        if next != cur {
                            tracing::debug!("abr: {} → {} kbps", cur / 1000, next / 1000);
                        }
                        stats.target_bps.store(next, Ordering::Relaxed);
                    }
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
            mic_muted,
            shared,
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
                stats,
                preset,
                params,
                applied_src: (sw, sh),
                ep,
                shutdown: sd_tx,
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

    /// Muda o limite da sala na hora; quem já está assistindo continua.
    pub fn set_max_viewers(&self, n: usize) {
        self.shared.max_viewers.store(n.max(1), Ordering::SeqCst);
    }

    /// Pedido de qualidade pendente de um espectador (preset, nome de quem pediu). Não bloqueia.
    pub fn take_quality_request(&self) -> Option<(Preset, String)> {
        self.shared.quality_req.lock().unwrap().take()
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

    /// Algum espectador entrou ou saiu desde o último apply_codec. Não bloqueia.
    pub fn codec_pending(&self) -> bool {
        self.shared.codec_dirty.load(Ordering::Acquire)
    }

    /// Tamanho atual da fonte (último quadro capturado) e Hz do monitor dela: base das opções
    /// de qualidade e da recomendação de pessoas.
    pub fn source_shape(&self) -> (u32, u32, u32) {
        let (w, h) = unpack_size(self.src_size.load(Ordering::Relaxed));
        (w, h, self.hz)
    }

    /// Troca a qualidade com a transmissão no ar. Bloqueia a thread chamadora (nunca chamar de
    /// dentro do runtime). Se o encoder novo falhar, volta ao anterior e devolve o erro.
    fn reconfigure(&self, r: &mut Resources, preset: Preset, kind: EncoderKind) -> Result<()> {
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
        let start = |k: &EncoderKind, p: livescreen_core::preset::EncodeParams| {
            start_encoder(
                k.clone(),
                &r.device,
                p,
                self.shared.video_tx.clone(),
                self.shared.force_key.clone(),
                r.stats.clone(),
            )
        };
        let (enc, tx, used_kind, applied, applied_preset, failed) = match start(&kind, params) {
            Ok((h, tx, uk)) => (h, tx, uk, params, preset, None),
            Err(e) => match start(&r.kind, r.params) {
                Ok((h, tx, uk)) => (h, tx, uk, r.params, r.preset, Some(e)),
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
        self.shared.force_key.store(true, Ordering::SeqCst);
        {
            let mut s = self.status.lock().unwrap();
            s.encoder = used_kind.label();
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

    pub fn set_preset(&self, preset: Preset) -> Result<()> {
        let mut g = self.res.lock().unwrap();
        let r = g.as_mut().ok_or_else(|| anyhow!("transmissão encerrada"))?;
        let (sw, sh) = unpack_size(r.src_size.load(Ordering::Relaxed));
        let params = encode_params(preset, sw, sh, r.hz);
        if r.preset == preset && r.params == params {
            return Ok(());
        }
        self.reconfigure(r, preset, r.kind.clone())
    }

    pub fn apply_codec(&self) -> Option<Result<Codec>> {
        if !self.shared.codec_dirty.swap(false, Ordering::AcqRel) {
            return None;
        }
        let mut g = self.res.lock().unwrap();
        let r = g.as_mut()?;
        let viewers_decoders: Vec<Vec<Codec>> = {
            let conns = self.shared.conns.lock().unwrap();
            conns.values().map(|p| p.decoders.clone()).collect()
        };
        let want = pick_encoder(&r.kinds, &viewers_decoders)?;
        if want == r.kind {
            return None;
        }
        let res = self.reconfigure(r, r.preset, want.clone());
        if res.is_err() || r.kind != want {
            r.kinds.retain(|k| k != &want);
            let err = res.err().unwrap_or_else(|| anyhow!("encoder {} indisponível", want.label()));
            return Some(Err(err));
        }
        Some(Ok(want.codec()))
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
            .map(|p| p.conn.clone());
        let Some(c) = conn else { return };
        self.shared.banned.lock().unwrap().insert(c.remote_id());
        self.shared
            .status
            .lock()
            .unwrap()
            .viewers
            .retain(|v| v.id_short != id_short);
        c.close(CLOSE_KICKED.into(), b"kicked");
    }

    pub fn new_invite(&self) {
        self.shared.new_invite();
    }

    pub fn set_friends(&self, ids: HashSet<iroh::EndpointId>) {
        *self.shared.friends.lock().unwrap() = ids;
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
            self.0.codec_dirty.store(true, Ordering::Release);
            self.0
                .status
                .lock()
                .unwrap()
                .viewers
                .retain(|v| &v.id_short != id);
        }
    }
}

async fn answer_presence(conn: Connection, sh: Arc<Shared>) {
    let remote_id = conn.remote_id();
    let is_friend = sh.friends.lock().unwrap().contains(&remote_id);
    let is_banned = sh.banned.lock().unwrap().contains(&remote_id);
    if is_banned {
        conn.close(CLOSE_DENIED.into(), livescreen_core::friends::REMOVED_REASON);
        return;
    }
    if !is_friend {
        conn.close(CLOSE_DENIED.into(), b"denied");
        return;
    }
    let res = async {
        let mut send = conn.open_uni().await?;
        write_msg(
            &mut send,
            &livescreen_core::friends::Presence {
                title: sh.title.clone(),
            },
        )
        .await?;
        send.finish()?;
        let _ = tokio::time::timeout(Duration::from_secs(3), conn.closed()).await;
        conn.close(CLOSE_NORMAL.into(), b"bye");
        Ok::<_, anyhow::Error>(())
    }
    .await;
    if let Err(e) = res {
        info!("presença encerrada: {e:#}");
    }
}

/// Removido nesta transmissão nunca entra; amigo entra sem convite; o resto precisa da prova da sala.
fn admit(banned: bool, friend: bool, proof: &str, proof_ok: bool) -> Result<(), &'static str> {
    if banned {
        return Err("você foi removido desta transmissão");
    }
    if friend || proof_ok {
        return Ok(());
    }
    if proof.is_empty() {
        Err("você ainda não está na lista de amigos de quem transmite: peça um convite")
    } else {
        Err("convite inválido ou substituído")
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
    let ViewerMsg::Hello {
        protocol,
        name,
        room_proof,
        decoders,
        ..
    } = hello else {
        conn.close(CLOSE_PROTOCOL.into(), b"protocol");
        return Ok(());
    };
    if protocol != PROTOCOL_VERSION {
        let _ = write_msg(
            &mut send,
            &HostMsg::Rejected {
                reason: if protocol < PROTOCOL_VERSION {
                    "LiveScreen desatualizado ou código de convite incompleto: atualize o app e use o código completo"
                } else {
                    "quem transmite está com um LiveScreen mais antigo e precisa atualizar"
                }
                .into(),
            },
        )
        .await;
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_millis(500), conn.closed()).await;
        conn.close(CLOSE_PROTOCOL.into(), b"protocol");
        return Ok(());
    }
    let remote_id = conn.remote_id();
    let is_banned = sh.banned.lock().unwrap().contains(&remote_id);
    let is_friend = sh.friends.lock().unwrap().contains(&remote_id);
    let proof_ok = if is_friend {
        false
    } else {
        let key = *sh.room_key.lock().unwrap();
        verify_room_proof(&key, &sh.host_id, &remote_id, &room_proof)
    };
    if let Err(reason) = admit(is_banned, is_friend, &room_proof, proof_ok) {
        let _ = write_msg(
            &mut send,
            &HostMsg::Rejected {
                reason: reason.into(),
            },
        )
        .await;
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_millis(500), conn.closed()).await;
        conn.close(CLOSE_DENIED.into(), b"denied");
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
    sh.conns.lock().unwrap().insert(
        id.clone(),
        Peer {
            conn: conn.clone(),
            name: sanitize(&name, 32),
            decoders,
        },
    );
    sh.codec_dirty.store(true, Ordering::Release);

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
                host_name: sh.host_name.clone(),
                light_presets: true,
            },
        ),
    )
    .await
    .map_err(|_| anyhow!("timeout enviando welcome"))??;
    // handshake concluído: a vaga volta para a próxima conexão; espectadores não têm teto
    drop(handshake);
    let mut vrx = sh.video_tx.subscribe();
    let mut arx = sh.audio_tx.subscribe();
    let mut shutdown = sh.shutdown.clone();

    // controle: pedidos de keyframe e de qualidade
    let sh2 = sh.clone();
    let who = sanitize(&name, 32);
    let visible = Arc::new(AtomicBool::new(true));
    let visible2 = visible.clone();
    let ctl = tokio::spawn(async move {
        while let Ok(m) = read_msg::<ViewerMsg>(&mut recv).await {
            match m {
                ViewerMsg::RequestKeyframe => sh2.request_keyframe(),
                ViewerMsg::SetQuality { preset } => sh2.request_quality(preset, &who),
                ViewerMsg::SetVisible { visible: v } => visible2.store(v, Ordering::Relaxed),
                _ => {}
            }
        }
    });

    let mut vsend: Option<iroh::endpoint::SendStream> = None;
    let mut gop_prio: i32 = 0;
    let mut waiting_key = true;
    // Um espectador lento não deve pedir keyframe a cada quadro perdido (inflaria o
    // bitrate de todos); no máximo um pedido por segundo, por espectador.
    let mut last_kf = Instant::now() - Duration::from_secs(2);
    sh.request_keyframe();
    let res: Result<()> = loop {
        tokio::select! {
            r = vrx.recv() => match r {
                Ok(f) => {
                    // espectador escondido: sem vídeo; o keyframe é pedido quando voltar
                    if !visible.load(Ordering::Relaxed) {
                        waiting_key = true;
                        continue;
                    }
                    if waiting_key && !f.keyframe {
                        if last_kf.elapsed() >= Duration::from_secs(1) {
                            last_kf = Instant::now();
                            sh.request_keyframe();
                        }
                        continue;
                    }
                    if f.keyframe {
                        if let Some(old) = vsend.take() { retire(old); }
                        let s = match tokio::time::timeout(HANDSHAKE_TIMEOUT, conn.open_uni()).await {
                            Ok(Ok(s)) => s,
                            Ok(Err(e)) => break Err(e.into()),
                            Err(_) => break Err(anyhow!("timeout abrindo stream de vídeo")),
                        };
                        // o GOP mais novo sai primeiro
                        gop_prio = gop_prio.wrapping_add(1);
                        let _ = s.set_priority(gop_prio);
                        vsend = Some(s);
                    }
                    waiting_key = false;
                    let Some(s) = vsend.as_mut() else { continue };
                    let mut chunks = [Bytes::copy_from_slice(&video_frame_header(&f)), f.data.clone()];
                    match tokio::time::timeout(STALE_VIDEO, s.write_all_chunks(&mut chunks)).await {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => break Err(e.into()),
                        Err(_) => {
                            // o quadro parcial some com o reset
                            if let Some(mut s) = vsend.take() { let _ = s.reset(0u32.into()); }
                            waiting_key = true;
                            sh.congested.store(true, Ordering::Relaxed);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    waiting_key = true;
                    sh.congested.store(true, Ordering::Relaxed);
                }
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
    use super::*;
    use crate::video::encoder::EncoderKind;
    use livescreen_core::protocol::Codec;

    #[test]
    fn abr_backs_off_and_recovers_slowly() {
        assert_eq!(Abr::default().next(10_000_000, 10_000_000, true), 7_000_000);
        let mut a = Abr::default();
        assert_eq!(a.next(7_000_000, 10_000_000, false), 7_000_000);
        assert_eq!(a.next(7_000_000, 10_000_000, false), 7_000_000);
        assert_eq!(a.next(7_000_000, 10_000_000, false), 7_560_000);
        assert_eq!(Abr::default().next(8_000_000, 4_000_000, false), 4_000_000);
        let (mut a, mut c) = (Abr::default(), 10_000_000);
        for _ in 0..20 {
            c = a.next(c, 10_000_000, true);
        }
        assert_eq!(c, 500_000);
        assert_eq!(Abr::default().next(1_000_000, 300_000, true), 300_000);
    }

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
            Some(h264_hw)
        );
        assert_eq!(
            pick_encoder(&kinds1, &[vec![Codec::Hevc]]),
            Some(hevc_hw.clone())
        );
        assert_eq!(pick_encoder(&kinds1, &[]), None);

        let kinds2 = [av1.clone(), hevc_hw, sw];
        assert_eq!(
            pick_encoder(&kinds2, &[vec![Codec::H264, Codec::Av1, Codec::Hevc]]),
            Some(av1)
        );
    }

    #[test]
    fn test_admit_precedence() {
        // banido vence amigo
        assert_eq!(
            admit(true, true, "", false),
            Err("você foi removido desta transmissão")
        );
        assert_eq!(
            admit(true, false, "proof", true),
            Err("você foi removido desta transmissão")
        );

        // amigo dispensa prova
        assert_eq!(admit(false, true, "", false), Ok(()));
        assert_eq!(admit(false, true, "bad_proof", false), Ok(()));

        // não amigo, com prova ok
        assert_eq!(admit(false, false, "valid_proof", true), Ok(()));

        // prova vazia vs. errada dão mensagens diferentes
        assert_eq!(
            admit(false, false, "", false),
            Err("você ainda não está na lista de amigos de quem transmite: peça um convite")
        );
        assert_eq!(
            admit(false, false, "wrong_proof", false),
            Err("convite inválido ou substituído")
        );
    }
}
