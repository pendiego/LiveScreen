//! Desenho do viewer web: chave da sala no fragmento do link (não chega ao servidor),
//! prova HMAC amarrada ao fingerprint DTLS do offer e STUN feito à mão porque o str0m
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
pub fn viewer_url(base: &str, secret: &[u8; ROOM_KEY_LEN], key: &[u8; ROOM_KEY_LEN]) -> String {
    format!(
        "{}/v/#{}.{}",
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

/// Extrai o primeiro fingerprint DTLS de um SDP (linha `a=fingerprint:`, sem o prefixo).
pub fn sdp_fingerprint(sdp: &str) -> Option<&str> {
    for line in sdp.lines() {
        let line = line.trim();
        if let Some(fp) = line.strip_prefix("a=fingerprint:") {
            return Some(fp.trim());
        }
    }
    None
}

/// Gera a prova HMAC-SHA256 vinculando a chave da sala ao fingerprint DTLS do offer do navegador.
pub fn web_proof(key: &[u8; ROOM_KEY_LEN], fingerprint: &str) -> String {
    let s_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let mut msg = Vec::with_capacity(17 + fingerprint.len());
    msg.extend_from_slice(b"livescreen/web/v1");
    msg.extend_from_slice(fingerprint.as_bytes());
    let tag = ring::hmac::sign(&s_key, &msg);
    hex::encode(tag.as_ref())
}

/// Verifica a prova HMAC enviada pelo navegador. Retorna `false` em caso de erro ou assinatura inválida.
pub fn verify_web_proof(key: &[u8; ROOM_KEY_LEN], fingerprint: &str, proof_hex: &str) -> bool {
    let Ok(proof_bytes) = hex::decode(proof_hex) else {
        return false;
    };
    let s_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let mut msg = Vec::with_capacity(17 + fingerprint.len());
    msg.extend_from_slice(b"livescreen/web/v1");
    msg.extend_from_slice(fingerprint.as_bytes());
    ring::hmac::verify(&s_key, &msg, &proof_bytes).is_ok()
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
        sdp: String,
        proof: String,
        #[serde(default)]
        name: String,
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
    Answer { id: String, sdp: String },
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

    #[test]
    fn test_web_proof_verification() {
        let key = [42u8; ROOM_KEY_LEN];
        let fp = "sha-256 01:23:45:67:89:AB:CD:EF";
        let proof = web_proof(&key, fp);

        assert!(verify_web_proof(&key, fp, &proof));

        let diff_key = [99u8; ROOM_KEY_LEN];
        assert!(!verify_web_proof(&diff_key, fp, &proof));

        let diff_fp = "sha-256 00:00:00:00:00:00:00:00";
        assert!(!verify_web_proof(&key, diff_fp, &proof));

        assert!(!verify_web_proof(&key, fp, "invalido"));
    }

    #[test]
    fn test_sdp_fingerprint_extraction() {
        let sdp = "v=0\r\no=- 12345 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\nm=video 9 UDP/TLS/RTP/SAVPF 96\r\nc=IN IP4 0.0.0.0\r\na=fingerprint:sha-256 AB:CD:EF:12:34\r\na=setup:actpass\r\n";
        assert_eq!(
            sdp_fingerprint(sdp),
            Some("sha-256 AB:CD:EF:12:34")
        );
        assert_eq!(sdp_fingerprint("v=0\r\ns=-\r\n"), None);
    }

    #[test]
    fn test_signal_serde() {
        let ans = SignalOut::Answer {
            id: "0a1b2c3d".into(),
            sdp: "v=0\r\n".into(),
        };
        let ans_json = serde_json::to_string(&ans).unwrap();
        assert_eq!(
            ans_json,
            r#"{"t":"answer","id":"0a1b2c3d","sdp":"v=0\r\n"}"#
        );

        let offer_json = r#"{"t":"offer","id":"0a1b2c3d","sdp":"x","proof":"y"}"#;
        let sig_in: SignalIn = serde_json::from_str(offer_json).unwrap();
        assert_eq!(
            sig_in,
            SignalIn::Offer {
                id: "0a1b2c3d".into(),
                sdp: "x".into(),
                proof: "y".into(),
                name: String::new(),
            }
        );
    }

    #[test]
    fn test_viewer_url_no_duplicate_slash() {
        let secret = [1u8; ROOM_KEY_LEN];
        let key = [2u8; ROOM_KEY_LEN];
        let url = viewer_url("http://127.0.0.1:8787/", &secret, &key);
        assert!(!url.contains("//v"));
        assert!(url.starts_with("http://127.0.0.1:8787/v/#"));
        assert!(url.contains(&room_id(&secret)));
        assert!(url.ends_with(&hex::encode(key)));
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
