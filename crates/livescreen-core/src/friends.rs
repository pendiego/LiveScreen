//! Identidade persistente, código = id hex, presença por ALPN separado respondida
//! só a amigos. Endpoints de quem apenas disca não publicam no pkarr (só o host
//! publica, para não sobrescrever o registro do host).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use iroh::endpoint::presets;
use iroh::{Endpoint, EndpointAddr, EndpointId, SecretKey};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

pub const PRESENCE_ALPN: &[u8] = b"livescreen/presence/1";
/// Motivo de fechamento da presença para quem foi removido da transmissão.
pub const REMOVED_REASON: &[u8] = b"removed";
pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(15);
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(8);

pub fn new_secret_key() -> SecretKey {
    use ring::rand::SecureRandom;
    let rng = ring::rand::SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("gerador de números aleatórios do sistema");
    SecretKey::from_bytes(&bytes)
}

pub fn secret_key_hex(k: &SecretKey) -> String {
    hex::encode(k.to_bytes())
}

pub fn parse_secret_key(s: &str) -> Option<SecretKey> {
    let s = s.trim();
    if s.len() != 64 {
        return None;
    }
    let mut bytes = [0u8; 32];
    hex::decode_to_slice(s, &mut bytes).ok()?;
    Some(SecretKey::from_bytes(&bytes))
}

pub fn friend_code(id: &EndpointId) -> String {
    id.to_string()
}

pub fn parse_friend_code(s: &str) -> Option<EndpointId> {
    let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let lower = clean.to_ascii_lowercase();
    lower.parse().ok()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Presence {
    #[serde(default)]
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PresenceState {
    Unknown,
    Live { title: String },
    Offline,
    NotFriend,
    Removed,
}

/// Disca para outros endpoints usando identidade persistente.
/// Não usa PkarrPublisher para não sobrescrever o registro do host quando apenas disca.
pub async fn dial_endpoint(identity: SecretKey) -> Result<Endpoint> {
    let transport = iroh::endpoint::QuicTransportConfig::builder()
        .max_concurrent_uni_streams(noq_proto::VarInt::from_u32(8))
        .max_concurrent_bidi_streams(noq_proto::VarInt::from_u32(0))
        .build();
    let ep = Endpoint::builder(presets::Minimal)
        .secret_key(identity)
        .address_lookup(iroh::address_lookup::PkarrResolver::n0_dns())
        .address_lookup(iroh::address_lookup::DnsAddressLookup::n0_dns())
        .relay_mode(iroh::endpoint::default_relay_mode())
        .transport_config(transport)
        .bind()
        .await?;
    Ok(ep)
}

pub async fn probe(ep: &Endpoint, id: EndpointId) -> PresenceState {
    let probe_fut = async {
        let conn = ep.connect(EndpointAddr::new(id), PRESENCE_ALPN).await?;
        let mut recv = conn.accept_uni().await?;
        let p: Presence = crate::protocol::read_msg(&mut recv).await?;
        conn.close(crate::protocol::CLOSE_NORMAL.into(), b"bye");
        Ok::<_, anyhow::Error>(p)
    };
    match tokio::time::timeout(PROBE_TIMEOUT, probe_fut).await {
        Ok(Ok(p)) => PresenceState::Live {
            title: crate::protocol::sanitize(&p.title, 120),
        },
        Ok(Err(e)) => match denied_reason(&e) {
            Some(r) if r == REMOVED_REASON => PresenceState::Removed,
            Some(_) => PresenceState::NotFriend,
            None => PresenceState::Offline,
        },
        _ => PresenceState::Offline,
    }
}

fn denied_reason(e: &anyhow::Error) -> Option<Vec<u8>> {
    use iroh::endpoint::{ConnectionError, ReadError, ReadExactError};
    let c = match (
        e.downcast_ref::<ConnectionError>(),
        e.downcast_ref::<ReadExactError>(),
    ) {
        (Some(ConnectionError::ApplicationClosed(c)), _) => c,
        (
            _,
            Some(ReadExactError::ReadError(ReadError::ConnectionLost(
                ConnectionError::ApplicationClosed(c),
            ))),
        ) => c,
        _ => return None,
    };
    (u64::from(c.error_code) == crate::protocol::CLOSE_DENIED as u64).then(|| c.reason.to_vec())
}

pub struct Prober {
    rt: Handle,
    ep: Endpoint,
    task: JoinHandle<()>,
    ids: tokio::sync::watch::Sender<Vec<EndpointId>>,
    states: Arc<Mutex<HashMap<EndpointId, PresenceState>>>,
    stopped: AtomicBool,
}

struct AbortJoinSet<T: 'static>(tokio::task::JoinSet<T>);
impl<T: 'static> std::ops::Deref for AbortJoinSet<T> {
    type Target = tokio::task::JoinSet<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: 'static> std::ops::DerefMut for AbortJoinSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T: 'static> Drop for AbortJoinSet<T> {
    fn drop(&mut self) {
        self.0.abort_all();
    }
}

impl Prober {
    pub fn start(
        rt: &Handle,
        identity: SecretKey,
        ids: Vec<EndpointId>,
        notify: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Prober> {
        let ep = rt.block_on(dial_endpoint(identity))?;
        let (ids_tx, mut ids_rx) = tokio::sync::watch::channel(ids);
        let states = Arc::new(Mutex::new(HashMap::new()));
        let states_loop = states.clone();
        let ep_loop = ep.clone();
        let task = rt.spawn(async move {
            loop {
                let current_ids: Vec<EndpointId> = ids_rx.borrow_and_update().clone();
                {
                    let mut s = states_loop.lock().unwrap();
                    s.retain(|k, _| current_ids.contains(k));
                }
                if !current_ids.is_empty() {
                    let sem = Arc::new(tokio::sync::Semaphore::new(8));
                    let mut set = AbortJoinSet(tokio::task::JoinSet::new());
                    for id in current_ids {
                        let ep = ep_loop.clone();
                        let sem = sem.clone();
                        set.spawn(async move {
                            let _permit = sem.acquire_owned().await.ok();
                            let state = probe(&ep, id).await;
                            (id, state)
                        });
                    }
                    while let Some(res) = set.join_next().await {
                        if let Ok((id, state)) = res {
                            {
                                let mut s = states_loop.lock().unwrap();
                                s.insert(id, state);
                            }
                            notify();
                        }
                    }
                }
                tokio::select! {
                    _ = tokio::time::sleep(PRESENCE_INTERVAL) => {}
                    res = ids_rx.changed() => {
                        if res.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(Prober {
            rt: rt.clone(),
            ep,
            task,
            ids: ids_tx,
            states,
            stopped: AtomicBool::new(false),
        })
    }

    pub fn set_ids(&self, ids: Vec<EndpointId>) {
        let _ = self.ids.send_replace(ids);
    }

    pub fn state(&self, id: &EndpointId) -> PresenceState {
        self.states
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .unwrap_or(PresenceState::Unknown)
    }

    pub fn snapshot(&self) -> HashMap<EndpointId, PresenceState> {
        self.states.lock().unwrap().clone()
    }

    pub fn states_json(&self) -> String {
        let map: HashMap<String, PresenceState> = self
            .snapshot()
            .into_iter()
            .map(|(id, st)| (friend_code(&id), st))
            .collect();
        serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
    }

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

impl Drop for Prober {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friend_code_roundtrip_and_normalization() {
        let key = new_secret_key();
        let id = key.public();
        let code = friend_code(&id);
        assert_eq!(parse_friend_code(&code), Some(id));

        // maiúsculas e quebras de linha no meio
        let mut messy = code.to_ascii_uppercase();
        messy.insert(10, '\n');
        messy.insert(35, ' ');
        messy.insert(50, '\r');
        assert_eq!(parse_friend_code(&messy), Some(id));

        // rejeita 63 chars, char não-hex e um convite
        assert_eq!(parse_friend_code(&code[..63]), None);
        let not_hex = format!("{}z", &code[..63]);
        assert_eq!(parse_friend_code(&not_hex), None);

        let ticket = iroh_tickets::endpoint::EndpointTicket::new(iroh::EndpointAddr::new(id));
        let inv = crate::protocol::format_invite(&ticket, &[1u8; crate::protocol::ROOM_KEY_LEN]);
        assert_eq!(parse_friend_code(&inv), None);
    }

    #[test]
    fn secret_key_roundtrip() {
        let key = new_secret_key();
        let hex = secret_key_hex(&key);
        let parsed = parse_secret_key(&hex).expect("secret key válida");
        assert_eq!(parsed.public(), key.public());
    }

    #[test]
    fn target_parse_friend_vs_invite() {
        let host_key = new_secret_key();
        let host_id = host_key.public();
        let friend_str = friend_code(&host_id);
        match crate::protocol::Target::parse(&friend_str) {
            Some(crate::protocol::Target::Friend(id)) => assert_eq!(id, host_id),
            other => panic!("esperava Target::Friend, veio {other:?}"),
        }

        let addr = iroh::EndpointAddr::new(host_id);
        let ticket = iroh_tickets::endpoint::EndpointTicket::new(addr);
        let room_key = [42u8; crate::protocol::ROOM_KEY_LEN];
        let invite_str = crate::protocol::format_invite(&ticket, &room_key);
        match crate::protocol::Target::parse(&invite_str) {
            Some(crate::protocol::Target::Invite(t, Some(k))) => {
                assert_eq!(t, ticket);
                assert_eq!(k, room_key);
            }
            other => panic!("esperava Target::Invite(_, Some(_)), veio {other:?}"),
        }
    }
}
