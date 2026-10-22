//! Embute o ícone do app no .exe. O `.res` é montado aqui a partir de `src/icon.rs` (mesmo
//! desenho da janela) e entregue ao linker do MSVC, que aceita `.res` direto: sem rc.exe.
use std::path::PathBuf;

#[path = "src/icon.rs"]
mod icon;

/// Tamanhos do Explorer/barra de tarefas em 100–200% de escala; 256 para "ícones extra grandes".
const SIZES: [u32; 7] = [16, 20, 24, 32, 48, 64, 256];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/icon.rs");
    let target_ok = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if !target_ok {
        return;
    }
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("icon.res");
    std::fs::write(&out, res_file()).expect("gravar icon.res");
    println!("cargo:rustc-link-arg-bins={}", out.display());
}

const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;

/// Arquivo .res de 32 bits: entrada vazia obrigatória, uma RT_ICON por tamanho e o grupo 1
/// (o primeiro grupo é o ícone que o Explorer mostra para o .exe).
fn res_file() -> Vec<u8> {
    let mut res = Vec::new();
    push_entry(&mut res, 0, 0, 0, &[]);
    let mut group = Vec::new();
    for v in [0u16, 1, SIZES.len() as u16] {
        group.extend(v.to_le_bytes());
    }
    for (i, &size) in SIZES.iter().enumerate() {
        let id = i as u16 + 1;
        let img = dib(size);
        push_entry(&mut res, RT_ICON, id, 0x1010, &img);
        // largura/altura 0 significa 256
        group.extend([size as u8, size as u8, 0, 0]);
        group.extend(1u16.to_le_bytes());
        group.extend(32u16.to_le_bytes());
        group.extend((img.len() as u32).to_le_bytes());
        group.extend(id.to_le_bytes());
    }
    push_entry(&mut res, RT_GROUP_ICON, 1, 0x1030, &group);
    res
}

/// Cabeçalho de 32 bytes com tipo e nome por ordinal, idioma neutro, dados alinhados a 4.
fn push_entry(res: &mut Vec<u8>, ty: u16, id: u16, flags: u16, data: &[u8]) {
    res.extend((data.len() as u32).to_le_bytes());
    res.extend(32u32.to_le_bytes());
    for v in [0xFFFFu16, ty, 0xFFFF, id] {
        res.extend(v.to_le_bytes());
    }
    res.extend(0u32.to_le_bytes()); // DataVersion
    res.extend(flags.to_le_bytes());
    res.extend(0u16.to_le_bytes()); // LanguageId neutro
    res.extend(0u32.to_le_bytes()); // Version
    res.extend(0u32.to_le_bytes()); // Characteristics
    res.extend(data);
    while res.len() % 4 != 0 {
        res.push(0);
    }
}

/// Imagem de ícone em DIB 32 bpp: BITMAPINFOHEADER (altura dobrada), pixels BGRA de baixo
/// para cima e máscara AND de 1 bpp (1 = transparente), cada linha alinhada a 4 bytes.
fn dib(size: u32) -> Vec<u8> {
    let px = icon::rgba(size);
    let n = size as usize;
    let mask_row = n.div_ceil(32) * 4;
    let mut d = Vec::with_capacity(40 + n * n * 4 + mask_row * n);
    d.extend(40u32.to_le_bytes());
    d.extend((size as i32).to_le_bytes());
    d.extend((2 * size as i32).to_le_bytes());
    d.extend(1u16.to_le_bytes());
    d.extend(32u16.to_le_bytes());
    d.extend(0u32.to_le_bytes()); // BI_RGB
    d.extend(((n * n * 4 + mask_row * n) as u32).to_le_bytes());
    d.extend([0u8; 16]);
    for y in (0..n).rev() {
        for x in 0..n {
            let i = (y * n + x) * 4;
            d.extend([px[i + 2], px[i + 1], px[i], px[i + 3]]);
        }
    }
    for y in (0..n).rev() {
        let mut row = vec![0u8; mask_row];
        for x in 0..n {
            if px[(y * n + x) * 4 + 3] == 0 {
                row[x / 8] |= 0x80 >> (x % 8);
            }
        }
        d.extend(row);
    }
    d
}
