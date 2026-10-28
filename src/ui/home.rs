use egui::{Align, Align2, CursorIcon, FontFamily, FontId, Layout, RichText, Sense, Stroke, StrokeKind, vec2};
use livescreen_core::protocol::parse_invite;

use super::theme::{self, I_CIRCLE_CHECK, I_CHEVRON, I_ERROR, I_INFO, I_SHARE, I_UPDATE, I_WARN, I_WATCH};
use super::toast::ToastKind;
use super::{App, EndKind, Screen};
use crate::video::encoder::EncoderKind;

/// Ação primária da home: tile cheio na cor `accent`. Devolve true no clique.
fn share_tile(ui: &mut egui::Ui, width: f32) -> bool {
    let p = theme::P;
    let enabled = ui.is_enabled();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 120.0), Sense::click());
    let fill = if !enabled {
        p.accent.gamma_multiply(0.4)
    } else if resp.is_pointer_button_down_on() {
        p.accent.lerp_to_gamma(egui::Color32::WHITE, 0.14)
    } else if resp.hovered() {
        p.accent.lerp_to_gamma(egui::Color32::WHITE, 0.08)
    } else {
        p.accent
    };
    let ring = if resp.has_focus() {
        Stroke::new(2.0, p.text)
    } else {
        Stroke::NONE
    };
    let painter = ui.painter();
    painter.rect(rect, theme::R_LG, fill, ring, StrokeKind::Outside);
    let fg = p.on_accent;
    let c_y = rect.center().y;
    painter.text(
        egui::pos2(rect.left() + 24.0, c_y),
        Align2::LEFT_CENTER,
        I_SHARE,
        FontId::proportional(28.0),
        fg,
    );
    let title_galley = painter.layout_no_wrap(
        "Compartilhar tela".into(),
        FontId::new(20.0, FontFamily::Name("display-medium".into())),
        fg,
    );
    let sub_galley = painter.layout_no_wrap(
        "Transmita uma tela ou janela".into(),
        FontId::proportional(14.0),
        fg.gamma_multiply(0.8),
    );
    let gap = 2.0;
    let total_h = title_galley.size().y + gap + sub_galley.size().y;
    let top_y = c_y - total_h / 2.0;
    let x = rect.left() + 68.0;
    painter.galley(egui::pos2(x, top_y), title_galley, fg);
    painter.galley(
        egui::pos2(x, top_y + total_h - sub_galley.size().y),
        sub_galley,
        fg.gamma_multiply(0.8),
    );
    painter.text(
        egui::pos2(rect.right() - 24.0, c_y),
        Align2::RIGHT_CENTER,
        I_CHEVRON,
        FontId::proportional(24.0),
        fg,
    );
    if enabled && resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, "Compartilhar tela"));
    resp.clicked()
}

/// Pílula de status do rodapé: altura fixa de 28 dp centrada no conteúdo. Um `Frame` aqui esticaria
/// até a altura do painel, porque `horizontal_centered` justifica os filhos no eixo vertical.
fn pill(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.add_space(4.0);
        add(ui);
        ui.add_space(4.0);
    });
    let rect = egui::Rect::from_center_size(
        inner.response.rect.center(),
        vec2(inner.response.rect.width() + 12.0, 28.0),
    );
    ui.painter().set(
        bg,
        egui::epaint::RectShape::filled(rect, theme::R_PILL, theme::P.raised),
    );
    inner.response
}

pub fn footer(app: &mut App, ui: &mut egui::Ui) {
    let enc = app.encoder.lock().unwrap().clone();
    let update = app.update.lock().unwrap().clone();
    let p = theme::P;
    let mut do_open_help = false;
    let mut do_open_logs = false;

    egui::Panel::bottom("footer")
        .exact_size(48.0)
        .frame(
            egui::Frame::new()
                .fill(p.surface)
                .inner_margin(egui::Margin::symmetric(24, 0)),
        )
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                match &enc {
                    None => {
                        pill(ui, |ui| {
                            ui.add(egui::Spinner::new().size(12.0));
                            ui.label(RichText::new("Detectando encoder de vídeo…").small().color(p.text_muted));
                        });
                    }
                    Some(kinds) => {
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
                            pill(ui, |ui| {
                                ui.label(RichText::new(I_CIRCLE_CHECK).size(16.0).color(p.success));
                                ui.label(
                                    RichText::new(format!("Hardware · {}", codecs.join(", ")))
                                        .small()
                                        .color(p.text_muted),
                                );
                            })
                            .on_hover_text(format!(
                                "Encoder com aceleração por hardware: {}",
                                hw_labels.join(", ")
                            ));
                        } else {
                            pill(ui, |ui| {
                                ui.label(RichText::new(I_INFO).size(16.0).color(p.warn));
                                ui.label(RichText::new("Encoder por software").small().color(p.text_muted));
                            })
                            .on_hover_text("Microsoft H.264 — pode usar mais CPU");
                        }
                    }
                }

                ui.add_space(8.0);
                let (sep_rect, _) = ui.allocate_exact_size(vec2(1.0, 16.0), Sense::hover());
                ui.painter().rect_filled(sep_rect, 0, p.border);
                ui.add_space(8.0);

                ui.label(
                    RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .family(FontFamily::Monospace)
                        .size(12.0)
                        .color(p.text_faint),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // O layout justifica no eixo vertical: sem altura fixa o botão esticaria até o painel.
                    ui.allocate_ui_with_layout(vec2(32.0, 32.0), Layout::left_to_right(Align::Center), |ui| super::settings::button(app, ui));
                    ui.add_space(16.0);

                    if app.updating.is_some() {
                        ui.label(RichText::new("Baixando atualização…").small().color(p.text_muted));
                        ui.add(egui::Spinner::new().size(12.0));
                        ui.add_space(16.0);
                    } else if let Some(rel) = update {
                        if theme::small_button(
                            ui,
                            theme::Kind::Secondary,
                            format!("{I_UPDATE}  Atualizar para a versão {}", rel.version),
                        )
                        .on_hover_text("Baixa a versão nova e reinicia o LiveScreen")
                        .clicked()
                        {
                            app.start_update(rel, ui.ctx());
                        }
                        ui.add_space(16.0);
                    }

                    if theme::small_button(
                        ui,
                        theme::Kind::Ghost,
                        RichText::new(format!("{I_INFO}  Ajuda"))
                            .family(FontFamily::Name("medium".into()))
                            .size(12.0)
                            .color(p.text_muted),
                    )
                    .on_hover_text("Abre a ajuda no navegador")
                    .clicked()
                    {
                        do_open_help = true;
                    }
                    ui.add_space(16.0);

                    if theme::small_button(
                        ui,
                        theme::Kind::Ghost,
                        RichText::new("Abrir pasta de logs")
                            .family(FontFamily::Name("medium".into()))
                            .size(12.0)
                            .color(p.text_muted),
                    )
                    .clicked()
                    {
                        do_open_logs = true;
                    }
                });
            });
        });

    if do_open_help {
        super::settings::open_help(app);
    }
    if do_open_logs {
        super::settings::open_logs(app);
    }
}

pub fn home(app: &mut App, ui: &mut egui::Ui) {
    footer(app, ui);
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
                    let font = FontId::new(32.0, FontFamily::Name("display".into()));
                    let title_galley = ui.painter().layout_no_wrap("LiveScreen".into(), font, theme::P.text);
                    let total_w = 10.0 + 10.0 + title_galley.size().x;
                    let left_pad = ((ui.available_width() - total_w) / 2.0).max(0.0);
                    let row_h = title_galley.size().y;
                    ui.allocate_ui_with_layout(vec2(ui.available_width(), row_h), Layout::left_to_right(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.add_space(left_pad);
                        let (dot_rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 5.0, theme::P.accent);
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("LiveScreen")
                                .text_style(egui::TextStyle::Name("title".into()))
                                .color(theme::P.text),
                        );
                    });
                    ui.label(
                        RichText::new("Compartilhe sua tela com seus amigos — direto, sem cadastro.")
                            .text_style(egui::TextStyle::Body)
                            .color(theme::P.text_muted),
                    );
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
    update_prompt(app, ui.ctx());
}

/// `extra`: espaço a mais no cartão Assistir, para igualar a altura da outra coluna.
fn left_column(app: &mut App, ui: &mut egui::Ui, share: &mut bool, watch: &mut bool, extra: f32) {
    let busy = app.updating.is_some();
    ui.add_enabled_ui(!busy, |ui| {
        *share = share_tile(ui, ui.available_width());
        ui.add_space(12.0);
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            theme::heading(ui, format!("{I_WATCH}  Assistir"));
            ui.label(
                RichText::new("Cole o código de convite que você recebeu.")
                    .text_style(egui::TextStyle::Body)
                    .color(theme::P.text_muted),
            );
            ui.add_space(4.0 + extra);
            let r = ui.add(theme::field(
                ui,
                egui::TextEdit::singleline(&mut app.ticket_input)
                    .font(egui::TextStyle::Monospace)
                    .hint_text(RichText::new("Cole o código aqui").text_style(egui::TextStyle::Monospace).color(theme::P.text_faint))
                    .desired_width(f32::INFINITY),
            ));
            if r.changed() {
                super::watch_view::normalize_ticket(&mut app.ticket_input);
            }
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                *watch = true;
            }
            ui.add_space(8.0);
            if theme::button_full(ui, theme::Kind::Secondary, "Assistir").clicked() {
                *watch = true;
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

    footer(app, ui);

    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() / 2.0 - 130.0).max(20.0));
            let p = theme::P;
            let (icon, color) = match kind {
                EndKind::Info => (I_CIRCLE_CHECK, p.accent),
                EndKind::Warn => (I_WARN, p.warn),
                EndKind::Error => (I_ERROR, p.danger),
            };
            ui.scope(|ui| {
                ui.set_max_width(408.0);
                egui::Frame::new()
                    .fill(p.surface)
                    .stroke(egui::Stroke::new(1.0, p.border))
                    .corner_radius(theme::R_LG)
                    .inner_margin(28.0)
                    .show(ui, |ui| {
                        ui.set_width(408.0 - 56.0 - 2.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(icon).size(32.0).color(color));
                            ui.add_space(12.0);
                            theme::heading(ui, &message);
                            ui.add_space(20.0);
                            if !retry {
                                if theme::button_full(ui, theme::Kind::Primary, "Voltar ao início").clicked() {
                                    go_home = true;
                                }
                            } else {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 12.0;
                                    let w = ui.available_width();
                                    let btn_w = (w - 12.0) / 2.0;
                                    ui.scope(|ui| {
                                        ui.set_width(btn_w);
                                        if theme::button_full(ui, theme::Kind::Secondary, "Tentar de novo").clicked() {
                                            again = true;
                                        }
                                    });
                                    ui.scope(|ui| {
                                        ui.set_width(btn_w);
                                        if theme::button_full(ui, theme::Kind::Primary, "Voltar ao início").clicked() {
                                            go_home = true;
                                        }
                                    });
                                });
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

pub fn update_prompt(app: &mut App, ctx: &egui::Context) {
    let update = app.update.lock().unwrap().clone();
    let Some(rel) = update else {
        return;
    };
    if app.update_prompt_dismissed
        || app.updating.is_some()
        || app.settings_open
        || app.add_friend.is_some()
    {
        return;
    }
    let mut dismiss = false;
    let mut accept = false;
    let modal = theme::modal(ctx, "update_prompt").show(ctx, |ui| {
        ui.set_width(420.0);
        ui.vertical_centered(|ui| {
            theme::icon_ring(ui, I_UPDATE, theme::P.accent);
            ui.add_space(12.0);
            theme::heading(ui, "Atualização disponível");
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "A versão {} do LiveScreen está pronta para instalar. Você está na {}. O app reinicia ao terminar.",
                    rel.version,
                    env!("CARGO_PKG_VERSION")
                ))
                .text_style(egui::TextStyle::Body)
                .color(theme::P.text_muted),
            );
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                let w = ui.available_width();
                let btn_w = (w - 12.0) / 2.0;
                ui.scope(|ui| {
                    ui.set_width(btn_w);
                    if theme::button_full(ui, theme::Kind::Ghost, "Depois").clicked() {
                        dismiss = true;
                    }
                });
                ui.scope(|ui| {
                    ui.set_width(btn_w);
                    if theme::button_full(
                        ui,
                        theme::Kind::Primary,
                        format!("{I_UPDATE}  Atualizar agora"),
                    )
                    .clicked()
                    {
                        accept = true;
                    }
                });
            });
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) || modal.should_close() {
        dismiss = true;
    }
    if accept {
        app.start_update(rel, ctx);
        app.update_prompt_dismissed = true;
    } else if dismiss {
        app.update_prompt_dismissed = true;
    }
}
