use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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

pub const MAX_VIEWERS: usize = 4;
/// Conexões simultâneas (handshake + espectadores ativos) aceitas; o excedente é recusado.
const MAX_PENDING: usize = 16;
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
}

type SinkTx = Arc<Mutex<Option<std::sync::mpsc::SyncSender<Nv12Frame>>>>;

struct Resources {
    capture: CaptureHandle,
    audio: Option<AudioHandle>,
    encoder: EncoderHandle,
    sink_tx: SinkTx,
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
    /// Teto de conexões simultâneas; sem ele, uma enxurrada de conexões vira enxurrada de tasks.
    pending: Arc<Semaphore>,
    /// Desempata ids de espectadores que compartilham o mesmo prefixo.
    next_id: AtomicUsize,
    video_tx: broadcast::Sender<Arc<EncodedFrame>>,
    audio_tx: broadcast::Sender<Bytes>,
    force_key: Arc<AtomicBool>,
    last_key: Mutex<Instant>,
    welcome: (u32, u32, u32, bool),
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

struct Sink {
    conv: Converter,
    tx: SinkTx,
    last: Instant,
    interval: Duration,
    start: Instant,
    status: Arc<Mutex<ShareStatus>>,
    dropped: u64,
}

impl FrameSink for Sink {
    fn on_frame(
        &mut self,
        tex: &windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
        _w: u32,
        _h: u32,
    ) {
        let now = Instant::now();
        if now.duration_since(self.last) + Duration::from_millis(2) < self.interval {
            return;
        }
        self.last = now;
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
            self.dropped += 1;
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
    ) -> Result<ShareSession> {
        let (device, context) = crate::gpu::create_device()?;
        let (sw, sh) = capture::source_size(&source.id)?;
        let params = encode_params(preset, sw, sh);
        let conv = Converter::new(&device, &context, params.width, params.height, params.fps)?;

        let (video_tx, _) = broadcast::channel::<Arc<EncodedFrame>>(16);
        let (audio_tx, _) = broadcast::channel::<Bytes>(64);
        let force_key = Arc::new(AtomicBool::new(true));
        let stats = Arc::new(EncoderStats::default());

        let (enc, tx, enc_name) = start_encoder(
            encoder_kind,
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
        }));

        let sink = Sink {
            conv,
            tx: sink_tx.clone(),
            last: Instant::now() - Duration::from_secs(1),
            interval: Duration::from_secs_f64(1.0 / params.fps as f64),
            start: Instant::now(),
            status: status.clone(),
            dropped: 0,
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
            pending: Arc::new(Semaphore::new(MAX_PENDING)),
            next_id: AtomicUsize::new(0),
            video_tx,
            audio_tx,
            force_key,
            last_key: Mutex::new(Instant::now() - Duration::from_secs(2)),
            welcome: (params.width, params.height, params.fps, audio_h.is_some()),
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
                        let _permit = permit;
                        match incoming.await {
                            Ok(conn) => {
                                if let Err(e) = handle_viewer(conn, shared).await {
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
            let (shared, stats) = (shared.clone(), stats.clone());
            rt.spawn(async move {
                let (mut pf, mut pb) = (0u64, 0u64);
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
                    let mut s = shared.status.lock().unwrap();
                    s.viewers = viewers;
                    s.fps = (f - pf) as f32;
                    s.kbps = ((b - pb) * 8 / 1000) as u32;
                    pf = f;
                    pb = b;
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
                encoder: enc,
                sink_tx,
                ep,
                shutdown: sd_tx,
            })),
        })
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
        r.sink_tx.lock().unwrap().take();
        r.encoder.stop();
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

async fn handle_viewer(conn: Connection, sh: Arc<Shared>) -> Result<()> {
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
    if sh.viewers.fetch_add(1, Ordering::SeqCst) >= MAX_VIEWERS {
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

    let (w, h, fps, audio) = sh.welcome;
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
    use super::{invite_addr, sanitize};

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
