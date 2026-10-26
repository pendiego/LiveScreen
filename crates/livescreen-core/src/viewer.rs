//! Sessão de quem assiste (viewer): conexão com o host, GOPs, keyframe, estatísticas e parada.

use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Result, anyhow};
use iroh::{Endpoint, SecretKey};
use iroh::endpoint::{Connection, ConnectionError};
use serde::Serialize;
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::preset::Preset;
use crate::protocol::*;

pub fn is_valid_invite(s: &str) -> bool {
    parse_invite(s).is_some()
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Phase {
    Connecting,
    Playing,
    Ended(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct WatchStatus {
    pub phase: Phase,
    pub host_id: String,
    pub host_name: String,
    pub path: String,
    pub rtt_ms: u32,
    pub fps: f32,
    pub width: u32,
    pub height: u32,
    pub audio: bool,
    pub title: String,
    pub decoder_hw: bool,
    pub decode_fps: f32,
    pub decode_ms: f32,
    pub codec: String,
    pub zero_copy: bool,
}

pub fn close_reason(e: &ConnectionError) -> &'static str {
    match e {
        ConnectionError::ApplicationClosed(c) => match u64::from(c.error_code) {
            CLOSE_FULL_U64 => "Sessão cheia",
            CLOSE_PROTOCOL_U64 => "Versão incompatível do LiveScreen",
            CLOSE_KICKED_U64 => "Removido pelo host",
            CLOSE_DENIED_U64 => "Convite inválido ou substituído",
            CLOSE_STOPPED_U64 => "Transmissão encerrada",
            _ => "Conexão perdida",
        },
        _ => "Conexão perdida",
    }
}

pub const CLOSE_FULL_U64: u64 = CLOSE_FULL as u64;
pub const CLOSE_PROTOCOL_U64: u64 = CLOSE_PROTOCOL as u64;
pub const CLOSE_KICKED_U64: u64 = CLOSE_KICKED as u64;
pub const CLOSE_STOPPED_U64: u64 = CLOSE_STOPPED as u64;
pub const CLOSE_DENIED_U64: u64 = CLOSE_DENIED as u64;

const BUDGET_MSG: &str = "Transmissão interrompida: o host enviou dados demais";

/// Contadores lidos pelo `stats_loop` do viewer.
pub struct DecodeStats {
    /// quadros publicados
    pub frames: AtomicU64,
    /// push + drain (decodificação + conversão), em µs
    pub busy_us: AtomicU64,
    /// modo atual do decoder
    pub hardware: AtomicBool,
    /// a thread/serviço do decoder acabou; o viewer encerra a sessão
    pub dead: AtomicBool,
    /// codec atualmente decodificado (`Codec::wire`); u8::MAX = nenhum
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
            dead: AtomicBool::new(false),
            codec: AtomicU8::new(u8::MAX),
            zero_copy: AtomicBool::new(false),
        }
    }
}

/// Chamado depois do Welcome (conexão, host manda áudio?): devolve a entrada do decoder e o que mantém o áudio vivo.
pub type StartOutputs = Box<
    dyn FnOnce(
            &Connection,
            bool,
        ) -> (
            std::sync::mpsc::SyncSender<EncodedFrame>,
            Option<Box<dyn Send>>,
        ) + Send,
>;

pub struct ViewerSession {
    rt: Handle,
    ep: Endpoint,
    task: JoinHandle<()>,
    pub status: Arc<Mutex<WatchStatus>>,
    ctl_tx: UnboundedSender<ViewerMsg>,
    stopped: AtomicBool,
}

impl ViewerSession {
    pub fn start(
        rt: &Handle,
        ticket: &str,
        identity: SecretKey,
        name: &str,
        app_version: &str,
        decoders: Vec<Codec>,
        need_key: Arc<AtomicBool>,
        stats: Arc<DecodeStats>,
        notify: Arc<dyn Fn() + Send + Sync>,
        outputs: StartOutputs,
    ) -> Result<ViewerSession> {
        let target = Target::parse(ticket).ok_or_else(|| anyhow!("Código de convite inválido"))?;
        let ep = rt.block_on(crate::friends::dial_endpoint(identity))?;
        let status = Arc::new(Mutex::new(WatchStatus {
            phase: Phase::Connecting,
            host_id: target.addr().id.to_string(),
            host_name: String::new(),
            path: "?".into(),
            rtt_ms: 0,
            fps: 0.0,
            width: 0,
            height: 0,
            audio: false,
            title: String::new(),
            decoder_hw: false,
            decode_fps: 0.0,
            decode_ms: 0.0,
            codec: String::new(),
            zero_copy: false,
        }));
        let (ctl_tx, ctl_rx) = tokio::sync::mpsc::unbounded_channel();
        let task = {
            let (ep, status) = (ep.clone(), status.clone());
            let (name, app_version) = (name.to_string(), app_version.to_string());
            let notify = notify.clone();
            rt.spawn(async move {
                let end = |m: String| {
                    let mut s = status.lock().unwrap();
                    if !matches!(s.phase, Phase::Ended(_)) {
                        s.phase = Phase::Ended(m);
                    }
                    notify();
                };
                match run(
                    &ep,
                    target,
                    &status,
                    name,
                    app_version,
                    decoders,
                    need_key,
                    stats,
                    notify.clone(),
                    outputs,
                    ctl_rx,
                )
                .await
                {
                    Ok(m) => end(m),
                    Err(e) => {
                        warn!("watch: {e:#}");
                        end(format!("{e}"));
                    }
                }
            })
        };
        Ok(ViewerSession {
            rt: rt.clone(),
            status,
            ep,
            task,
            ctl_tx,
            stopped: AtomicBool::new(false),
        })
    }

    /// Pede ao host a qualidade da transmissão; vale para a sala toda e o host pode demorar
    /// ou ignorar (há um intervalo mínimo entre pedidos).
    pub fn request_quality(&self, preset: Preset) {
        let _ = self.ctl_tx.send(ViewerMsg::SetQuality { preset });
    }

    /// Avisa o host que a janela foi minimizada/restaurada: escondido, ele para de mandar vídeo.
    pub fn set_visible(&self, v: bool) {
        let _ = self.ctl_tx.send(ViewerMsg::SetVisible { visible: v });
    }

    /// Idempotente: chamado explicitamente e de novo pelo `Drop`, sem pagar o block_on duas vezes.
    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        self.task.abort();
        let ep = self.ep.clone();
        self.rt.block_on(async {
            let _ = tokio::time::timeout(Duration::from_millis(3500), ep.close()).await;
        });
    }
}

impl Drop for ViewerSession {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run(
    ep: &Endpoint,
    target: Target,
    status: &Arc<Mutex<WatchStatus>>,
    name: String,
    app_version: String,
    decoders: Vec<Codec>,
    need_key: Arc<AtomicBool>,
    stats: Arc<DecodeStats>,
    notify: Arc<dyn Fn() + Send + Sync>,
    outputs: StartOutputs,
    mut ctl_rx: tokio::sync::mpsc::UnboundedReceiver<ViewerMsg>,
) -> Result<String> {
    let addr = target.addr();
    let (protocol, proof) = match &target {
        Target::Invite(_, None) => {
            info!("convite sem chave de sala: host 0.4, protocolo {LEGACY_PROTOCOL_VERSION}");
            (LEGACY_PROTOCOL_VERSION, String::new())
        }
        Target::Invite(_, Some(k)) => (PROTOCOL_VERSION, room_proof(k, &addr.id, &ep.id())),
        Target::Friend(_) => (PROTOCOL_VERSION, String::new()),
    };
    let conn = tokio::time::timeout(Duration::from_secs(30), ep.connect(addr.clone(), ALPN))
        .await
        .map_err(|_| anyhow!("Não foi possível conectar (tempo esgotado)"))?
        .map_err(|e| anyhow!("Não foi possível conectar: {e}"))?;
    let (mut send, mut recv) = conn.open_bi().await?;
    write_msg(
        &mut send,
        &ViewerMsg::Hello {
            protocol,
            app_version,
            name,
            room_proof: proof,
            decoders,
        },
    )
    .await?;
    let welcome: HostMsg = tokio::time::timeout(Duration::from_secs(15), read_msg(&mut recv))
        .await
        .map_err(|_| anyhow!("Sem resposta do host"))??;
    let (w, h, audio, title, host_name) = match welcome {
        HostMsg::Welcome {
            width,
            height,
            audio,
            title,
            host_name,
            ..
        } => (width, height, audio, title, host_name),
        HostMsg::Rejected { reason } => return Ok(format!("Recusado: {reason}")),
        HostMsg::StreamEnded => return Ok("Transmissão encerrada".into()),
        HostMsg::StreamParams { .. } => return Ok("Resposta inesperada do host".into()),
    };
    {
        let mut s = status.lock().unwrap();
        s.phase = Phase::Playing;
        s.width = w;
        s.height = h;
        s.audio = audio;
        s.title = crate::protocol::sanitize(&title, 120);
        s.host_name = crate::protocol::sanitize(&host_name, 32);
    }
    let (tx, _audio) = outputs(&conn, audio);

    let count = Arc::new(AtomicU64::new(0));
    let budget = Arc::new(FrameBudget::default());
    let budget_hit = Arc::new(AtomicBool::new(false));

    let read_loop = async {
        let (ftx, mut frx) = tokio::sync::mpsc::channel::<(u64, EncodedFrame)>(32);
        let mut readers = tokio::task::JoinSet::new(); // dropar aborta os leitores
        let (mut next_gop, mut cur_gop) = (0u64, 0u64);
        loop {
            tokio::select! {
                s = conn.accept_uni() => {
                    let mut s = match s {
                        Ok(s) => s,
                        Err(e) => return Err::<(), anyhow::Error>(e.into()),
                    };
                    let (gop, ftx, conn, budget, budget_hit) = (
                        next_gop,
                        ftx.clone(),
                        conn.clone(),
                        budget.clone(),
                        budget_hit.clone(),
                    );
                    next_gop += 1;
                    readers.spawn(async move {
                        // reset do host (GOP descartado) é o fim normal deste stream
                        loop {
                            match read_video_frame(&mut s, &budget).await {
                                Ok(Some(f)) => {
                                    if ftx.send((gop, f)).await.is_err() {
                                        break;
                                    }
                                }
                                Ok(None) => break,
                                Err(e) if e.is::<BudgetExceeded>() => {
                                    budget_hit.store(true, Ordering::SeqCst);
                                    conn.close(CLOSE_PROTOCOL.into(), b"budget");
                                    break;
                                }
                                Err(_) => break,
                            }
                        }
                    });
                    while readers.try_join_next().is_some() {}
                }
                Some((gop, f)) = frx.recv() => {
                    if gop < cur_gop {
                        continue;
                    } // GOP velho chegando atrasado
                    cur_gop = gop;
                    if need_key.load(Ordering::SeqCst) && !f.keyframe {
                        continue;
                    }
                    if f.keyframe && need_key.load(Ordering::SeqCst) {
                        need_key.store(false, Ordering::SeqCst);
                    }
                    count.fetch_add(1, Ordering::Relaxed);
                    if tx.try_send(f).is_err() {
                        need_key.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
    };
    let key_loop = async {
        let mut tick = tokio::time::interval(Duration::from_millis(200));
        let mut last = tokio::time::Instant::now() - Duration::from_secs(2);
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    if need_key.load(Ordering::SeqCst) && last.elapsed() >= Duration::from_secs(1) {
                        last = tokio::time::Instant::now();
                        if write_msg(&mut send, &ViewerMsg::RequestKeyframe).await.is_err() {
                            return;
                        }
                    }
                }
                Some(msg) = ctl_rx.recv() => {
                    // host 0.4 não conhece SetVisible: a mensagem derrubaria o canal de controle dele
                    if matches!(target, Target::Invite(_, None)) && matches!(msg, ViewerMsg::SetVisible { .. }) {
                        continue;
                    }
                    if write_msg(&mut send, &msg).await.is_err() {
                        return;
                    }
                }
            }
        }
    };
    let stats_loop = async {
        let mut tick = tokio::time::interval(Duration::from_secs(1));
        let mut prev = 0;
        let (mut prev_frames, mut prev_busy) = (0u64, 0u64);
        // acumulados desde o último log (a cada 10 s)
        let (mut acc_frames, mut acc_busy, mut ticks) = (0u64, 0u64, 0u32);
        loop {
            tick.tick().await;
            if stats.dead.load(Ordering::Relaxed) {
                return "Não foi possível iniciar o decodificador de vídeo".to_string();
            }
            let c = count.load(Ordering::Relaxed);
            let (mut path, mut rtt) = ("?".to_string(), 0u32);
            for p in conn.paths().iter() {
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
            let frames = stats.frames.load(Ordering::Relaxed);
            let busy = stats.busy_us.load(Ordering::Relaxed);
            let (df, db) = (frames - prev_frames, busy - prev_busy);
            (prev_frames, prev_busy) = (frames, busy);
            let hw = stats.hardware.load(Ordering::Relaxed);
            {
                let mut s = status.lock().unwrap();
                s.path = path;
                s.rtt_ms = rtt;
                s.fps = (c - prev) as f32;
                s.decoder_hw = hw;
                s.decode_fps = df as f32;
                s.decode_ms = if df == 0 {
                    0.0
                } else {
                    db as f32 / df as f32 / 1000.0
                };
                s.codec = Codec::from_wire(stats.codec.load(Ordering::Relaxed))
                    .map(|c| c.label().to_string())
                    .unwrap_or_default();
                s.zero_copy = stats.zero_copy.load(Ordering::Relaxed);
            }
            prev = c;
            (acc_frames, acc_busy, ticks) = (acc_frames + df, acc_busy + db, ticks + 1);
            if ticks >= 10 {
                if acc_frames > 0 {
                    info!(
                        "decoder {}{}: {:.0} quadros/s, {:.1} ms por quadro",
                        if hw { "hardware" } else { "software" },
                        if stats.zero_copy.load(Ordering::Relaxed) { " (sem cópia)" } else { "" },
                        acc_frames as f32 / ticks as f32,
                        acc_busy as f32 / acc_frames as f32 / 1000.0
                    );
                }
                (acc_frames, acc_busy, ticks) = (0, 0, 0);
            }
            notify();
        }
    };

    tokio::select! {
        r = read_loop => {
            if let Err(e) = r {
                info!("leitura de vídeo: {e:#}");
            }
        }
        _ = key_loop => {}
        m = stats_loop => return Ok(m),
        ended = async {
            loop {
                match read_msg::<HostMsg>(&mut recv).await {
                    Ok(HostMsg::StreamParams { width, height, .. }) => {
                        let mut s = status.lock().unwrap();
                        s.width = width;
                        s.height = height;
                    }
                    Ok(HostMsg::StreamEnded) => break true,
                    Ok(_) => {}
                    Err(_) => break false,
                }
            }
        } => {
            if ended {
                return Ok("Transmissão encerrada".into());
            }
        }
        e = conn.closed() => {
            if budget_hit.load(Ordering::SeqCst) {
                return Ok(BUDGET_MSG.into());
            }
            return Ok(close_reason(&e).into());
        }
    }
    // o stream acabou; o código de fechamento diz o motivo
    if budget_hit.load(Ordering::SeqCst) {
        return Ok(BUDGET_MSG.into());
    }
    Ok(
        match tokio::time::timeout(Duration::from_secs(1), conn.closed()).await {
            Ok(e) => close_reason(&e).into(),
            Err(_) => "Conexão perdida".into(),
        },
    )
}
