//! Preferências da interface, persistidas em `%APPDATA%\LiveScreen\ui.json`.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::preset::Preset;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub preset: Preset,
    pub share_audio: bool,
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    /// Última janela em estado normal: x e y externos, largura e altura internas (pontos).
    pub window: Option<[f32; 4]>,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            preset: Preset::P1080p60,
            share_audio: true,
            name: String::new(),
            volume: 1.0,
            muted: false,
            window: None,
        }
    }
}

impl Prefs {
    pub fn parse(s: &str) -> Prefs {
        serde_json::from_str(s).unwrap_or_else(|e| {
            warn!("ui.json inválido: {e}");
            Prefs::default()
        })
    }

    pub fn load() -> Prefs {
        match std::fs::read_to_string(crate::config::app_dir().join("ui.json")) {
            Ok(s) => Prefs::parse(&s),
            Err(_) => Prefs::default(),
        }
    }

    pub fn save(&self) {
        let dir = crate::config::app_dir();
        let tmp = dir.join("ui.json.tmp");
        let res = serde_json::to_string_pretty(self)
            .map_err(|e| e.to_string())
            .and_then(|j| std::fs::write(&tmp, j).map_err(|e| e.to_string()))
            .and_then(|_| std::fs::rename(&tmp, dir.join("ui.json")).map_err(|e| e.to_string()));
        if let Err(e) = res {
            warn!("não foi possível salvar ui.json: {e}");
        }
    }

    /// Ganho linear para o áudio do espectador (curva perceptual volume²).
    pub fn gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume.clamp(0.0, 1.0).powi(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_json_gives_default() {
        assert_eq!(Prefs::parse("{"), Prefs::default());
    }

    #[test]
    fn partial_json_keeps_defaults() {
        let p = Prefs::parse(r#"{"preset":"Native60"}"#);
        assert_eq!(p.preset, Preset::Native60);
        assert_eq!(p.volume, 1.0);
    }

    #[test]
    fn window_roundtrip() {
        assert_eq!(
            Prefs::parse(r#"{"window":[10.0,20.0,900.0,600.0]}"#).window,
            Some([10.0, 20.0, 900.0, 600.0])
        );
    }
}
