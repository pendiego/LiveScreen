#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Preset {
    P480p30,
    P720p30,
    P720p60,
    P1080p30,
    #[default]
    P1080p60,
    P1440p60,
    P2160p60,
    Native60,
    /// Resolução da fonte na taxa de atualização do monitor (ex.: 144 Hz).
    NativeHigh,
}

impl Preset {
    /// Da mais leve à mais pesada; `options` mantém a primeira de cada imagem repetida.
    pub const ALL: [Preset; 9] = [
        Preset::P480p30,
        Preset::P720p30,
        Preset::P720p60,
        Preset::P1080p30,
        Preset::P1080p60,
        Preset::P1440p60,
        Preset::P2160p60,
        Preset::Native60,
        Preset::NativeHigh,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeParams {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate_bps: u32,
}

/// Quantas pessoas o upload aguenta: cada uma recebe a própria cópia do vídeo (o bitrate é o
/// teto, atingido com muito movimento) e do áudio (Opus a 128 kbps, em `audio::capture`).
/// 20% do upload fica livre para o resto da casa.
pub fn viewers_for(upload_bps: u64, video_bps: u32) -> usize {
    const AUDIO_BPS: u64 = 128_000;
    (upload_bps / 10 * 8 / (video_bps as u64 + AUDIO_BPS)) as usize
}

/// Teto do H.264 nível 5.2, o máximo que os encoders de GPU aceitam na prática: 36 864
/// macroblocos por quadro, 2 073 600 por segundo, lado até 4096 (NVENC recusa 5120×1440).
const MAX_SIDE: u64 = 4096;
const MAX_MBS: u64 = 36_864;
const MAX_MBS_PER_SEC: u64 = 2_073_600;

fn macroblocks(w: u64, h: u64) -> u64 {
    w.div_ceil(16) * h.div_ceil(16)
}

/// Tamanho alinhado (par, mínimo 64) e dentro do nível 5.2. A checagem vale para o tamanho
/// já alinhado, que é o devolvido. Escala proporcional: converge em 1–2 passos; o teto de
/// passos só existe para nunca virar laço infinito.
fn fit_level(w: f64, h: f64, fps: u64) -> (u64, u64) {
    let max_mbs = MAX_MBS.min(MAX_MBS_PER_SEC / fps.max(1)) as f64;
    let align = |v: f64| ((v.round().max(0.0) as u64) & !1).max(64);
    let mut s = 1.0f64;
    for _ in 0..64 {
        let (aw, ah) = (align(w * s), align(h * s));
        let k = (MAX_SIDE as f64 / aw as f64)
            .min(MAX_SIDE as f64 / ah as f64)
            .min((max_mbs / macroblocks(aw, ah) as f64).sqrt());
        if k >= 1.0 {
            return (aw, ah);
        }
        s *= k;
        // o arredondamento do alinhamento pode devolver o mesmo tamanho: aí força um passo
        if (align(w * s), align(h * s)) == (aw, ah) {
            s *= 0.995;
        }
    }
    (64, 64)
}

/// Teto de fps da "Nativa (monitor)": acima disso a decodificação de quem assiste (por
/// software) e o nível 5.2 começam a cortar resolução.
const MAX_FPS: u32 = 144;

/// Orçamento de pixels, fps e bitrate de cada faixa fixa.
fn tier(preset: Preset) -> Option<(u64, u64, u64)> {
    Some(match preset {
        Preset::P480p30 => (854 * 480, 30, 1_500_000),
        Preset::P720p30 => (1280 * 720, 30, 4_000_000),
        Preset::P720p60 => (1280 * 720, 60, 6_000_000),
        Preset::P1080p30 => (1920 * 1080, 30, 5_000_000),
        Preset::P1080p60 => (1920 * 1080, 60, 8_000_000),
        Preset::P1440p60 => (2560 * 1440, 60, 14_000_000),
        Preset::P2160p60 => (3840 * 2160, 60, 25_000_000),
        Preset::Native60 | Preset::NativeHigh => return None,
    })
}

/// `hz`: taxa de atualização do monitor da fonte (só a "Nativa (monitor)" usa).
pub fn encode_params(preset: Preset, src_w: u32, src_h: u32, hz: u32) -> EncodeParams {
    let (sw, sh) = (src_w.max(1) as u64, src_h.max(1) as u64);
    let native_fps = match preset {
        Preset::NativeHigh => hz.clamp(60, MAX_FPS) as u64,
        _ => 60,
    };
    // orçamento de pixels, não caixa fixa: ultrawide e tela em pé recebem a mesma área
    // (e o mesmo custo) que uma 16:9, sem virar faixa fina
    let (budget, fps) = tier(preset).map_or((u64::MAX, native_fps), |(b, f, _)| (b, f));
    // nunca amplia
    let s = (budget as f64 / (sw * sh) as f64).sqrt().min(1.0);
    let (w, h) = fit_level(sw as f64 * s, sh as f64 * s, fps);
    let bitrate_bps = match tier(preset) {
        // fonte menor que a faixa: imagem menor, bitrate proporcional
        Some((b, _, bps)) => (bps as f64 * ((w * h) as f64 / b as f64).min(1.0)).round() as u64,
        None => {
            let base = 8_000_000 * w * h / (1920 * 1080);
            if preset == Preset::Native60 {
                base.clamp(6_000_000, 25_000_000)
            } else {
                (base * fps / 60).clamp(6_000_000, 40_000_000)
            }
        }
    };
    EncodeParams {
        width: w as u32,
        height: h as u32,
        fps: fps as u32,
        bitrate_bps: bitrate_bps as u32,
    }
}

/// Opções que fazem sentido para esta fonte, da mais leve à mais pesada: sem repetidas (numa
/// tela 1080p, "1440p" e "Nativa 60" dão a mesma imagem que "1080p 60") e com a taxa do
/// monitor quando ele passa de 60 Hz.
pub fn options(src_w: u32, src_h: u32, hz: u32) -> Vec<(Preset, EncodeParams)> {
    let mut v: Vec<(Preset, EncodeParams)> = Vec::new();
    for pr in Preset::ALL {
        let p = encode_params(pr, src_w, src_h, hz);
        if !v.iter().any(|(_, q)| same_output(q, &p)) {
            v.push((pr, p));
        }
    }
    v
}

fn same_output(a: &EncodeParams, b: &EncodeParams) -> bool {
    (a.width, a.height, a.fps) == (b.width, b.height, b.fps)
}

/// A opção da lista que dá a mesma imagem que `preset` (a preferência salva pode ter sido
/// escolhida em outro monitor).
pub fn normalize(preset: Preset, src_w: u32, src_h: u32, hz: u32) -> Preset {
    let p = encode_params(preset, src_w, src_h, hz);
    options(src_w, src_h, hz)
        .into_iter()
        .find(|(_, q)| same_output(q, &p))
        .map_or(preset, |(pr, _)| pr)
}

/// Nome para a pessoa: "Nativa (2560×1440) 144fps" quando a imagem sai do tamanho da fonte,
/// senão a faixa ("720p 30fps").
pub fn label(preset: Preset, p: &EncodeParams, src_w: u32, src_h: u32) -> String {
    let native = ((src_w & !1).max(64), (src_h & !1).max(64));
    if (p.width, p.height) == native {
        return format!("Nativa ({}×{}) {}fps", p.width, p.height, p.fps);
    }
    let name = match preset {
        Preset::P480p30 => "480p",
        Preset::P720p30 | Preset::P720p60 => "720p",
        Preset::P1080p30 | Preset::P1080p60 => "1080p",
        Preset::P1440p60 => "1440p",
        Preset::P2160p60 => "4K",
        // nativa reduzida pelo limite do encoder
        Preset::Native60 | Preset::NativeHigh => {
            return format!("{}×{} {}fps", p.width, p.height, p.fps);
        }
    };
    format!("{name} {}fps", p.fps)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sizes() {
        let p = encode_params(Preset::P1080p60, 2560, 1440, 60);
        assert_eq!((p.width, p.height), (1920, 1080));
        let p = encode_params(Preset::P1080p60, 1366, 768, 60);
        assert_eq!((p.width, p.height), (1366, 768));
        let p = encode_params(Preset::Native60, 1001, 601, 60);
        assert_eq!((p.width, p.height), (1000, 600));
    }

    #[test]
    fn presets_keep_pixel_budget_for_any_aspect() {
        // em pé: mesma área que 1080p/720p, orientação mantida
        let p = encode_params(Preset::P1080p60, 1440, 2560, 60);
        assert_eq!((p.width, p.height), (1080, 1920));
        let p = encode_params(Preset::P720p30, 1080, 1920, 60);
        assert_eq!((p.width, p.height), (720, 1280));
        // ultrawide 32:9 em "1080p": antes virava 1920×540 (metade da área)
        let p = encode_params(Preset::P1080p60, 5120, 1440, 60);
        let area = p.width as u64 * p.height as u64;
        assert!(area <= 1920 * 1080 && area > 1920 * 1080 * 98 / 100, "{p:?}");
        assert!((p.width as f64 / p.height as f64 - 32.0 / 9.0).abs() < 0.02, "{p:?}");
    }

    #[test]
    fn native_fits_h264_level_5_2() {
        // 4K60 cabe; ultrawide e 8K não cabem e são reduzidos sem distorcer
        for (sw, sh) in [(3840, 2160), (5120, 1440), (7680, 2160), (7680, 4320), (2160, 3840)] {
            let p = encode_params(Preset::Native60, sw, sh, 60);
            let (w, h) = (p.width as u64, p.height as u64);
            assert!(w <= MAX_SIDE && h <= MAX_SIDE, "{sw}x{sh} -> {w}x{h}");
            assert!(macroblocks(w, h) * 60 <= MAX_MBS_PER_SEC, "{sw}x{sh} -> {w}x{h}");
            let (src, out) = (sw as f64 / sh as f64, w as f64 / h as f64);
            assert!((src - out).abs() / src < 0.02, "{sw}x{sh} -> {w}x{h}");
        }
        let p = encode_params(Preset::Native60, 3840, 2160, 60);
        assert_eq!((p.width, p.height), (3840, 2160));
    }

    #[test]
    fn extreme_shapes_never_break_limits() {
        // estreitas/altas, transpostas, gigantes e degeneradas: o tamanho devolvido (já
        // alinhado e com o mínimo de 64) é que precisa caber no nível 5.2
        let cases = [
            (8, 5000),
            (5000, 8),
            (1, 1),
            (0, 0),
            (16, 65_535),
            (65_535, 16),
            (100_000, 100_000),
            (u32::MAX, u32::MAX),
            (u32::MAX, 1),
            (4097, 4097),
        ];
        for pr in Preset::ALL {
            for (sw, sh) in cases {
                let p = encode_params(pr, sw, sh, 240);
                let (w, h) = (p.width as u64, p.height as u64);
                let ctx = format!("{pr:?} {sw}x{sh} -> {w}x{h}");
                assert!(w >= 64 && h >= 64 && w % 2 == 0 && h % 2 == 0, "{ctx}");
                assert!(w <= MAX_SIDE && h <= MAX_SIDE, "{ctx}");
                assert!(macroblocks(w, h) <= MAX_MBS, "{ctx}");
                assert!(macroblocks(w, h) * p.fps as u64 <= MAX_MBS_PER_SEC, "{ctx}");
            }
        }
        // alta e fina: o lado longo cai para 4096, o curto fica no mínimo
        let p = encode_params(Preset::Native60, 8, 5000, 60);
        assert_eq!((p.width, p.height), (64, 4096));
        let p = encode_params(Preset::Native60, 5000, 8, 60);
        assert_eq!((p.width, p.height), (4096, 64));
    }
    #[test]
    fn viewers_for_upload() {
        // 88 Mbps (fibra medida aqui): 80% / (vídeo + 128 kbps de áudio)
        assert_eq!(viewers_for(88_000_000, 8_000_000), 8);
        assert_eq!(viewers_for(88_000_000, 4_000_000), 17);
        // upload fraco: nem uma pessoa em 1080p, uma em 720p
        assert_eq!(viewers_for(5_000_000, 8_000_000), 0);
        assert_eq!(viewers_for(6_000_000, 4_000_000), 1);
        assert_eq!(viewers_for(0, 4_000_000), 0);
    }

    #[test]
    fn bitrate() {
        assert_eq!(
            encode_params(Preset::Native60, 640, 480, 60).bitrate_bps,
            6_000_000
        );
        assert_eq!(
            encode_params(Preset::Native60, 7680, 4320, 60).bitrate_bps,
            25_000_000
        );
        assert_eq!(
            encode_params(Preset::P720p30, 1920, 1080, 60).bitrate_bps,
            4_000_000
        );
        // fonte menor que a faixa: bitrate cai junto com a imagem
        let p = encode_params(Preset::P1080p60, 960, 540, 60);
        assert_eq!((p.width, p.height, p.bitrate_bps), (960, 540, 2_000_000));
        // 144 Hz custa mais que 60 na mesma resolução, com teto
        let lo = encode_params(Preset::Native60, 2560, 1440, 144).bitrate_bps;
        let hi = encode_params(Preset::NativeHigh, 2560, 1440, 144).bitrate_bps;
        assert!(hi > lo && hi <= 40_000_000, "{lo} {hi}");
    }

    fn names(sw: u32, sh: u32, hz: u32) -> Vec<String> {
        options(sw, sh, hz)
            .iter()
            .map(|(pr, p)| label(*pr, p, sw, sh))
            .collect()
    }

    #[test]
    fn options_follow_the_monitor() {
        // 1080p a 60 Hz: sem 1440p/4K e sem "Nativa 60" repetindo o 1080p60
        assert_eq!(
            names(1920, 1080, 60),
            [
                "480p 30fps",
                "720p 30fps",
                "720p 60fps",
                "Nativa (1920×1080) 30fps",
                "Nativa (1920×1080) 60fps",
            ]
        );
        // 1440p a 144 Hz: ganha a faixa 1440p (nativa) e a taxa do monitor
        assert_eq!(
            names(2560, 1440, 144)[4..],
            [
                "1080p 60fps",
                "Nativa (2560×1440) 60fps",
                "Nativa (2560×1440) 144fps",
            ]
        );
        // 4K a 240 Hz: a taxa para no teto e a resolução cai para caber no nível 5.2
        let v = options(3840, 2160, 240);
        let (pr, p) = v.last().unwrap();
        assert_eq!((*pr, p.fps), (Preset::NativeHigh, MAX_FPS));
        assert!(p.width < 3840);
        // janela pequena: as faixas grandes viram a própria janela e não se repetem
        assert_eq!(
            names(800, 600, 60),
            ["480p 30fps", "Nativa (800×600) 30fps", "Nativa (800×600) 60fps"]
        );
    }

    #[test]
    fn saved_choice_maps_to_the_same_image_on_another_monitor() {
        // "Nativa 60" salva num monitor 1440p vira a opção "1080p 60" que aparece num 1080p
        assert_eq!(normalize(Preset::Native60, 1920, 1080, 60), Preset::P1080p60);
        // 144 Hz salva, monitor de 60 Hz: dá a mesma imagem que a nativa a 60
        assert_eq!(normalize(Preset::NativeHigh, 1920, 1080, 60), Preset::P1080p60);
        assert_eq!(normalize(Preset::P720p60, 1920, 1080, 60), Preset::P720p60);
    }
}
