//! Preferências da interface, persistidas em `%APPDATA%\LiveScreen\ui.json`.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::Overrides;
use livescreen_core::preset::Preset;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub preset: Preset,
    pub share_audio: bool,
    /// Transmitir também o microfone (desligado por padrão, por privacidade).
    pub share_mic: bool,
    /// Id do microfone escolhido; `None` = padrão do Windows.
    pub mic_device: Option<String>,
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    /// Última janela em estado normal: x e y externos, largura e altura internas (pontos).
    pub window: Option<[f32; 4]>,
    /// Limite de pessoas na sala; quem transmite aumenta quando quiser.
    pub room_limit: usize,
    /// Baixar a qualidade sozinho quando o encoder perde quadros (ShareStatus.overloaded).
    pub auto_quality: bool,
    /// Aplicar pedidos de qualidade mais leve de quem assiste.
    pub accept_quality_requests: bool,
    pub hardware_decode: bool,
    /// Mostrar o quadro decodificado direto da textura da placa de vídeo (sem cópia pela CPU).
    pub zero_copy: bool,
    pub hardware_encoder: bool,
    pub show_stats: bool,
    pub check_updates: bool,
    pub graphics_capture: bool,
    pub dxgi_capture: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            preset: Preset::P1080p60,
            share_audio: true,
            share_mic: false,
            mic_device: None,
            name: String::new(),
            volume: 1.0,
            muted: false,
            window: None,
            room_limit: 4,
            auto_quality: true,
            accept_quality_requests: true,
            hardware_decode: true,
            zero_copy: true,
            hardware_encoder: true,
            show_stats: false,
            check_updates: true,
            graphics_capture: true,
            dxgi_capture: true,
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

    /// `config.json` e a variável de ambiente só negam, então sempre vencem.
    pub fn apply(&self, o: Overrides) -> Overrides {
        Overrides {
            deny_graphics_capture: o.deny_graphics_capture || !self.graphics_capture,
            deny_dxgi: o.deny_dxgi || !self.dxgi_capture,
            deny_hardware_encoder: o.deny_hardware_encoder || !self.hardware_encoder,
            deny_hardware_decoder: o.deny_hardware_decoder || !self.hardware_decode,
            deny_zero_copy: o.deny_zero_copy || !self.zero_copy,
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

    #[test]
    fn old_ui_json_keeps_new_defaults() {
        let p = Prefs::parse(r#"{"preset":"P720p30"}"#);
        assert!(p.hardware_decode && p.hardware_encoder && p.check_updates);
        assert!(p.graphics_capture && p.dxgi_capture);
        assert!(!p.show_stats);
    }

    #[test]
    fn old_theme_field_is_ignored() {
        assert_eq!(Prefs::parse(r#"{"theme":"Light","volume":0.5}"#).volume, 0.5);
    }

    #[test]
    fn file_overrides_win() {
        let o = Overrides {
            deny_hardware_decoder: true,
            ..Default::default()
        };
        assert!(Prefs::default().apply(o).deny_hardware_decoder);
        let p = Prefs {
            hardware_encoder: false,
            ..Default::default()
        };
        let o = p.apply(Overrides::default());
        assert!(o.deny_hardware_encoder);
        assert!(!o.deny_hardware_decoder && !o.deny_graphics_capture && !o.deny_dxgi);
    }

    #[test]
    fn zero_copy_pref_denies_zero_copy() {
        assert!(!Prefs::default().apply(Overrides::default()).deny_zero_copy);
        assert!(Prefs { zero_copy: false, ..Prefs::default() }.apply(Overrides::default()).deny_zero_copy);
        assert!(Prefs::default().apply(Overrides { deny_zero_copy: true, ..Default::default() }).deny_zero_copy);
    }
}
