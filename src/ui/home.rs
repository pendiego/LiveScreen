use egui::{Align, Align2, CursorIcon, FontFamily, FontId, Layout, RichText, Sense, Stroke, StrokeKind, vec2};
use livescreen_core::protocol::parse_invite;

use super::theme::{self, I_CHECK, I_CHEVRON, I_ERROR, I_INFO, I_SHARE, I_UPDATE, I_WARN, I_WATCH};
use super::toast::ToastKind;
use super::{App, EndKind, Screen};
use crate::video::encoder::EncoderKind;

/// Ação primária da home: tile cheio na cor `primary`. Devolve true no clique.
fn share_tile(ui: &mut egui::Ui, width: f32) -> bool {
    let p = theme::palette(ui);
    let enabled = ui.is_enabled();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 88.0), Sense::click());
    let fill = if !enabled {
        p.primary.gamma_multiply(0.4)
    } else if resp.is_pointer_button_down_on() {
        p.primary.lerp_to_gamma(p.on_primary, 0.25)
    } else if resp.hovered() {
        p.primary.lerp_to_gamma(p.on_primary, 0.15)
    } else {
        p.primary
    };
    let ring = if resp.has_focus() { Stroke::new(2.0, p.accent) } else { Stroke::NONE };
    let painter = ui.painter();
    painter.add(p.shadow.as_shape(rect, 12));
    painter.rect(rect, 12, fill, ring, StrokeKind::Outside);
    let fg = p.on_primary;
    let c = rect.left_center();
    painter.text(c + vec2(24.0, 0.0), Align2::LEFT_CENTER, I_SHARE, FontId::proportional(28.0), fg);
    painter.text(
        c + vec2(72.0, 1.0),
        Align2::LEFT_BOTTOM,
        "Compartilhar tela",
        FontId::new(18.0, FontFamily::Name("semibold".into())),
        fg,
    );
    painter.text(
        c + vec2(72.0, 4.0),
        Align2::LEFT_TOP,
        "Transmita uma tela ou janela",
        FontId::proportional(13.0),
        fg.gamma_multiply(0.7),
    );
    painter.text(
        rect.right_center() - vec2(20.0, 0.0),
        Align2::RIGHT_CENTER,
        I_CHEVRON,
        FontId::proportional(16.0),
        fg,
    );
    if enabled && resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, "Compartilhar tela"));
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
                Some(kinds) => {
                    let p = theme::palette(ui);
                    let hw_labels: Vec<_> = kinds
                        .iter()
                        .filter_map(|k| match k {
                            EncoderKind::Hardware { .. } => Some(k.label()),
                            EncoderKind::Software => None,
                        })
                        .collect();
                    if !hw_labels.is_empty() {
                        let mut codecs: Vec<&'static str> = Vec::new();
                        for k in kinds.iter() {
                            if let EncoderKind::Hardware { .. } = k {
                                let c = k.codec().label();
                                if !codecs.contains(&c) {
                                    codecs.push(c);
                                }
                            }
                        }
                        theme::badge(ui, &format!("{I_CHECK}  Hardware · {}", codecs.join(", ")), p.success)
                            .on_hover_text(format!(
                                "Encoder com aceleração por hardware: {}",
                                hw_labels.join(", ")
                            ));
                    } else {
                        theme::badge(ui, &format!("{I_INFO}  Encoder por software"), p.warn)
                            .on_hover_text("Microsoft H.264 — pode usar mais CPU");
                    }
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
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                const LEFT: f32 = 400.0;
                const GAP: f32 = 24.0;
                const RIGHT: f32 = 416.0;
                let (mut share, mut watch) = (false, false);
                let wide = ui.available_width() >= 880.0;
                // centraliza na vertical com a altura do quadro anterior
                let hid = ui.make_persistent_id("home_h");
                let last_h: f32 = ui.data(|d| d.get_temp(hid)).unwrap_or(0.0);
                ui.add_space(((ui.available_height() - last_h) / 2.0).max(24.0));
                let top = ui.cursor().min.y;
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("LiveScreen")
                            .text_style(egui::TextStyle::Name("title".into()))
                            .extra_letter_spacing(-1.1),
                    );
                    ui.label(RichText::new("Compartilhe sua tela com seus amigos — direto, sem cadastro.").weak());
                });
                ui.add_space(32.0);
                if wide {
                    let total = LEFT + GAP + RIGHT;
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        // sem o espaçamento entre itens, que deslocaria o centro
                        let sp = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
                        // colunas com a mesma altura: a esquerda estica o cartão Assistir, a lista
                        // de amigos estica ou rola (até MAX_H). Medidas do quadro anterior.
                        const MAX_H: f32 = 320.0;
                        let lid = ui.make_persistent_id("home_lh");
                        let rid = ui.make_persistent_id("home_rh");
                        let lh: f32 = ui.data(|d| d.get_temp(lid)).unwrap_or(0.0);
                        let rh: f32 = ui.data(|d| d.get_temp(rid)).unwrap_or(0.0);
                        let target = lh.max(rh.min(MAX_H));
                        let extra = (target - lh).max(0.0);
                        let l = ui.allocate_ui_with_layout(vec2(LEFT, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.spacing_mut().item_spacing = sp;
                            ui.set_width(LEFT);
                            left_column(app, ui, &mut share, &mut watch, extra);
                        });
                        ui.add_space(GAP);
                        let mut nr = 0.0;
                        ui.allocate_ui_with_layout(vec2(RIGHT, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.spacing_mut().item_spacing = sp;
                            ui.set_width(RIGHT);
                            nr = super::friends::card(app, ui, Some(target));
                        });
                        let nl = l.response.rect.height() - extra;
                        ui.data_mut(|d| {
                            d.insert_temp(lid, nl);
                            d.insert_temp(rid, nr);
                        });
                        if (nl - lh).abs() > 0.5 || (nr - rh).abs() > 0.5 {
                            ui.ctx().request_repaint();
                        }
                    });
                } else {
                    let col = ui.available_width().min(520.0);
                    ui.vertical_centered(|ui| {
                        ui.allocate_ui_with_layout(vec2(col, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(col);
                            left_column(app, ui, &mut share, &mut watch, 0.0);
                            ui.add_space(16.0);
                            super::friends::card(app, ui, None);
                        });
                    });
                }
                let h = ui.min_rect().max.y - top;
                ui.data_mut(|d| d.insert_temp(hid, h));
                if (h - last_h).abs() > 0.5 {
                    ui.ctx().request_repaint();
                }
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
                    if app.ticket_input.trim().is_empty() {
                        // convite já copiado? entra preenchido: falta só "Conectar"
                        if let Ok(t) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                            if let Some(t) = livescreen_core::protocol::find_invite(&t) {
                                app.ticket_input = t;
                                app.toasts.push(
                                    "Código de convite colado da área de transferência",
                                    ToastKind::Info,
                                );
                            }
                        }
                        app.screen = Screen::WatchInput;
                    } else if matches!(parse_invite(&app.ticket_input), Some((_, Some(_)))) {
                        let ctx = ui.ctx().clone();
                        super::watch_view::connect(app, &ctx, app.ticket_input.clone());
                        if matches!(app.screen, Screen::Home) {
                            if let Some(err) = app.error.take() {
                                app.toasts.push(err, ToastKind::Warn);
                            }
                        }
                    } else {
                        app.screen = Screen::WatchInput;
                    }
                }
            });
    });
}

/// `extra`: espaço a mais no cartão Assistir, para igualar a altura da outra coluna.
fn left_column(app: &mut App, ui: &mut egui::Ui, share: &mut bool, watch: &mut bool, extra: f32) {
    let busy = app.updating.is_some();
    ui.add_enabled_ui(!busy, |ui| {
        *share = share_tile(ui, ui.available_width());
        ui.add_space(12.0);
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            theme::section_title(ui, format!("{I_WATCH}  Assistir"));
            ui.label(RichText::new("Cole o código de convite que você recebeu.").weak());
            ui.add_space(4.0 + extra);
            ui.horizontal(|ui| {
                let r = ui.add(theme::field(
                    ui,
                    egui::TextEdit::singleline(&mut app.ticket_input)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("Cole o código aqui")
                        .desired_width(ui.available_width() - 96.0),
                ));
                if r.changed() {
                    super::watch_view::normalize_ticket(&mut app.ticket_input);
                }
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    *watch = true;
                }
                if ui
                    .add(egui::Button::new("Assistir").min_size(vec2(88.0, ui.spacing().interact_size.y)))
                    .clicked()
                {
                    *watch = true;
                }
            });
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
        super::watch_view::connect(app, &ui.ctx().clone(), app.watch_target.clone());
    } else if go_home {
        app.screen = Screen::Home;
    }
}
