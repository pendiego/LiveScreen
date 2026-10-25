use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::runtime::Handle;
use tracing::warn;

pub use livescreen_core::viewer::WatchStatus;
use livescreen_core::protocol::EncodedFrame;
use livescreen_core::viewer::{DecodeStats, ViewerSession};

use crate::video::decoder::{self, RgbaFrame};

pub struct WatchSession {
    inner: ViewerSession,
    pub status: Arc<Mutex<WatchStatus>>,
    pub frame: Arc<Mutex<Option<RgbaFrame>>>,
    gain: Arc<AtomicU32>,
}

impl WatchSession {
    pub fn start(
        rt: &Handle,
        ticket: &str,
        name: &str,
        gain: f32,
        hardware_decode: bool,
        ctx: egui::Context,
    ) -> Result<WatchSession> {
        let frame = Arc::new(Mutex::new(None));
        let gain = Arc::new(AtomicU32::new(gain.to_bits()));
        let need_key = Arc::new(AtomicBool::new(true));
        let stats = Arc::new(DecodeStats::default());

        let start_outputs = {
            let (frame, ctx) = (frame.clone(), ctx.clone());
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
            name,
            env!("CARGO_PKG_VERSION"),
            need_key,
            stats,
            notify,
            start_outputs,
        )?;

        let status = inner.status.clone();
        Ok(WatchSession {
            inner,
            status,
            frame,
            gain,
        })
    }

    pub fn request_quality(&self, preset: livescreen_core::preset::Preset) {
        self.inner.request_quality(preset);
    }

    pub fn set_gain(&self, g: f32) {
        self.gain.store(g.to_bits(), Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.inner.stop();
    }
}
