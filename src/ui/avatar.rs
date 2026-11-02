//! Desenho e seleção de avatares portáveis (64×64) em egui.
//!
//! Os desenhos vêm do núcleo (`livescreen_core::avatar`) como primitivas vetoriais:
//! círculo, elipse, retângulo arredondado, polígono convexo, linhas abertas e anéis.
//! Todos os polígonos dos 7 animais originais são convexos (triângulos e quadriláteros convexos/losangos),
//! o que permite o preenchimento exato via `epaint::Shape::convex_polygon`.

use egui::{epaint, vec2, Color32, FontFamily, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, WidgetInfo};
use livescreen_core::avatar::{self, Ink, Shape, BACK, COUNT, DARK, LIGHT};

use super::theme::P;

/// Converte cor 0xRRGGBB em Color32 do egui.
fn color_from_u32(c: u32) -> Color32 {
    let r = ((c >> 16) & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = (c & 0xFF) as u8;
    Color32::from_rgb(r, g, b)
}

/// Mapeia a tinta (Ink) para a cor final no egui.
fn ink_color(ink: Ink, accent: u32) -> Color32 {
    match ink {
        Ink::Accent => color_from_u32(accent),
        Ink::Dark => color_from_u32(DARK),
        Ink::Light => color_from_u32(LIGHT),
        Ink::Back => color_from_u32(BACK),
    }
}

/// Desenha o avatar `id` dentro do `rect` informado, escalando o viewBox 64×64.
///
/// Fundo sempre circular em `BACK` (raio 32 no quadro 64×64). Se `id` for inválido ou 0
/// sem desenho cadastrado, desenha apenas o círculo de fundo.
pub fn paint_avatar(painter: &egui::Painter, rect: Rect, id: u8) {
    let s = rect.width().min(rect.height()) / 64.0;
    if s <= 0.0 {
        return;
    }
    let ox = rect.center().x - 32.0 * s;
    let oy = rect.center().y - 32.0 * s;
    let pt = |x: f32, y: f32| egui::pos2(ox + x * s, oy + y * s);

    // Círculo de fundo obrigatório para todo avatar
    let bg_color = color_from_u32(BACK);
    painter.circle_filled(pt(32.0, 32.0), 32.0 * s, bg_color);

    let Some(av) = avatar::get(id) else {
        return;
    };

    for shape in av.shapes {
        match shape {
            Shape::Circle { ink, cx, cy, r } => {
                let color = ink_color(*ink, av.accent);
                painter.circle_filled(pt(*cx, *cy), *r * s, color);
            }
            Shape::Ellipse { ink, cx, cy, rx, ry } => {
                let color = ink_color(*ink, av.accent);
                const N: usize = 24;
                let points: Vec<Pos2> = (0..N)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / N as f32;
                        pt(cx + rx * a.cos(), cy + ry * a.sin())
                    })
                    .collect();
                painter.add(epaint::Shape::convex_polygon(points, color, Stroke::NONE));
            }
            Shape::Rect { ink, x, y, w, h, rx } => {
                let color = ink_color(*ink, av.accent);
                let r = Rect::from_min_size(pt(*x, *y), vec2(*w * s, *h * s));
                painter.rect_filled(r, *rx * s, color);
            }
            Shape::Poly { ink, pts } => {
                let color = ink_color(*ink, av.accent);
                let points: Vec<Pos2> = pts.chunks_exact(2).map(|c| pt(c[0], c[1])).collect();
                let stroke = Stroke::new((1.0 * s).max(0.5), color);
                painter.add(epaint::Shape::convex_polygon(points, color, stroke));
            }
            Shape::Line { ink, w, x1, y1, x2, y2 } => {
                let color = ink_color(*ink, av.accent);
                let p1 = pt(*x1, *y1);
                let p2 = pt(*x2, *y2);
                let stroke_w = *w * s;
                painter.line_segment([p1, p2], Stroke::new(stroke_w, color));
                // Pontas redondas simuladas com círculos nas extremidades
                let r_cap = stroke_w * 0.5;
                painter.circle_filled(p1, r_cap, color);
                painter.circle_filled(p2, r_cap, color);
            }
            Shape::Pline { ink, w, pts } => {
                let color = ink_color(*ink, av.accent);
                let points: Vec<Pos2> = pts.chunks_exact(2).map(|c| pt(c[0], c[1])).collect();
                let stroke_w = *w * s;
                if points.len() >= 2 {
                    let path = epaint::PathShape::line(points.clone(), Stroke::new(stroke_w, color));
                    painter.add(path);
                    // Juntas e pontas arredondadas
                    let r_cap = stroke_w * 0.5;
                    for p in &points {
                        painter.circle_filled(*p, r_cap, color);
                    }
                }
            }
            Shape::Ring { ink, w, cx, cy, rx, ry } => {
                let color = ink_color(*ink, av.accent);
                const N: usize = 24;
                let points: Vec<Pos2> = (0..N)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / N as f32;
                        pt(cx + rx * a.cos(), cy + ry * a.sin())
                    })
                    .collect();
                let stroke = Stroke::new(*w * s, color);
                let path = epaint::PathShape::closed_line(points, stroke);
                painter.add(path);
            }
        }
    }
}

/// Widget do egui que aloca espaço quadrado `size` e desenha o avatar correspondente.
pub fn avatar(ui: &mut Ui, id: u8, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    paint_avatar(ui.painter(), rect, id);
    response
}

/// Grade de seleção com a opção "Automático" + os 7 avatares fixos.
///
/// `chosen`: 0 = automático, 1..=COUNT = animal escolhido.
/// `current_name`: nome atual da pessoa, usado para gerar a prévia do "Automático".
pub fn picker(ui: &mut Ui, chosen: &mut u8, current_name: &str) {
    let auto_id = avatar::auto(current_name);
    let auto_name = avatar::get(auto_id).map(|a| a.name).unwrap_or("Animal");

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(10.0, 10.0);

        // Opção 0: Automático
        let auto_selected = *chosen == 0;
        let auto_label = format!("Avatar automático ({auto_name})");
        if draw_avatar_item(ui, auto_id, "Automático", auto_selected, &auto_label).clicked() {
            *chosen = 0;
        }

        // Opções 1..=COUNT
        for id in 1..=COUNT {
            let av = avatar::get(id);
            let name = av.map(|a| a.name).unwrap_or("");
            let selected = *chosen == id;
            let accessible = format!("Avatar {name}");
            if draw_avatar_item(ui, id, name, selected, &accessible).clicked() {
                *chosen = id;
            }
        }
    });
}

/// Item clicável da grade de escolha de avatares com prévia, nome e anel de seleção.
fn draw_avatar_item(
    ui: &mut Ui,
    display_id: u8,
    label: &str,
    selected: bool,
    accessible_name: &str,
) -> Response {
    const CARD_W: f32 = 56.0;
    const CARD_H: f32 = 64.0;
    const AVATAR_SIZE: f32 = 36.0;

    let (rect, response) = ui.allocate_exact_size(vec2(CARD_W, CARD_H), Sense::click());
    let painter = ui.painter();

    let avatar_center = egui::pos2(rect.center().x, rect.top() + (AVATAR_SIZE / 2.0) + 4.0);
    let avatar_rect = Rect::from_center_size(avatar_center, vec2(AVATAR_SIZE, AVATAR_SIZE));

    // Desenha o avatar em si
    paint_avatar(painter, avatar_rect, display_id);

    // Anel de foco / seleção conforme o tema
    let r_ring = (AVATAR_SIZE * 0.5) + 3.0;
    if selected {
        painter.circle_stroke(avatar_center, r_ring, Stroke::new(2.5, P.accent));
    } else if response.hovered() {
        painter.circle_stroke(avatar_center, r_ring, Stroke::new(1.5, P.border_strong));
    }

    // Rótulo textual abaixo do avatar
    let font_id = FontId::new(11.0, FontFamily::Name("medium".into()));
    let text_color = if selected {
        P.accent
    } else if response.hovered() {
        P.text
    } else {
        P.text_muted
    };
    let galley = painter.layout_no_wrap(label.to_string(), font_id, text_color);
    let text_pos = egui::pos2(rect.center().x - galley.size().x / 2.0, rect.bottom() - galley.size().y - 2.0);
    painter.galley(text_pos, galley, text_color);

    // Acessibilidade e dica de foco
    let resp = response.on_hover_text(accessible_name);
    resp.widget_info(|| WidgetInfo::selected(egui::Role::RadioButton, true, selected, accessible_name));
    resp
}
