//! Ferramenta de assinatura offline para as releases e o manifesto web do LiveScreen.
//! Gera chaves Ed25519, extrai a chave pública e assina executáveis e manifestos web.
use std::path::Path;
use std::process::ExitCode;

use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
const MANIFEST_PREFIX: &[u8] = b"livescreen/web-manifest/v1\n";

fn usage() -> ExitCode {
    eprintln!(
        "uso:\n  \
         sign_release keygen <arquivo.pk8>\n  \
         sign_release pubkey <arquivo.pk8>\n  \
         sign_release sign <arquivo.pk8> <versão> <arquivo.exe>\n  \
         sign_release sign-web <arquivo.pk8> <manifest.json>"
    );
    ExitCode::from(2)
}

fn keygen(path: &str) -> Result<(), (String, u8)> {
    let p = Path::new(path);
    if p.exists() {
        return Err((format!("já existe: {path}"), 1));
    }
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| (format!("falha ao criar diretórios: {e}"), 1))?;
    }
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|e| (format!("falha ao gerar chave: {e}"), 1))?;
    std::fs::write(p, pkcs8.as_ref())
        .map_err(|e| (format!("falha ao gravar chave: {e}"), 1))?;
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())
        .map_err(|e| (format!("falha ao ler chave gerada: {e}"), 1))?;
    println!("{}", hex::encode(pair.public_key().as_ref()));
    Ok(())
}

fn pubkey(path: &str) -> Result<(), (String, u8)> {
    let bytes = std::fs::read(path)
        .map_err(|e| (format!("falha ao ler chave {path}: {e}"), 1))?;
    let pair = Ed25519KeyPair::from_pkcs8(&bytes)
        .map_err(|e| (format!("chave PKCS#8 inválida: {e}"), 1))?;
    println!("{}", hex::encode(pair.public_key().as_ref()));
    Ok(())
}

fn sign(key_path: &str, version: &str, exe_path: &str) -> Result<(), (String, u8)> {
    let key_bytes = std::fs::read(key_path)
        .map_err(|e| (format!("falha ao ler chave {key_path}: {e}"), 1))?;
    let pair = Ed25519KeyPair::from_pkcs8(&key_bytes)
        .map_err(|e| (format!("chave PKCS#8 inválida: {e}"), 1))?;
    let exe_bytes = std::fs::read(exe_path)
        .map_err(|e| (format!("falha ao ler executável {exe_path}: {e}"), 1))?;

    let digest = ring::digest::digest(&ring::digest::SHA256, &exe_bytes);
    let sha = hex::encode(digest.as_ref());
    // Mesmo formato de update::signed_message
    let msg = format!("livescreen-update-v1\n{version}\n{sha}");

    let sig = pair.sign(msg.as_bytes());
    UnparsedPublicKey::new(&ED25519, pair.public_key().as_ref())
        .verify(msg.as_bytes(), sig.as_ref())
        .map_err(|e| (format!("falha ao verificar assinatura gerada: {e}"), 1))?;

    println!("{}", hex::encode(sig.as_ref()));
    Ok(())
}

fn sign_web(key_path: &str, manifest_path: &str) -> Result<(), (String, u8)> {
    let key_bytes = std::fs::read(key_path)
        .map_err(|e| (format!("falha ao ler chave {key_path}: {e}"), 1))?;
    let pair = Ed25519KeyPair::from_pkcs8(&key_bytes)
        .map_err(|e| (format!("chave PKCS#8 inválida: {e}"), 1))?;
    let manifest_bytes = std::fs::read(manifest_path)
        .map_err(|e| (format!("falha ao ler manifesto {manifest_path}: {e}"), 1))?;

    let mut msg = Vec::with_capacity(MANIFEST_PREFIX.len() + manifest_bytes.len());
    msg.extend_from_slice(MANIFEST_PREFIX);
    msg.extend_from_slice(&manifest_bytes);

    let sig = pair.sign(&msg);
    UnparsedPublicKey::new(&ED25519, pair.public_key().as_ref())
        .verify(&msg, sig.as_ref())
        .map_err(|e| (format!("falha ao verificar assinatura gerada: {e}"), 1))?;

    println!("{}", hex::encode(sig.as_ref()));
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return usage();
    }
    let res = match (args[1].as_str(), args.len()) {
        ("keygen", 3) => keygen(&args[2]),
        ("pubkey", 3) => pubkey(&args[2]),
        ("sign", 5) => sign(&args[2], &args[3], &args[4]),
        ("sign-web", 4) => sign_web(&args[2], &args[3]),
        _ => return usage(),
    };
    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err((msg, code)) => {
            eprintln!("{msg}");
            ExitCode::from(code)
        }
    }
}
