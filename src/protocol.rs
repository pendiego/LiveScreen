use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Result, bail};
use bytes::Bytes;
use iroh::endpoint::{RecvStream, SendStream};
use iroh_tickets::endpoint::EndpointTicket;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub const ALPN: &[u8] = b"livescreen/1";
pub const PROTOCOL_VERSION: u32 = 5;
pub const MAX_MSG: usize = 64 * 1024;
pub const MAX_FRAME: usize = 8 * 1024 * 1024;

pub const CLOSE_NORMAL: u32 = 0;
pub const CLOSE_FULL: u32 = 1;
pub const CLOSE_PROTOCOL: u32 = 2;
pub const CLOSE_KICKED: u32 = 3;
pub const CLOSE_STOPPED: u32 = 4;
pub const CLOSE_DENIED: u32 = 5;

pub const ROOM_KEY_LEN: usize = 16;
pub const VIDEO_BUDGET: usize = 64 * 1024 * 1024;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    H264,
    Hevc,
    Av1,
}

impl Codec {
    pub fn wire(&self) -> u8 {
        match self {
            Codec::H264 => 0,
            Codec::Hevc => 1,
            Codec::Av1 => 2,
        }
    }

    pub fn from_wire(v: u8) -> Option<Codec> {
        match v {
            0 => Some(Codec::H264),
            1 => Some(Codec::Hevc),
            2 => Some(Codec::Av1),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Codec::H264 => "H.264",
            Codec::Hevc => "HEVC",
            Codec::Av1 => "AV1",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ViewerMsg {
    Hello {
        protocol: u32,
        app_version: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        room_proof: String,
        #[serde(default)]
        decoders: Vec<Codec>,
    },
    RequestKeyframe,
    /// Pede ao host a qualidade da transmissão (vale para a sala toda; o host aplica).
    SetQuality {
        preset: crate::preset::Preset,
    },
    SetVisible {
        visible: bool,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostMsg {
    Welcome {
        width: u32,
        height: u32,
        fps: u32,
        audio: bool,
        #[serde(default)]
        title: String,
    },
    /// Qualidade trocada com a transmissão no ar; o próximo keyframe já vem no novo tamanho.
    StreamParams {
        width: u32,
        height: u32,
        fps: u32,
    },
    Rejected {
        reason: String,
    },
    StreamEnded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedFrame {
    pub seq: u64,
    pub keyframe: bool,
    pub pts_us: u64,
    pub codec: Codec,
    pub data: Bytes,
}

pub fn encode_msg<T: Serialize>(m: &T) -> Result<Vec<u8>> {
    let body = serde_json::to_vec(m)?;
    if body.len() > MAX_MSG {
        bail!("mensagem grande demais");
    }
    let mut v = (body.len() as u32).to_le_bytes().to_vec();
    v.extend(body);
    Ok(v)
}

pub async fn write_msg<T: Serialize>(send: &mut SendStream, m: &T) -> Result<()> {
    send.write_all(&encode_msg(m)?).await?;
    Ok(())
}

pub async fn read_msg<T: DeserializeOwned>(recv: &mut RecvStream) -> Result<T> {
    let mut l = [0u8; 4];
    recv.read_exact(&mut l).await?;
    let len = u32::from_le_bytes(l) as usize;
    if len > MAX_MSG {
        bail!("erro de protocolo: mensagem > 64 KiB");
    }
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await?;
    Ok(serde_json::from_slice(&buf)?)
}

pub fn video_frame_header(f: &EncodedFrame) -> [u8; 21] {
    let len = (17 + f.data.len()) as u32;
    let flags = (if f.keyframe { 1u8 } else { 0u8 }) | ((f.codec.wire() & 0x03) << 1);
    let mut h = [0u8; 21];
    h[..4].copy_from_slice(&len.to_le_bytes());
    h[4] = flags;
    h[5..13].copy_from_slice(&f.seq.to_le_bytes());
    h[13..21].copy_from_slice(&f.pts_us.to_le_bytes());
    h
}

/// Valida o cabeçalho e devolve o tamanho do corpo (len).
fn check_len(len: usize) -> Result<usize> {
    if len < 17 || len > MAX_FRAME {
        bail!("erro de protocolo: tamanho de quadro inválido ({len})");
    }
    Ok(len)
}

fn frame_from_body(body: Bytes) -> Result<EncodedFrame> {
    if body.len() < 17 {
        bail!("erro de protocolo: corpo de quadro curto demais");
    }
    let flags = body[0];
    if flags & 0b1111_1000 != 0 {
        bail!("erro de protocolo: flags de quadro inválidas");
    }
    let codec = Codec::from_wire((flags >> 1) & 0x03)
        .ok_or_else(|| anyhow::anyhow!("erro de protocolo: flags de quadro inválidas"))?;
    Ok(EncodedFrame {
        keyframe: (flags & 1) != 0,
        seq: u64::from_le_bytes(body[1..9].try_into().unwrap()),
        pts_us: u64::from_le_bytes(body[9..17].try_into().unwrap()),
        codec,
        data: body.slice(17..),
    })
}

pub fn parse_video_body(body: &[u8]) -> Result<EncodedFrame> {
    frame_from_body(Bytes::copy_from_slice(body))
}

pub struct FrameBudget(AtomicUsize);

impl Default for FrameBudget {
    fn default() -> Self {
        Self(AtomicUsize::new(0))
    }
}

impl FrameBudget {
    pub fn reserve(&self, n: usize) -> bool {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |cur| {
                cur.checked_add(n).filter(|&sum| sum <= VIDEO_BUDGET)
            })
            .is_ok()
    }
}

#[derive(Debug)]
pub struct BudgetExceeded;

impl std::fmt::Display for BudgetExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "orçamento de memória de vídeo excedido")
    }
}

impl std::error::Error for BudgetExceeded {}

struct Budgeted {
    buf: Vec<u8>,
    budget: Arc<FrameBudget>,
}

impl AsRef<[u8]> for Budgeted {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}

impl Drop for Budgeted {
    fn drop(&mut self) {
        self.budget.0.fetch_sub(self.buf.len(), Ordering::Relaxed);
    }
}

pub async fn read_video_frame(
    recv: &mut RecvStream,
    budget: &Arc<FrameBudget>,
) -> Result<Option<EncodedFrame>> {
    let mut l = [0u8; 4];
    match recv.read_exact(&mut l).await {
        Ok(()) => {}
        Err(iroh::endpoint::ReadExactError::FinishedEarly(0)) => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let len = check_len(u32::from_le_bytes(l) as usize)?;
    if !budget.reserve(len) {
        return Err(BudgetExceeded.into());
    }
    let mut budgeted = Budgeted {
        buf: vec![0u8; len],
        budget: budget.clone(),
    };
    recv.read_exact(&mut budgeted.buf).await?;
    let body = Bytes::from_owner(budgeted);
    frame_from_body(body).map(Some)
}

pub fn format_invite(ticket: &EndpointTicket, key: &[u8; ROOM_KEY_LEN]) -> String {
    format!("{ticket}.{}", hex::encode(key))
}

pub fn parse_invite(s: &str) -> Option<(EndpointTicket, [u8; ROOM_KEY_LEN])> {
    let s = s.trim();
    let (ticket_str, hex_str) = s.rsplit_once('.')?;
    if hex_str.len() != ROOM_KEY_LEN * 2 {
        return None;
    }
    let mut key = [0u8; ROOM_KEY_LEN];
    hex::decode_to_slice(hex_str, &mut key).ok()?;
    let ticket: EndpointTicket = ticket_str.parse().ok()?;
    Some((ticket, key))
}

pub fn new_room_key() -> [u8; ROOM_KEY_LEN] {
    use ring::rand::SecureRandom;
    let rng = ring::rand::SystemRandom::new();
    let mut key = [0u8; ROOM_KEY_LEN];
    rng.fill(&mut key).expect("gerador aleatório do sistema");
    key
}

pub fn room_proof(
    key: &[u8; ROOM_KEY_LEN],
    host: &iroh::EndpointId,
    viewer: &iroh::EndpointId,
) -> String {
    let s_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let mut msg = Vec::with_capacity(18 + 32 + 32);
    msg.extend_from_slice(b"livescreen/room/v1");
    msg.extend_from_slice(host.as_bytes());
    msg.extend_from_slice(viewer.as_bytes());
    let tag = ring::hmac::sign(&s_key, &msg);
    hex::encode(tag.as_ref())
}

pub fn verify_room_proof(
    key: &[u8; ROOM_KEY_LEN],
    host: &iroh::EndpointId,
    viewer: &iroh::EndpointId,
    proof_hex: &str,
) -> bool {
    let Ok(proof_bytes) = hex::decode(proof_hex) else {
        return false;
    };
    let s_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let mut msg = Vec::with_capacity(18 + 32 + 32);
    msg.extend_from_slice(b"livescreen/room/v1");
    msg.extend_from_slice(host.as_bytes());
    msg.extend_from_slice(viewer.as_bytes());
    ring::hmac::verify(&s_key, &msg, &proof_bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for codec in [Codec::H264, Codec::Hevc, Codec::Av1] {
            let f = EncodedFrame {
                seq: 7,
                keyframe: true,
                pts_us: 123456,
                codec,
                data: Bytes::from_static(&[1, 2, 3]),
            };
            let hdr = video_frame_header(&f);
            let mut full = Vec::new();
            full.extend_from_slice(&hdr);
            full.extend_from_slice(&f.data);
            let len = u32::from_le_bytes(full[..4].try_into().unwrap()) as usize;
            assert_eq!(len, 20);
            let parsed = parse_video_body(&full[4..]).unwrap();
            assert_eq!(parsed, f);
        }
    }

    #[test]
    fn frame_flags_rejects_unknown_bits() {
        // bit 3 set (0b1000)
        let mut body = vec![0u8; 17];
        body[0] = 0b1000;
        assert!(parse_video_body(&body).is_err());

        // codec 3 (bits 1-2 = 0b11)
        body[0] = 0b0110;
        assert!(parse_video_body(&body).is_err());
    }

    #[test]
    fn invite_roundtrip_and_proof_binding() {
        let host_id = iroh::SecretKey::from_bytes(&[1u8; 32]).public();
        let viewer_id = iroh::SecretKey::from_bytes(&[2u8; 32]).public();
        let other_viewer = iroh::SecretKey::from_bytes(&[3u8; 32]).public();
        let addr = iroh::EndpointAddr::new(host_id);
        let ticket = EndpointTicket::new(addr);
        let key = [42u8; ROOM_KEY_LEN];

        let inv = format_invite(&ticket, &key);
        let (parsed_t, parsed_k) = parse_invite(&inv).expect("invite válido");
        assert_eq!(parsed_t, ticket);
        assert_eq!(parsed_k, key);

        let proof = room_proof(&key, &host_id, &viewer_id);
        assert!(verify_room_proof(&key, &host_id, &viewer_id, &proof));
        assert!(!verify_room_proof(&key, &host_id, &other_viewer, &proof));

        let bad_key = [99u8; ROOM_KEY_LEN];
        assert!(!verify_room_proof(&bad_key, &host_id, &viewer_id, &proof));

        assert!(parse_invite(&ticket.to_string()).is_none());
        assert!(parse_invite(&format!("{ticket}.short")).is_none());
    }

    #[test]
    fn budget_released_on_drop() {
        let budget = Arc::new(FrameBudget::default());
        assert!(budget.reserve(VIDEO_BUDGET));
        assert!(!budget.reserve(1));
        budget.0.store(0, Ordering::Relaxed);

        assert!(budget.reserve(100));
        let b = Bytes::from_owner(Budgeted {
            buf: vec![0u8; 100],
            budget: budget.clone(),
        });
        let s1 = b.slice(0..50);
        let s2 = b.slice(50..100);
        drop(b);
        assert_eq!(budget.0.load(Ordering::Relaxed), 100);
        drop(s1);
        assert_eq!(budget.0.load(Ordering::Relaxed), 100);
        drop(s2);
        assert_eq!(budget.0.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn limits() {
        assert!(check_len(MAX_FRAME + 1).is_err());
        assert!(check_len(MAX_FRAME).is_ok());
        let big = ViewerMsg::Hello {
            protocol: 1,
            app_version: "x".repeat(MAX_MSG),
            name: String::new(),
            room_proof: String::new(),
            decoders: Vec::new(),
        };
        assert!(encode_msg(&big).is_err());
        assert!(encode_msg(&ViewerMsg::RequestKeyframe).is_ok());
    }

    #[test]
    fn hello_without_name() {
        let m = serde_json::from_str::<ViewerMsg>(
            r#"{"type":"hello","protocol":1,"app_version":"0.1.0"}"#,
        )
        .unwrap();
        let ViewerMsg::Hello {
            name,
            room_proof,
            decoders,
            ..
        } = m
        else {
            panic!()
        };
        assert_eq!(name, "");
        assert_eq!(room_proof, "");
        assert!(decoders.is_empty());
    }

    #[test]
    fn welcome_without_title() {
        let m = serde_json::from_str::<HostMsg>(
            r#"{"type":"welcome","width":1,"height":2,"fps":30,"audio":false}"#,
        )
        .unwrap();
        let HostMsg::Welcome { title, .. } = m else {
            panic!()
        };
        assert_eq!(title, "");
    }
}
