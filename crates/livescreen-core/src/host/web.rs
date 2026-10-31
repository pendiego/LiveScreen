//! Uma sessão Rtc e um socket UDP por navegador (mais um IPv6, se houver). Sem TURN:
//! o host anuncia host e srflx (um por mapeamento de NAT distinto, v4 e v6), prevê as
//! próximas portas do próprio NAT (regra do delta do Radmin VPN) e as do NAT do navegador,
//! que o str0m testa como candidatos remotos para abrir o caminho direto. Sinalização sem trickle.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
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
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc};
use tokio_websockets::{ClientBuilder, Message};

use super::{CountGuard, Peer, PeerConn, Shared, WebState, admit, rtt_congested};

const WEB_BASE: &str = "https://livescreen.pendiego.com";
/// Destinos com IPs distintos: o NAT só cria mapeamento novo para destino novo. A ordem é a ordem de envio.
const STUN_SERVERS: [&str; 5] = [
    "stun.cloudflare.com:3478",
    "stun.l.google.com:19302",
    "global.stun.twilio.com:3478",
    "stun.nextcloud.com:3478",
    "stun.sipgate.net:3478",
];
const STUN_TIMEOUT: Duration = Duration::from_millis(400);
/// Portas previstas à frente da última do NAT do host.
const HOST_PREDICT: i32 = 8;
/// Portas previstas à frente do srflx do navegador (~35 pares ICE, abaixo do limite de 100).
const BROWSER_PREDICT: u16 = 32;
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
    pub(super) pair_logged: std::sync::atomic::AtomicBool,
    pub(super) rtt_ms: AtomicU32,
    pub(super) h264_baseline: std::sync::atomic::AtomicBool,
    /// Aba do navegador escondida (sem PiP): o host para de mandar vídeo, o áudio segue.
    pub(super) hidden: std::sync::atomic::AtomicBool,
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

/// IPv6 global do host (rota até a Cloudflare); `None` sem IPv6, link-local ou ULA.
fn local_v6() -> Option<Ipv6Addr> {
    let s = std::net::UdpSocket::bind("[::]:0").ok()?;
    s.connect("[2606:4700:4700::1111]:80").ok()?;
    match s.local_addr().ok()?.ip() {
        IpAddr::V6(ip)
            if !ip.is_loopback()
                && ip.segments()[0] & 0xffc0 != 0xfe80
                && ip.segments()[0] & 0xfe00 != 0xfc00 =>
        {
            Some(ip)
        }
        _ => None,
    }
}

/// IPv4 da rota padrão (a que sai para a internet); UNSPECIFIED se não houver.
fn default_v4() -> IpAddr {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("1.1.1.1:80")?;
            s.local_addr()
        })
        .map(|a| a.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
}

/// Nomes de adaptador virtual/VPN (Windows reporta Radmin e TAP como Ethernet).
fn looks_virtual(names: &[&str]) -> bool {
    const MARKS: [&str; 13] = [
        "vpn", "tap-", "tunnel", "wireguard", "virtual", "radmin", "hamachi", "zerotier",
        "tailscale", "nordlynx", "vethernet", "hyper-v", "wsl",
    ];
    names.iter().any(|n| {
        let n = n.to_lowercase();
        MARKS.iter().any(|m| n.contains(m))
    })
}

/// Placa física: ligada, com gateway, tipo Ethernet/Wi-Fi/celular e nome que não é de VPN.
fn is_physical(i: &netdev::Interface) -> bool {
    use netdev::interface::types::InterfaceType as T;
    i.is_up()
        && !i.is_loopback()
        && i.gateway.is_some()
        && matches!(i.if_type, T::Ethernet | T::GigabitEthernet | T::Wireless80211 | T::Wwanpp | T::Wwanpp2)
        && !looks_virtual(&[
            &i.name,
            i.friendly_name.as_deref().unwrap_or(""),
            i.description.as_deref().unwrap_or(""),
        ])
}

/// Rota padrão por VPN: nome da VPN e IPv4 da placa física para um socket extra.
struct RouteInfo {
    vpn: Option<String>,
    physical: Option<Ipv4Addr>,
}

fn route_info(default_ip: IpAddr) -> RouteInfo {
    let none = RouteInfo { vpn: None, physical: None };
    let ifaces = netdev::get_interfaces();
    let Some(owner) = ifaces
        .iter()
        .find(|i| i.ipv4.iter().any(|n| IpAddr::V4(n.addr()) == default_ip))
    else {
        return none;
    };
    if is_physical(owner) {
        return none;
    }
    let physical = ifaces.iter().filter(|i| is_physical(i)).find_map(|i| {
        i.ipv4.iter().map(|n| n.addr()).find(|a| !(a.octets()[0] == 169 && a.octets()[1] == 254))
    });
    RouteInfo {
        vpn: Some(owner.friendly_name.clone().unwrap_or_else(|| owner.name.clone())),
        physical,
    }
}

/// Sockets de um navegador: rota padrão (v4), placa física quando a rota padrão é VPN (v4) e IPv6.
struct Socks {
    v4: UdpSocket,
    v4_addr: SocketAddr,
    alt: Option<(UdpSocket, SocketAddr)>,
    v6: Option<(UdpSocket, SocketAddr)>,
}

impl Socks {
    /// Espera um datagrama em qualquer socket; devolve (bytes, origem, endereço local que recebeu).
    /// ICMP "port unreachable" vira ConnectionReset no Windows e não pode derrubar o peer.
    async fn recv(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr, SocketAddr)> {
        loop {
            tokio::select! {
                r = self.v4.readable() => r?,
                r = async { self.alt.as_ref().unwrap().0.readable().await }, if self.alt.is_some() => r?,
                r = async { self.v6.as_ref().unwrap().0.readable().await }, if self.v6.is_some() => r?,
            }
            let all = [
                Some((&self.v4, self.v4_addr)),
                self.alt.as_ref().map(|(s, a)| (s, *a)),
                self.v6.as_ref().map(|(s, a)| (s, *a)),
            ];
            for (s, local) in all.into_iter().flatten() {
                match s.try_recv_from(buf) {
                    Ok((n, src)) => return Ok((n, src, local)),
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::ConnectionReset
                        ) => {}
                    Err(e) => return Err(e),
                }
            }
        }
    }

    /// Envia pelo socket cujo endereço é `t.source`; sem correspondência, pela família do destino.
    async fn send(&self, t: &str0m::net::Transmit) {
        let s = if t.source == self.v4_addr {
            Some(&self.v4)
        } else if let Some((s, _)) = self.alt.as_ref().filter(|(_, a)| *a == t.source) {
            Some(s)
        } else if let Some((s, _)) = self.v6.as_ref().filter(|(_, a)| *a == t.source) {
            Some(s)
        } else if t.destination.is_ipv6() {
            self.v6.as_ref().map(|(s, _)| s)
        } else {
            Some(&self.v4)
        };
        if let Some(s) = s {
            let _ = s.send_to(&t.contents, t.destination).await;
        }
    }
}

/// Consulta os STUN em sequência no mesmo socket; o índice segue STUN_SERVERS (None = sem resposta).
async fn stun_mappings(sock: &UdpSocket) -> Vec<Option<SocketAddr>> {
    let resolved = futures_util::future::join_all(
        STUN_SERVERS.iter().map(|s| tokio::net::lookup_host(*s)),
    )
    .await;
    let rng = ring::rand::SystemRandom::new();
    let mut out = Vec::with_capacity(STUN_SERVERS.len());
    // sequencial: o delta depende da ordem em que o NAT alocou as portas
    for r in resolved {
        let target = r.ok().and_then(|mut it| it.find(SocketAddr::is_ipv4));
        let mut txid = [0u8; 12];
        let Some(target) = target.filter(|_| rng.fill(&mut txid).is_ok()) else {
            out.push(None);
            continue;
        };
        let _ = sock
            .send_to(&crate::web::stun_binding_request(&txid), target)
            .await;
        let deadline = tokio::time::Instant::now() + STUN_TIMEOUT;
        let mut sbuf = [0u8; 512];
        let mut found = None;
        loop {
            match tokio::time::timeout_at(deadline, sock.recv_from(&mut sbuf)).await {
                Ok(Ok((n, _))) => {
                    if let Some(a) = crate::web::stun_mapped_addr(&sbuf[..n], &txid) {
                        found = Some(a);
                        break;
                    }
                }
                Ok(Err(e)) if e.kind() == std::io::ErrorKind::ConnectionReset => continue,
                _ => break,
            }
        }
        out.push(found);
    }
    out
}

/// Regra do Radmin: ≥4 deltas consecutivos, nenhum zero, moda com ≥50 % dos deltas; prevê as próximas 8 portas.
fn host_predictions(maps: &[Option<SocketAddr>]) -> Vec<SocketAddr> {
    let some: Vec<SocketAddr> = maps.iter().flatten().copied().collect();
    let Some(last) = some.last().copied() else {
        return Vec::new();
    };
    if some.iter().any(|a| a.ip() != last.ip()) {
        return Vec::new();
    }
    let deltas: Vec<i32> = some
        .windows(2)
        .map(|w| i32::from(w[1].port()) - i32::from(w[0].port()))
        .collect();
    if deltas.len() < 4 || deltas.contains(&0) {
        return Vec::new();
    }
    let mut counts: Vec<(i32, usize)> = Vec::new();
    let (mut best, mut best_n) = (0, 0);
    for &d in &deltas {
        let n = match counts.iter_mut().find(|(x, _)| *x == d) {
            Some(e) => {
                e.1 += 1;
                e.1
            }
            None => {
                counts.push((d, 1));
                1
            }
        };
        if n > best_n {
            (best, best_n) = (d, n);
        }
    }
    if best_n * 100 / deltas.len() < 50 {
        return Vec::new();
    }
    (1..=HOST_PREDICT)
        .map(|k| i32::from(last.port()) + best * k)
        .filter(|p| (1024..=65535).contains(p))
        .map(|p| SocketAddr::new(last.ip(), p as u16))
        .collect()
}

/// O NAT do celular costuma alocar portas em sequência: a porta que vai falar com o host vem logo depois da do STUN.
fn browser_predictions(sdp: &str) -> Vec<SocketAddr> {
    for l in sdp.lines() {
        let f: Vec<&str> = l.split_whitespace().collect();
        if f.len() < 8
            || !f[0].starts_with("a=candidate:")
            || !f[2].eq_ignore_ascii_case("udp")
            || f[6] != "typ"
            || f[7] != "srflx"
        {
            continue;
        }
        if let (Ok(IpAddr::V4(ip)), Ok(port)) = (f[4].parse::<IpAddr>(), f[5].parse::<u16>()) {
            return (1..=BROWSER_PREDICT)
                .filter_map(|k| port.checked_add(k))
                .map(|p| SocketAddr::new(IpAddr::V4(ip), p))
                .collect();
        }
    }
    Vec::new()
}

async fn run(sh: Arc<Shared>) {
    let mut shutdown = sh.shutdown.clone();
    let backoff_secs = [1, 2, 4, 8, 16, 30];
    let mut retry_idx = 0;

    // canal e mapa duram a transmissão inteira: peers abertos sobrevivem à queda da sinalização
    let (tx, mut rx) = mpsc::channel::<SignalOut>(64);
    let mut peers: HashMap<String, Weak<WebPeer>> = HashMap::new();
    while !*shutdown.borrow() {
        sh.status.lock().unwrap().web_vpn = route_info(default_v4()).vpn;
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
    socks: &Socks,
    events: &mut Vec<Event>,
) -> Result<Instant, anyhow::Error> {
    loop {
        match rtc.poll_output()? {
            Output::Timeout(t) => return Ok(t),
            Output::Transmit(t) => socks.send(&t).await,
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

    let local_ip = default_v4();
    let route = route_info(local_ip);
    sh.status.lock().unwrap().web_vpn = route.vpn.clone();

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

    let sock6 = match local_v6() {
        Some(ip) => UdpSocket::bind(SocketAddr::new(IpAddr::V6(ip), 0)).await.ok(),
        None => None,
    };
    let local6 = sock6.as_ref().and_then(|s| s.local_addr().ok());

    // Rota padrão por VPN: socket extra na placa física (modelo de host forte do Windows).
    let alt = match route.physical {
        Some(ip) => UdpSocket::bind(SocketAddr::new(IpAddr::V4(ip), 0)).await.ok(),
        None => None,
    };
    let alt_addr = alt.as_ref().and_then(|s| s.local_addr().ok());

    let (maps, alt_maps) = tokio::join!(stun_mappings(&sock), async {
        match &alt {
            Some(s) => stun_mappings(s).await,
            None => Vec::new(),
        }
    });
    let distinct_of = |maps: &[Option<SocketAddr>]| {
        let mut d: Vec<SocketAddr> = Vec::new();
        for m in maps.iter().flatten() {
            if !d.contains(m) {
                d.push(*m);
            }
        }
        d
    };
    let distinct = distinct_of(&maps);
    let alt_distinct = distinct_of(&alt_maps);
    let n_srflx = distinct.len() + alt_distinct.len();
    let host_pred = host_predictions(&maps);
    let alt_pred = host_predictions(&alt_maps);
    let n_host = host_pred.len() + alt_pred.len();

    let mut rtc = RtcConfig::new()
        .clear_codecs()
        .enable_h264(true)
        .enable_opus(true, false)
        .set_stats_interval(Some(Duration::from_secs(1)))
        .build(Instant::now());

    if let Ok(c) = Candidate::host(local_addr, "udp") {
        rtc.add_local_candidate(c);
    }
    if let Some(a) = local6 {
        if let Ok(c) = Candidate::host(a, "udp") {
            rtc.add_local_candidate(c);
        }
    }
    for m in distinct.iter().chain(host_pred.iter()) {
        if let Ok(c) = Candidate::server_reflexive(*m, local_addr, "udp") {
            rtc.add_local_candidate(c);
        }
    }
    if let Some(a) = alt_addr {
        if let Ok(c) = Candidate::host(a, "udp") {
            rtc.add_local_candidate(c);
        }
        for m in alt_distinct.iter().chain(alt_pred.iter()) {
            if let Ok(c) = Candidate::server_reflexive(*m, a, "udp") {
                rtc.add_local_candidate(c);
            }
        }
    }
    let socks = Socks {
        v4: sock,
        v4_addr: local_addr,
        alt: alt.zip(alt_addr),
        v6: sock6.zip(local6),
    };

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

    let browser_pred = browser_predictions(&sdp);
    let n_browser = browser_pred.len();
    for a in browser_pred {
        if let Ok(c) = Candidate::server_reflexive(a, a, "udp") {
            rtc.add_remote_candidate(c);
        }
    }
    tracing::info!(
        "peer {id}: ipv6={} srflx={} previstos_host={} previstos_navegador={} vpn={:?} placa_fisica={:?}",
        local6.is_some(),
        n_srflx,
        n_host,
        n_browser,
        route.vpn,
        alt_addr
    );
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
    let mut next_timeout = match drain_outputs(&mut rtc, &socks, &mut events).await {
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
                        let mut pick = |high: bool| rtc.writer(m.mid).and_then(|w| {
                            w.payload_params().find(|p| {
                                let spec = p.spec();
                                let k = spec.format.profile_level_id.map(|p| p >> 16);
                                spec.codec == str0m::format::Codec::H264
                                    && spec.format.packetization_mode == Some(1)
                                    && if high { k == Some(0x64) } else { matches!(k, Some(0x42 | 0x4d)) }
                            }).map(|p| p.pt())
                        });
                        let found = pick(true).map(|pt| (pt, false)).or_else(|| pick(false).map(|pt| (pt, true)));
                        if let Some((pt, baseline)) = found {
                            *video_mid = Some(m.mid);
                            *video_pt = Some(pt);
                            peer.h264_baseline.store(baseline, Ordering::Relaxed);
                            if baseline {
                                tracing::info!("peer {id}: navegador sem H.264 High; a sala passa a usar Baseline");
                            }
                        } else {
                            *close_reason = Some("este navegador não decodifica H.264: use Chrome, Edge, Firefox ou Safari".into());
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
                            h264_baseline: peer.h264_baseline.load(Ordering::Relaxed),
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
                Event::PeerStats(s) => {
                    if let Some(p) = s.selected_candidate_pair {
                        if !peer.pair_logged.swap(true, Ordering::Relaxed) {
                            tracing::info!(
                                "peer {id}: par ICE selecionado local={} remoto={} rtt={:?} (remoto ipv6={})",
                                p.local.addr,
                                p.remote.addr,
                                p.current_round_trip_time,
                                p.remote.addr.is_ipv6()
                            );
                        }
                    }
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
                        // `ctl`: este host trata set_visible e set_quality; hosts antigos não mandam
                        let _ = ch.write(false, serde_json::json!({"type":"chat_ready","chat_id":chat_id,"donate":sh.donate,"ctl":true}).to_string().as_bytes());
                    }
                    *flush_now = true;
                }
                Event::ChannelData(d) if Some(d.id) == *chat_ch && d.data.len() <= MAX_WEB_CHAT => {
                    match serde_json::from_slice(&d.data) {
                        Ok(ViewerMsg::Chat { text, donate }) => {
                            if chat_limiter.allow(Instant::now()) {
                                if let Some(l) = ChatLine::incoming(chat_id, name, false, &text, donate, sh.donate) {
                                    sh.post_chat(l);
                                }
                            }
                        }
                        Ok(ViewerMsg::SetVisible { visible }) => peer.hidden.store(!visible, Ordering::Relaxed),
                        Ok(ViewerMsg::SetQuality { preset }) => sh.request_quality(preset, name),
                        _ => {}
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
    let started = Instant::now();
    let mut rx_count = 0u32;
    let mut first_src: Option<SocketAddr> = None;
    let exit: &str;
    loop {
        if stop_requested {
            exit = "stop_requested (ICE desconectado/fechado ou codec sem suporte)";
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
                exit = "timeout de ICE (20 s)";
                break;
            }
            recv_res = socks.recv(&mut buf) => {
                match recv_res {
                    Ok((n, src, dst)) => {
                        rx_count += 1;
                        first_src.get_or_insert(src);
                        let now = Instant::now();
                        if let Ok(recv) = Receive::new(Protocol::Udp, src, dst, &buf[..n]) {
                            if rtc.handle_input(Input::Receive(now, recv)).is_ok() {
                                next_timeout = match drain_outputs(&mut rtc, &socks, &mut events).await {
                                    Ok(t) => t,
                                    Err(_) => { exit = "erro em drain_outputs (recepção)"; break; }
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
                    Err(_) => { exit = "erro no socket UDP"; break; }
                }
            }
            _ = tokio::time::sleep(sleep_dur) => {
                let now = Instant::now();
                if rtc.handle_input(Input::Timeout(now)).is_ok() {
                    next_timeout = match drain_outputs(&mut rtc, &socks, &mut events).await {
                        Ok(t) => t,
                        Err(_) => { exit = "erro em drain_outputs (timeout)"; break; }
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
                        // aba escondida: sem vídeo; o keyframe é pedido quando ela voltar
                        if peer.hidden.load(Ordering::Relaxed) {
                            waiting_key = true;
                            continue;
                        }
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
                        next_timeout = match drain_outputs(&mut rtc, &socks, &mut events).await {
                            Ok(t) => t,
                            Err(_) => { exit = "erro em drain_outputs (vídeo)"; break; }
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
                    Err(broadcast::error::RecvError::Closed) => { exit = "feed de vídeo fechado"; break; }
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
                            next_timeout = match drain_outputs(&mut rtc, &socks, &mut events).await {
                                Ok(t) => t,
                                Err(_) => { exit = "erro em drain_outputs (áudio)"; break; }
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
                    Err(broadcast::error::RecvError::Closed) => { exit = "feed de áudio fechado"; break; }
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
                exit = "peer.stop (viewer saiu pela sinalização ou foi removido)";
                break;
            }
            _ = shutdown.changed() => {
                close_reason = Some("a transmissão terminou".into());
                exit = "shutdown do host (transmissão terminou)";
                break;
            }
        }
    }

    if !connected {
        // contagem bruta de UDP, sem validar STUN/DTLS: só indica se algo chegou ao socket
        tracing::warn!(
            "peer {id}: encerrado sem conectar após {:?} (saída: {exit}); datagramas UDP brutos recebidos={rx_count} primeira_origem={first_src:?}",
            started.elapsed()
        );
    }

    if let Some(reason) = close_reason {
        let _ = tx.try_send(SignalOut::Close {
            id: id.clone(),
            reason,
        });
    }

    rtc.disconnect();
    let _ = drain_outputs(&mut rtc, &socks, &mut Vec::new()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url_default() {
        assert_eq!(base_url(), WEB_BASE);
    }

    /// Formato que a página (`viewer.js`) manda pelo canal do chat.
    #[test]
    fn page_control_messages_parse() {
        let v: ViewerMsg = serde_json::from_str(r#"{"type":"set_visible","visible":false}"#).unwrap();
        assert!(matches!(v, ViewerMsg::SetVisible { visible: false }));
        for p in ["P1080p30", "P720p60", "P720p30", "P480p30", "P360p30", "P240p30"] {
            let m: ViewerMsg = serde_json::from_str(&format!(r#"{{"type":"set_quality","preset":"{p}"}}"#)).unwrap();
            assert!(matches!(m, ViewerMsg::SetQuality { .. }), "{p}");
        }
    }

    #[test]
    fn looks_virtual_names() {
        for n in [
            "Famatech Radmin VPN Ethernet Adapter",
            "NordLynx Tunnel",
            "TAP-NordVPN Windows Adapter V9",
            "Hyper-V Virtual Ethernet Adapter",
            "Tailscale Tunnel",
        ] {
            assert!(looks_virtual(&[n]), "{n}");
        }
        for n in ["Realtek PCIe GbE Family Controller", "Intel(R) Wi-Fi 6 AX201 160MHz"] {
            assert!(!looks_virtual(&[n]), "{n}");
        }
    }

    fn sa(ip: &str, p: u16) -> SocketAddr {
        SocketAddr::new(ip.parse().unwrap(), p)
    }

    fn ports(v: &[SocketAddr]) -> Vec<u16> {
        v.iter().map(|a| a.port()).collect()
    }

    fn maps(ps: &[Option<u16>]) -> Vec<Option<SocketAddr>> {
        ps.iter().map(|p| p.map(|p| sa("1.2.3.4", p))).collect()
    }

    #[test]
    fn browser_predictions_follow_first_v4_srflx() {
        let sdp = "a=candidate:1 1 udp 2122 abcd.local 5000 typ host\r\n\
                   a=candidate:2 1 udp 1686 2001:db8::1 6000 typ srflx raddr :: rport 0\r\n\
                   a=candidate:3 1 udp 41 198.51.100.7 7000 typ relay raddr 0.0.0.0 rport 0\r\n\
                   a=candidate:4 1 udp 1686 203.0.113.9 6000 typ srflx raddr 0.0.0.0 rport 0\r\n";
        let p = browser_predictions(sdp);
        assert_eq!(p.len(), 32);
        assert!(p.iter().all(|a| a.ip() == "203.0.113.9".parse::<IpAddr>().unwrap()));
        assert_eq!(ports(&p), (6001..=6032).collect::<Vec<_>>());
        let hi = "a=candidate:4 1 udp 1686 203.0.113.9 65530 typ srflx raddr 0.0.0.0 rport 0\r\n";
        assert_eq!(ports(&browser_predictions(hi)), vec![65531, 65532, 65533, 65534, 65535]);
        let none = "a=candidate:1 1 udp 2122 192.168.0.5 5000 typ host\r\n";
        assert!(browser_predictions(none).is_empty());
        let tcp = "a=candidate:4 1 tcp 1686 203.0.113.9 6000 typ srflx\r\n";
        assert!(browser_predictions(tcp).is_empty());
    }

    #[test]
    fn host_predictions_radmin_rule() {
        let p = |v: &[Option<u16>]| ports(&host_predictions(&maps(v)));
        let s = |x: &[u16]| x.to_vec();
        assert_eq!(p(&[Some(5000), Some(5002), Some(5004), Some(5006), Some(5008)]), (1..=8).map(|k| 5008 + 2 * k).collect::<Vec<_>>());
        assert_eq!(p(&[Some(5000), Some(5002), Some(5004), Some(5009), Some(5011)]), (1..=8).map(|k| 5011 + 2 * k).collect::<Vec<_>>());
        assert!(p(&[Some(5000), Some(5002), Some(5005), Some(5010), Some(5017)]).is_empty());
        assert!(p(&[Some(5000), Some(5000), Some(5002), Some(5004), Some(5006)]).is_empty());
        assert_eq!(p(&[Some(6000), Some(5998), Some(5996), Some(5994), Some(5992)]), (1..=8).map(|k| 5992 - 2 * k).collect::<Vec<_>>());
        assert!(p(&[Some(5000), None, Some(5004), Some(5006), Some(5008)]).is_empty());
        assert_eq!(p(&[Some(65522), Some(65524), Some(65526), Some(65528), Some(65530)]), s(&[65532, 65534]));
        let mut m = maps(&[Some(5000), Some(5002), Some(5004), Some(5006), Some(5008)]);
        m[2] = Some(sa("5.6.7.8", 5004));
        assert!(host_predictions(&m).is_empty());
    }
}
