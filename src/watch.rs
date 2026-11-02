use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::runtime::Handle;
use tracing::warn;

use livescreen_core::protocol::EncodedFrame;
pub use livescreen_core::viewer::WatchStatus;
use livescreen_core::viewer::{DecodeStats, ViewerSession};

use crate::video::decoder::{self, VideoFrame};
use crate::video::present;

pub struct WatchSession {
    inner: ViewerSession,
    pub status: Arc<Mutex<WatchStatus>>,
    pub frame: Arc<Mutex<Option<VideoFrame>>>,
    gain: Arc<AtomicU32>,
    readback_only: Arc<AtomicBool>,
    pub chat: Arc<Mutex<livescreen_core::chat::ChatLog>>,
}

impl WatchSession {
    pub fn start(
        rt: &Handle,
        ticket: &str,
        identity: iroh::SecretKey,
        name: &str,
        gain: f32,
        hardware_decode: bool,
        target: Option<present::Target>,
        ctx: egui::Context,
    ) -> Result<WatchSession> {
        let frame = Arc::new(Mutex::new(None));
        let readback_only = Arc::new(AtomicBool::new(false));
        let gain = Arc::new(AtomicU32::new(gain.to_bits()));
        let need_key = Arc::new(AtomicBool::new(true));
        let stats = Arc::new(DecodeStats::default());

        let start_outputs = {
            let (frame, ctx, readback_only) = (frame.clone(), ctx.clone(), readback_only.clone());
            let (need_key, stats) = (need_key.clone(), stats.clone());
            let gain = gain.clone();
            Box::new(move |conn: &iroh::endpoint::Connection, audio: bool| {
                let (tx, rx) = sync_channel::<EncodedFrame>(8);
                let _ = decoder::spawn(
                    rx,
                    frame,
                    ctx,
                    need_key,
                    stats,
                    hardware_decode,
                    target,
                    readback_only,
                );
                let audio_handle = if audio {
                    match crate::audio::playback::start(&Handle::current(), conn.clone(), gain) {
                        Ok(a) => Some(Box::new(a) as Box<dyn Send>),
                        Err(e) => {
                            warn!("áudio do espectador: {e:#}");
                            None
                        }
                    }
                } else {
                    None
                };
                (tx, audio_handle)
            })
        };

        let notify = {
            let ctx = ctx.clone();
            Arc::new(move || {
                ctx.request_repaint();
            })
        };

        let inner = ViewerSession::start(
            rt,
            ticket,
            identity,
            name,
            env!("CARGO_PKG_VERSION"),
            decoder::supported_codecs(hardware_decode),
            need_key,
            stats,
            notify,
            start_outputs,
        )?;

        let status = inner.status.clone();
        let chat = inner.chat.clone();
        Ok(WatchSession {
            inner,
            status,
            frame,
            gain,
            readback_only,
            chat,
        })
    }

    pub fn request_quality(&self, preset: livescreen_core::preset::Preset) {
        self.inner.request_quality(preset);
    }

    pub fn send_chat(&self, text: &str) {
        self.inner.send_chat(text);
    }

    pub fn donate(&self, amount: u32, text: &str) {
        self.inner.donate(amount, text);
    }

    /// Avisa o host que a janela foi minimizada/restaurada: escondido, ele para de mandar vídeo.
    pub fn set_visible(&self, v: bool) {
        self.inner.set_visible(v);
    }

    /// A UI não conseguiu importar a textura compartilhada: o decoder passa ao readback.
    pub fn disable_zero_copy(&self) {
        self.readback_only.store(true, Ordering::Relaxed);
    }

    pub fn set_gain(&self, g: f32) {
        self.gain.store(g.to_bits(), Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.inner.stop();
    }
}
