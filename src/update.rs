//! Atualização pelo GitHub Releases: compara a release `latest` com a versão do app, baixa o
//! `livescreen.exe` publicado nela, confere o sha256 e a assinatura Ed25519 offline sobre versão
//! e sha256 antes de trocar o executável.
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use sha2::{Digest, Sha256};
use tracing::{info, warn};

const API: &str = "https://api.github.com/repos/pendiego/LiveScreen/releases/latest";
const ASSET: &str = "livescreen.exe";
const SIG_ASSET: &str = "livescreen.exe.sig";
const UPDATE_KEY: &str = "c46025005551cda3e63968e0b6a71bccd47a6411276b85aff966f93d81290930";

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub sig_url: String,
    pub sha256: String,
}

#[derive(serde::Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(serde::Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}

pub fn client() -> Result<reqwest::Client> {
    // reqwest vem sem provider de cripto (feature do iroh); instala o ring, já compilado
    let _ = rustls::crypto::ring::default_provider().install_default();
    Ok(reqwest::Client::builder()
        .user_agent(concat!("LiveScreen/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300))
        .build()?)
}

/// `Some` só quando a release publicada é mais nova e traz o executável com hash.
pub async fn check() -> Result<Option<Release>> {
    let body = client()?
        .get(API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(pick(
        serde_json::from_slice(&body)?,
        env!("CARGO_PKG_VERSION"),
    ))
}

fn version(v: &str) -> Option<(u64, u64, u64)> {
    let core = v.trim_start_matches('v').split(['-', '+']).next()?;
    let mut it = core.split('.').map(|p| p.parse::<u64>().ok());
    let r = (it.next()??, it.next()??, it.next()??);
    it.next().is_none().then_some(r)
}

fn pick(r: GhRelease, current: &str) -> Option<Release> {
    let newer = version(&r.tag_name)? > version(current)?;
    if !newer {
        return None;
    }
    let sig_url = r
        .assets
        .iter()
        .find(|a| a.name == SIG_ASSET)
        .map(|a| a.browser_download_url.clone());
    let asset = r.assets.into_iter().find(|a| a.name == ASSET);
    let found = asset.and_then(|a| {
        let sha = a.digest?.strip_prefix("sha256:")?.to_ascii_lowercase();
        let sig_url = sig_url?;
        Some(Release {
            version: r.tag_name.trim_start_matches('v').to_string(),
            url: a.browser_download_url,
            sig_url,
            sha256: sha,
        })
    });
    if found.is_none() {
        warn!(
            "release {} sem {ASSET} com sha256 ou sem {SIG_ASSET}; ignorada",
            r.tag_name
        );
    }
    found
}

fn signed_message(version: &str, sha256_hex: &str) -> String {
    format!("livescreen-update-v1\n{version}\n{sha256_hex}")
}

fn signature_ok(pubkey: &[u8], version: &str, sha256_hex: &str, sig_hex: &str) -> bool {
    let Ok(sig) = hex::decode(sig_hex.trim()) else {
        return false;
    };
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, pubkey)
        .verify(signed_message(version, sha256_hex).as_bytes(), &sig)
        .is_ok()
}

/// Baixa e confere o hash e a assinatura; o executável só é trocado se ambos baterem.
pub async fn download(rel: &Release) -> Result<Vec<u8>> {
    let sig_bytes = client()?
        .get(&rel.sig_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    if sig_bytes.len() > 1024 {
        bail!("assinatura da atualização inválida");
    }
    let sig = String::from_utf8_lossy(&sig_bytes);

    let bytes = client()?
        .get(&rel.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let got = hex::encode(Sha256::digest(&bytes));
    if got != rel.sha256 {
        bail!(
            "download corrompido (sha256 {got}, esperado {})",
            rel.sha256
        );
    }
    if !signature_ok(&hex::decode(UPDATE_KEY)?, &rel.version, &got, &sig) {
        bail!("assinatura da atualização inválida: release ignorada");
    }
    Ok(bytes.to_vec())
}

fn sidecars(exe: &Path) -> (PathBuf, PathBuf) {
    (exe.with_extension("exe.new"), exe.with_extension("exe.old"))
}

/// Põe o novo executável no lugar do atual e o inicia. Um exe em uso no Windows pode ser
/// renomeado, mas não apagado: o antigo vira `.old` e a próxima execução o remove.
/// Qualquer falha devolve o executável original ao lugar.
pub fn install(bytes: &[u8]) -> Result<()> {
    install_at(&std::env::current_exe()?, bytes, |exe| {
        std::process::Command::new(exe).spawn().map(drop)
    })
}

fn install_at(
    exe: &Path,
    bytes: &[u8],
    launch: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<()> {
    let (new, old) = sidecars(exe);
    std::fs::write(&new, bytes)
        .with_context(|| format!("sem permissão para gravar em {}", new.display()))?;
    let _ = std::fs::remove_file(&old);
    if let Err(e) = std::fs::rename(exe, &old) {
        let _ = std::fs::remove_file(&new);
        return Err(anyhow!(e).context("não foi possível mover o executável atual"));
    }
    if let Err(e) = std::fs::rename(&new, exe) {
        let _ = std::fs::rename(&old, exe);
        let _ = std::fs::remove_file(&new);
        return Err(anyhow!(e).context("não foi possível instalar o novo executável"));
    }
    if let Err(e) = launch(exe) {
        let _ = std::fs::remove_file(exe);
        let _ = std::fs::rename(&old, exe);
        return Err(anyhow!(e).context("o novo executável não iniciou"));
    }
    info!("atualização instalada em {}", exe.display());
    Ok(())
}

/// Apaga sobras de uma atualização. O processo antigo pode levar alguns segundos para
/// sair e liberar o `.old`, então tenta por até 30 s em segundo plano.
pub fn cleanup() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    std::thread::spawn(move || {
        let (new, old) = sidecars(&exe);
        let _ = std::fs::remove_file(new);
        for _ in 0..60 {
            if !old.exists() || std::fs::remove_file(&old).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(tag: &str, assets: &[(&str, Option<&str>)]) -> GhRelease {
        GhRelease {
            tag_name: tag.into(),
            assets: assets
                .iter()
                .map(|(n, d)| GhAsset {
                    name: (*n).into(),
                    browser_download_url: format!("https://x/{n}"),
                    digest: d.map(Into::into),
                })
                .collect(),
        }
    }

    #[test]
    fn picks_only_newer_release_with_hashed_exe() {
        let ok = [
            ("livescreen.exe", Some("sha256:ABCD")),
            ("livescreen.exe.sig", None),
        ];
        assert_eq!(
            pick(rel("v0.2.0", &ok), "0.1.0"),
            Some(Release {
                version: "0.2.0".into(),
                url: "https://x/livescreen.exe".into(),
                sig_url: "https://x/livescreen.exe.sig".into(),
                sha256: "abcd".into(),
            })
        );
        // numérico, não lexicográfico
        assert!(pick(rel("v0.10.0", &ok), "0.9.0").is_some());
        assert!(pick(rel("v0.1.0", &ok), "0.1.0").is_none());
        assert!(pick(rel("v0.0.9", &ok), "0.1.0").is_none());
        assert!(pick(rel("nightly", &ok), "0.1.0").is_none());
        // só o zip, ou exe sem hash: não dá para conferir, não atualiza
        assert!(
            pick(
                rel(
                    "v0.2.0",
                    &[
                        ("livescreen-windows-x64.zip", Some("sha256:ab")),
                        ("livescreen.exe.sig", None),
                    ]
                ),
                "0.1.0"
            )
            .is_none()
        );
        assert!(pick(rel("v0.2.0", &[("livescreen.exe", None), ("livescreen.exe.sig", None)]), "0.1.0").is_none());
        // exe com digest mas sem .sig dá None
        assert!(pick(rel("v0.2.0", &[("livescreen.exe", Some("sha256:ABCD"))]), "0.1.0").is_none());
    }

    #[test]
    fn signature_binds_version_and_hash() {
        use ring::rand::SystemRandom;
        use ring::signature::{Ed25519KeyPair, KeyPair};

        let rng = SystemRandom::new();
        let doc1 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pair1 = Ed25519KeyPair::from_pkcs8(doc1.as_ref()).unwrap();

        let doc2 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pair2 = Ed25519KeyPair::from_pkcs8(doc2.as_ref()).unwrap();

        let sha = "ab".repeat(32);
        let msg = signed_message("0.9.0", &sha);
        let sig = pair1.sign(msg.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());

        assert!(signature_ok(pair1.public_key().as_ref(), "0.9.0", &sha, &sig_hex));
        // versão errada
        assert!(!signature_ok(pair1.public_key().as_ref(), "0.9.1", &sha, &sig_hex));
        // sha diferente
        assert!(!signature_ok(pair1.public_key().as_ref(), "0.9.0", &"cd".repeat(32), &sig_hex));
        // chave pública de outro par
        assert!(!signature_ok(pair2.public_key().as_ref(), "0.9.0", &sha, &sig_hex));
        // sig_hex inválido ou vazio
        assert!(!signature_ok(pair1.public_key().as_ref(), "0.9.0", &sha, "zz"));
        assert!(!signature_ok(pair1.public_key().as_ref(), "0.9.0", &sha, ""));
    }

    #[test]
    fn embedded_key_matches_offline_signer() {
        const FIXTURE_SIG: &str = "860b6ace56570dd406d6e000a39fc287f27b67aee08a2bef2024212b0292c7e533b5abd2393371da8f33f63e843ccfbccc5b4b6b518eaf57c0e36d90d640db01";
        assert!(signature_ok(
            &hex::decode(UPDATE_KEY).unwrap(),
            "0.0.0",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            FIXTURE_SIG
        ));
    }

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ls-upd-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d.join("livescreen.exe")
    }

    #[test]
    fn install_swaps_and_keeps_old_aside() {
        let exe = tmp("ok");
        std::fs::write(&exe, b"old").unwrap();
        install_at(&exe, b"new", |_| Ok(())).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"new");
        let (new, old) = sidecars(&exe);
        assert_eq!(std::fs::read(old).unwrap(), b"old");
        assert!(!new.exists());
    }

    #[test]
    fn install_rolls_back_when_launch_fails() {
        let exe = tmp("fail");
        std::fs::write(&exe, b"old").unwrap();
        let r = install_at(&exe, b"new", |_| Err(std::io::Error::other("216")));
        assert!(r.is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"old");
        let (new, old) = sidecars(&exe);
        assert!(!new.exists() && !old.exists());
    }
}
