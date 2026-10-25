use std::time::{Duration, Instant};

use egui::{Align, Align2, Color32, CursorIcon, Layout, Margin, Order, RichText, Sense, vec2};

use super::theme::{self, I_BACK, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_MUTE, I_PASTE, I_VOLUME};
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
        }
    }
}

#[derive(PartialEq)]
enum ControlAction {
    None,
    Leave,
    ToggleFullscreen,
}

pub fn connect(app: &mut App, ctx: &egui::Context) {
    app.save_prefs();
    let hw_decode = !app.effective_overrides().deny_hardware_decoder;
    let zero_copy = hw_decode && !app.effective_overrides().deny_zero_copy;
    let target = if zero_copy {
        app.display.as_ref().map(Display::target)
    } else {
        None
    };
    if let Some(d) = app.display.as_mut() {
        d.reset();
    }
    match WatchSession::start(
        &app.rt,
        &app.ticket_input,
        &app.prefs.name,
        app.prefs.gain(),
        hw_decode,
        target,
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
fn normalize_ticket(s: &mut String) {
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
    egui::Panel::bottom("input_footer").show(ui, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            super::settings::button(app, ui);
        });
    });
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() / 2.0 - 220.0).max(16.0));
            ui.scope(|ui| {
                ui.set_max_width(560.0);
                theme::card(ui, |ui| {
                    ui.set_width(528.0);
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        theme::heading(ui, "Assistir a uma transmissão");
                        ui.label(
                            RichText::new(
                                "Cole o código de convite que você recebeu e clique em Conectar.",
                            )
                            .weak(),
                        );
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
                        if ui.button(format!("{I_PASTE}  Colar")).clicked() {
                            match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                                Ok(t) => paste = Some(t),
                                Err(_) => app.toasts.push(
                                    "Não foi possível ler a área de transferência",
                                    ToastKind::Warn,
                                ),
                            }
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
                            ui.label(
                                RichText::new(
                                    "Código inválido — confira se copiou o código inteiro.",
                                )
                                .color(ui.visuals().error_fg_color),
                            );
                        }
                        if valid && legacy {
                            ui.label(
                                RichText::new(
                                    "Código sem chave de sala: só conecta em quem transmite com o LiveScreen 0.4. Se veio de uma versão mais nova, falta o final do código (a parte depois do ponto).",
                                )
                                .color(ui.visuals().warn_fg_color),
                            );
                        }
                        ui.label("Seu nome (opcional)");
                        ui.add(theme::field(
                            ui,
                            egui::TextEdit::singleline(&mut app.prefs.name)
                                .id(name_id)
                                .char_limit(32)
                                .hint_text("Como você aparece para quem compartilha")
                                .desired_width(f32::INFINITY),
                        ));
                        if valid && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            go = true;
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Voltar").clicked() {
                                back = true;
                            }
                            let r = ui
                                .add_enabled_ui(valid, |ui| theme::primary_button(ui, "Conectar"))
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
        connect(app, &ctx);
    }
}

fn controls(app: &mut App, ui: &mut egui::Ui, st: &WatchStatus) -> ControlAction {
    let mut action = ControlAction::None;
    let p = theme::palette(ui);
    ui.horizontal(|ui| {
        if ui.button(format!("{I_BACK}  Sair")).clicked() {
            action = ControlAction::Leave;
        }
        let (text, color) = if st.phase == Phase::Connecting {
            ("Conectando…", ui.visuals().weak_text_color())
        } else if st.path == "Relay" {
            ("Via servidor", p.warn)
        } else {
            ("Conexão direta", p.success)
        };
        let tip_path = if st.path == "Relay" {
            "Via servidor"
        } else {
            &st.path
        };
        theme::badge(ui, text, color).on_hover_text(format!(
            "{} · {} ms · {} fps · {}×{}",
            tip_path, st.rtt_ms, st.fps, st.width, st.height
        ));
        if st.audio {
            let (icon, tip) = if app.prefs.muted {
                (I_MUTE, "Ativar som")
            } else {
                (I_VOLUME, "Silenciar")
            };
            if theme::icon_button(ui, icon, tip).clicked() {
                app.prefs.muted = !app.prefs.muted;
                app.save_prefs();
            }
            // Grava ao soltar o slider, não a cada quadro do arrasto.
            let r = ui.add(egui::Slider::new(&mut app.prefs.volume, 0.0..=1.0).show_value(false));
            if r.drag_stopped() {
                app.save_prefs();
            }
            let pct = (app.prefs.volume * 100.0).round() as i32;
            ui.add_sized(
                vec2(36.0, ui.spacing().interact_size.y),
                egui::Label::new(format!("{pct}%")),
            );
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (icon, tip) = if app.watch_view.fullscreen {
                (I_EXIT_FULLSCREEN, "Sair da tela cheia (Esc)")
            } else {
                (I_FULLSCREEN, "Tela cheia (F11)")
            };
            if theme::icon_button(ui, icon, tip).clicked() {
                action = ControlAction::ToggleFullscreen;
            }
            super::settings::button(app, ui);
            ui.menu_button("Qualidade", |ui| {
                ui.label(RichText::new("Só vale se for mais leve que a atual").small().weak());
                for p in livescreen_core::preset::Preset::ALL {
                    if ui.button(p.name()).clicked() {
                        if let Some(w) = app.watch.as_ref() {
                            w.request_quality(p);
                        }
                        ui.close();
                    }
                }
            })
            .response
            .on_hover_text("Pedir ao host outra qualidade (resolução e fps)");
            if !st.title.is_empty() {
                ui.add(egui::Label::new(RichText::new(&st.title).strong()).truncate());
            }
        });
    });
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
    if !app.watch_view.fullscreen {
        egui::Panel::top("watch_top").show(ui, |ui| {
            let a = controls(app, ui, &st);
            if a != ControlAction::None {
                action = a;
            }
        });
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
                    let galley = painter.layout_no_wrap(
                        text,
                        egui::FontId::monospace(12.0),
                        Color32::WHITE,
                    );
                    let pos = rect.min + vec2(8.0, 8.0);
                    painter.rect_filled(
                        egui::Rect::from_min_size(pos, galley.size()).expand(6.0),
                        4.0,
                        Color32::from_black_alpha(160),
                    );
                    painter.galley(pos, galley, Color32::WHITE);
                }
            } else {
                let connecting = st.phase == Phase::Connecting;
                let secs = app.watch_view.started.elapsed().as_secs();
                let msg = if connecting {
                    format!("Conectando ao host… ({secs} s)")
                } else {
                    "Aguardando a primeira imagem…".to_string()
                };
                let c = Color32::from_gray(200);
                ui.painter().text(
                    rect.center() + vec2(0.0, 24.0),
                    Align2::CENTER_CENTER,
                    msg,
                    egui::FontId::proportional(14.0),
                    c,
                );
                if connecting {
                    if secs >= 10 {
                        ui.painter().text(
                            rect.center() + vec2(0.0, 46.0),
                            Align2::CENTER_CENTER,
                            "Confira se quem compartilha ainda está transmitindo.",
                            egui::FontId::proportional(13.0),
                            Color32::from_gray(180),
                        );
                    }
                    ui.ctx().request_repaint_after(Duration::from_secs(1));
                }
                let spinner =
                    egui::Rect::from_center_size(rect.center() - vec2(0.0, 12.0), vec2(24.0, 24.0));
                ui.put(spinner, egui::Spinner::new().size(24.0).color(c));
            }
            double = resp.double_clicked();
        });
    if double {
        action = ControlAction::ToggleFullscreen;
    }

    if app.watch_view.fullscreen {
        let active = ctx.input(|i| {
            i.pointer.delta() != egui::Vec2::ZERO
                || i.pointer.any_pressed()
                || !i.keys_down.is_empty()
        });
        if active {
            app.watch_view.last_activity = Instant::now();
        }
        let idle = app.watch_view.last_activity.elapsed();
        if idle < OVERLAY_TIMEOUT {
            egui::Area::new(egui::Id::unique("watch_overlay"))
                .anchor(Align2::CENTER_BOTTOM, [0.0, -24.0])
                .order(Order::Foreground)
                .show(&ctx, |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_black_alpha(190))
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            let a = controls(app, ui, &st);
                            if a != ControlAction::None {
                                action = a;
                            }
                        });
                });
            ctx.request_repaint_after(OVERLAY_TIMEOUT - idle);
        } else {
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
