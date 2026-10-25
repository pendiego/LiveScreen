use egui::{Align2, CursorIcon, FontId, RichText, Sense, Stroke, vec2};

use super::theme::{self, I_CHECK, I_ERROR, I_INFO, I_SHARE, I_UPDATE, I_WARN, I_WATCH};
use super::{App, EndKind, Screen};
use crate::video::encoder::EncoderKind;

/// Peça clicável da home. Devolve true no clique.
fn tile(ui: &mut egui::Ui, icon: &str, title: &str, desc: &str) -> bool {
    let p = theme::palette(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(280.0, 150.0), Sense::click());
    let hovered = resp.hovered();
    let focused = resp.has_focus();
    let stroke = if focused {
        Stroke::new(2.0, p.accent)
    } else if hovered {
        Stroke::new(1.0, p.border_strong)
    } else {
        Stroke::new(1.0, p.border)
    };
    theme::paint_surface(ui, rect, stroke);
    let painter = ui.painter();
    painter.text(
        rect.center_top() + vec2(0.0, 24.0),
        Align2::CENTER_TOP,
        icon,
        FontId::proportional(32.0),
        p.accent,
    );
    painter.text(
        rect.center_top() + vec2(0.0, 76.0),
        Align2::CENTER_TOP,
        title,
        FontId::new(22.0, egui::FontFamily::Name("semibold".into())),
        ui.visuals().text_color(),
    );
    painter.text(
        rect.center_top() + vec2(0.0, 112.0),
        Align2::CENTER_TOP,
        desc,
        FontId::proportional(13.0),
        ui.visuals().weak_text_color(),
    );
    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn home(app: &mut App, ui: &mut egui::Ui) {
    let enc = app.encoder.lock().unwrap().clone();
    let update = app.update.lock().unwrap().clone();
    egui::Panel::bottom("home_footer").show(ui, |ui| {
        ui.horizontal(|ui| {
            let weak = ui.visuals().weak_text_color();
            match &enc {
                None => {
                    ui.add(egui::Spinner::new().size(12.0));
                    ui.label(
                        RichText::new("Detectando encoder de vídeo…")
                            .small()
                            .color(weak),
                    );
                }
                Some(EncoderKind::Hardware { name }) => {
                    ui.label(
                        RichText::new(format!(
                            "{I_CHECK}  Encoder: {name} (aceleração por hardware)"
                        ))
                        .small()
                        .color(weak),
                    );
                }
                Some(EncoderKind::Software) => {
                    ui.label(
                        RichText::new(format!(
                            "{I_INFO}  Encoder: software (Microsoft H.264) — pode usar mais CPU"
                        ))
                        .small()
                        .color(weak),
                    );
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                super::settings::button(app, ui);
                if app.updating.is_some() {
                    ui.label(RichText::new("Baixando atualização…").small().color(weak));
                    ui.add(egui::Spinner::new().size(12.0));
                } else if let Some(rel) = update
                    && theme::small_button(ui, format!(
                            "{I_UPDATE}  Atualizar para a versão {}",
                            rel.version
                        ))
                        .on_hover_text("Baixa a versão nova e reinicia o LiveScreen")
                        .clicked()
                {
                    app.start_update(rel, ui.ctx());
                }
            });
        });
    });
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() / 2.0 - 150.0).max(20.0));
            ui.label(
                RichText::new("LiveScreen")
                    .text_style(egui::TextStyle::Name("title".into()))
                    .extra_letter_spacing(-1.1),
            );
            ui.label(
                RichText::new("Compartilhe sua tela com seus amigos — direto, sem cadastro.")
                    .weak(),
            );
            ui.add_space(24.0);
            let (mut share, mut watch) = (false, false);
            let total = 280.0 * 2.0 + ui.spacing().item_spacing.x;
            let busy = app.updating.is_some();
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
                    share = tile(
                        ui,
                        I_SHARE,
                        "Compartilhar tela",
                        "Transmita uma tela ou janela",
                    );
                    watch = tile(ui, I_WATCH, "Assistir", "Entre com um código de convite");
                });
            });
            if share {
                app.error = None;
                app.picker = Some(super::share_view::Picker::new(
                    ui.ctx().clone(),
                    app.prefs.preset,
                    app.prefs.share_audio,
                    app.prefs.share_mic,
                    app.prefs.mic_device.clone(),
                ));
                app.screen = Screen::SharePicker;
                // mede antes de transmitir, quando o upload ainda está livre
                if *app.upload.lock().unwrap() == crate::speedtest::Upload::Unknown {
                    app.measure_upload(ui.ctx());
                }
            }
            if watch {
                app.error = None;
                // convite já copiado? entra preenchido: falta só "Conectar"
                if let Ok(t) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                    let t: String = t.chars().filter(|c| !c.is_whitespace()).collect();
                    if crate::watch::is_valid_ticket(&t) {
                        app.ticket_input = t;
                        app.toasts.push(
                            "Código de convite colado da área de transferência",
                            super::toast::ToastKind::Info,
                        );
                    }
                }
                app.screen = Screen::WatchInput;
            }
        });
    });
}

pub fn ended(app: &mut App, ui: &mut egui::Ui) {
    let Screen::Ended {
        message,
        kind,
        retry,
    } = &app.screen
    else {
        return;
    };
    let (message, kind, retry) = (message.clone(), *kind, *retry);
    let mut go_home = false;
    let mut again = false;
    egui::Panel::bottom("ended_footer").show(ui, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            super::settings::button(app, ui);
        });
    });
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() / 2.0 - 130.0).max(20.0));
            let p = theme::palette(ui);
            let (icon, color) = match kind {
                EndKind::Info => (I_CHECK, p.accent),
                EndKind::Warn => (I_WARN, p.warn),
                EndKind::Error => (I_ERROR, p.danger),
            };
            ui.scope(|ui| {
                ui.set_max_width(440.0);
                theme::card(ui, |ui| {
                    ui.set_width(408.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(icon).size(32.0).color(color));
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&message)
                                .size(18.0)
                                .family(egui::FontFamily::Name("semibold".into())),
                        );
                        ui.add_space(12.0);
                        if theme::primary_button(ui, "Voltar ao início").clicked() {
                            go_home = true;
                        }
                        if retry && ui.button("Tentar de novo").clicked() {
                            again = true;
                        }
                    });
                });
            });
        });
    });
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
        if retry {
            again = true;
        } else {
            go_home = true;
        }
    }
    if again {
        super::watch_view::connect(app, &ui.ctx().clone());
    } else if go_home {
        app.screen = Screen::Home;
    }
}
