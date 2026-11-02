//! Desenho do viewer web: chave da sala no fragmento do link (não chega ao servidor),
//! offer e answer cifrados com essa chave (AES-256-GCM) e STUN feito à mão porque o str0m
//! não descobre o endereço server-reflexive (srflx).

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use serde::{Deserialize, Serialize};

pub use crate::protocol::ROOM_KEY_LEN;

/// Identificador público da sala no servidor de sinalização: os primeiros 32 caracteres hex
/// do SHA-256 do segredo do host.
pub fn room_id(secret: &[u8; ROOM_KEY_LEN]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, secret);
    hex::encode(&digest.as_ref()[..16])
}

/// Monta a URL completa do espectador com o segredo da sala e a chave simétrica no fragmento `#`.
/// O sufixo `.v2` avisa a página que offer e answer vão cifrados com a chave da sala (AES-256-GCM).
pub fn viewer_url(base: &str, secret: &[u8; ROOM_KEY_LEN], key: &[u8; ROOM_KEY_LEN]) -> String {
    format!(
        "{}/v/#{}.{}.v2",
        base.trim_end_matches('/'),
        room_id(secret),
        hex::encode(key)
    )
}

/// Converte a URL base (HTTP/HTTPS) na URL do WebSocket de sinalização do host.
pub fn host_ws_url(base: &str, secret: &[u8; ROOM_KEY_LEN]) -> String {
    let base = base.trim_end_matches('/');
    let ws_base = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        base.to_string()
    };
    format!("{ws_base}/ws/host/{}", room_id(secret))
}

const SIGNAL_SALT: &[u8] = b"livescreen/web-signal/v2";
const NONCE_LEN: usize = 12;

/// Conteúdo do offer cifrado pelo navegador (JSON).
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct SealedOffer {
    pub sdp: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub avatar: u8,
}

#[derive(Serialize)]
struct SealedAnswer<'a> {
    sdp: &'a str,
}

/// AES-256-GCM com chave HKDF-SHA256(salt = SIGNAL_SALT, ikm = chave da sala, info = `dir`).
fn signal_key(key: &[u8; ROOM_KEY_LEN], dir: &[u8]) -> ring::aead::LessSafeKey {
    let prk = ring::hkdf::Salt::new(ring::hkdf::HKDF_SHA256, SIGNAL_SALT).extract(key);
    let info = [dir];
    let okm = prk.expand(&info, &ring::aead::AES_256_GCM).expect("tamanho fixo do AES-256");
    ring::aead::LessSafeKey::new(ring::aead::UnboundKey::from(okm))
}

/// hex(nonce || cifra || tag).
fn seal_with(k: &ring::aead::LessSafeKey, nonce: [u8; NONCE_LEN], aad: &[u8], plain: &[u8]) -> String {
    let mut buf = plain.to_vec();
    k.seal_in_place_append_tag(
        ring::aead::Nonce::assume_unique_for_key(nonce),
        ring::aead::Aad::from(aad),
        &mut buf,
    )
    .expect("tamanho dentro do limite do AES-GCM");
    format!("{}{}", hex::encode(nonce), hex::encode(buf))
}

/// Inverso de `seal_with`; `None` se o hex for inválido, curto (< 28 bytes) ou se a tag não conferir.
fn open_with(
    k: &ring::aead::LessSafeKey,
    aad: &[u8],
    sealed_hex: &str,
) -> Option<(Vec<u8>, [u8; NONCE_LEN])> {
    let bytes = hex::decode(sealed_hex).ok()?;
    if bytes.len() < NONCE_LEN + 16 {
        return None;
    }
    let (n, ct) = bytes.split_at(NONCE_LEN);
    let nonce: [u8; NONCE_LEN] = n.try_into().ok()?;
    let mut buf = ct.to_vec();
    let plain = k
        .open_in_place(
            ring::aead::Nonce::assume_unique_for_key(nonce),
            ring::aead::Aad::from(aad),
            &mut buf,
        )
        .ok()?;
    Some((plain.to_vec(), nonce))
}

/// Abre o offer do navegador. AAD = "offer:" + sala (32 hex). Devolve também o nonce, que amarra a resposta.
pub fn open_offer(
    key: &[u8; ROOM_KEY_LEN],
    room: &str,
    sealed_hex: &str,
) -> Option<(SealedOffer, [u8; NONCE_LEN])> {
    let aad = format!("offer:{room}");
    let (plain, nonce) = open_with(&signal_key(key, b"offer"), aad.as_bytes(), sealed_hex)?;
    Some((serde_json::from_slice(&plain).ok()?, nonce))
}

/// Cifra a answer com nonce aleatório. AAD = "answer:" + sala + ":" + hex(nonce do offer).
pub fn seal_answer(
    key: &[u8; ROOM_KEY_LEN],
    room: &str,
    offer_nonce: &[u8; NONCE_LEN],
    sdp: &str,
) -> String {
    use ring::rand::SecureRandom;
    let mut nonce = [0u8; NONCE_LEN];
    ring::rand::SystemRandom::new().fill(&mut nonce).expect("gerador aleatório do sistema");
    let aad = format!("answer:{room}:{}", hex::encode(offer_nonce));
    let plain = serde_json::to_vec(&SealedAnswer { sdp }).expect("JSON de string");
    seal_with(&signal_key(key, b"answer"), nonce, aad.as_bytes(), &plain)
}

const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

/// Monta um pacote STUN Binding Request (RFC 5389): tipo 0x0001, tamanho 0, cookie e transaction ID.
pub fn stun_binding_request(txid: &[u8; 12]) -> [u8; 20] {
    let mut buf = [0u8; 20];
    buf[0..2].copy_from_slice(&0x0001u16.to_be_bytes());
    buf[2..4].copy_from_slice(&0x0000u16.to_be_bytes());
    buf[4..8].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
    buf[8..20].copy_from_slice(txid);
    buf
}

/// Extrai o endereço mapeado (XOR-MAPPED-ADDRESS ou fallback MAPPED-ADDRESS) de uma resposta STUN.
/// Rejeita respostas com transaction ID ou cookie divergentes, ou comprimentos inconsistentes.
pub fn stun_mapped_addr(resp: &[u8], txid: &[u8; 12]) -> Option<SocketAddr> {
    if resp.len() < 20 {
        return None;
    }
    let msg_type = u16::from_be_bytes([resp[0], resp[1]]);
    if msg_type != 0x0101 {
        return None;
    }
    let msg_len = u16::from_be_bytes([resp[2], resp[3]]) as usize;
    if resp.len() != 20 + msg_len {
        return None;
    }
    if &resp[4..8] != &STUN_MAGIC_COOKIE.to_be_bytes() {
        return None;
    }
    if &resp[8..20] != txid {
        return None;
    }

    let mut attrs = &resp[20..];
    let mut xor_mapped = None;
    let mut mapped = None;

    while !attrs.is_empty() {
        if attrs.len() < 4 {
            return None;
        }
        let attr_type = u16::from_be_bytes([attrs[0], attrs[1]]);
        let attr_len = u16::from_be_bytes([attrs[2], attrs[3]]) as usize;
        let padded_len = (attr_len + 3) & !3;
        if attrs.len() < 4 + padded_len {
            return None;
        }
        let val = &attrs[4..4 + attr_len];
        attrs = &attrs[4 + padded_len..];

        match attr_type {
            // XOR-MAPPED-ADDRESS
            0x0020 => {
                if val.len() < 4 {
                    return None;
                }
                let family = val[1];
                let xport = u16::from_be_bytes([val[2], val[3]]);
                let port = xport ^ 0x2112;

                match family {
                    0x01 => {
                        if val.len() != 8 {
                            return None;
                        }
                        let cookie_bytes = STUN_MAGIC_COOKIE.to_be_bytes();
                        let ip = Ipv4Addr::new(
                            val[4] ^ cookie_bytes[0],
                            val[5] ^ cookie_bytes[1],
                            val[6] ^ cookie_bytes[2],
                            val[7] ^ cookie_bytes[3],
                        );
                        xor_mapped = Some(SocketAddr::new(IpAddr::V4(ip), port));
                    }
                    0x02 => {
                        if val.len() != 20 {
                            return None;
                        }
                        let mut xkey = [0u8; 16];
                        xkey[..4].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
                        xkey[4..].copy_from_slice(txid);
                        let mut ip_bytes = [0u8; 16];
                        for i in 0..16 {
                            ip_bytes[i] = val[4 + i] ^ xkey[i];
                        }
                        let ip = Ipv6Addr::from(ip_bytes);
                        xor_mapped = Some(SocketAddr::new(IpAddr::V6(ip), port));
                    }
                    _ => return None,
                }
            }
            // MAPPED-ADDRESS
            0x0001 => {
                if val.len() < 4 {
                    return None;
                }
                let family = val[1];
                let port = u16::from_be_bytes([val[2], val[3]]);

                match family {
                    0x01 => {
                        if val.len() != 8 {
                            return None;
                        }
                        let ip = Ipv4Addr::new(val[4], val[5], val[6], val[7]);
                        mapped = Some(SocketAddr::new(IpAddr::V4(ip), port));
                    }
                    0x02 => {
                        if val.len() != 20 {
                            return None;
                        }
                        let mut ip_bytes = [0u8; 16];
                        ip_bytes.copy_from_slice(&val[4..20]);
                        let ip = Ipv6Addr::from(ip_bytes);
                        mapped = Some(SocketAddr::new(IpAddr::V6(ip), port));
                    }
                    _ => return None,
                }
            }
            _ => {}
        }
    }

    xor_mapped.or(mapped)
}

/// Mensagens enviadas do servidor de sinalização para o host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
pub enum SignalIn {
    Ready,
    Pong,
    Offer {
        id: String,
        #[serde(default)]
        sealed: String,
    },
    Left {
        id: String,
    },
}

/// Mensagens enviadas do host para o servidor de sinalização.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
pub enum SignalOut {
    Host { secret: String },
    Ping,
    Answer {
        id: String,
        sealed: String,
    },
    Reject { id: String, reason: String },
    Close { id: String, reason: String },
}

/// Valida o identificador de um peer web (exatamente 8 caracteres hexadecimais minúsculos).
pub fn valid_peer_id(id: &str) -> bool {
    id.len() == 8 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stun_rfc5769_vector() {
        let txid_hex = "b7e7a701bc34d686fa87dfae";
        let mut txid = [0u8; 12];
        hex::decode_to_slice(txid_hex, &mut txid).unwrap();

        // Cabeçalho (20B) + Atributo XOR-MAPPED-ADDRESS (12B):
        // 01 01 (Binding Success Response)
        // 00 0c (Comprimento dos atributos = 12 bytes)
        // 21 12 a4 42 (Magic cookie)
        // b7 e7 a7 01 bc 34 d6 86 fa 87 df ae (TxID)
        // 00 20 00 08 00 01 a1 47 e1 12 a6 43 (Attr XOR-MAPPED-ADDRESS)
        let resp_hex = "0101000c2112a442b7e7a701bc34d686fa87dfae002000080001a147e112a643";
        let resp = hex::decode(resp_hex).unwrap();

        let addr = stun_mapped_addr(&resp, &txid).expect("deve decodificar o endereço");
        assert_eq!(addr, "192.0.2.1:32853".parse().unwrap());

        // Mesmo pacote com txid diferente retorna None
        let mut diff_txid = txid;
        diff_txid[0] ^= 0xff;
        assert_eq!(stun_mapped_addr(&resp, &diff_txid), None);
    }

    const SALA: &str = "00112233445566778899aabbccddeeff";
    const OFFER_VEC: &str = "0101010101010101010101019ad3a09ee5d4a444d16c2dde72f783f7cea499b7594f64533a5f5479a8498c50e3ff5ea0b0087a13d3d5f0d59092";

    #[test]
    fn test_signal_vectors_match_webcrypto() {
        let key = [9u8; ROOM_KEY_LEN];
        let (offer, nonce) = open_offer(&key, SALA, OFFER_VEC).expect("offer do WebCrypto");
        assert_eq!(offer, SealedOffer { sdp: "v=0\r\n".into(), name: "Ana".into(), avatar: 0 });
        assert_eq!(nonce, [1u8; 12]);

        let sealed = seal_with(
            &signal_key(&key, b"answer"),
            [2u8; 12],
            b"answer:00112233445566778899aabbccddeeff:010101010101010101010101",
            br#"{"sdp":"v=1\r\n"}"#,
        );
        assert_eq!(sealed, "0202020202020202020202021e0a51645b5320a1314e55d9b83a5520af7b8d351fceefca7f27f829e800359e2b");
    }

    #[test]
    fn test_signal_rejects_tampering() {
        let key = [9u8; ROOM_KEY_LEN];
        let mut bad = OFFER_VEC.to_string();
        let last = if bad.ends_with('0') { '1' } else { '0' };
        bad.pop();
        bad.push(last);
        assert!(open_offer(&key, SALA, &bad).is_none());
        assert!(open_offer(&key, "ffeeddccbbaa99887766554433221100", OFFER_VEC).is_none());
        assert!(open_offer(&[8u8; ROOM_KEY_LEN], SALA, OFFER_VEC).is_none());
        for s in ["", "zz", "01"] {
            assert!(open_offer(&key, SALA, s).is_none());
        }
        let answer = "0202020202020202020202021e0a51645b5320a1314e55d9b83a5520af7b8d351fceefca7f27f829e800359e2b";
        assert!(open_offer(&key, SALA, answer).is_none());

        let a = seal_answer(&key, SALA, &[1u8; 12], "v=1\r\n");
        let aad = format!("answer:{SALA}:{}", hex::encode([1u8; 12]));
        let (plain, _) = open_with(&signal_key(&key, b"answer"), aad.as_bytes(), &a).unwrap();
        assert_eq!(plain, br#"{"sdp":"v=1\r\n"}"#);
        assert_ne!(a, seal_answer(&key, SALA, &[1u8; 12], "v=1\r\n"));
    }

    #[test]
    fn test_signal_serde() {
        let ans = SignalOut::Answer { id: "0a1b2c3d".into(), sealed: "s".into() };
        assert_eq!(
            serde_json::to_string(&ans).unwrap(),
            r#"{"t":"answer","id":"0a1b2c3d","sealed":"s"}"#
        );

        let sig_in: SignalIn =
            serde_json::from_str(r#"{"t":"offer","id":"0a1b2c3d","sealed":"y"}"#).unwrap();
        assert_eq!(sig_in, SignalIn::Offer { id: "0a1b2c3d".into(), sealed: "y".into() });

        let legacy: SignalIn = serde_json::from_str(
            r#"{"t":"offer","id":"0a1b2c3d","sdp":"x","proof":"y","name":"n"}"#,
        )
        .unwrap();
        assert_eq!(legacy, SignalIn::Offer { id: "0a1b2c3d".into(), sealed: String::new() });
    }

    #[test]
    fn test_viewer_url_no_duplicate_slash() {
        let secret = [1u8; ROOM_KEY_LEN];
        let key = [2u8; ROOM_KEY_LEN];
        let url = viewer_url("http://127.0.0.1:8787/", &secret, &key);
        assert!(!url.contains("//v"));
        assert!(url.starts_with("http://127.0.0.1:8787/v/#"));
        assert!(url.contains(&room_id(&secret)));
        assert!(url.ends_with(&format!("{}.v2", hex::encode(key))));
    }

    #[test]
    fn test_host_ws_url() {
        let secret = [1u8; ROOM_KEY_LEN];
        let ws = host_ws_url("https://example.com/prefix/", &secret);
        assert_eq!(
            ws,
            format!("wss://example.com/prefix/ws/host/{}", room_id(&secret))
        );

        let ws_http = host_ws_url("http://127.0.0.1:8787", &secret);
        assert_eq!(
            ws_http,
            format!("ws://127.0.0.1:8787/ws/host/{}", room_id(&secret))
        );
    }

    #[test]
    fn test_valid_peer_id() {
        assert!(valid_peer_id("0a1b2c3d"));
        assert!(valid_peer_id("12345678"));
        assert!(valid_peer_id("abcdef01"));
        assert!(!valid_peer_id("0A1B2C3D")); // maiúsculo não permitido
        assert!(!valid_peer_id("0a1b2c3")); // 7 chars
        assert!(!valid_peer_id("0a1b2c3de")); // 9 chars
        assert!(!valid_peer_id("0a1b2c3g")); // 'g' inválido
    }

    #[test]
    fn test_stun_binding_request() {
        let txid = [7u8; 12];
        let req = stun_binding_request(&txid);
        assert_eq!(&req[0..2], &[0x00, 0x01]);
        assert_eq!(&req[2..4], &[0x00, 0x00]);
        assert_eq!(&req[4..8], &0x2112A442u32.to_be_bytes());
        assert_eq!(&req[8..20], &txid);
    }
}
