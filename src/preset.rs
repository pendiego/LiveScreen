#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Preset {
    P720p30,
    #[default]
    P1080p60,
    Native60,
}

impl Preset {
    pub const ALL: [Preset; 3] = [Preset::P720p30, Preset::P1080p60, Preset::Native60];
    pub fn label(self) -> &'static str {
        match self {
            Preset::P720p30 => "720p 30fps",
            Preset::P1080p60 => "1080p 60fps",
            Preset::Native60 => "Nativa 60fps",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeParams {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate_bps: u32,
}

pub fn encode_params(preset: Preset, src_w: u32, src_h: u32) -> EncodeParams {
    let (bw, bh, fps) = match preset {
        Preset::P720p30 => (1280, 720, 30),
        Preset::P1080p60 => (1920, 1080, 60),
        Preset::Native60 => (src_w.max(1), src_h.max(1), 60),
    };
    let (sw, sh) = (src_w.max(1) as u64, src_h.max(1) as u64);
    // encaixa sem ampliar
    let (mut w, mut h) = if sw <= bw as u64 && sh <= bh as u64 {
        (sw, sh)
    } else if sw * bh as u64 >= sh * bw as u64 {
        (bw as u64, sh * bw as u64 / sw)
    } else {
        (sw * bh as u64 / sh, bh as u64)
    };
    w = (w & !1).max(64);
    h = (h & !1).max(64);
    let bitrate_bps = match preset {
        Preset::P720p30 => 4_000_000,
        Preset::P1080p60 => 8_000_000,
        Preset::Native60 => {
            (8_000_000u64 * w * h / (1920 * 1080)).clamp(6_000_000, 25_000_000) as u32
        }
    };
    EncodeParams {
        width: w as u32,
        height: h as u32,
        fps,
        bitrate_bps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sizes() {
        let p = encode_params(Preset::P1080p60, 2560, 1440);
        assert_eq!((p.width, p.height), (1920, 1080));
        let p = encode_params(Preset::P1080p60, 1366, 768);
        assert_eq!((p.width, p.height), (1366, 768));
        let p = encode_params(Preset::Native60, 1001, 601);
        assert_eq!((p.width, p.height), (1000, 600));
    }
    #[test]
    fn bitrate() {
        assert_eq!(
            encode_params(Preset::Native60, 640, 480).bitrate_bps,
            6_000_000
        );
        assert_eq!(
            encode_params(Preset::Native60, 7680, 4320).bitrate_bps,
            25_000_000
        );
        assert_eq!(
            encode_params(Preset::P720p30, 1920, 1080).bitrate_bps,
            4_000_000
        );
    }
}
