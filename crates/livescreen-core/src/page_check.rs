//! Verificação da integridade da página web do espectador contra manifesto assinado offline.
//! O host LiveScreen só abre o link do navegador se a página servida conferir com a assinatura
//! Ed25519 emitida no deploy oficial.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Chave pública Ed25519 oficial usada para assinar as releases do desktop e o manifesto web.
pub const RELEASE_PUBLIC_KEY: &str =
    "c46025005551cda3e63968e0b6a71bccd47a6411276b85aff966f93d81290930";

pub const MANIFEST_PREFIX: &[u8] = b"livescreen/web-manifest/v1\n";
pub const MAX_MANIFEST_BYTES: usize = 16 * 1024; // 16 KiB
pub const MAX_FILE_BYTES: usize = 2 * 1024 * 1024; // 2 MiB
pub const MAX_FILES: usize = 64;
pub const MAX_SIG_BYTES: usize = 256; // 256 B

#[derive(serde::Deserialize)]
struct Manifest {
    files: HashMap<String, String>,
}

static VERIFY_CACHE: Mutex<Option<(String, Instant)>> = Mutex::new(None);

/// Valida se um caminho de arquivo dentro do manifesto é seguro.
/// Não permite `..`, caminhos absolutos e restringe caracteres a [A-Za-z0-9._/-].
pub fn validate_path(path: &str) -> anyhow::Result<()> {
    if path.is_empty() {
        anyhow::bail!("caminho vazio no manifesto");
    }
    if path.contains("..") {
        anyhow::bail!("caminho contém '..' proibido: {path}");
    }
    if path.starts_with('/') || path.starts_with('\\') {
        anyhow::bail!("caminho absoluto proibido: {path}");
    }
    for b in path.bytes() {
        let ok = matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'/' | b'-');
        if !ok {
            anyhow::bail!("caractere inválido no caminho: {path}");
        }
    }
    Ok(())
}

/// Confere a assinatura Ed25519 sobre `MANIFEST_PREFIX || manifest_bytes`.
pub fn verify_signature(manifest: &[u8], sig_hex: &str, key: &[u8]) -> anyhow::Result<()> {
    if manifest.len() > MAX_MANIFEST_BYTES {
        anyhow::bail!("manifesto da página excede o limite de tamanho (16 KiB)");
    }
    let sig_bytes = hex::decode(sig_hex.trim())
        .map_err(|e| anyhow::anyhow!("formato de assinatura inválido: {e}"))?;
    if sig_bytes.len() != 64 {
        anyhow::bail!("tamanho de assinatura Ed25519 inválido");
    }

    let mut msg = Vec::with_capacity(MANIFEST_PREFIX.len() + manifest.len());
    msg.extend_from_slice(MANIFEST_PREFIX);
    msg.extend_from_slice(manifest);

    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key)
        .verify(&msg, &sig_bytes)
        .map_err(|_| {
            anyhow::anyhow!("a página do espectador não confere com a assinatura do LiveScreen")
        })?;

    Ok(())
}

/// Validação pura e síncrona do manifesto e dos conteúdos fornecidos por `fetch`.
pub fn check(
    manifest_bytes: &[u8],
    sig_hex: &str,
    key: &[u8],
    mut fetch: impl FnMut(&str) -> Option<Vec<u8>>,
) -> anyhow::Result<()> {
    verify_signature(manifest_bytes, sig_hex, key)?;

    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| anyhow::anyhow!("manifesto JSON inválido: {e}"))?;

    if manifest.files.is_empty() {
        anyhow::bail!("manifesto não contém arquivos");
    }
    if manifest.files.len() > MAX_FILES {
        anyhow::bail!(
            "manifesto excede o limite de 64 arquivos ({})",
            manifest.files.len()
        );
    }

    for (rel_path, expected_hash) in &manifest.files {
        validate_path(rel_path)?;

        let content = fetch(rel_path)
            .ok_or_else(|| anyhow::anyhow!("arquivo não encontrado: {rel_path}"))?;

        if content.len() > MAX_FILE_BYTES {
            anyhow::bail!("arquivo {rel_path} excede o limite de tamanho (2 MiB)");
        }

        let digest = ring::digest::digest(&ring::digest::SHA256, &content);
        let got_hash = hex::encode(digest.as_ref());

        if !got_hash.eq_ignore_ascii_case(expected_hash.trim()) {
            anyhow::bail!("conteúdo do arquivo {rel_path} não confere com o hash assinado");
        }
    }

    Ok(())
}

/// Identifica se o endereço base aponta para loopback local (dev com wrangler).
pub fn is_loopback(base: &str) -> bool {
    if let Ok(url) = reqwest::Url::parse(base) {
        if let Some(host) = url.host_str() {
            let h = host.trim_start_matches('[').trim_end_matches(']');
            if h.eq_ignore_ascii_case("localhost") {
                return true;
            }
            if let Ok(ip) = h.parse::<std::net::IpAddr>() {
                return ip.is_loopback();
            }
        }
        false
    } else {
        let lower = base.to_ascii_lowercase();
        lower.contains("localhost") || lower.contains("127.0.0.1") || lower.contains("::1")
    }
}

/// Confere se o endereço base já está verificado com sucesso ou se é loopback.
pub fn is_verified(base: &str) -> bool {
    if is_loopback(base) {
        return true;
    }
    let cache = VERIFY_CACHE.lock().unwrap();
    if let Some((cached_base, instant)) = cache.as_ref() {
        if cached_base == base && instant.elapsed() < Duration::from_secs(600) {
            return true;
        }
    }
    false
}

/// Limpa o cache de verificação (útil em testes).
#[cfg(test)]
pub fn clear_cache() {
    let mut cache = VERIFY_CACHE.lock().unwrap();
    *cache = None;
}

/// Lê o corpo de uma resposta HTTP em streaming até `max_bytes`.
/// Aborta imediatamente assim que o limite de bytes for excedido.
pub async fn read_capped_body(
    mut resp: reqwest::Response,
    max_bytes: usize,
) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        if buf.len() + chunk.len() > max_bytes {
            anyhow::bail!("resposta excede o limite de tamanho ({max_bytes} bytes)");
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// Lê o corpo de uma resposta HTTP em streaming até `max_bytes`, computando o SHA-256
/// diretamente nos pedaços sem acumular o arquivo inteiro na memória.
/// Aborta imediatamente se o limite for excedido.
pub async fn stream_sha256_capped(
    mut resp: reqwest::Response,
    max_bytes: usize,
) -> anyhow::Result<[u8; 32]> {
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    let mut total: usize = 0;
    while let Some(chunk) = resp.chunk().await? {
        total = total
            .checked_add(chunk.len())
            .ok_or_else(|| anyhow::anyhow!("overflow no tamanho do arquivo"))?;
        if total > max_bytes {
            anyhow::bail!("arquivo excede o limite de tamanho ({max_bytes} bytes)");
        }
        ctx.update(&chunk);
    }
    let digest = ctx.finish();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(digest.as_ref());
    Ok(hash)
}

/// Baixa o manifesto da página web remota e valida sua assinatura e os hashes de todos os arquivos.
/// Endereços de loopback pulam a verificação. Sucessos são cacheados por ~10 minutos.
pub async fn verify(base: &str) -> anyhow::Result<()> {
    if is_loopback(base) {
        return Ok(());
    }

    // Cache de sucesso por ~10 min
    if is_verified(base) {
        return Ok(());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let base_trimmed = base.trim_end_matches('/');
    let manifest_url = format!("{base_trimmed}/v/manifest.json");
    let sig_url = format!("{base_trimmed}/v/manifest.json.sig");

    let manifest_resp = client
        .get(&manifest_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("falha ao baixar {manifest_url}: {e}"))?;
    if !manifest_resp.status().is_success() {
        anyhow::bail!(
            "falha ao baixar {manifest_url}: status {}",
            manifest_resp.status()
        );
    }
    let manifest_bytes = read_capped_body(manifest_resp, MAX_MANIFEST_BYTES)
        .await
        .map_err(|e| anyhow::anyhow!("falha ao ler {manifest_url}: {e}"))?;

    let sig_resp = client
        .get(&sig_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("falha ao baixar {sig_url}: {e}"))?;
    if !sig_resp.status().is_success() {
        anyhow::bail!("falha ao baixar {sig_url}: status {}", sig_resp.status());
    }
    let sig_bytes = read_capped_body(sig_resp, MAX_SIG_BYTES)
        .await
        .map_err(|e| anyhow::anyhow!("falha ao ler {sig_url}: {e}"))?;
    let sig_text = std::str::from_utf8(&sig_bytes)
        .map_err(|e| anyhow::anyhow!("assinatura inválida (não UTF-8): {e}"))?
        .trim();

    let pubkey_bytes = hex::decode(RELEASE_PUBLIC_KEY)
        .map_err(|e| anyhow::anyhow!("chave pública de release inválida: {e}"))?;

    // Pré-valida assinatura e manifesto
    verify_signature(&manifest_bytes, sig_text, &pubkey_bytes)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| anyhow::anyhow!("manifesto JSON inválido: {e}"))?;
    if manifest.files.is_empty() {
        anyhow::bail!("manifesto não contém arquivos");
    }
    if manifest.files.len() > MAX_FILES {
        anyhow::bail!(
            "manifesto excede o limite de 64 arquivos ({})",
            manifest.files.len()
        );
    }

    // Baixa cada arquivo em streaming e confere hash sem acumular os arquivos na memória
    for (rel_path, expected_hash) in &manifest.files {
        validate_path(rel_path)?;
        let file_url = format!("{base_trimmed}/v/{rel_path}");
        let resp = client
            .get(&file_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("falha ao baixar {file_url}: {e}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("falha ao baixar {file_url}: status {}", resp.status());
        }
        let hash = stream_sha256_capped(resp, MAX_FILE_BYTES)
            .await
            .map_err(|e| anyhow::anyhow!("falha ao ler {file_url}: {e}"))?;
        let got_hash = hex::encode(hash);
        if !got_hash.eq_ignore_ascii_case(expected_hash.trim()) {
            anyhow::bail!("conteúdo do arquivo {rel_path} não confere com o hash assinado");
        }
    }

    // Sucesso: atualiza o cache
    {
        let mut cache = VERIFY_CACHE.lock().unwrap();
        *cache = Some((base.to_string(), Instant::now()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn make_test_keypair() -> (Ed25519KeyPair, Vec<u8>) {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let pubkey = pair.public_key().as_ref().to_vec();
        (pair, pubkey)
    }

    fn sign_manifest(pair: &Ed25519KeyPair, manifest_bytes: &[u8]) -> String {
        let mut msg = Vec::new();
        msg.extend_from_slice(MANIFEST_PREFIX);
        msg.extend_from_slice(manifest_bytes);
        hex::encode(pair.sign(&msg).as_ref())
    }

    #[test]
    fn test_valid_manifest_new_file_set_passes() {
        let (pair, pubkey) = make_test_keypair();
        // Modela host release N conferindo página N+1 com novos arquivos
        let files_data = [
            ("index.html", b"<html>v3</html>".as_slice()),
            ("pq.js", b"console.log('pq')".as_slice()),
            ("future/extra.js", b"function future() {}".as_slice()),
        ];
        let mut files_json = String::from("{\"files\":{");
        for (i, (name, content)) in files_data.iter().enumerate() {
            let digest = hex::encode(ring::digest::digest(&ring::digest::SHA256, content).as_ref());
            if i > 0 {
                files_json.push(',');
            }
            files_json.push_str(&format!("\"{name}\":\"{digest}\""));
        }
        files_json.push_str("}}");

        let manifest_bytes = files_json.into_bytes();
        let sig = sign_manifest(&pair, &manifest_bytes);

        let res = check(&manifest_bytes, &sig, &pubkey, |path| {
            files_data
                .iter()
                .find(|(n, _)| *n == path)
                .map(|(_, c)| c.to_vec())
        });
        assert!(res.is_ok(), "validação falhou: {:?}", res.err());
    }

    #[test]
    fn test_tampered_file_fails() {
        let (pair, pubkey) = make_test_keypair();
        let manifest = r#"{"files":{"viewer.js":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}}"#;
        let manifest_bytes = manifest.as_bytes();
        let sig = sign_manifest(&pair, manifest_bytes);

        // Retorna bytes adulterados (não bate com o hash)
        let res = check(manifest_bytes, &sig, &pubkey, |_| {
            Some(b"conteudo adulterado".to_vec())
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("não confere com o hash assinado"));
    }

    #[test]
    fn test_tampered_manifest_fails() {
        let (pair, pubkey) = make_test_keypair();
        let manifest = r#"{"files":{"index.html":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}}"#;
        let sig = sign_manifest(&pair, manifest.as_bytes());

        let tampered = r#"{"files":{"index.html":"0000000000000000000000000000000000000000000000000000000000000000"}}"#;
        let res = check(tampered.as_bytes(), &sig, &pubkey, |_| Some(vec![]));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("não confere com a assinatura"));
    }

    #[test]
    fn test_tampered_signature_fails() {
        let (_pair, pubkey) = make_test_keypair();
        let manifest = r#"{"files":{"index.html":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}}"#;
        let bad_sig = "00".repeat(64);

        let res = check(manifest.as_bytes(), &bad_sig, &pubkey, |_| Some(vec![]));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("não confere com a assinatura"));
    }

    #[test]
    fn test_bad_paths_rejected() {
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("/absolute/path").is_err());
        assert!(validate_path("\\windows\\path").is_err());
        assert!(validate_path("path with spaces.js").is_err());
        assert!(validate_path("valid-path_1.2/sub.dir/file.ext").is_ok());
    }

    #[test]
    fn test_limits_rejected() {
        let (pair, pubkey) = make_test_keypair();

        // Limite de manifesto (16 KiB)
        let huge_manifest = vec![b'a'; MAX_MANIFEST_BYTES + 1];
        let huge_sig = sign_manifest(&pair, &huge_manifest);
        assert!(check(&huge_manifest, &huge_sig, &pubkey, |_| None).is_err());

        // Limite de arquivo (2 MiB)
        let manifest = r#"{"files":{"huge.bin":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}}"#;
        let sig = sign_manifest(&pair, manifest.as_bytes());
        let res = check(manifest.as_bytes(), &sig, &pubkey, |_| {
            Some(vec![0u8; MAX_FILE_BYTES + 1])
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("excede o limite de tamanho"));
    }

    #[test]
    fn test_loopback_skip() {
        assert!(is_loopback("http://127.0.0.1:8787"));
        assert!(is_loopback("http://localhost:8787"));
        assert!(is_loopback("http://[::1]:8787"));
        assert!(!is_loopback("https://livescreen.pendiego.com"));
    }

    #[tokio::test]
    async fn test_verify_loopback_skips_network() {
        clear_cache();
        assert!(verify("http://127.0.0.1:8787").await.is_ok());
        assert!(verify("http://localhost:8787").await.is_ok());
    }

    async fn spawn_test_server(body: Vec<u8>) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = sock.read(&mut buf).await;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = sock.write_all(header.as_bytes()).await;
                let _ = sock.write_all(&body).await;
            }
        });
        format!("http://{addr}")
    }

    fn ensure_crypto() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    #[tokio::test]
    async fn test_read_capped_body_ok_and_exceeded() {
        ensure_crypto();
        let client = reqwest::Client::new();

        let url_ok = spawn_test_server(vec![b'x'; 64]).await;
        let resp = client.get(&url_ok).send().await.unwrap();
        let bytes = read_capped_body(resp, 128).await.unwrap();
        assert_eq!(bytes.len(), 64);

        let url_over = spawn_test_server(vec![b'x'; 200]).await;
        let resp = client.get(&url_over).send().await.unwrap();
        let err = read_capped_body(resp, 100).await.unwrap_err();
        assert!(err.to_string().contains("excede o limite de tamanho"));
    }

    #[tokio::test]
    async fn test_stream_sha256_capped_ok_and_exceeded() {
        ensure_crypto();
        let client = reqwest::Client::new();
        let data = b"hello streaming sha256 verification test payload".to_vec();
        let expected_hash = ring::digest::digest(&ring::digest::SHA256, &data);

        let url_ok = spawn_test_server(data.clone()).await;
        let resp = client.get(&url_ok).send().await.unwrap();
        let hash = stream_sha256_capped(resp, 1024).await.unwrap();
        assert_eq!(hash, expected_hash.as_ref());

        let url_over = spawn_test_server(vec![b'z'; 500]).await;
        let resp = client.get(&url_over).send().await.unwrap();
        let err = stream_sha256_capped(resp, 100).await.unwrap_err();
        assert!(err.to_string().contains("excede o limite de tamanho"));
    }
}
