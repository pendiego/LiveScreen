//! Painel de chat compartilhado entre host e espectador: lista rolável + campo de envio.

use egui::{Align, Layout, RichText};
use livescreen_core::chat::{ChatLog, ChatRole, MAX_CHAT_CHARS};

use super::theme::{self, Kind, P};

/// Desenha o chat ocupando exatamente `height` (lista rolável + campo de envio); devolve o
/// texto a enviar quando a pessoa confirma (botão ou Enter).
/// `me` identifica as linhas próprias, mostradas como "Você".
pub fn panel(
    ui: &mut egui::Ui,
    log: &ChatLog,
    me: (ChatRole, u32),
    draft: &mut String,
    height: f32,
) -> Option<String> {
    const INPUT_H: f32 = 40.0;
    const GAP: f32 = 12.0;
    let list_h = (height - INPUT_H - GAP).max(120.0);
    egui::ScrollArea::vertical()
        .max_height(list_h)
        .min_scrolled_height(list_h)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            // sem isso a linha do nome herda a altura mínima de botão (36 px) e a lista rola à toa
            ui.spacing_mut().interact_size.y = 20.0;
            let mut empty = true;
            for l in log.lines() {
                empty = false;
                let host = l.role == ChatRole::Host;
                let who = if (l.role, l.sender) == me { "Você".to_string() } else { l.label() };
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(who)
                            .size(13.0)
                            .family(egui::FontFamily::Name("medium".into()))
                            .color(if host { P.accent } else { P.text_muted }),
                    );
                    if host {
                        theme::badge(ui, "anfitrião", P.accent);
                    }
                    if l.friend {
                        theme::badge(ui, "amigo", P.success);
                    }
                });
                ui.add_space(3.0);
                ui.add(egui::Label::new(RichText::new(&l.text).size(15.0).color(P.text)).wrap());
                ui.add_space(14.0);
            }
            if empty {
                ui.label(RichText::new("Nenhuma mensagem ainda").size(14.0).color(P.text_muted));
            }
        });
    ui.add_space(GAP);
    let mut out = None;
    ui.horizontal(|ui| {
        // campo e botão com a mesma altura (interact_size manda no TextEdit e no botão)
        ui.spacing_mut().interact_size.y = INPUT_H;
        ui.spacing_mut().item_spacing.x = 8.0;
        let btn_w = 96.0;
        let resp = ui.add(
            egui::TextEdit::singleline(draft)
                .hint_text("Escreva uma mensagem")
                .char_limit(MAX_CHAT_CHARS)
                .font(egui::TextStyle::Body)
                .margin(egui::Margin::symmetric(12, 10))
                .align(egui::Align2::LEFT_CENTER)
                .desired_width((ui.available_width() - btn_w - ui.spacing().item_spacing.x).max(60.0)),
        );
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let click = ui
            .with_layout(Layout::right_to_left(Align::Center), |ui| {
                theme::button(ui, Kind::Primary, "Enviar").clicked()
            })
            .inner;
        if enter || click {
            let t = draft.trim().to_string();
            if !t.is_empty() {
                out = Some(t);
            }
            draft.clear();
            resp.request_focus();
        }
    });
    out
}
