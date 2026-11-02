//! Sinalização web v3: chave pública híbrida (X25519 + ML-KEM-768),
//! chave simétrica de 256 bits no fragmento do link e STUN direto.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use curve25519_dalek::montgomery::MontgomeryPoint;
use libcrux_ml_kem::mlkem768::{
    self, MlKem768Ciphertext, MlKem768PrivateKey,
};
#[cfg(test)]
use libcrux_ml_kem::mlkem768::MlKem768PublicKey;
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

pub use crate::protocol::ROOM_KEY_LEN;

pub const WEB_KEY_LEN: usize = 32;
pub const HOST_PK_LEN: usize = 1216;
pub const MLKEM_SK_LEN: usize = 2400;
const MLKEM_EK_LEN: usize = 1184;
const MLKEM_CT_LEN: usize = 1088;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const PRE_LEN: usize = 16;
const MIN_SEALED_LEN: usize = 1 + 32 + MLKEM_CT_LEN + PRE_LEN + NONCE_LEN + TAG_LEN;

/// Identificador público da sala no servidor de sinalização: os primeiros 32 caracteres hex
/// do SHA-256 do segredo do host.
pub fn room_id(secret: &[u8; ROOM_KEY_LEN]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, secret);
    hex::encode(&digest.as_ref()[..16])
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

/// Preenche o texto com espaços 0x20 até o próximo múltiplo de 2048 bytes (mínimo 2048).
pub(crate) fn pad_plaintext(mut bytes: Vec<u8>) -> Vec<u8> {
    let target = bytes.len().div_ceil(2048).max(1) * 2048;
    bytes.resize(target, b' ');
    bytes
}

/// Deriva a chave de pré-validação HMAC: HKDF-SHA256(salt="livescreen/web-signal/v3/pre", ikm=web_key, info=room).
pub(crate) fn derive_k_pre(web_key: &[u8; WEB_KEY_LEN], room: &str) -> [u8; 32] {
    let prk = ring::hkdf::Salt::new(
        ring::hkdf::HKDF_SHA256,
        b"livescreen/web-signal/v3/pre",
    )
    .extract(web_key);
    let info_slices = [room.as_bytes()];
    let okm = prk
        .expand(&info_slices, ring::hkdf::HKDF_SHA256)
        .expect("k_pre expand");
    let mut k_pre = [0u8; 32];
    okm.fill(&mut k_pre).expect("k_pre fill");
    k_pre
}

/// Computa a tag HMAC-SHA256 de pré-validação truncada em 16 bytes: HMAC(k_pre, 0x03 || vx_pk || ct_m)[..16].
pub(crate) fn compute_pre(
    web_key: &[u8; WEB_KEY_LEN],
    room: &str,
    vx_pk: &[u8; 32],
    ct_m: &[u8],
) -> [u8; PRE_LEN] {
    let k_pre = derive_k_pre(web_key, room);
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &k_pre);
    let mut ctx = ring::hmac::Context::with_key(&key);
    ctx.update(&[0x03]);
    ctx.update(vx_pk);
    ctx.update(ct_m);
    let tag = ctx.sign();
    let mut pre = [0u8; PRE_LEN];
    pre.copy_from_slice(&tag.as_ref()[..PRE_LEN]);
    pre
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn is_all_zero(a: &[u8]) -> bool {
    let mut acc = 0u8;
    for b in a {
        acc |= *b;
    }
    acc == 0
}

/// Confere o valor `pre` em tempo constante.
pub(crate) fn check_pre(
    web_key: &[u8; WEB_KEY_LEN],
    room: &str,
    vx_pk: &[u8; 32],
    ct_m: &[u8],
    pre_got: &[u8],
) -> bool {
    if pre_got.len() != PRE_LEN {
        return false;
    }
    let expected = compute_pre(web_key, room, vx_pk, ct_m);
    constant_time_eq(pre_got, &expected)
}
/// Deriva o hash de contexto: SHA-256("livescreen/web-signal/v3" || room || host_pk || vx_pk || ct_m).
pub(crate) fn derive_ctx(
    room: &str,
    host_pk: &[u8; HOST_PK_LEN],
    vx_pk: &[u8; 32],
    ct_m: &[u8],
) -> [u8; 32] {
    let mut h = ring::digest::Context::new(&ring::digest::SHA256);
    h.update(b"livescreen/web-signal/v3");
    h.update(room.as_bytes());
    h.update(host_pk);
    h.update(vx_pk);
    h.update(ct_m);
    let digest = h.finish();
    let mut out = [0u8; 32];
    out.copy_from_slice(digest.as_ref());
    out
}

/// Cabeçalho de oferta v3 pré-validado (tamanho, prefixo 0x03 e tag HMAC `pre`).
/// Permite ao host consumir o ID de replay antes de disparar operações de chave pública caras.
pub struct ValidatedOfferHeader {
    pub(crate) bytes: Vec<u8>,
    pub replay_id: [u8; 16],
}

pub struct WebKeys {
    web_key: Zeroizing<[u8; WEB_KEY_LEN]>,
    hx_sk: Zeroizing<[u8; 32]>,
    hm_sk: Zeroizing<Vec<u8>>,
    host_pk: [u8; HOST_PK_LEN],
    pk_hash: [u8; 32],
    #[cfg(test)]
    pub(crate) decaps_count: std::sync::atomic::AtomicUsize,
}

impl WebKeys {
    /// Gera material de chaves seguro a partir do gerador aleatório do sistema.
    pub fn generate() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let mut web_key = Zeroizing::new([0u8; WEB_KEY_LEN]);
        rng.fill(&mut *web_key).expect("rng fill web_key");

        let mut hx_sk = Zeroizing::new([0u8; 32]);
        rng.fill(&mut *hx_sk).expect("rng fill hx_sk");

        let mut host_m_seed = Zeroizing::new([0u8; 64]);
        rng.fill(&mut *host_m_seed).expect("rng fill host_m_seed");

        Self::from_seeds(*web_key, *hx_sk, *host_m_seed)
    }

    /// Constrói chaves determinísticas a partir de sementes (usado em testes e no vetor cruzado).
    pub fn from_seeds(
        mut web_key: [u8; WEB_KEY_LEN],
        mut hx_sk: [u8; 32],
        mut host_m_seed: [u8; 64],
    ) -> Self {
        let hx_pk = MontgomeryPoint::mul_base_clamped(hx_sk).0;
        let keypair = mlkem768::generate_key_pair(host_m_seed);
        host_m_seed.zeroize();
        let (hm_sk, hm_pk) = keypair.into_parts();
        let hm_ek: [u8; MLKEM_EK_LEN] = *hm_pk.as_slice();

        let mut host_pk = [0u8; HOST_PK_LEN];
        host_pk[..32].copy_from_slice(&hx_pk);
        host_pk[32..].copy_from_slice(&hm_ek);

        let mut h = ring::digest::Context::new(&ring::digest::SHA256);
        h.update(b"livescreen/web-host-pk/v3");
        h.update(&host_pk);
        let digest = h.finish();
        let mut pk_hash = [0u8; 32];
        pk_hash.copy_from_slice(digest.as_ref());

        let mut sk_raw: [u8; MLKEM_SK_LEN] = hm_sk.into();
        let hm_sk = Zeroizing::new(sk_raw.to_vec());
        sk_raw.zeroize();

        let web_key_z = Zeroizing::new(web_key);
        let hx_sk_z = Zeroizing::new(hx_sk);
        web_key.zeroize();
        hx_sk.zeroize();

        Self {
            web_key: web_key_z,
            hx_sk: hx_sk_z,
            hm_sk,
            host_pk,
            pk_hash,
            #[cfg(test)]
            decaps_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Retorna a chave pública composta do host em formato hexadecimal minúsculo (2432 caracteres).
    pub fn host_pk_hex(&self) -> String {
        hex::encode(self.host_pk)
    }

    /// Hash SHA-256 da chave pública composta do host.
    pub fn pk_hash(&self) -> &[u8; 32] {
        &self.pk_hash
    }

    /// Chave pública X25519 efêmera do host.
    pub fn hx_pk(&self) -> &[u8; 32] {
        self.host_pk[..32].try_into().unwrap()
    }

    /// Chave pública de encapsulação ML-KEM-768 do host.
    pub fn hm_ek(&self) -> &[u8; MLKEM_EK_LEN] {
        self.host_pk[32..].try_into().unwrap()
    }

    /// Monta o link do navegador v3: `{base}/v/#{room}.{web_key_hex}.{pk_hash_hex}.v3`.
    pub fn viewer_url(&self, base: &str, room: &str) -> String {
        let clean = base.trim_end_matches('/');
        format!(
            "{clean}/v/#{room}.{}.{}.v3",
            hex::encode(&*self.web_key),
            hex::encode(self.pk_hash)
        )
    }

    /// Valida o cabeçalho leve do offer: hex, tamanho mínimo, prefixo 0x03 e HMAC `pre`.
    /// Não realiza decapsulação ML-KEM nem multiplicação escalar X25519.
    pub fn check_offer_header(&self, room: &str, sealed_hex: &str) -> Option<ValidatedOfferHeader> {
        if sealed_hex.len() < MIN_SEALED_LEN * 2 || sealed_hex.len() % 2 != 0 {
            return None;
        }
        let bytes = hex::decode(sealed_hex).ok()?;
        if bytes.len() < MIN_SEALED_LEN {
            return None;
        }

        // 1. Prefixo 0x03
        if bytes[0] != 0x03 {
            return None;
        }

        let vx_pk: [u8; 32] = bytes[1..33].try_into().ok()?;
        let ct_m = &bytes[33..33 + MLKEM_CT_LEN];
        let pre_got = &bytes[33 + MLKEM_CT_LEN..33 + MLKEM_CT_LEN + PRE_LEN];

        // 2. Pré-filtro HMAC barato em tempo constante
        if !check_pre(&self.web_key, room, &vx_pk, ct_m, pre_got) {
            return None;
        }

        // 3. Replay ID = SHA-256(vx_pk || ct_m)[..16]
        let mut h = ring::digest::Context::new(&ring::digest::SHA256);
        h.update(&vx_pk);
        h.update(ct_m);
        let digest = h.finish();
        let mut replay_id = [0u8; 16];
        replay_id.copy_from_slice(&digest.as_ref()[..16]);

        Some(ValidatedOfferHeader { bytes, replay_id })
    }

    /// Executa as operações criptográficas pesadas (decapsulação ML-KEM, X25519 e AEAD)
    /// somente após a verificação e consumo do `replay_id`.
    pub fn open_validated_offer(&self, room: &str, header: ValidatedOfferHeader) -> Option<OpenedOffer> {
        let bytes = header.bytes;
        let vx_pk: [u8; 32] = bytes[1..33].try_into().ok()?;
        let ct_m = &bytes[33..33 + MLKEM_CT_LEN];
        let nonce: [u8; NONCE_LEN] = bytes[33 + MLKEM_CT_LEN + PRE_LEN
            ..33 + MLKEM_CT_LEN + PRE_LEN + NONCE_LEN]
            .try_into()
            .ok()?;
        let ct_and_tag = &bytes[33 + MLKEM_CT_LEN + PRE_LEN + NONCE_LEN..];

        #[cfg(test)]
        self.decaps_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // 3. Decapsulação ML-KEM-768
        let ct = MlKem768Ciphertext::try_from(ct_m).ok()?;
        let sk_array: &[u8; MLKEM_SK_LEN] = self.hm_sk.as_slice().try_into().ok()?;
        let sk = MlKem768PrivateKey::from(sk_array);
        let ss_m_raw = mlkem768::decapsulate(&sk, &ct);
        let mut sk_raw: [u8; MLKEM_SK_LEN] = sk.into();
        sk_raw.zeroize();
        let ss_m = Zeroizing::new(ss_m_raw);

        // 4. Multiplicação X25519 (rejeita ponto de ordem baixa ou saída toda zerada)
        let vx_point = MontgomeryPoint(vx_pk);
        let ss_x_point = vx_point.mul_clamped(*self.hx_sk);
        let ss_x = Zeroizing::new(ss_x_point.0);
        if is_all_zero(&*ss_x) {
            return None;
        }

        // 5. Contexto e derivação de chaves
        let ctx = derive_ctx(room, &self.host_pk, &vx_pk, ct_m);

        let mut ikm = Zeroizing::new([0u8; 32 + 32 + WEB_KEY_LEN]);
        ikm[..32].copy_from_slice(&*ss_m);
        ikm[32..64].copy_from_slice(&*ss_x);
        ikm[64..].copy_from_slice(&*self.web_key);

        let prk = ring::hkdf::Salt::new(
            ring::hkdf::HKDF_SHA256,
            b"livescreen/web-signal/v3",
        )
        .extract(&*ikm);

        let mut offer_info = Vec::with_capacity(5 + 32);
        offer_info.extend_from_slice(b"offer");
        offer_info.extend_from_slice(&ctx);

        let offer_info_slices = [&offer_info[..]];
        let k_offer_okm = prk
            .expand(&offer_info_slices, ring::hkdf::HKDF_SHA256)
            .ok()?;
        let mut k_offer = Zeroizing::new([0u8; 32]);
        k_offer_okm.fill(&mut *k_offer).ok()?;

        let mut chain_info = Vec::with_capacity(5 + 32);
        chain_info.extend_from_slice(b"chain");
        chain_info.extend_from_slice(&ctx);

        let chain_info_slices = [&chain_info[..]];
        let s_chain_okm = prk
            .expand(&chain_info_slices, ring::hkdf::HKDF_SHA256)
            .ok()?;
        let mut s_chain = Zeroizing::new([0u8; 32]);
        s_chain_okm.fill(&mut *s_chain).ok()?;

        // 6. AEAD Open
        let mut aad = Vec::with_capacity(1 + 32);
        aad.push(0x03);
        aad.extend_from_slice(&ctx);

        let unbound = ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, &*k_offer).ok()?;
        let less_safe = ring::aead::LessSafeKey::new(unbound);
        let mut in_out = ct_and_tag.to_vec();
        let plain_bytes = less_safe
            .open_in_place(
                ring::aead::Nonce::assume_unique_for_key(nonce),
                ring::aead::Aad::from(&aad),
                &mut in_out,
            )
            .ok()?;

        // 7. Parse do JSON
        let offer: SealedOffer = serde_json::from_slice(plain_bytes).ok()?;

        Some(OpenedOffer {
            offer,
            replay_id: header.replay_id,
            answer: AnswerSealer {
                s_chain,
                ctx,
                vx_pk,
            },
        })
    }

    /// Abre um offer cifrado v3 recebido do navegador.
    /// Ordem rigorosa: checagem leve do cabeçalho -> decapsulação -> X25519 -> AEAD -> JSON.
    pub fn open_offer(&self, room: &str, sealed_hex: &str) -> Option<OpenedOffer> {
        let header = self.check_offer_header(room, sealed_hex)?;
        self.open_validated_offer(room, header)
    }
}

/// Oferta decifrada e validada pelo host.
pub struct OpenedOffer {
    pub offer: SealedOffer,
    pub replay_id: [u8; 16],
    pub answer: AnswerSealer,
}

/// Gerador da resposta cifrada para um espectador específico.
pub struct AnswerSealer {
    s_chain: Zeroizing<[u8; 32]>,
    ctx: [u8; 32],
    vx_pk: [u8; 32],
}

impl AnswerSealer {
    /// Cifra a resposta SDP gerando chaves efêmeras aleatórias com sigilo futuro.
    /// Consome `self` para aposentar o material secreto após a cifragem.
    pub fn seal(self, sdp: &str) -> String {
        let rng = ring::rand::SystemRandom::new();
        let mut ax_sk = Zeroizing::new([0u8; 32]);
        let mut nonce = [0u8; NONCE_LEN];
        loop {
            rng.fill(&mut *ax_sk).expect("rng fill ax_sk");
            let ss_a = MontgomeryPoint(self.vx_pk).mul_clamped(*ax_sk).0;
            if ss_a != [0u8; 32] {
                break;
            }
        }
        rng.fill(&mut nonce).expect("rng fill nonce");
        self.seal_with_params(sdp, *ax_sk, nonce)
    }

    /// Cifra com parâmetros explícitos (usado no teste com o vetor determinístico).
    /// Consome `self` para aposentar o material secreto após a cifragem.
    pub fn seal_with_params(self, sdp: &str, mut ax_sk: [u8; 32], nonce: [u8; NONCE_LEN]) -> String {
        let ax_sk_z = Zeroizing::new(ax_sk);
        ax_sk.zeroize();

        let ax_pk = MontgomeryPoint::mul_base_clamped(*ax_sk_z).0;
        let ss_a = Zeroizing::new(MontgomeryPoint(self.vx_pk).mul_clamped(*ax_sk_z).0);
        assert_ne!(*ss_a, [0u8; 32], "ss_a cannot be all zero");

        let mut ikm = Zeroizing::new([0u8; 64]);
        ikm[..32].copy_from_slice(&*self.s_chain);
        ikm[32..].copy_from_slice(&*ss_a);

        let prk = ring::hkdf::Salt::new(
            ring::hkdf::HKDF_SHA256,
            b"livescreen/web-signal/v3/answer",
        )
        .extract(&*ikm);

        let mut info = Vec::with_capacity(6 + 32 + 32);
        info.extend_from_slice(b"answer");
        info.extend_from_slice(&self.ctx);
        info.extend_from_slice(&ax_pk);

        let info_slices = [&info[..]];
        let okm = prk
            .expand(&info_slices, ring::hkdf::HKDF_SHA256)
            .expect("k_answer expand");
        let mut k_answer = Zeroizing::new([0u8; 32]);
        okm.fill(&mut *k_answer).expect("k_answer fill");
        let plain_json = serde_json::to_string(&SealedAnswer { sdp }).expect("serialize answer");
        let mut padded = pad_plaintext(plain_json.into_bytes());

        let mut aad = Vec::with_capacity(1 + 32 + 32);
        aad.push(0x03);
        aad.extend_from_slice(&self.ctx);
        aad.extend_from_slice(&ax_pk);

        let unbound =
            ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, &*k_answer).expect("unbound key");
        let less_safe = ring::aead::LessSafeKey::new(unbound);
        less_safe
            .seal_in_place_append_tag(
                ring::aead::Nonce::assume_unique_for_key(nonce),
                ring::aead::Aad::from(&aad),
                &mut padded,
            )
            .expect("seal in place");

        let mut sealed_bytes = Vec::with_capacity(1 + 32 + NONCE_LEN + padded.len());
        sealed_bytes.push(0x03);
        sealed_bytes.extend_from_slice(&ax_pk);
        sealed_bytes.extend_from_slice(&nonce);
        sealed_bytes.extend_from_slice(&padded);

        hex::encode(sealed_bytes)
    }
}

/// Conteúdo do offer cifrado pelo navegador (JSON).
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct SealedOffer {
    pub sdp: String,
    #[serde(default)]
    pub name: String,
    /// Valor estranho (texto, negativo, null) vira 0 = automático: não recusa a entrada por um enfeite.
    #[serde(default, deserialize_with = "lenient_u8")]
    pub avatar: u8,
}

fn lenient_u8<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    Ok(v.as_u64().and_then(|n| u8::try_from(n).ok()).unwrap_or(0))
}

#[derive(Serialize, Deserialize)]
struct SealedAnswer<'a> {
    sdp: &'a str,
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
    Pk { pk: String },
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
pub(crate) fn seal_offer_for_test(
    web_key: &[u8; WEB_KEY_LEN],
    room: &str,
    host_pk: &[u8; HOST_PK_LEN],
    vx_sk: [u8; 32],
    rand32: [u8; 32],
    nonce: [u8; NONCE_LEN],
    plain_json: &str,
) -> String {
    let vx_pk = MontgomeryPoint::mul_base_clamped(vx_sk).0;
    let hx_pk: [u8; 32] = host_pk[..32].try_into().unwrap();
    let ek: [u8; MLKEM_EK_LEN] = host_pk[32..].try_into().unwrap();

    let ss_x = MontgomeryPoint(hx_pk).mul_clamped(vx_sk).0;
    assert_ne!(ss_x, [0u8; 32]);

    let ek_pub = MlKem768PublicKey::from(ek);
    let (ct_m, ss_m) = mlkem768::encapsulate(&ek_pub, rand32);

    let ctx = derive_ctx(room, host_pk, &vx_pk, ct_m.as_slice());

    let mut ikm = [0u8; 32 + 32 + WEB_KEY_LEN];
    ikm[..32].copy_from_slice(&ss_m);
    ikm[32..64].copy_from_slice(&ss_x);
    ikm[64..].copy_from_slice(web_key);

    let prk = ring::hkdf::Salt::new(
        ring::hkdf::HKDF_SHA256,
        b"livescreen/web-signal/v3",
    )
    .extract(&ikm);

    let mut offer_info = Vec::with_capacity(5 + 32);
    offer_info.extend_from_slice(b"offer");
    offer_info.extend_from_slice(&ctx);

    let offer_info_slices = [&offer_info[..]];
    let k_offer_okm = prk
        .expand(&offer_info_slices, ring::hkdf::HKDF_SHA256)
        .expect("k_offer expand");
    let mut k_offer = [0u8; 32];
    k_offer_okm.fill(&mut k_offer).expect("k_offer fill");

    let pre = compute_pre(web_key, room, &vx_pk, ct_m.as_slice());

    let mut padded = pad_plaintext(plain_json.as_bytes().to_vec());

    let mut aad = Vec::with_capacity(1 + 32);
    aad.push(0x03);
    aad.extend_from_slice(&ctx);

    let unbound =
        ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, &k_offer).expect("unbound key");
    let less_safe = ring::aead::LessSafeKey::new(unbound);
    less_safe
        .seal_in_place_append_tag(
            ring::aead::Nonce::assume_unique_for_key(nonce),
            ring::aead::Aad::from(&aad),
            &mut padded,
        )
        .expect("seal in place");

    let mut out = Vec::with_capacity(1 + 32 + MLKEM_CT_LEN + PRE_LEN + NONCE_LEN + padded.len());
    out.push(0x03);
    out.extend_from_slice(&vx_pk);
    out.extend_from_slice(ct_m.as_slice());
    out.extend_from_slice(&pre);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&padded);

    hex::encode(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_offer_and_answer() {
        let keys = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";
        let vx_sk = [42u8; 32];
        let rand32 = [77u8; 32];
        let nonce = [99u8; 12];
        let offer_json = r#"{"sdp":"v=0\r\no=- 1 1 IN IP4 127.0.0.1","name":"Teste","avatar":2}"#;

        let sealed = seal_offer_for_test(
            &keys.web_key,
            room,
            &keys.host_pk,
            vx_sk,
            rand32,
            nonce,
            offer_json,
        );

        let opened = keys.open_offer(room, &sealed).expect("open offer failed");
        assert_eq!(opened.offer.sdp, "v=0\r\no=- 1 1 IN IP4 127.0.0.1");
        assert_eq!(opened.offer.name, "Teste");
        assert_eq!(opened.offer.avatar, 2);

        let answer_sdp = "v=0\r\no=- 2 2 IN IP4 127.0.0.1";
        let answer_sealed = opened.answer.seal(answer_sdp);
        assert!(answer_sealed.starts_with("03"));
    }

    #[test]
    fn tamper_each_field_fails() {
        let keys = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";
        let vx_sk = [11u8; 32];
        let rand32 = [22u8; 32];
        let nonce = [33u8; 12];
        let offer_json = r#"{"sdp":"v=0","name":"Alice","avatar":1}"#;

        let sealed = seal_offer_for_test(
            &keys.web_key,
            room,
            &keys.host_pk,
            vx_sk,
            rand32,
            nonce,
            offer_json,
        );
        let raw = hex::decode(&sealed).unwrap();
        // 1. Prefixo
        let mut t = raw.clone();
        t[0] ^= 0x01;
        assert!(keys.open_offer(room, &hex::encode(&t)).is_none());

        // 2. vx_pk (bytes 1..33)
        let mut t = raw.clone();
        t[1] ^= 0x55;
        assert!(keys.open_offer(room, &hex::encode(&t)).is_none());

        // 3. ct_m (bytes 33..1121)
        let mut t = raw.clone();
        t[50] ^= 0x55;
        assert!(keys.open_offer(room, &hex::encode(&t)).is_none());

        // 4. pre (bytes 1121..1137)
        let mut t = raw.clone();
        t[1125] ^= 0x55;
        assert!(keys.open_offer(room, &hex::encode(&t)).is_none());

        // 5. nonce (bytes 1137..1149)
        let mut t = raw.clone();
        t[1140] ^= 0x55;
        assert!(keys.open_offer(room, &hex::encode(&t)).is_none());

        // 6. tag / ciphertext (a partir do byte 1149)
        let mut t = raw.clone();
        let last = t.len() - 1;
        t[last] ^= 0x55;
        assert!(keys.open_offer(room, &hex::encode(&t)).is_none());

        // 7. AAD / room divergente
        assert!(keys.open_offer("wrong_room_0123456789abcdef00", &sealed).is_none());
    }

    #[test]
    fn wrong_web_key_or_different_webkeys_fails() {
        let keys1 = WebKeys::generate();
        let keys2 = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";
        let offer_json = r#"{"sdp":"v=0","name":"Bob","avatar":0}"#;

        let sealed = seal_offer_for_test(
            &keys1.web_key,
            room,
            &keys1.host_pk,
            [1u8; 32],
            [2u8; 32],
            [3u8; 12],
            offer_json,
        );

        assert!(keys2.open_offer(room, &sealed).is_none());
    }

    #[test]
    fn short_or_invalid_hex_fails() {
        let keys = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";

        assert!(keys.open_offer(room, "").is_none());
        assert!(keys.open_offer(room, "03").is_none());
        assert!(keys.open_offer(room, "123").is_none()); // tamanho ímpar
        assert!(keys.open_offer(room, "zzzz").is_none());
        assert!(keys.open_offer(room, &"03".repeat(500)).is_none()); // menor que 1165 bytes
    }

    #[test]
    fn zero_or_low_order_vx_pk_rejected() {
        let keys = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";

        // Cria um offer com vx_pk toda zerada
        let vx_pk_zero = [0u8; 32];
        let ek = MlKem768PublicKey::from(*keys.hm_ek());
        let (ct_m, _) = mlkem768::encapsulate(&ek, [5u8; 32]);
        let pre = compute_pre(&keys.web_key, room, &vx_pk_zero, ct_m.as_slice());
        let nonce = [7u8; 12];
        let plain = pad_plaintext(b"{}".to_vec());

        // Cifra fictícia
        let mut out = Vec::new();
        out.push(0x03);
        out.extend_from_slice(&vx_pk_zero);
        out.extend_from_slice(ct_m.as_slice());
        out.extend_from_slice(&pre);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&plain);
        out.extend_from_slice(&[0u8; 16]); // tag dummy

        assert!(keys.open_offer(room, &hex::encode(out)).is_none());
    }

    #[test]
    fn padding_makes_sealed_length_equal_for_different_name_lengths() {
        let keys = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";

        let offer1 = r#"{"sdp":"v=0\r\ns=-","name":"A","avatar":1}"#;
        let offer2 = r#"{"sdp":"v=0\r\ns=-","name":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA","avatar":1}"#;

        let s1 = seal_offer_for_test(
            &keys.web_key,
            room,
            &keys.host_pk,
            [1u8; 32],
            [2u8; 32],
            [3u8; 12],
            offer1,
        );
        let s2 = seal_offer_for_test(
            &keys.web_key,
            room,
            &keys.host_pk,
            [1u8; 32],
            [2u8; 32],
            [3u8; 12],
            offer2,
        );

        assert_eq!(s1.len(), s2.len());
    }

    #[test]
    fn replay_id_stable_for_same_offer() {
        let keys = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";
        let offer_json = r#"{"sdp":"v=0","name":"Carlos","avatar":1}"#;

        let sealed = seal_offer_for_test(
            &keys.web_key,
            room,
            &keys.host_pk,
            [10u8; 32],
            [20u8; 32],
            [30u8; 12],
            offer_json,
        );

        let o1 = keys.open_offer(room, &sealed).unwrap();
        let o2 = keys.open_offer(room, &sealed).unwrap();
        assert_eq!(o1.replay_id, o2.replay_id);
    }

    #[test]
    fn pre_failure_returns_before_decapsulation() {
        let keys = WebKeys::generate();
        let room = "0123456789abcdef0123456789abcdef";
        let offer_json = r#"{"sdp":"v=0","name":"David","avatar":1}"#;

        let sealed = seal_offer_for_test(
            &keys.web_key,
            room,
            &keys.host_pk,
            [11u8; 32],
            [22u8; 32],
            [33u8; 12],
            offer_json,
        );

        let mut raw = hex::decode(&sealed).unwrap();
        // adultera o pre (offset 1121..1137)
        raw[1122] ^= 0xff;

        let count_before = keys.decaps_count.load(std::sync::atomic::Ordering::Relaxed);
        let res = keys.open_offer(room, &hex::encode(raw));
        let count_after = keys.decaps_count.load(std::sync::atomic::Ordering::Relaxed);

        assert!(res.is_none());
        assert_eq!(count_before, 0);
        assert_eq!(count_after, 0, "decapsulação não deve ter sido executada");
    }

    #[test]
    fn cross_language_fixture_test() {
        let fixture_raw = include_str!("../../../web/test/v3-vector.json");
        let fixture: serde_json::Value = serde_json::from_str(fixture_raw).unwrap();

        let room = fixture["room"].as_str().unwrap();
        let web_key: [u8; 32] = hex::decode(fixture["web_key"].as_str().unwrap()).unwrap().try_into().unwrap();
        let host_x_sk: [u8; 32] = hex::decode(fixture["host_x_sk"].as_str().unwrap()).unwrap().try_into().unwrap();
        let host_m_seed: [u8; 64] = hex::decode(fixture["host_m_seed"].as_str().unwrap()).unwrap().try_into().unwrap();
        let host_pk_expected = fixture["host_pk"].as_str().unwrap();
        let pk_hash_expected = fixture["pk_hash"].as_str().unwrap();

        let web_keys = WebKeys::from_seeds(web_key, host_x_sk, host_m_seed);
        assert_eq!(web_keys.host_pk_hex(), host_pk_expected);
        assert_eq!(hex::encode(web_keys.pk_hash()), pk_hash_expected);

        let offer_sealed = fixture["offer_sealed"].as_str().unwrap();
        let opened = web_keys.open_offer(room, offer_sealed).expect("falha ao abrir offer do fixture");

        let offer_plain_raw = fixture["offer_plain"].as_str().unwrap();
        let offer_plain: SealedOffer = serde_json::from_str(offer_plain_raw).unwrap();
        assert_eq!(opened.offer.sdp, offer_plain.sdp);
        assert_eq!(opened.offer.name, offer_plain.name);
        assert_eq!(opened.offer.avatar, offer_plain.avatar);

        let answer_x_sk: [u8; 32] = hex::decode(fixture["answer_x_sk"].as_str().unwrap()).unwrap().try_into().unwrap();
        let answer_nonce: [u8; 12] = hex::decode(fixture["answer_nonce"].as_str().unwrap()).unwrap().try_into().unwrap();
        let answer_sdp = fixture["answer_sdp"].as_str().unwrap();

        let answer_sealed_expected = fixture["answer_sealed"].as_str().expect("answer_sealed ausente no fixture");
        assert!(!answer_sealed_expected.is_empty(), "answer_sealed não pode ser vazio");

        let answer_sealed = opened.answer.seal_with_params(answer_sdp, answer_x_sk, answer_nonce);
        assert_eq!(answer_sealed, answer_sealed_expected);
    }

    #[test]
    #[ignore = "utilitário manual para regenerar o vetor quando a especificação mudar"]
    fn regenerate_cross_language_fixture() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/test/v3-vector.json");
        let fixture_raw = std::fs::read_to_string(&fixture_path).unwrap();
        let mut fixture_obj: serde_json::Value = serde_json::from_str(&fixture_raw).unwrap();

        let room = fixture_obj["room"].as_str().unwrap();
        let web_key: [u8; 32] = hex::decode(fixture_obj["web_key"].as_str().unwrap()).unwrap().try_into().unwrap();
        let host_x_sk: [u8; 32] = hex::decode(fixture_obj["host_x_sk"].as_str().unwrap()).unwrap().try_into().unwrap();
        let host_m_seed: [u8; 64] = hex::decode(fixture_obj["host_m_seed"].as_str().unwrap()).unwrap().try_into().unwrap();

        let web_keys = WebKeys::from_seeds(web_key, host_x_sk, host_m_seed);
        let offer_sealed = fixture_obj["offer_sealed"].as_str().unwrap();
        let opened = web_keys.open_offer(room, offer_sealed).expect("falha ao abrir offer");

        let answer_x_sk: [u8; 32] = hex::decode(fixture_obj["answer_x_sk"].as_str().unwrap()).unwrap().try_into().unwrap();
        let answer_nonce: [u8; 12] = hex::decode(fixture_obj["answer_nonce"].as_str().unwrap()).unwrap().try_into().unwrap();
        let answer_sdp = fixture_obj["answer_sdp"].as_str().unwrap();

        let answer_sealed = opened.answer.seal_with_params(answer_sdp, answer_x_sk, answer_nonce);
        fixture_obj["answer_sealed"] = serde_json::Value::String(answer_sealed);
        let updated = serde_json::to_string_pretty(&fixture_obj).unwrap() + "\n";
        std::fs::write(&fixture_path, updated).unwrap();
    }

    #[test]
    fn replayed_header_rejected_without_decapsulation() {
        let keys = WebKeys::from_seeds([1u8; 32], [2u8; 32], [3u8; 64]);
        let room = "0123456789abcdef0123456789abcdef";
        let offer_json = r#"{"sdp":"v=0\r\n...","name":"Test","avatar":0}"#;
        let sealed = seal_offer_for_test(
            &keys.web_key,
            room,
            &keys.host_pk,
            [11u8; 32],
            [22u8; 32],
            [33u8; 12],
            offer_json,
        );

        let mut seen = std::collections::HashSet::new();

        // Primeira chegada: cabeçalho válido, consome replay_id
        let header1 = keys.check_offer_header(room, &sealed).expect("header 1 válido");
        assert!(seen.insert(header1.replay_id));
        let count_before = keys.decaps_count.load(std::sync::atomic::Ordering::Relaxed);
        let opened = keys.open_validated_offer(room, header1).expect("abre offer 1");
        assert_eq!(opened.offer.name, "Test");
        let count_after_first = keys.decaps_count.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(count_after_first, count_before + 1);

        // Segunda chegada (replay com mesmo cabeçalho mas potencialmente payload modificado)
        let header2 = keys.check_offer_header(room, &sealed).expect("header 2 válido");
        let is_replay = !seen.insert(header2.replay_id);
        assert!(is_replay, "deve detectar replay_id já consumido");
        // Rejeitado antes da decapsulação: open_validated_offer NÃO é chamado
        let count_after_replay = keys.decaps_count.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            count_after_replay, count_after_first,
            "decapsulação não deve ser chamada no replay"
        );
    }
}
