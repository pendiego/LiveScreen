//! Avatares da sala: dados portáveis (quadro 64×64) com desenho procedural fixo de 7 animais.
//! Pela rede só trafega o id (u8: 1..=COUNT, 0 = automático).

pub const COUNT: u8 = 7;

pub const BACK: u32 = 0x1C2027;
pub const DARK: u32 = 0x0B0D10;
pub const LIGHT: u32 = 0xF3F5F7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Accent,
    Dark,
    Light,
    Back,
}

impl Ink {
    pub fn code(&self) -> &'static str {
        match self {
            Ink::Accent => "a",
            Ink::Dark => "d",
            Ink::Light => "l",
            Ink::Back => "b",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle { ink: Ink, cx: f32, cy: f32, r: f32 },
    Ellipse { ink: Ink, cx: f32, cy: f32, rx: f32, ry: f32 },
    Rect { ink: Ink, x: f32, y: f32, w: f32, h: f32, rx: f32 },
    Poly { ink: Ink, pts: &'static [f32] },
    Line { ink: Ink, w: f32, x1: f32, y1: f32, x2: f32, y2: f32 },
    Pline { ink: Ink, w: f32, pts: &'static [f32] },
    Ring { ink: Ink, w: f32, cx: f32, cy: f32, rx: f32, ry: f32 },
}

pub struct Avatar {
    pub id: u8,
    pub name: &'static str,
    pub accent: u32,
    pub shapes: &'static [Shape],
}

static GATO_SHAPES: &[Shape] = &[
    Shape::Poly { ink: Ink::Accent, pts: &[16.0, 28.0, 18.0, 10.0, 30.0, 20.0] },
    Shape::Poly { ink: Ink::Accent, pts: &[48.0, 28.0, 46.0, 10.0, 34.0, 20.0] },
    Shape::Circle { ink: Ink::Accent, cx: 32.0, cy: 36.0, r: 18.0 },
    Shape::Circle { ink: Ink::Dark, cx: 25.0, cy: 35.0, r: 2.6 },
    Shape::Circle { ink: Ink::Dark, cx: 39.0, cy: 35.0, r: 2.6 },
    Shape::Poly { ink: Ink::Dark, pts: &[29.5, 41.0, 34.5, 41.0, 32.0, 44.0] },
];

static RAPOSA_SHAPES: &[Shape] = &[
    Shape::Poly { ink: Ink::Accent, pts: &[12.0, 24.0, 14.0, 8.0, 27.0, 19.0] },
    Shape::Poly { ink: Ink::Accent, pts: &[52.0, 24.0, 50.0, 8.0, 37.0, 19.0] },
    Shape::Poly { ink: Ink::Accent, pts: &[10.0, 22.0, 32.0, 52.0, 54.0, 22.0, 32.0, 16.0] },
    // focinho em V: dois triângulos, pois o egui só preenche polígonos convexos
    Shape::Poly { ink: Ink::Light, pts: &[18.0, 30.0, 32.0, 52.0, 32.0, 38.0] },
    Shape::Poly { ink: Ink::Light, pts: &[46.0, 30.0, 32.0, 52.0, 32.0, 38.0] },
    Shape::Circle { ink: Ink::Dark, cx: 24.0, cy: 28.0, r: 2.6 },
    Shape::Circle { ink: Ink::Dark, cx: 40.0, cy: 28.0, r: 2.6 },
    Shape::Circle { ink: Ink::Dark, cx: 32.0, cy: 49.0, r: 2.6 },
];

static URSO_SHAPES: &[Shape] = &[
    Shape::Circle { ink: Ink::Accent, cx: 18.0, cy: 21.0, r: 7.0 },
    Shape::Circle { ink: Ink::Accent, cx: 46.0, cy: 21.0, r: 7.0 },
    Shape::Circle { ink: Ink::Accent, cx: 32.0, cy: 35.0, r: 18.0 },
    Shape::Ellipse { ink: Ink::Light, cx: 32.0, cy: 42.0, rx: 8.0, ry: 6.0 },
    Shape::Circle { ink: Ink::Dark, cx: 25.0, cy: 32.0, r: 2.6 },
    Shape::Circle { ink: Ink::Dark, cx: 39.0, cy: 32.0, r: 2.6 },
    Shape::Ellipse { ink: Ink::Dark, cx: 32.0, cy: 40.0, rx: 3.0, ry: 2.2 },
];

static COELHO_SHAPES: &[Shape] = &[
    Shape::Ellipse { ink: Ink::Accent, cx: 25.0, cy: 17.0, rx: 4.5, ry: 12.0 },
    Shape::Ellipse { ink: Ink::Accent, cx: 39.0, cy: 17.0, rx: 4.5, ry: 12.0 },
    Shape::Circle { ink: Ink::Accent, cx: 32.0, cy: 39.0, r: 15.0 },
    Shape::Circle { ink: Ink::Dark, cx: 26.5, cy: 37.0, r: 2.4 },
    Shape::Circle { ink: Ink::Dark, cx: 37.5, cy: 37.0, r: 2.4 },
    Shape::Poly { ink: Ink::Dark, pts: &[30.0, 42.0, 34.0, 42.0, 32.0, 44.5] },
];

static SAPO_SHAPES: &[Shape] = &[
    Shape::Circle { ink: Ink::Accent, cx: 21.0, cy: 27.0, r: 7.5 },
    Shape::Circle { ink: Ink::Accent, cx: 43.0, cy: 27.0, r: 7.5 },
    Shape::Ellipse { ink: Ink::Accent, cx: 32.0, cy: 39.0, rx: 21.0, ry: 13.0 },
    Shape::Circle { ink: Ink::Dark, cx: 21.0, cy: 27.0, r: 3.6 },
    Shape::Circle { ink: Ink::Dark, cx: 43.0, cy: 27.0, r: 3.6 },
    Shape::Pline {
        ink: Ink::Dark,
        w: 2.6,
        pts: &[
            40.7, 39.0, 39.1, 41.1, 37.0, 42.7, 34.6, 43.7, 32.0, 44.0, 29.4, 43.7, 27.0, 42.7,
            24.9, 41.1, 23.3, 39.0,
        ],
    },
];

static POLVO_SHAPES: &[Shape] = &[
    Shape::Line { ink: Ink::Accent, w: 5.0, x1: 19.0, y1: 34.0, x2: 15.0, y2: 52.0 },
    Shape::Line { ink: Ink::Accent, w: 5.0, x1: 26.0, y1: 36.0, x2: 24.0, y2: 54.0 },
    Shape::Line { ink: Ink::Accent, w: 5.0, x1: 32.0, y1: 36.0, x2: 32.0, y2: 55.0 },
    Shape::Line { ink: Ink::Accent, w: 5.0, x1: 38.0, y1: 36.0, x2: 40.0, y2: 54.0 },
    Shape::Line { ink: Ink::Accent, w: 5.0, x1: 45.0, y1: 34.0, x2: 49.0, y2: 52.0 },
    Shape::Ellipse { ink: Ink::Accent, cx: 32.0, cy: 26.0, rx: 16.0, ry: 15.0 },
    Shape::Circle { ink: Ink::Dark, cx: 26.0, cy: 26.0, r: 2.6 },
    Shape::Circle { ink: Ink::Dark, cx: 38.0, cy: 26.0, r: 2.6 },
];

static PEIXE_SHAPES: &[Shape] = &[
    Shape::Poly { ink: Ink::Accent, pts: &[42.0, 32.0, 56.0, 21.0, 56.0, 43.0] },
    Shape::Ellipse { ink: Ink::Accent, cx: 29.0, cy: 32.0, rx: 18.0, ry: 12.0 },
    Shape::Circle { ink: Ink::Dark, cx: 20.0, cy: 29.0, r: 2.6 },
    Shape::Pline {
        ink: Ink::Dark,
        w: 2.4,
        pts: &[
            30.4, 24.9, 31.8, 27.1, 32.7, 29.5, 33.0, 32.0, 32.7, 34.5, 31.8, 36.9, 30.4, 39.1,
        ],
    },
];

static ALL: [Avatar; 7] = [
    Avatar { id: 1, name: "Gato", accent: 0xC8F135, shapes: GATO_SHAPES },
    Avatar { id: 2, name: "Raposa", accent: 0xFFB547, shapes: RAPOSA_SHAPES },
    Avatar { id: 3, name: "Urso", accent: 0xFFB547, shapes: URSO_SHAPES },
    Avatar { id: 4, name: "Coelho", accent: 0xF3F5F7, shapes: COELHO_SHAPES },
    Avatar { id: 5, name: "Sapo", accent: 0x3DDC97, shapes: SAPO_SHAPES },
    Avatar { id: 6, name: "Polvo", accent: 0xFF5A4E, shapes: POLVO_SHAPES },
    Avatar { id: 7, name: "Peixe", accent: 0x5AB8FF, shapes: PEIXE_SHAPES },
];

pub fn all() -> &'static [Avatar] {
    &ALL
}

pub fn get(id: u8) -> Option<&'static Avatar> {
    if (1..=COUNT).contains(&id) {
        Some(&ALL[(id - 1) as usize])
    } else {
        None
    }
}

/// FNV-1a 32-bit (offset 0x811C9DC5, primo 0x01000193), resultado em 1..=COUNT.
pub fn auto(key: &str) -> u8 {
    let mut hash: u32 = 0x811C9DC5;
    for b in key.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    (hash % (COUNT as u32) + 1) as u8
}

pub fn resolve(chosen: u8, key: &str) -> u8 {
    if (1..=COUNT).contains(&chosen) {
        chosen
    } else {
        auto(key)
    }
}

/// Número curto e exato no JSON: 2.6f32 vira 2.6, não 2.5999999046325684.
fn num(v: f32) -> serde_json::Value {
    serde_json::from_str(&v.to_string()).expect("f32 finito")
}

/// Os desenhos em JSON para a página do navegador (`web/public/v/avatars.json`) e o Android.
pub fn json() -> String {
    use serde_json::{Value, json};
    let row = |kind: &str, ink: Ink, head: &[f32], pts: &[f32]| {
        let mut v = vec![json!(kind), json!(ink.code())];
        v.extend(head.iter().chain(pts).map(|&x| num(x)));
        Value::Array(v)
    };
    let avatars: Vec<Value> = ALL
        .iter()
        .map(|av| {
            let shapes: Vec<Value> = av
                .shapes
                .iter()
                .map(|s| match *s {
                    Shape::Circle { ink, cx, cy, r } => row("circle", ink, &[cx, cy, r], &[]),
                    Shape::Ellipse { ink, cx, cy, rx, ry } => row("ellipse", ink, &[cx, cy, rx, ry], &[]),
                    Shape::Rect { ink, x, y, w, h, rx } => row("rect", ink, &[x, y, w, h, rx], &[]),
                    Shape::Poly { ink, pts } => row("poly", ink, &[], pts),
                    Shape::Line { ink, w, x1, y1, x2, y2 } => row("line", ink, &[w, x1, y1, x2, y2], &[]),
                    Shape::Pline { ink, w, pts } => row("pline", ink, &[w], pts),
                    Shape::Ring { ink, w, cx, cy, rx, ry } => row("ring", ink, &[w, cx, cy, rx, ry], &[]),
                })
                .collect();
            json!({"id": av.id, "name": av.name, "accent": format!("#{:06X}", av.accent), "shapes": shapes})
        })
        .collect();
    json!({
        "back": format!("#{BACK:06X}"),
        "dark": format!("#{DARK:06X}"),
        "light": format!("#{LIGHT:06X}"),
        "avatars": avatars,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vetores de teste de auto(key) para conferência entre plataformas (Desktop, Android, Web):
    /// 1. "Diego" -> 3 (Urso)
    /// 2. "Alice" -> 6 (Polvo)
    /// 3. "#0"    -> 7 (Peixe)
    #[test]
    fn test_auto_vectors() {
        assert_eq!(auto("Diego"), 3);
        assert_eq!(auto("Alice"), 6);
        assert_eq!(auto("#0"), 7);
    }

    #[test]
    fn test_resolve() {
        // 0 deve cair no automático
        assert_eq!(resolve(0, "Diego"), 3);
        // Válido (1..=COUNT) deve ser preservado
        assert_eq!(resolve(1, "Diego"), 1);
        assert_eq!(resolve(7, "Diego"), 7);
        // Fora da faixa deve cair no automático
        assert_eq!(resolve(8, "Diego"), 3);
        assert_eq!(resolve(255, "Diego"), 3);
    }

    #[test]
    fn test_get_and_all() {
        assert_eq!(all().len(), COUNT as usize);
        assert!(get(0).is_none());
        assert_eq!(get(1).unwrap().name, "Gato");
        assert_eq!(get(7).unwrap().name, "Peixe");
        assert!(get(8).is_none());
    }

    #[test]
    fn test_json_matches_web_public_file() {
        let expected = include_str!("../../../web/public/v/avatars.json");
        let generated = json();
        assert_eq!(
            generated, expected,
            "Regere web/public/v/avatars.json com avatar::json()"
        );
    }
}
