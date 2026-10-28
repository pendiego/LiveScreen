use std::time::{Duration, Instant};

use egui::{Align, Align2, Color32, CursorIcon, Layout, RichText, Sense, vec2};

use super::theme::{
    self, CallTone, I_ADD_FRIEND, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE,
    I_PERSON, I_QUALITY, I_SETTINGS, I_VOLUME, I_WATCH,
};
use super::toast::ToastKind;
use super::{App, Screen};
use livescreen_core::protocol::{find_invite, parse_invite};
use livescreen_core::viewer::{Phase, WatchStatus};
use crate::video::decoder::VideoFrame;
use crate::video::present::Display;
use crate::watch::WatchSession;

const OVERLAY_TIMEOUT: Duration = Duration::from_millis(2500);

pub struct WatchView {
    cpu_tex: Option<egui::TextureHandle>,
    shown: Option<(egui::TextureId, egui::Vec2)>,
    fullscreen: bool,
    last_activity: Instant,
    started: Instant,
    sent_visible: Option<bool>,
    bar_rect: Option<egui::Rect>,
}

impl Default for WatchView {
    fn default() -> Self {
        WatchView {
            cpu_tex: None,
            shown: None,
            fullscreen: false,
            last_activity: Instant::now(),
            started: Instant::now(),
            sent_visible: None,
            bar_rect: None,
        }
    }
}

#[derive(PartialEq)]
enum ControlAction {
    None,
    Leave,
    ToggleFullscreen,
}

pub fn connect(app: &mut App, ctx: &egui::Context, target: String) {
    if let Some(p) = app.take_prober() {
        p.stop();
    }
    app.join_background();
    app.watch_target = target.clone();
    app.save_prefs();
    let hw_decode = !app.effective_overrides().deny_hardware_decoder;
    let zero_copy = hw_decode && !app.effective_overrides().deny_zero_copy;
    let display_target = if zero_copy {
        app.display.as_ref().map(Display::target)
    } else {
        None
    };
    if let Some(d) = app.display.as_mut() {
        d.reset();
    }
    match WatchSession::start(
        &app.rt,
        &target,
        app.identity.clone(),
        &app.prefs.name,
        app.prefs.gain(),
        hw_decode,
        display_target,
        ctx.clone(),
    ) {
        Ok(w) => {
            app.watch = Some(w);
            app.error = None;
            app.watch_view = Default::default();
            app.screen = Screen::Watching;
        }
        Err(e) => app.error = Some(e.to_string()),
    }
}

/// Deixa só o convite quando o texto tem um; senão só tira os espaços (código quebrado em linhas).
pub(super) fn normalize_ticket(s: &mut String) {
    match find_invite(s) {
        Some(inv) => *s = inv,
        None => s.retain(|c| !c.is_whitespace()),
    }
}

pub fn input(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let mut back = false;
    let mut go = false;
    let ticket_id = egui::Id::unique("watch_ticket");
    let name_id = egui::Id::unique("watch_name");
    let pasted: Option<String> = ctx.input(|i| {
        i.events.iter().rev().find_map(|e| match e {
            egui::Event::Paste(t) => Some(t.clone()),
            _ => None,
        })
    });
    let text_focus = ctx
        .memory(|m| m.focused())
        .is_some_and(|f| f == ticket_id || f == name_id);
    let mut paste: Option<String> = None;
    let mut auto = false;
    super::home::footer(app, ui);
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() / 2.0 - 220.0).max(16.0));
            ui.scope(|ui| {
                ui.set_max_width(560.0);
                theme::card(ui, |ui| {
                    ui.set_width(528.0);
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        theme::heading(ui, format!("{I_WATCH}  Assistir a uma transmissão"));
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(
                                "Cole o código de convite que você recebeu e clique em Conectar.",
                            )
                            .color(theme::P.text_muted),
                        );
                        ui.add_space(12.0);
                        let r = ui.add(
                            egui::TextEdit::multiline(&mut app.ticket_input)
                                .id(ticket_id)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(3)
                                .hint_text("Cole o código aqui")
                                .desired_width(f32::INFINITY),
                        );
                        if r.changed() {
                            normalize_ticket(&mut app.ticket_input);
                            // Ctrl+V dentro do campo
                            auto = pasted.is_some();
                        }
                        // Ctrl+V fora dos campos de texto
                        if !text_focus && paste.is_none() {
                            paste = pasted.clone();
                        }
                        if let Some(t) = paste {
                            match find_invite(&t) {
                                Some(inv) => {
                                    app.ticket_input = inv;
                                    auto = true;
                                }
                                None => app.toasts.push(
                                    "O texto colado não tem um código de convite",
                                    ToastKind::Warn,
                                ),
                            }
                        }
                        let parsed = parse_invite(&app.ticket_input);
                        let valid = parsed.is_some();
                        let legacy = matches!(parsed, Some((_, None)));
                        // convite sem chave nunca conecta sozinho: o aviso precisa ser lido
                        if auto && valid && !legacy {
                            go = true;
                        }
                        if !app.ticket_input.trim().is_empty() && !valid {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(
                                    "Código inválido — confira se copiou o código inteiro.",
                                )
                                .color(ui.visuals().error_fg_color),
                            );
                        }
                        if valid && legacy {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(
                                    "Código sem chave de sala: só conecta em quem transmite com o LiveScreen 0.4. Se veio de uma versão mais nova, falta o final do código (a parte depois do ponto).",
                                )
                                .color(ui.visuals().warn_fg_color),
                            );
                        }
                        ui.add_space(12.0);
                        ui.add(theme::field(
                            ui,
                            egui::TextEdit::singleline(&mut app.prefs.name)
                                .id(name_id)
                                .char_limit(32)
                                .hint_text(
                                    RichText::new(format!("{I_PERSON}  Seu nome (opcional)"))
                                        .color(theme::P.text_muted),
                                )
                                .desired_width(f32::INFINITY),
                        ));
                        if valid && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            go = true;
                        }
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if theme::button(ui, theme::Kind::Ghost, "Voltar").clicked() {
                                back = true;
                            }
                            if theme::button(ui, theme::Kind::Ghost, format!("{I_PASTE}  Colar")).clicked() {
                                match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                                    Ok(t) => match find_invite(&t) {
                                        Some(inv) => {
                                            app.ticket_input = inv;
                                            let p = parse_invite(&app.ticket_input);
                                            if p.is_some() && !matches!(p, Some((_, None))) {
                                                go = true;
                                            }
                                        }
                                        None => app.toasts.push(
                                            "O texto colado não tem um código de convite",
                                            ToastKind::Warn,
                                        ),
                                    },
                                    Err(_) => app.toasts.push(
                                        "Não foi possível ler a área de transferência",
                                        ToastKind::Warn,
                                    ),
                                }
                            }
                            let r = ui
                                .add_enabled_ui(valid, |ui| {
                                    theme::button_full(ui, theme::Kind::Primary, "Conectar")
                                })
                                .inner;
                            if r.clicked() {
                                go = true;
                            }
                        });
                        // não rouba o foco de botão (Tab) nem do campo de nome
                        if ctx.memory(|m| m.focused().is_none()) && !valid {
                            ctx.memory_mut(|m| m.request_focus(ticket_id));
                        }
                        if let Some(e) = &app.error {
                            ui.add_space(8.0);
                            ui.label(RichText::new(e).color(ui.visuals().error_fg_color));
                        }
                    });
                });
            });
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        back = true;
    }
    if back {
        app.error = None;
        app.save_prefs();
        app.screen = Screen::Home;
    } else if go {
        connect(app, &ctx, app.ticket_input.clone());
    }
}

fn controls(app: &mut App, ui: &mut egui::Ui, st: &WatchStatus) -> ControlAction {
    let mut action = ControlAction::None;
    if st.audio {
        let (icon, tip, tone) = if app.prefs.muted {
            (I_MUTE, "Ativar som", CallTone::Alert)
        } else {
            (I_VOLUME, "Silenciar", CallTone::Plain)
        };
        if theme::call_button(ui, icon, tip, tone).clicked() {
            app.prefs.muted = !app.prefs.muted;
            app.save_prefs();
        }
        ui.spacing_mut().slider_width = 96.0;
        let r = ui
            .scope(|ui| {
                let v = ui.visuals_mut();
                v.selection.bg_fill = theme::P.accent;
                v.widgets.inactive.bg_fill = theme::P.border_strong;
                v.widgets.hovered.bg_fill = theme::P.text;
                v.widgets.active.bg_fill = theme::P.text;
                ui.add(
                    egui::Slider::new(&mut app.prefs.volume, 0.0..=1.0)
                        .show_value(false)
                        .trailing_fill(true),
                )
            })
            .inner;
        if r.drag_stopped() {
            app.save_prefs();
        }
        let pct = (app.prefs.volume * 100.0).round() as i32;
        ui.label(
            RichText::new(format!("{pct}%"))
                .small()
                .color(theme::P.text_muted),
        );
    }

    let q = theme::call_button(
        ui,
        I_QUALITY,
        "Qualidade: pedir ao host outra resolução e fps",
        CallTone::Neutral,
    );
    egui::Popup::menu(&q)
        .align(egui::RectAlign::TOP_START)
        .show(|ui| {
            ui.label(RichText::new("Só vale se for mais leve que a atual").small().weak());
            for p in livescreen_core::preset::Preset::ALL {
                if ui.button(p.name()).clicked() {
                    if let Some(w) = app.watch.as_ref() {
                        w.request_quality(p);
                    }
                    ui.close();
                }
            }
        });

    if matches!(st.phase, Phase::Playing) && !app.identity_temporary {
        if let Some(id) = livescreen_core::friends::parse_friend_code(&st.host_id) {
            let is_friend = app.friends.iter().filter_map(|f| f.id()).any(|fid| fid == id);
            if !is_friend
                && theme::call_button(
                    ui,
                    I_ADD_FRIEND,
                    "Adicionar quem transmite aos amigos",
                    CallTone::Neutral,
                )
                .clicked()
            {
                app.add_friend = Some(super::friends::Draft {
                    name: st.host_name.clone(),
                    code: st.host_id.clone(),
                    error: None,
                });
            }
        }
    }

    if theme::call_button(
        ui,
        I_SETTINGS,
        super::settings::SETTINGS_LABEL,
        if app.settings_open {
            CallTone::Active
        } else {
            CallTone::Neutral
        },
    )
    .clicked()
    {
        app.settings_open = true;
    }

    let (icon, tip, tone) = if app.watch_view.fullscreen {
        (
            I_EXIT_FULLSCREEN,
            "Sair da tela cheia (Esc)",
            CallTone::Active,
        )
    } else {
        (
            I_FULLSCREEN,
            "Tela cheia (F11)",
            CallTone::Neutral,
        )
    };
    if theme::call_button(ui, icon, tip, tone).clicked() {
        action = ControlAction::ToggleFullscreen;
    }

    if theme::hangup_button(ui, I_HANGUP, "Sair").clicked() {
        action = ControlAction::Leave;
    }

    action
}

/// Avisa o host quando a janela some ou volta. Roda em `App::logic`: com a janela minimizada o
/// eframe não executa `App::ui`, só a lógica, então é o único lugar que enxerga a minimização.
pub fn sync_visible(app: &mut App, ctx: &egui::Context) {
    let Some(w) = app.watch.as_ref() else {
        app.watch_view.sent_visible = None;
        return;
    };
    let visible = !ctx.input(|i| i.viewport().minimized.unwrap_or(false));
    if app.watch_view.sent_visible != Some(visible) {
        w.set_visible(visible);
        app.watch_view.sent_visible = Some(visible);
    }
}

pub fn watching(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let Some(w) = app.watch.as_ref() else { return };
    let st = w.status.lock().unwrap().clone();
    if let Phase::Ended(m) = &st.phase {
        if app.watch_view.fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }
        app.end(m.clone(), true);
        return;
    }
    w.set_gain(app.prefs.gain());

    if let Some(f) = w.frame.lock().unwrap().take() {
        match f {
            VideoFrame::Cpu(f) => {
                let img = egui::ColorImage::new([f.width as usize, f.height as usize], f.pixels);
                let t = match &mut app.watch_view.cpu_tex {
                    Some(t) => {
                        t.set(img, egui::TextureOptions::LINEAR);
                        t
                    }
                    None => app
                        .watch_view
                        .cpu_tex
                        .insert(ctx.load_texture("video", img, egui::TextureOptions::LINEAR)),
                };
                app.watch_view.shown = Some((t.id(), t.size_vec2()));
            }
            VideoFrame::Gpu(g) => match app.display.as_mut().map(|d| d.show(&g)) {
                Some(Ok(s)) => app.watch_view.shown = Some(s),
                other => {
                    let why = match other {
                        Some(Err(e)) => format!("{e:#}"),
                        _ => "sem exibição DX12".into(),
                    };
                    tracing::warn!("zero-copy: {why}");
                    w.disable_zero_copy();
                }
            },
        }
    }

    let mut action = ControlAction::None;
    if ctx.input(|i| i.key_pressed(egui::Key::F11))
        || (app.watch_view.fullscreen && ctx.input(|i| i.key_pressed(egui::Key::Escape)))
    {
        action = ControlAction::ToggleFullscreen;
    }
    let mut double = false;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(Color32::BLACK))
        .show(ui, |ui| {
            let (rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click());
            if let Some((tex_id, tex_size)) = app.watch_view.shown {
                let (fw, fh) = (tex_size.x, tex_size.y);
                // o decoder devolve altura alinhada (ex.: 1088); recorta ao tamanho anunciado no Welcome
                let (vw, vh) = if st.width > 0 && st.width as f32 <= fw && st.height as f32 <= fh {
                    (st.width as f32, st.height as f32)
                } else {
                    (fw, fh)
                };
                let scale = (rect.width() / vw).min(rect.height() / vh);
                let r = egui::Rect::from_center_size(rect.center(), vec2(vw, vh) * scale);
                let uv =
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(vw / fw, vh / fh));
                ui.painter().image(tex_id, r, uv, Color32::WHITE);
                if app.prefs.show_stats {
                    let path = match st.path.as_str() {
                        "Direto" => "Direto",
                        "Relay" => "Via servidor",
                        _ => "?",
                    };
                    let text = format!(
                        "Decodificação: {} · {} · {:.0} fps · {:.1} ms/quadro · {}\nRede: {} · {} ms · {:.0} fps recebidos · {}×{}",
                        if st.decoder_hw { "placa de vídeo" } else { "processador" },
                        if st.codec.is_empty() { "—" } else { &st.codec },
                        st.decode_fps,
                        st.decode_ms,
                        if st.zero_copy { "sem cópia" } else { "cópia via CPU" },
                        path,
                        st.rtt_ms,
                        st.fps,
                        st.width,
                        st.height
                    );
                    let painter = ui.painter();
                    let font_id = egui::FontId::new(12.0, egui::FontFamily::Monospace);
                    let galley = painter.layout_no_wrap(text, font_id, theme::P.text);
                    let padding = 10.0;
                    let bg_size = galley.size() + vec2(padding * 2.0, padding * 2.0);
                    // abaixo da pílula do título (y 16 + ~36), que o texto longo de estatísticas cobriria
                    let top_right = rect.right_top() + vec2(-16.0, 64.0);
                    let bg_rect = egui::Rect::from_min_size(
                        egui::pos2(top_right.x - bg_size.x, top_right.y),
                        bg_size,
                    );
                    painter.rect_filled(
                        bg_rect,
                        theme::R_SM as f32,
                        Color32::from_black_alpha(153),
                    );
                    painter.galley(bg_rect.min + vec2(padding, padding), galley, theme::P.text);
                }
            } else {
                let connecting = st.phase == Phase::Connecting;
                let secs = app.watch_view.started.elapsed().as_secs();
                let msg = if connecting {
                    format!("Conectando ao host… ({secs} s)")
                } else {
                    "Aguardando a primeira imagem…".to_string()
                };
                ui.painter().text(
                    rect.center() + vec2(0.0, 24.0),
                    Align2::CENTER_CENTER,
                    msg,
                    egui::FontId::new(20.0, egui::FontFamily::Name("display-medium".into())),
                    theme::P.text,
                );
                if connecting {
                    if secs >= 10 {
                        ui.painter().text(
                            rect.center() + vec2(0.0, 50.0),
                            Align2::CENTER_CENTER,
                            "Confira se quem compartilha ainda está transmitindo.",
                            egui::FontId::new(13.0, egui::FontFamily::Proportional),
                            theme::P.text_muted,
                        );
                    }
                    ui.ctx().request_repaint_after(Duration::from_secs(1));
                }
                let spinner =
                    egui::Rect::from_center_size(rect.center() - vec2(0.0, 12.0), vec2(24.0, 24.0));
                ui.put(spinner, egui::Spinner::new().size(24.0).color(theme::P.accent));
            }
            double = resp.double_clicked();
        });
    if double {
        action = ControlAction::ToggleFullscreen;
    }

    let active = ctx.input(|i| {
        i.pointer.delta() != egui::Vec2::ZERO || i.pointer.any_pressed() || !i.keys_down.is_empty()
    });
    if active {
        app.watch_view.last_activity = Instant::now();
    }
    let over_bar = app
        .watch_view
        .bar_rect
        .zip(ctx.pointer_hover_pos())
        .is_some_and(|(r, p)| r.contains(p));
    // sem imagem, com menu/modal aberto ou mouse na barra, os controles não somem
    let keep = app.watch_view.shown.is_none()
        || over_bar
        || egui::Popup::is_any_open(&ctx)
        || app.settings_open
        || app.add_friend.is_some();
    let idle = app.watch_view.last_activity.elapsed();
    if keep || idle < OVERLAY_TIMEOUT {
        egui::Area::new(egui::Id::unique("watch_path_badge"))
            .anchor(Align2::LEFT_TOP, vec2(24.0, 24.0))
            .show(&ctx, |ui| {
                let (text, color) = if st.phase == Phase::Connecting {
                    ("Conectando…", theme::P.text_muted)
                } else if st.path == "Relay" {
                    ("Via servidor", theme::P.warn)
                } else {
                    ("Conexão direta", theme::P.success)
                };
                let tip_path = if st.path == "Relay" {
                    "Via servidor"
                } else {
                    &st.path
                };
                let hint = if st.path == "Relay" && st.phase != Phase::Connecting {
                    "\nServidor público compartilhado, com limite de banda: se travar, baixe a qualidade."
                } else {
                    ""
                };
                theme::badge(ui, text, color).on_hover_text(format!(
                    "{} · {} ms · {} fps · {}×{}{}",
                    tip_path, st.rtt_ms, st.fps, st.width, st.height, hint
                ));
            });

        if !st.title.is_empty() {
            theme::call_title(&ctx, &st.title);
        }
        let bar = theme::call_bar(&ctx, "watch_bar", |ui| controls(app, ui, &st));
        app.watch_view.bar_rect = Some(bar.response.rect);
        if bar.inner != ControlAction::None {
            action = bar.inner;
        }
        if !keep {
            ctx.request_repaint_after(OVERLAY_TIMEOUT - idle);
        }
    } else {
        app.watch_view.bar_rect = None;
        if app.watch_view.fullscreen {
            ctx.set_cursor_icon(CursorIcon::None);
        }
    }

    match action {
        ControlAction::None => {}
        ControlAction::ToggleFullscreen => {
            app.watch_view.fullscreen = !app.watch_view.fullscreen;
            app.watch_view.last_activity = Instant::now();
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(app.watch_view.fullscreen));
        }
        ControlAction::Leave => {
            if app.watch_view.fullscreen {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            app.stop_all();
            app.screen = Screen::Home;
        }
    }
}
