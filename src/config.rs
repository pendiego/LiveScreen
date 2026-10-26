use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub capture_overrides: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Overrides {
    pub deny_graphics_capture: bool,
    pub deny_dxgi: bool,
    pub deny_hardware_encoder: bool,
    pub deny_hardware_decoder: bool,
    pub deny_zero_copy: bool,
}

impl Overrides {
    pub fn parse(items: &[String]) -> Overrides {
        let mut o = Overrides::default();
        for x in items {
            match x.trim() {
                "deny-graphics-capture" => o.deny_graphics_capture = true,
                "deny-dxgi" => o.deny_dxgi = true,
                "deny-hardware-encoder" => o.deny_hardware_encoder = true,
                "deny-hardware-decoder" => o.deny_hardware_decoder = true,
                "deny-zero-copy" => o.deny_zero_copy = true,
                "" => {}
                other => warn!("override desconhecido: {other}"),
            }
        }
        o
    }
}

pub fn app_dir() -> std::path::PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let d = base.join("LiveScreen");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn load() -> Config {
    if let Ok(v) = std::env::var("LIVESCREEN_CAPTURE_OVERRIDES") {
        return Config {
            capture_overrides: v.split(',').map(|s| s.to_string()).collect(),
        };
    }
    match std::fs::read_to_string(app_dir().join("config.json")) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            warn!("config.json inválido: {e}");
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

pub fn overrides() -> Overrides {
    Overrides::parse(&load().capture_overrides)
}

/// Identidade desta janela. `temporary` = outra janela já reservou a persistente
/// (ou identity.key está ilegível): amigos ficam desligados nesta execução.
pub struct Identity {
    pub key: iroh::SecretKey,
    pub temporary: bool,
    _lock: Option<std::fs::File>,
}

impl Identity {
    pub fn into_parts(self) -> (iroh::SecretKey, bool, Option<std::fs::File>) {
        (self.key, self.temporary, self._lock)
    }
}

pub fn identity() -> Identity {
    reserve_identity(&app_dir(), std::time::Duration::from_secs(3))
}

fn temporary_identity() -> Identity {
    Identity {
        key: livescreen_core::friends::new_secret_key(),
        temporary: true,
        _lock: None,
    }
}

fn reserve_identity(dir: &std::path::Path, wait: std::time::Duration) -> Identity {
    use std::fs::TryLockError;
    let file = match std::fs::File::options()
        .create(true)
        .write(true)
        .truncate(false)
        .open(dir.join("identity.lock"))
    {
        Ok(f) => Some(f),
        Err(e) => {
            warn!("não foi possível abrir identity.lock: {e}");
            None
        }
    };
    let mut lock = None;
    if let Some(f) = file {
        let deadline = std::time::Instant::now() + wait;
        loop {
            match f.try_lock() {
                Ok(()) => {
                    lock = Some(f);
                    break;
                }
                Err(TryLockError::WouldBlock) => {
                    if std::time::Instant::now() >= deadline {
                        warn!("identidade em uso por outra janela; usando chave temporária");
                        return temporary_identity();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(TryLockError::Error(e)) => {
                    warn!("não foi possível travar identity.lock: {e}");
                    break;
                }
            }
        }
    }
    match load_identity(&dir.join("identity.key")) {
        Some(key) => Identity {
            key,
            temporary: false,
            _lock: lock,
        },
        None => temporary_identity(),
    }
}

fn load_identity(path: &std::path::Path) -> Option<iroh::SecretKey> {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            if let Some(key) = livescreen_core::friends::parse_secret_key(&s) {
                return Some(key);
            }
            let bad = {
                let mut s = path.as_os_str().to_os_string();
                s.push(".bad");
                std::path::PathBuf::from(s)
            };
            warn!("identity.key inválido; renomeando para {}", bad.display());
            let _ = std::fs::rename(path, bad);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            warn!("não foi possível ler identity.key: {e}");
            return None;
        }
    }
    let key = livescreen_core::friends::new_secret_key();
    let hex = livescreen_core::friends::secret_key_hex(&key);
    let tmp = {
        let mut s = path.as_os_str().to_os_string();
        s.push(".tmp");
        std::path::PathBuf::from(s)
    };
    if let Err(e) = std::fs::write(&tmp, &hex).and_then(|_| std::fs::rename(&tmp, path)) {
        warn!("não foi possível salvar identity.key: {e}");
    }
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_overrides() {
        let o = Overrides::parse(&[
            " deny-dxgi ".into(),
            "foo".into(),
            "deny-hardware-encoder".into(),
            "deny-hardware-decoder".into(),
            "deny-zero-copy".into(),
        ]);
        assert!(o.deny_dxgi && o.deny_hardware_encoder && !o.deny_graphics_capture);
        assert!(o.deny_hardware_decoder && o.deny_zero_copy);
    }

    #[test]
    fn identity_persistence_and_bad_recovery() {
        let dir = std::env::temp_dir().join(format!(
            "ls_test_id_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("identity.key");

        let k1 = load_identity(&path).unwrap();
        let k2 = load_identity(&path).unwrap();
        assert_eq!(k1.public(), k2.public());

        std::fs::write(&path, "lixo-invalido-de-tamanho-incorreto").unwrap();
        let k3 = load_identity(&path).unwrap();
        assert_ne!(k3.public(), k1.public());
        let bad_path = dir.join("identity.key.bad");
        assert!(bad_path.exists());
        assert_eq!(
            std::fs::read_to_string(&bad_path).unwrap(),
            "lixo-invalido-de-tamanho-incorreto"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn second_window_gets_temporary_identity() {
        let dir = std::env::temp_dir().join(format!(
            "ls_test_lock_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let zero = std::time::Duration::ZERO;
        let a = reserve_identity(&dir, zero);
        let b = reserve_identity(&dir, zero);
        assert!(!a.temporary);
        assert!(b.temporary);
        assert_ne!(b.key.public(), a.key.public());
        let a_pub = a.key.public();
        drop(a);
        let c = reserve_identity(&dir, zero);
        assert!(!c.temporary);
        assert_eq!(c.key.public(), a_pub);
        drop((b, c));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
