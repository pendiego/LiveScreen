//! Ícone do app desenhado em código num espaço de 64×64 unidades: monitor com "play" branco
//! sobre quadrado arredondado azul. Sem dependências: o `build.rs` inclui este arquivo para
//! gerar o ícone do .exe, e a UI o usa no ícone da janela; os dois nunca divergem.

/// RGBA (não pré-multiplicado) `size`×`size`, com antialiasing por supersampling 4×4.
pub fn rgba(size: u32) -> Vec<u8> {
    const SS: u32 = 4;
    let n = size.max(1);
    let scale = 64.0 / n as f32;
    let mut out = vec![0u8; (n * n * 4) as usize];
    for y in 0..n {
        for x in 0..n {
            let (mut acc, mut hits) = ([0f32; 3], 0u32);
            for sy in 0..SS {
                for sx in 0..SS {
                    let fx = (x as f32 + (sx as f32 + 0.5) / SS as f32) * scale;
                    let fy = (y as f32 + (sy as f32 + 0.5) / SS as f32) * scale;
                    if let Some(c) = sample(fx, fy) {
                        for k in 0..3 {
                            acc[k] += c[k] as f32;
                        }
                        hits += 1;
                    }
                }
            }
            if hits > 0 {
                let i = ((y * n + x) * 4) as usize;
                for k in 0..3 {
                    out[i + k] = (acc[k] / hits as f32).round() as u8;
                }
                out[i + 3] = (hits * 255 / (SS * SS)) as u8;
            }
        }
    }
    out
}

/// Cor no ponto (x, y) do espaço 64×64; `None` fora do quadrado arredondado.
fn sample(x: f32, y: f32) -> Option<[u8; 3]> {
    // quadrado arredondado de raio 14
    let (cx, cy) = (x.clamp(14.0, 50.0), y.clamp(14.0, 50.0));
    if (x - cx).powi(2) + (y - cy).powi(2) > 14.0 * 14.0 {
        return None;
    }
    let inside = |x0: f32, y0: f32, x1: f32, y1: f32| x >= x0 && x < x1 && y >= y0 && y < y1;
    let frame = inside(14.0, 16.0, 50.0, 42.0) && !inside(18.0, 20.0, 46.0, 38.0);
    let foot = inside(29.0, 42.0, 35.0, 47.0) || inside(22.0, 47.0, 42.0, 50.0);
    // triângulo (27,22) (27,36) (39,29)
    let e = |ax: f32, ay: f32, bx: f32, by: f32| (bx - ax) * (y - ay) - (by - ay) * (x - ax);
    let (a, b, c) = (
        e(27.0, 22.0, 27.0, 36.0),
        e(27.0, 36.0, 39.0, 29.0),
        e(39.0, 29.0, 27.0, 22.0),
    );
    let play = (a >= 0.0 && b >= 0.0 && c >= 0.0) || (a <= 0.0 && b <= 0.0 && c <= 0.0);
    Some(if frame || foot || play {
        [255, 255, 255]
    } else {
        [0x00, 0x67, 0xC0]
    })
}
