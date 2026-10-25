//! Sessão de quem assiste (viewer): conexão com o host, GOPs, keyframe, estatísticas e parada.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Result, anyhow};
use iroh::Endpoint;
use iroh::endpoint::{Connection, ConnectionError, presets};
use iroh_tickets::endpoint::EndpointTicket;
use serde::Serialize;
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::preset::Preset;
use crate::protocol::*;

pub fn is_valid_ticket(s: &str) -> bool {
    s.trim().parse::<EndpointTicket>().is_ok()
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
}

pub fn close_reason(e: &ConnectionError) -> &'static str {
    match e {
        ConnectionError::ApplicationClosed(c) => match u64::from(c.error_code) {
            CLOSE_FULL_U64 => "Sessão cheia",
            CLOSE_PROTOCOL_U64 => "Versão incompatível do LiveScreen",
            CLOSE_KICKED_U64 => "Removido pelo host",
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

/// Contadores lidos pelo `stats_loop` do viewer.
#[derive(Default)]
pub struct DecodeStats {
    /// quadros publicados
    pub frames: AtomicU64,
    /// push + drain (decodificação + conversão), em µs
    pub busy_us: AtomicU64,
    /// modo atual do decoder
    pub hardware: AtomicBool,
    /// a thread/serviço do decoder acabou; o viewer encerra a sessão
    pub dead: AtomicBool,
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
    quality_tx: UnboundedSender<Preset>,
    stopped: AtomicBool,
}

impl ViewerSession {
    pub fn start(
        rt: &Handle,
        ticket: &str,
        name: &str,
        app_version: &str,
        need_key: Arc<AtomicBool>,
        stats: Arc<DecodeStats>,
        notify: Arc<dyn Fn() + Send + Sync>,
        outputs: StartOutputs,
    ) -> Result<ViewerSession> {
        let t: EndpointTicket = ticket
            .trim()
            .parse()
            .map_err(|_| anyhow!("Código de convite inválido"))?;
        let ep = rt.block_on(async { Endpoint::builder(presets::N0).bind().await })?;
        let status = Arc::new(Mutex::new(WatchStatus {
            phase: Phase::Connecting,
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
        }));
        let (quality_tx, quality_rx) = tokio::sync::mpsc::unbounded_channel();
        let task = {
            let (ep, status) = (ep.clone(), status.clone());
            let (name, app_version) = (name.to_string(), app_version.to_string());
            let addr = t.endpoint_addr().clone();
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
                    addr,
                    &status,
                    name,
                    app_version,
                    need_key,
                    stats,
                    notify.clone(),
                    outputs,
                    quality_rx,
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
            quality_tx,
            stopped: AtomicBool::new(false),
        })
    }

    /// Pede ao host a qualidade da transmissão; vale para a sala toda e o host pode demorar
    /// ou ignorar (há um intervalo mínimo entre pedidos).
    pub fn request_quality(&self, preset: Preset) {
        let _ = self.quality_tx.send(preset);
    }

    /// Idempotente: chamado explicitamente e de novo pelo `Drop`, sem pagar o block_on duas vezes.
    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        self.task.abort();
        let ep = self.ep.clone();
        self.rt.block_on(async {
            let _ = tokio::time::timeout(Duration::from_secs(1), ep.close()).await;
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
    addr: iroh::EndpointAddr,
    status: &Arc<Mutex<WatchStatus>>,
    name: String,
    app_version: String,
    need_key: Arc<AtomicBool>,
    stats: Arc<DecodeStats>,
    notify: Arc<dyn Fn() + Send + Sync>,
    outputs: StartOutputs,
    mut quality_rx: tokio::sync::mpsc::UnboundedReceiver<Preset>,
) -> Result<String> {
    let conn = tokio::time::timeout(Duration::from_secs(30), ep.connect(addr, ALPN))
        .await
        .map_err(|_| anyhow!("Não foi possível conectar (tempo esgotado)"))?
        .map_err(|e| anyhow!("Não foi possível conectar: {e}"))?;
    let (mut send, mut recv) = conn.open_bi().await?;
    write_msg(
        &mut send,
        &ViewerMsg::Hello {
            protocol: PROTOCOL_VERSION,
            app_version,
            name,
        },
    )
    .await?;
    let welcome: HostMsg = tokio::time::timeout(Duration::from_secs(15), read_msg(&mut recv))
        .await
        .map_err(|_| anyhow!("Sem resposta do host"))??;
    let (w, h, audio, title) = match welcome {
        HostMsg::Welcome {
            width,
            height,
            audio,
            title,
            ..
        } => (width, height, audio, title),
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
    }
    let (tx, _audio) = outputs(&conn, audio);

    let count = Arc::new(AtomicU64::new(0));

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
                    let (gop, ftx) = (next_gop, ftx.clone());
                    next_gop += 1;
                    readers.spawn(async move {
                        // reset do host (GOP descartado) é o fim normal deste stream
                        while let Ok(Some(f)) = read_video_frame(&mut s).await {
                            if ftx.send((gop, f)).await.is_err() {
                                break;
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
                Some(preset) = quality_rx.recv() => {
                    if write_msg(&mut send, &ViewerMsg::SetQuality { preset }).await.is_err() {
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
            }
            prev = c;
            (acc_frames, acc_busy, ticks) = (acc_frames + df, acc_busy + db, ticks + 1);
            if ticks >= 10 {
                if acc_frames > 0 {
                    info!(
                        "decoder {}: {:.0} quadros/s, {:.1} ms por quadro",
                        if hw { "hardware" } else { "software" },
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
        e = conn.closed() => return Ok(close_reason(&e).into()),
    }
    // o stream acabou; o código de fechamento diz o motivo
    Ok(
        match tokio::time::timeout(Duration::from_secs(1), conn.closed()).await {
            Ok(e) => close_reason(&e).into(),
            Err(_) => "Conexão perdida".into(),
        },
    )
}
