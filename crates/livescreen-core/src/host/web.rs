//! Uma sessão Rtc e um socket UDP por navegador. O host não tem cliente TURN:
//! o navegador usa o TURN dele, e o host anuncia os candidatos host e srflx.
//! Sinalização sem trickle.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, Once, Weak};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use crate::protocol::sanitize;
use crate::chat::{ChatLimiter, ChatLine};
use crate::protocol::ViewerMsg;
use str0m::channel::ChannelId;
use crate::web::{
    SignalIn, SignalOut, host_ws_url, sdp_fingerprint, valid_peer_id, verify_web_proof,
};
use ring::rand::SecureRandom;
use str0m::change::SdpOffer;
use str0m::media::{Frequency, MediaTime, Mid, Pt};
use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Event, IceConnectionState, Input, Output, Rtc, RtcConfig};
use tokio::sync::{broadcast, mpsc};
use tokio_websockets::{ClientBuilder, Message};

use super::{CountGuard, Peer, PeerConn, Shared, WebState, admit, rtt_congested};

const WEB_BASE: &str = "https://livescreen.pendiego.com";
const STUN_SERVER: &str = "stun.cloudflare.com:3478";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_SDP: usize = 32 * 1024;
/// Tamanho máximo, em bytes, de uma mensagem do canal de chat.
const MAX_WEB_CHAT: usize = 4096;

fn base_url() -> String {
    std::env::var("LIVESCREEN_WEB_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| WEB_BASE.to_string())
}

static INIT_CRYPTO: Once = Once::new();

pub(super) fn init_crypto() {
    INIT_CRYPTO.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
        str0m::crypto::from_feature_flags().install_process_default();
    });
}

#[derive(Default)]
pub(crate) struct WebPeer {
    pub(super) stop: tokio::sync::Notify,
    pub(super) reason: Mutex<Option<String>>,
    pub(super) rtt_ms: AtomicU32,
}

impl super::Host {
    pub fn web_link(&self) -> String {
        init_crypto();
        if !self.shared.web_started.swap(true, Ordering::SeqCst) {
            self.rt.spawn(run(self.shared.clone()));
        }
        crate::web::viewer_url(
            &base_url(),
            &self.shared.web_secret,
            &*self.shared.room_key.lock().unwrap(),
        )
    }
}

async fn run(sh: Arc<Shared>) {
    let mut shutdown = sh.shutdown.clone();
    let backoff_secs = [1, 2, 4, 8, 16, 30];
    let mut retry_idx = 0;

    // canal e mapa duram a transmissão inteira: peers abertos sobrevivem à queda da sinalização
    let (tx, mut rx) = mpsc::channel::<SignalOut>(64);
    let mut peers: HashMap<String, Weak<WebPeer>> = HashMap::new();
    while !*shutdown.borrow() {
        let base = base_url();
        let ws_url = host_ws_url(&base, &sh.web_secret);
        let uri = match ws_url.parse::<http::Uri>() {
            Ok(u) => u,
            Err(e) => {
                let err_msg = format!("URL inválida: {e}");
                sh.status.lock().unwrap().web = WebState::Retrying(err_msg);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let conn_res = ClientBuilder::from_uri(uri).connect().await;
        let mut ws = match conn_res {
            Ok((ws, _resp)) => ws,
            Err(e) => {
                let err_msg = format!("{e:#}");
                sh.status.lock().unwrap().web = WebState::Retrying(err_msg);
                let delay = Duration::from_secs(backoff_secs[retry_idx.min(backoff_secs.len() - 1)]);
                retry_idx += 1;
                tokio::select! {
                    _ = tokio::time::sleep(delay) => continue,
                    _ = shutdown.changed() => break,
                }
            }
        };

        let host_msg = SignalOut::Host {
            secret: hex::encode(sh.web_secret),
        };
        let host_json = match serde_json::to_string(&host_msg) {
            Ok(j) => j,
            Err(_) => break,
        };
        if ws.send(Message::text(host_json)).await.is_err() {
            sh.status.lock().unwrap().web = WebState::Retrying("falha enviando host".into());
            let delay = Duration::from_secs(backoff_secs[retry_idx.min(backoff_secs.len() - 1)]);
            retry_idx += 1;
            tokio::select! {
                _ = tokio::time::sleep(delay) => continue,
                _ = shutdown.changed() => break,
            }
        }

        let ready_res = tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(msg_res) = ws.next().await {
                let msg = msg_res?;
                if let Some(text) = msg.as_text() {
                    if let Ok(SignalIn::Ready) = serde_json::from_str::<SignalIn>(text) {
                        return Ok(());
                    }
                }
            }
            Err(anyhow!("conexão fechada pelo servidor"))
        })
        .await;

        if ready_res.is_err() || ready_res.unwrap().is_err() {
            sh.status.lock().unwrap().web = WebState::Retrying("timeout aguardando ready".into());
            let delay = Duration::from_secs(backoff_secs[retry_idx.min(backoff_secs.len() - 1)]);
            retry_idx += 1;
            tokio::select! {
                _ = tokio::time::sleep(delay) => continue,
                _ = shutdown.changed() => break,
            }
        }

        sh.status.lock().unwrap().web = WebState::Online;
        let online_at = Instant::now();
        let mut last_pong = Instant::now();
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
        ping_interval.tick().await;

        loop {
            peers.retain(|_, w| w.upgrade().is_some());

            tokio::select! {
                Some(out) = rx.recv() => {
                    if let Ok(json) = serde_json::to_string(&out) {
                        if ws.send(Message::text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                _ = ping_interval.tick() => {
                    if last_pong.elapsed() > Duration::from_secs(75) {
                        break;
                    }
                    if let Ok(json) = serde_json::to_string(&SignalOut::Ping) {
                        if ws.send(Message::text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                msg_opt = ws.next() => {
                    let Some(msg_res) = msg_opt else { break };
                    let Ok(msg) = msg_res else { break };
                    if let Some(text) = msg.as_text() {
                        match serde_json::from_str::<SignalIn>(text) {
                            Ok(SignalIn::Ready) => {}
                            Ok(SignalIn::Pong) => last_pong = Instant::now(),
                            Ok(SignalIn::Left { id }) => {
                                if let Some(w) = peers.remove(&id) {
                                    if let Some(p) = w.upgrade() {
                                        p.stop.notify_one();
                                    }
                                }
                            }
                            Ok(SignalIn::Offer { id, sdp, proof, name }) => {
                                if !valid_peer_id(&id) || sdp.len() > MAX_SDP {
                                    continue;
                                }
                                if peers.get(&id).and_then(|w| w.upgrade()).is_some()
                                    || sh.conns.lock().unwrap().contains_key(&id)
                                {
                                    continue;
                                }
                                let key = *sh.room_key.lock().unwrap();
                                let proof_ok = sdp_fingerprint(&sdp)
                                    .is_some_and(|fp| verify_web_proof(&key, fp, &proof));
                                if let Err(reason) = admit(false, false, &proof, proof_ok) {
                                    let _ = tx.try_send(SignalOut::Reject {
                                        id,
                                        reason: reason.to_string(),
                                    });
                                    continue;
                                }
                                let Ok(permit) = sh.pending.clone().try_acquire_owned() else {
                                    let _ = tx.try_send(SignalOut::Reject {
                                        id,
                                        reason: "muitas conexões ao mesmo tempo, tente de novo".to_string(),
                                    });
                                    continue;
                                };
                                if sh.viewers.fetch_add(1, Ordering::SeqCst) >= sh.max_viewers.load(Ordering::SeqCst) {
                                    sh.viewers.fetch_sub(1, Ordering::SeqCst);
                                    let _ = tx.try_send(SignalOut::Reject {
                                        id,
                                        reason: "sala cheia".to_string(),
                                    });
                                    continue;
                                }
                                let peer_obj = Arc::new(WebPeer::default());
                                peers.insert(id.clone(), Arc::downgrade(&peer_obj));
                                let clean_name = sanitize(&name, 32);
                                tokio::spawn(peer(
                                    sh.clone(),
                                    id,
                                    sdp,
                                    clean_name,
                                    permit,
                                    tx.clone(),
                                    peer_obj,
                                ));
                            }
                            Err(e) => {
                                tracing::warn!("sinalização: json inválido ({e:#}): {text}");
                            }
                        }
                    }
                }
                _ = shutdown.changed() => {
                    let drain_deadline = tokio::time::Instant::now() + Duration::from_millis(500);
                    while let Ok(Some(out)) = tokio::time::timeout_at(drain_deadline, rx.recv()).await {
                        if let Ok(json) = serde_json::to_string(&out) {
                            let _ = ws.send(Message::text(json)).await;
                        }
                    }
                    break;
                }
            }
        }

        if *shutdown.borrow() {
            break;
        }
        // só zera o backoff se a sessão durou; senão um servidor que derruba logo após o ready vira laço quente
        if online_at.elapsed() >= Duration::from_secs(60) {
            retry_idx = 0;
        }
        sh.status.lock().unwrap().web = WebState::Retrying("conexão com o servidor do link caiu".into());
        let delay = Duration::from_secs(backoff_secs[retry_idx.min(backoff_secs.len() - 1)]);
        retry_idx += 1;
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown.changed() => break,
        }
    }
}

async fn drain_outputs(
    rtc: &mut Rtc,
    sock: &tokio::net::UdpSocket,
    events: &mut Vec<Event>,
) -> Result<Instant, anyhow::Error> {
    loop {
        match rtc.poll_output()? {
            Output::Timeout(t) => return Ok(t),
            Output::Transmit(t) => {
                let _ = sock.send_to(&t.contents, t.destination).await;
            }
            Output::Event(e) => {
                events.push(e);
            }
        }
    }
}

async fn peer(
    sh: Arc<Shared>,
    id: String,
    sdp: String,
    name: String,
    permit: tokio::sync::OwnedSemaphorePermit,
    tx: mpsc::Sender<SignalOut>,
    peer: Arc<WebPeer>,
) {
    let mut shutdown = sh.shutdown.clone();
    let mut guard = CountGuard(&sh, None);
    let mut permit_opt = Some(permit);

    let local_ip = match std::net::UdpSocket::bind("0.0.0.0:0").and_then(|s| {
        s.connect("1.1.1.1:80")?;
        s.local_addr()
    }) {
        Ok(addr) => addr.ip(),
        Err(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
    };

    let sock = match tokio::net::UdpSocket::bind(SocketAddr::new(local_ip, 0)).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("peer {id}: erro bind UDP: {e:#}");
            let _ = tx.try_send(SignalOut::Reject {
                id,
                reason: "erro interno de rede".into(),
            });
            return;
        }
    };
    let local_addr = match sock.local_addr() {
        Ok(a) => a,
        Err(_) => {
            let _ = tx.try_send(SignalOut::Reject {
                id,
                reason: "erro interno de rede".into(),
            });
            return;
        }
    };

    let mut mapped = None;
    let mut txid = [0u8; 12];
    if ring::rand::SystemRandom::new().fill(&mut txid).is_ok() {
        let req = crate::web::stun_binding_request(&txid);
        if let Ok(mut addrs) = tokio::net::lookup_host(STUN_SERVER).await {
            if let Some(target) = addrs.next() {
                let _ = sock.send_to(&req, target).await;
                let mut sbuf = [0u8; 512];
                let stun_deadline = Instant::now() + Duration::from_secs(2);
                while Instant::now() < stun_deadline {
                    let left = stun_deadline.saturating_duration_since(Instant::now());
                    match tokio::time::timeout(left, sock.recv_from(&mut sbuf)).await {
                        Ok(Ok((n, _src))) => {
                            if let Some(addr) = crate::web::stun_mapped_addr(&sbuf[..n], &txid) {
                                mapped = Some(addr);
                                break;
                            }
                        }
                        _ => break,
                    }
                }
            }
        }
    }

    let mut rtc = RtcConfig::new()
        .clear_codecs()
        .enable_h264(true)
        .enable_opus(true, false)
        .set_stats_interval(Some(Duration::from_secs(1)))
        .build(Instant::now());

    if let Ok(c) = Candidate::host(local_addr, "udp") {
        rtc.add_local_candidate(c);
    }
    if let Some(m) = mapped {
        if let Ok(c) = Candidate::server_reflexive(m, local_addr, "udp") {
            rtc.add_local_candidate(c);
        }
    }

    let offer = match SdpOffer::from_sdp_string(&sdp) {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!("peer {id}: SDP inválido: {e:#}");
            let _ = tx.try_send(SignalOut::Reject {
                id,
                reason: "navegador incompatível".into(),
            });
            return;
        }
    };

    let answer = match rtc.sdp_api().accept_offer(offer) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!("peer {id}: erro aceitando oferta SDP: {e:#}");
            let _ = tx.try_send(SignalOut::Reject {
                id,
                reason: "navegador incompatível".into(),
            });
            return;
        }
    };

    let _ = tx.try_send(SignalOut::Answer {
        id: id.clone(),
        sdp: answer.to_sdp_string(),
    });

    let mut vrx = sh.video_tx.subscribe();
    let mut arx = sh.audio_tx.subscribe();

    let mut video_mid: Option<Mid> = None;
    let mut video_pt: Option<Pt> = None;
    let mut audio_mid: Option<Mid> = None;
    let mut audio_pt: Option<Pt> = None;
    let mut audio_clock_rate: Option<Frequency> = None;

    let mut connected = false;
    let mut waiting_key = true;
    let mut last_kf = Instant::now() - Duration::from_secs(2);
    let mut min_rtt: Option<Duration> = None;
    let mut close_reason: Option<String> = None;
    let mut stop_requested = false;

    let connect_deadline = Instant::now() + CONNECT_TIMEOUT;

    let chat_id = sh.next_chat_id.fetch_add(1, Ordering::Relaxed);
    let mut crx = sh.chat_tx.subscribe();
    let mut chat_ch: Option<ChannelId> = None;
    let mut chat_limiter = ChatLimiter::default();
    let mut flush_now = false;

    let mut events = Vec::new();
    let mut next_timeout = match drain_outputs(&mut rtc, &sock, &mut events).await {
        Ok(t) => t,
        Err(_) => return,
    };

    let process_events = |events: &mut Vec<Event>,
                          rtc: &mut Rtc,
                          sh: &Arc<Shared>,
                          peer: &Arc<WebPeer>,
                          id: &str,
                          name: &str,
                          guard: &mut CountGuard,
                          permit_opt: &mut Option<tokio::sync::OwnedSemaphorePermit>,
                          video_mid: &mut Option<Mid>,
                          video_pt: &mut Option<Pt>,
                          audio_mid: &mut Option<Mid>,
                          audio_pt: &mut Option<Pt>,
                          audio_clock_rate: &mut Option<Frequency>,
                          connected: &mut bool,
                          min_rtt: &mut Option<Duration>,
                          close_reason: &mut Option<String>,
                          stop_requested: &mut bool,
                          chat_ch: &mut Option<ChannelId>,
                          chat_limiter: &mut ChatLimiter,
                          chat_id: u32,
                          flush_now: &mut bool| {
        for e in events.drain(..) {
            match e {
                Event::MediaAdded(m) => {
                    if m.kind.is_video() {
                        let found_pt = rtc.writer(m.mid).and_then(|w| {
                            w.payload_params().find(|p| {
                                let spec = p.spec();
                                spec.codec == str0m::format::Codec::H264
                                    && spec.format.packetization_mode == Some(1)
                                    && spec.format.profile_level_id.map(|p| p >> 16) == Some(0x64)
                            }).map(|p| p.pt())
                        });
                        if let Some(pt) = found_pt {
                            *video_mid = Some(m.mid);
                            *video_pt = Some(pt);
                        } else {
                            *close_reason = Some("este navegador não decodifica H.264 High: use Chrome, Edge ou Safari".into());
                            *stop_requested = true;
                            break;
                        }
                    } else if m.kind.is_audio() {
                        if let Some(w) = rtc.writer(m.mid) {
                            if let Some(p) = w.payload_params().find(|p| p.spec().codec == str0m::format::Codec::Opus) {
                                *audio_mid = Some(m.mid);
                                *audio_pt = Some(p.pt());
                                *audio_clock_rate = Some(p.spec().clock_rate);
                            }
                        }
                    }
                }
                Event::Connected => {
                    *connected = true;
                    sh.conns.lock().unwrap().insert(
                        id.to_string(),
                        Peer {
                            conn: PeerConn::Web(peer.clone()),
                            name: name.to_string(),
                            decoders: vec![],
                        },
                    );
                    guard.1 = Some(id.to_string());
                    sh.codec_dirty.store(true, Ordering::Release);
                    sh.request_keyframe();
                    drop(permit_opt.take());
                }
                Event::KeyframeRequest(_) => {
                    sh.request_keyframe();
                }
                Event::MediaEgressStats(s) => {
                    if Some(s.mid) == *video_mid {
                        if let Some(rtt) = s.rtt {
                            peer.rtt_ms.store(rtt.as_millis() as u32, Ordering::Relaxed);
                            *min_rtt = Some(min_rtt.map_or(rtt, |m| m.min(rtt)));
                            if let Some(min) = *min_rtt {
                                if rtt_congested(rtt, min) {
                                    sh.congested.store(true, Ordering::Relaxed);
                                }
                            }
                        }
                        if s.loss.is_some_and(|l| l > 0.05) {
                            sh.congested.store(true, Ordering::Relaxed);
                        }
                    }
                }
                Event::IceConnectionStateChange(IceConnectionState::Disconnected) | Event::Closed => {
                    *stop_requested = true;
                    break;
                }
                Event::ChannelOpen(cid, label) if label == "chat" => {
                    *chat_ch = Some(cid);
                    if let Some(mut ch) = rtc.channel(cid) {
                        let _ = ch.write(false, serde_json::json!({"type":"chat_ready","chat_id":chat_id}).to_string().as_bytes());
                    }
                    *flush_now = true;
                }
                Event::ChannelData(d) if Some(d.id) == *chat_ch => {
                    if d.data.len() <= MAX_WEB_CHAT && chat_limiter.allow(Instant::now()) {
                        if let Ok(ViewerMsg::Chat { text }) = serde_json::from_slice(&d.data) {
                            if let Some(l) = ChatLine::from_viewer(chat_id, name, false, &text) {
                                sh.post_chat(l);
                            }
                        }
                    }
                }
                Event::ChannelClose(cid) if Some(cid) == *chat_ch => {
                    *chat_ch = None;
                }
                _ => {}
            }
        }
    };

    process_events(
        &mut events,
        &mut rtc,
        &sh,
        &peer,
        &id,
        &name,
        &mut guard,
        &mut permit_opt,
        &mut video_mid,
        &mut video_pt,
        &mut audio_mid,
        &mut audio_pt,
        &mut audio_clock_rate,
        &mut connected,
        &mut min_rtt,
        &mut close_reason,
        &mut stop_requested,
        &mut chat_ch,
        &mut chat_limiter,
        chat_id,
        &mut flush_now,
    );

    let mut buf = [0u8; 2000];

    loop {
        if stop_requested {
            break;
        }

        if flush_now {
            flush_now = false;
            next_timeout = Instant::now();
        }
        let sleep_dur = next_timeout.saturating_duration_since(Instant::now());
        let conn_timeout_fut = async {
            if !connected {
                let left = connect_deadline.saturating_duration_since(Instant::now());
                tokio::time::sleep(left).await;
                true
            } else {
                std::future::pending::<bool>().await
            }
        };

        tokio::select! {
            _ = conn_timeout_fut => {
                tracing::warn!("peer {id}: ICE sem conexão em {CONNECT_TIMEOUT:?}");
                close_reason = Some("não foi possível conectar: a rede bloqueou a conexão direta".into());
                break;
            }
            recv_res = sock.recv_from(&mut buf) => {
                match recv_res {
                    Ok((n, src)) => {
                        let now = Instant::now();
                        if let Ok(recv) = Receive::new(Protocol::Udp, src, local_addr, &buf[..n]) {
                            if rtc.handle_input(Input::Receive(now, recv)).is_ok() {
                                next_timeout = match drain_outputs(&mut rtc, &sock, &mut events).await {
                                    Ok(t) => t,
                                    Err(_) => break,
                                };
                                process_events(
                                    &mut events,
                                    &mut rtc,
                                    &sh,
                                    &peer,
                                    &id,
                                    &name,
                                    &mut guard,
                                    &mut permit_opt,
                                    &mut video_mid,
                                    &mut video_pt,
                                    &mut audio_mid,
                                    &mut audio_pt,
                                    &mut audio_clock_rate,
                                    &mut connected,
                                    &mut min_rtt,
                                    &mut close_reason,
                                    &mut stop_requested,
                                    &mut chat_ch,
                                    &mut chat_limiter,
                                    chat_id,
                                    &mut flush_now,
                                );
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            _ = tokio::time::sleep(sleep_dur) => {
                let now = Instant::now();
                if rtc.handle_input(Input::Timeout(now)).is_ok() {
                    next_timeout = match drain_outputs(&mut rtc, &sock, &mut events).await {
                        Ok(t) => t,
                        Err(_) => break,
                    };
                    process_events(
                        &mut events,
                        &mut rtc,
                        &sh,
                        &peer,
                        &id,
                        &name,
                        &mut guard,
                        &mut permit_opt,
                        &mut video_mid,
                        &mut video_pt,
                        &mut audio_mid,
                        &mut audio_pt,
                        &mut audio_clock_rate,
                        &mut connected,
                        &mut min_rtt,
                        &mut close_reason,
                        &mut stop_requested,
                        &mut chat_ch,
                        &mut chat_limiter,
                        chat_id,
                        &mut flush_now,
                    );
                }
            }
            r = vrx.recv() => {
                match r {
                    Ok(f) => {
                        if !connected {
                            continue;
                        }
                        let (Some(mid), Some(pt)) = (video_mid, video_pt) else {
                            continue;
                        };
                        if f.codec != crate::protocol::Codec::H264 {
                            waiting_key = true;
                            if last_kf.elapsed() >= Duration::from_secs(1) {
                                last_kf = Instant::now();
                                sh.request_keyframe();
                            }
                            continue;
                        }
                        if waiting_key && !f.keyframe {
                            if last_kf.elapsed() >= Duration::from_secs(1) {
                                last_kf = Instant::now();
                                sh.request_keyframe();
                            }
                            continue;
                        }
                        waiting_key = false;
                        // shortcut: uma cópia por espectador web por quadro (str0m pede Arc<[u8]>); trocar EncodedFrame.data para Arc<[u8]> se aparecer no perfil
                        let data = Arc::<[u8]>::from(&f.data[..]);
                        let rtp_time = MediaTime::from_90khz(f.pts_us * 9 / 100);
                        if let Some(w) = rtc.writer(mid) {
                            let _ = w.write(pt, Instant::now(), rtp_time, data);
                        }
                        next_timeout = match drain_outputs(&mut rtc, &sock, &mut events).await {
                            Ok(t) => t,
                            Err(_) => break,
                        };
                        process_events(
                            &mut events,
                            &mut rtc,
                            &sh,
                            &peer,
                            &id,
                            &name,
                            &mut guard,
                            &mut permit_opt,
                            &mut video_mid,
                            &mut video_pt,
                            &mut audio_mid,
                            &mut audio_pt,
                            &mut audio_clock_rate,
                            &mut connected,
                            &mut min_rtt,
                            &mut close_reason,
                            &mut stop_requested,
                            &mut chat_ch,
                            &mut chat_limiter,
                            chat_id,
                            &mut flush_now,
                        );
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        waiting_key = true;
                        sh.congested.store(true, Ordering::Relaxed);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            r = arx.recv() => {
                match r {
                    Ok(pkt) => {
                        if !connected {
                            continue;
                        }
                        let (Some(mid), Some(pt), Some(freq)) = (audio_mid, audio_pt, audio_clock_rate) else {
                            continue;
                        };
                        if let Some((seq, opus)) = crate::protocol::parse_audio_datagram(&pkt) {
                            let rtp_time = MediaTime::new(seq as u64 * 960, freq);
                            let data = Arc::<[u8]>::from(opus);
                            if let Some(w) = rtc.writer(mid) {
                                let _ = w.write(pt, Instant::now(), rtp_time, data);
                            }
                            next_timeout = match drain_outputs(&mut rtc, &sock, &mut events).await {
                                Ok(t) => t,
                                Err(_) => break,
                            };
                            process_events(
                                &mut events,
                                &mut rtc,
                                &sh,
                                &peer,
                                &id,
                                &name,
                                &mut guard,
                                &mut permit_opt,
                                &mut video_mid,
                                &mut video_pt,
                                &mut audio_mid,
                                &mut audio_pt,
                                &mut audio_clock_rate,
                                &mut connected,
                                &mut min_rtt,
                                &mut close_reason,
                                &mut stop_requested,
                                &mut chat_ch,
                                &mut chat_limiter,
                                chat_id,
                                &mut flush_now,
                            );
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            r = crx.recv() => if let (Ok(m), Some(cid)) = (r, chat_ch) {
                if let (Ok(json), Some(mut ch)) = (serde_json::to_string(&*m), rtc.channel(cid)) {
                    let _ = ch.write(false, json.as_bytes());
                }
                flush_now = true;
            },
            _ = peer.stop.notified() => {
                let r = peer.reason.lock().unwrap().clone();
                close_reason = r;
                break;
            }
            _ = shutdown.changed() => {
                close_reason = Some("a transmissão terminou".into());
                break;
            }
        }
    }

    if let Some(reason) = close_reason {
        let _ = tx.try_send(SignalOut::Close {
            id: id.clone(),
            reason,
        });
    }

    rtc.disconnect();
    let _ = drain_outputs(&mut rtc, &sock, &mut Vec::new()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url_default() {
        assert_eq!(base_url(), WEB_BASE);
    }
}
