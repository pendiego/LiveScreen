use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Result, anyhow};
use iroh::Endpoint;
use iroh::endpoint::{Connection, ConnectionError, presets};
use iroh_tickets::endpoint::EndpointTicket;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::protocol::*;
use crate::video::decoder::{self, RgbaFrame};

pub fn is_valid_ticket(s: &str) -> bool {
    s.trim().parse::<EndpointTicket>().is_ok()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Connecting,
    Playing,
    Ended(String),
}

#[derive(Clone, Debug)]
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

pub struct WatchSession {
    rt: Handle,
    pub status: Arc<Mutex<WatchStatus>>,
    pub frame: Arc<Mutex<Option<RgbaFrame>>>,
    ep: Endpoint,
    task: JoinHandle<()>,
    gain: Arc<AtomicU32>,
    quality_tx: tokio::sync::mpsc::UnboundedSender<crate::preset::Preset>,
    stopped: AtomicBool,
}

fn close_reason(e: &ConnectionError) -> &'static str {
    match e {
        ConnectionError::ApplicationClosed(c) => match u64::from(c.error_code) {
            CLOSE_KICKED_U64 => "Você foi removido",
            CLOSE_STOPPED_U64 => "Transmissão encerrada",
            CLOSE_FULL_U64 => "Sala cheia",
            _ => "Conexão perdida",
        },
        _ => "Conexão perdida",
    }
}
const CLOSE_FULL_U64: u64 = CLOSE_FULL as u64;
const CLOSE_KICKED_U64: u64 = CLOSE_KICKED as u64;
const CLOSE_STOPPED_U64: u64 = CLOSE_STOPPED as u64;

impl WatchSession {
    pub fn start(
        rt: &Handle,
        ticket: &str,
        name: &str,
        gain: f32,
        hardware_decode: bool,
        ctx: egui::Context,
    ) -> Result<WatchSession> {
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
        let frame = Arc::new(Mutex::new(None));
        let gain = Arc::new(AtomicU32::new(gain.to_bits()));
        let (quality_tx, quality_rx) = tokio::sync::mpsc::unbounded_channel();
        let task = {
            let (ep, status, frame) = (ep.clone(), status.clone(), frame.clone());
            let (name, gain2) = (name.to_string(), gain.clone());
            let addr = t.endpoint_addr().clone();
            rt.spawn(async move {
                let end = |m: String| {
                    let mut s = status.lock().unwrap();
                    if !matches!(s.phase, Phase::Ended(_)) {
                        s.phase = Phase::Ended(m);
                    }
                    ctx.request_repaint();
                };
                match run(
                    &ep,
                    addr,
                    &status,
                    frame,
                    ctx.clone(),
                    name,
                    gain2,
                    hardware_decode,
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
        Ok(WatchSession {
            rt: rt.clone(),
            status,
            frame,
            ep,
            task,
            gain,
            quality_tx,
            stopped: AtomicBool::new(false),
        })
    }

    /// Pede ao host a qualidade da transmissão; vale para a sala toda e o host pode demorar
    /// ou ignorar (há um intervalo mínimo entre pedidos).
    pub fn request_quality(&self, preset: crate::preset::Preset) {
        let _ = self.quality_tx.send(preset);
    }

    pub fn set_gain(&self, g: f32) {
        self.gain.store(g.to_bits(), Ordering::Relaxed);
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

impl Drop for WatchSession {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run(
    ep: &Endpoint,
    addr: iroh::EndpointAddr,
    status: &Arc<Mutex<WatchStatus>>,
    frame: Arc<Mutex<Option<RgbaFrame>>>,
    ctx: egui::Context,
    name: String,
    gain: Arc<AtomicU32>,
    hardware_decode: bool,
    mut quality_rx: tokio::sync::mpsc::UnboundedReceiver<crate::preset::Preset>,
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
            app_version: env!("CARGO_PKG_VERSION").into(),
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
        s.title = crate::share::sanitize(&title, 120);
    }
    let _audio = if audio {
        match crate::audio::playback::start(&Handle::current(), conn.clone(), gain) {
            Ok(a) => Some(a),
            Err(e) => {
                warn!("áudio do espectador: {e:#}");
                None
            }
        }
    } else {
        None
    };

    let mut vrecv = conn.accept_uni().await?;
    let need_key = Arc::new(AtomicBool::new(true));
    let (tx, rx) = std::sync::mpsc::sync_channel::<EncodedFrame>(8);
    let dstats = Arc::new(decoder::DecodeStats::default());
    let dec = decoder::spawn(
        rx,
        frame,
        ctx.clone(),
        need_key.clone(),
        dstats.clone(),
        hardware_decode,
    );
    let count = Arc::new(AtomicU64::new(0));

    let read_loop = async {
        loop {
            match read_video_frame(&mut vrecv).await? {
                None => return Ok::<(), anyhow::Error>(()),
                Some(f) => {
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
            if dec.is_finished() {
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
            let frames = dstats.frames.load(Ordering::Relaxed);
            let busy = dstats.busy_us.load(Ordering::Relaxed);
            let (df, db) = (frames - prev_frames, busy - prev_busy);
            (prev_frames, prev_busy) = (frames, busy);
            let hw = dstats.hardware.load(Ordering::Relaxed);
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
            ctx.request_repaint();
        }
    };

    tokio::select! {
        r = read_loop => { if let Err(e) = r { info!("leitura de vídeo: {e:#}"); } }
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
            if ended { return Ok("Transmissão encerrada".into()); }
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

fn _assert_conn(_: &Connection) {}
