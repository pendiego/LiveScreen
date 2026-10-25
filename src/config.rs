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
}
