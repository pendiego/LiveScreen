//! Host sem plataforma: aceita espectadores (iroh e navegador), reparte vídeo/áudio/chat e
//! controla o bitrate. A plataforma entrega quadros codificados por `Feeds`.

mod web;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use bytes::Bytes;
use iroh::Endpoint;
use iroh::endpoint::{Connection, presets};
use iroh_tickets::endpoint::EndpointTicket;
use tokio::runtime::Handle;
use tokio::sync::{Semaphore, broadcast, watch};
use tracing::info;

use crate::chat::{ChatLimiter, ChatLine, ChatLog};
use crate::preset::Preset;
use crate::protocol::*;

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

pub(crate) fn rtt_congested(rtt: Duration, min: Duration) -> bool {
    rtt > min + min.max(Duration::from_millis(80))
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
    pub id: Option<iroh::EndpointId>,
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

#[derive(Clone, Debug, PartialEq)]
pub enum WebState {
    Off,
    Connecting,
    Online,
    Retrying(String),
}

#[derive(Clone, Debug)]
pub struct ShareStatus {
    pub ticket: Option<String>,
    pub online_warning: bool,
    pub web: WebState,
    /// VPN que é a rota padrão (nome do adaptador); o link do navegador também tenta pela placa física.
    pub web_vpn: Option<String>,
    pub backend: &'static str,
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
#[derive(Clone)]
pub(crate) enum PeerConn {
    Iroh(Connection),
    Web(Arc<web::WebPeer>),
}

pub(crate) struct Peer {
    pub(crate) conn: PeerConn,
    pub(crate) name: String,
    pub(crate) decoders: Vec<Codec>,
    pub(crate) h264_baseline: bool,
}

pub(crate) struct Shared {
    pub(crate) status: Arc<Mutex<ShareStatus>>,
    pub(crate) viewers: AtomicUsize,
    pub(crate) conns: Mutex<HashMap<String, Peer>>,
    pub(crate) codec_dirty: AtomicBool,
    /// Teto de handshakes simultâneos; sem ele, uma enxurrada de conexões vira enxurrada de tasks.
    pub(crate) pending: Arc<Semaphore>,
    /// Desempata ids de espectadores que compartilham o mesmo prefixo.
    next_id: AtomicUsize,
    /// Limite da sala; ajustável durante a transmissão.
    pub(crate) max_viewers: AtomicUsize,
    pub(crate) video_tx: broadcast::Sender<Arc<EncodedFrame>>,
    pub(crate) audio_tx: broadcast::Sender<Bytes>,
    force_key: Arc<AtomicBool>,
    last_key: Mutex<Instant>,
    quality_req: Mutex<Option<(Preset, String)>>,
    last_quality: Mutex<Instant>,
    params: watch::Sender<(u32, u32, u32, bool)>,
    title: String,
    pub(crate) shutdown: watch::Receiver<bool>,
    /// Algum espectador ficou para trás desde o último tick.
    pub(crate) congested: AtomicBool,
    pub(crate) room_key: Mutex<[u8; ROOM_KEY_LEN]>,
    pub(crate) web_secret: [u8; ROOM_KEY_LEN],
    pub(crate) web_started: AtomicBool,
    ticket: Mutex<Option<EndpointTicket>>,
    host_id: iroh::EndpointId,
    friends: Mutex<HashSet<iroh::EndpointId>>,
    banned: Mutex<HashSet<iroh::EndpointId>>,
    host_name: String,
    pub(crate) chat_tx: broadcast::Sender<Arc<HostMsg>>,
    pub(crate) chat: Arc<Mutex<ChatLog>>,
    /// Ids de remetente de chat; o 0 é do host.
    pub(crate) next_chat_id: AtomicU32,
    /// Doações simuladas ligadas nesta transmissão.
    pub(crate) donate: bool,
}

impl Shared {
    /// Registra a linha e repassa para a sala toda (quem enviou também recebe: é o eco).
    pub(crate) fn post_chat(&self, line: ChatLine) {
        self.chat.lock().unwrap().push(line.clone());
        let _ = self.chat_tx.send(Arc::new(HostMsg::Chat(line)));
    }

    pub(crate) fn request_keyframe(&self) {
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

pub(crate) struct CountGuard<'a>(pub(crate) &'a Shared, pub(crate) Option<String>);
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
        conn.close(CLOSE_DENIED.into(), crate::friends::REMOVED_REASON);
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
            &crate::friends::Presence {
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
pub(crate) fn admit(banned: bool, friend: bool, proof: &str, proof_ok: bool) -> Result<(), &'static str> {
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
        chat: wants_chat,
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
            conn: PeerConn::Iroh(conn.clone()),
            name: sanitize(&name, 32),
            decoders,
            h264_baseline: false,
        },
    );
    sh.codec_dirty.store(true, Ordering::Release);

    let chat_id = sh.next_chat_id.fetch_add(1, Ordering::Relaxed);
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
                chat: true,
                chat_id,
                donate: sh.donate,
            },
        ),
    )
    .await
    .map_err(|_| anyhow!("timeout enviando welcome"))??;
    // handshake concluído: a vaga volta para a próxima conexão; espectadores não têm teto
    drop(handshake);
    let mut vrx = sh.video_tx.subscribe();
    let mut arx = sh.audio_tx.subscribe();
    let mut crx = wants_chat.then(|| sh.chat_tx.subscribe());
    let mut shutdown = sh.shutdown.clone();

    // controle: pedidos de keyframe e de qualidade
    let sh2 = sh.clone();
    let who = sanitize(&name, 32);
    let mut limiter = ChatLimiter::default();
    let visible = Arc::new(AtomicBool::new(true));
    let sender_name = who.clone();
    let visible2 = visible.clone();
    let ctl = tokio::spawn(async move {
        while let Ok(m) = read_msg::<ViewerMsg>(&mut recv).await {
            match m {
                ViewerMsg::RequestKeyframe => sh2.request_keyframe(),
                ViewerMsg::SetQuality { preset } => sh2.request_quality(preset, &who),
                ViewerMsg::SetVisible { visible: v } => visible2.store(v, Ordering::Relaxed),
                ViewerMsg::Chat { text, donate } => {
                    if limiter.allow(Instant::now())
                        && let Some(l) = ChatLine::incoming(chat_id, &sender_name, is_friend, &text, donate, sh2.donate)
                    {
                        sh2.post_chat(l);
                    }
                }
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
            r = async { crx.as_mut().unwrap().recv().await }, if crx.is_some() => match r {
                Ok(m) => if let Err(e) = write_msg(&mut send, &*m).await { break Err(e); },
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => crx = None,
            },
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

/// Contadores compartilhados entre o encoder da plataforma e o host.
#[derive(Default)]
pub struct HostStats {
    pub frames: AtomicU64,
    pub bytes: AtomicU64,
    /// Quadros jogados fora com a fila do encoder cheia: o encoder não está dando conta.
    pub dropped: AtomicU64,
    /// alvo do controle adaptativo; 0 = teto do preset
    pub target_bps: AtomicU32,
    /// bitrate do preset do encoder atual
    pub ceiling_bps: AtomicU32,
}

pub struct HostConfig {
    pub identity: iroh::SecretKey,
    pub host_name: String,
    pub title: String,
    pub max_viewers: usize,
    pub friends: HashSet<iroh::EndpointId>,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub audio: bool,
    /// Doações simuladas no chat (experimental); vale pela transmissão inteira.
    pub donate: bool,
}

/// Entradas do encoder e da captura de áudio da plataforma. Áudio: datagrama `[seq u32 LE][Opus 20 ms]`.
#[derive(Clone)]
pub struct Feeds {
    pub video: broadcast::Sender<Arc<EncodedFrame>>,
    pub audio: broadcast::Sender<Bytes>,
    pub force_key: Arc<AtomicBool>,
    pub stats: Arc<HostStats>,
}

impl Feeds {
    pub fn new() -> Feeds {
        Feeds {
            video: broadcast::channel(16).0,
            audio: broadcast::channel(64).0,
            force_key: Arc::new(AtomicBool::new(true)),
            stats: Arc::new(HostStats::default()),
        }
    }
}

impl Default for Feeds {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Host {
    rt: Handle,
    pub status: Arc<Mutex<ShareStatus>>,
    pub chat: Arc<Mutex<ChatLog>>,
    pub(crate) shared: Arc<Shared>,
    ep: Endpoint,
    shutdown: watch::Sender<bool>,
    stopped: AtomicBool,
}

impl Host {
    /// Liga o endpoint (BBR, ALPN + PRESENCE_ALPN) e dispara o accept loop e o tick. Bloqueia; nunca chamar de dentro do runtime.
    pub fn start(
        rt: &Handle,
        cfg: HostConfig,
        feeds: &Feeds,
        status: Arc<Mutex<ShareStatus>>,
    ) -> Result<Host> {
        let ep = rt.block_on(async {
            // BBR: fila curta e tolerância a perda aleatória (Wi-Fi); o CUBIC padrão enche o buffer e aumenta a latência
            let transport = iroh::endpoint::QuicTransportConfig::builder()
                .congestion_controller_factory(Arc::new(noq_proto::congestion::Bbr3Config::default()))
                .max_concurrent_uni_streams(noq_proto::VarInt::from_u32(0))
                .max_concurrent_bidi_streams(noq_proto::VarInt::from_u32(1))
                .datagram_receive_buffer_size(None)
                .build();
            Endpoint::builder(presets::N0)
                .secret_key(cfg.identity)
                .alpns(vec![ALPN.to_vec(), crate::friends::PRESENCE_ALPN.to_vec()])
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
            max_viewers: AtomicUsize::new(cfg.max_viewers.max(1)),
            video_tx: feeds.video.clone(),
            audio_tx: feeds.audio.clone(),
            force_key: feeds.force_key.clone(),
            last_key: Mutex::new(Instant::now() - Duration::from_secs(2)),
            quality_req: Mutex::new(None),
            last_quality: Mutex::new(Instant::now() - Duration::from_secs(10)),
            params: watch::channel((cfg.width, cfg.height, cfg.fps, cfg.audio)).0,
            title: cfg.title,
            shutdown: sd_rx,
            congested: AtomicBool::new(false),
            room_key: Mutex::new(new_room_key()),
            web_secret: new_room_key(),
            web_started: AtomicBool::new(false),
            ticket: Mutex::new(None),
            host_id: ep.id(),
            friends: Mutex::new(cfg.friends),
            banned: Mutex::new(HashSet::new()),
            host_name: cfg.host_name,
            chat_tx: broadcast::channel(64).0,
            chat: Arc::new(Mutex::new(ChatLog::default())),
            next_chat_id: AtomicU32::new(1),
            donate: cfg.donate,
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
                                if conn.alpn() == crate::friends::PRESENCE_ALPN {
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
            let (shared, stats) = (shared.clone(), feeds.stats.clone());
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
                    let mut has_web = false;
                    let viewers = shared
                        .conns
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|(id, peer)| {
                            let (c, name) = (&peer.conn, &peer.name);
                            match c {
                                PeerConn::Iroh(c) => {
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
                                        id: Some(peer_id),
                                        id_short: id.clone(),
                                        name: name.clone(),
                                        friend,
                                        path,
                                        rtt_ms: rtt,
                                    }
                                }
                                PeerConn::Web(w) => {
                                    has_web = true;
                                    ViewerInfo {
                                        id: None,
                                        id_short: id.clone(),
                                        name: name.clone(),
                                        friend: false,
                                        path: "Navegador".into(),
                                        rtt_ms: w.rtt_ms.load(Ordering::Relaxed),
                                    }
                                }
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
                        if rtt_congested(*rtt, e.1) {
                            congested = true;
                        }
                    }
                    min_rtt.retain(|id, _| samples.iter().any(|(s, _, _)| s == id));
                    let ceiling = stats.ceiling_bps.load(Ordering::Relaxed);
                    if samples.is_empty() && !has_web {
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
                    let d = stats.dropped.load(Ordering::Relaxed);
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

        let chat = shared.chat.clone();
        Ok(Host { rt: rt.clone(), status, chat, shared, ep, shutdown: sd_tx, stopped: AtomicBool::new(false) })
    }

    pub fn shutdown_signal(&self) -> watch::Receiver<bool> {
        self.shared.shutdown.clone()
    }

    /// Mensagem do próprio host: sempre `role = Host`, sem limitador.
    pub fn send_chat(&self, text: &str) {
        if let Some(l) = ChatLine::from_host(&self.shared.host_name, text) {
            self.shared.post_chat(l);
        }
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
        match c {
            PeerConn::Iroh(c) => {
                self.shared.banned.lock().unwrap().insert(c.remote_id());
                self.shared
                    .status
                    .lock()
                    .unwrap()
                    .viewers
                    .retain(|v| v.id_short != id_short);
                c.close(CLOSE_KICKED.into(), b"kicked");
            }
            PeerConn::Web(w) => {
                *w.reason.lock().unwrap() = Some("você foi removido desta transmissão".into());
                w.stop.notify_one();
                self.shared
                    .status
                    .lock()
                    .unwrap()
                    .viewers
                    .retain(|v| v.id_short != id_short);
            }
        }
    }

    pub fn new_invite(&self) {
        self.shared.new_invite();
    }

    pub fn set_friends(&self, ids: HashSet<iroh::EndpointId>) {
        *self.shared.friends.lock().unwrap() = ids;
    }

    /// Muda o limite da sala na hora; quem já está assistindo continua.
    pub fn set_max_viewers(&self, n: usize) {
        self.shared.max_viewers.store(n.max(1), Ordering::SeqCst);
    }

    /// Pedido de qualidade pendente de um espectador (preset, nome de quem pediu). Não bloqueia.
    pub fn take_quality_request(&self) -> Option<(Preset, String)> {
        self.shared.quality_req.lock().unwrap().take()
    }

    /// Algum espectador entrou ou saiu desde o último `take_viewer_decoders`. Não bloqueia.
    pub fn codec_pending(&self) -> bool {
        self.shared.codec_dirty.load(Ordering::Acquire)
    }

    /// Decoders de quem assiste, se alguém entrou ou saiu desde a última chamada (zera `codec_dirty`).
    pub fn take_viewer_decoders(&self) -> Option<Vec<Vec<Codec>>> {
        if !self.shared.codec_dirty.swap(false, Ordering::AcqRel) {
            return None;
        }
        let conns = self.shared.conns.lock().unwrap();
        Some(conns.values().map(|p| p.decoders.clone()).collect())
    }

    /// Algum espectador do navegador só decodifica H.264 Baseline/Main (Firefox): o encoder deve gerar Baseline.
    pub fn needs_h264_baseline(&self) -> bool {
        self.shared.conns.lock().unwrap().values().any(|p| p.h264_baseline)
    }

    /// Novo tamanho no ar, repassado aos espectadores e ao status.
    pub fn set_stream(&self, width: u32, height: u32, fps: u32, audio: bool) {
        {
            let mut s = self.shared.status.lock().unwrap();
            s.width = width;
            s.height = height;
            s.target_fps = fps;
        }
        self.shared.params.send_replace((width, height, fps, audio));
    }

    /// Idempotente e bloqueante (nunca chamar de dentro do runtime): avisa os espectadores,
    /// espera até 1,5 s por eles e fecha o endpoint (teto de 2 s).
    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        let _ = self.shutdown.send(true);
        let shared = self.shared.clone();
        self.rt.block_on(async {
            let deadline = Instant::now() + Duration::from_millis(1500);
            while shared.viewers.load(Ordering::SeqCst) > 0 && Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            let _ = tokio::time::timeout(Duration::from_secs(2), self.ep.close()).await;
        });
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_rtt_congested() {
        assert!(rtt_congested(
            Duration::from_millis(201),
            Duration::from_millis(100)
        ));
        assert!(!rtt_congested(
            Duration::from_millis(199),
            Duration::from_millis(100)
        ));
        assert!(rtt_congested(
            Duration::from_millis(81),
            Duration::from_millis(0)
        ));
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
