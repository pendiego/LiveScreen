use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::time::{Duration, Instant};

use egui::{
    Align, Align2, Color32, CursorIcon, Layout, RichText, Sense, Stroke, StrokeKind, UiBuilder,
    pos2, vec2,
};

use super::theme::{
    self, I_BACK, I_CHECK, I_COPY, I_MONITOR, I_PEOPLE, I_PERSON, I_PLAY, I_REFRESH, I_REMOVE,
    I_SEARCH, I_STOP, I_VOLUME, I_WARN, I_WINDOW,
};
use super::toast::ToastKind;
use super::{App, Screen};
use crate::capture::{self, SourceId, SourceInfo};
use crate::preset::Preset;
use crate::share::{AudioState, MAX_VIEWERS, ShareSession, ShareStatus};

type Thumb = Option<(u32, u32, Vec<u8>)>;
struct Update {
    sources: Vec<SourceInfo>,
    thumbs: Vec<(SourceId, Thumb)>,
}

fn to_image((w, h, rgba): (u32, u32, Vec<u8>)) -> egui::ColorImage {
    egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba)
}

pub struct Picker {
    sources: Vec<SourceInfo>,
    textures: HashMap<SourceId, egui::TextureHandle>,
    selected: Option<SourceId>,
    rx: Receiver<Update>,
    refresh: Sender<()>,
    filter: String,
    filter_focused: bool,
    /// Filtro já aplicado em `monitors`/`windows` (evita refiltrar a cada quadro).
    applied: String,
    monitors: Vec<usize>,
    windows: Vec<usize>,
    pub preset: Preset,
    pub audio: bool,
}

impl Picker {
    pub fn new(ctx: egui::Context, preset: Preset, audio: bool) -> Picker {
        let (tx, rx) = channel::<Update>();
        let (refresh, refresh_rx) = channel::<()>();
        std::thread::spawn(move || {
            loop {
                let sources = capture::enumerate();
                let thumbs = sources
                    .iter()
                    .map(|s| (s.id.clone(), capture::thumbnail(&s.id, 240)))
                    .collect();
                if tx.send(Update { sources, thumbs }).is_err() {
                    return;
                }
                ctx.request_repaint();
                if let Err(std::sync::mpsc::RecvTimeoutError::Disconnected) =
                    refresh_rx.recv_timeout(Duration::from_secs(3))
                {
                    return;
                }
            }
        });
        Picker {
            sources: vec![],
            textures: HashMap::new(),
            selected: None,
            rx,
            refresh,
            filter: String::new(),
            applied: String::new(),
            filter_focused: false,
            monitors: vec![],
            windows: vec![],
            preset,
            audio,
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        let mut changed = false;
        while let Ok(u) = self.rx.try_recv() {
            self.sources = u.sources;
            changed = true;
            if let Some(sel) = &self.selected
                && !self.sources.iter().any(|s| &s.id == sel)
            {
                self.selected = None;
            }
            for (id, t) in u.thumbs {
                let Some(t) = t else { continue };
                let img = to_image(t);
                match self.textures.get_mut(&id) {
                    Some(tex) => tex.set(img, egui::TextureOptions::LINEAR),
                    None => {
                        let tex = ctx.load_texture(
                            format!("thumb{:?}", id),
                            img,
                            egui::TextureOptions::LINEAR,
                        );
                        self.textures.insert(id, tex);
                    }
                }
            }
        }
        if changed {
            // Miniaturas de janelas fechadas não podem ficar ocupando textura para sempre.
            let sources = &self.sources;
            self.textures
                .retain(|id, _| sources.iter().any(|s| &s.id == id));
            self.refresh_filter();
        }
    }

    /// Recalcula as listas visíveis a partir de `sources`/`filter`.
    fn refresh_filter(&mut self) {
        let f = self.filter.to_lowercase();
        self.monitors = (0..self.sources.len())
            .filter(|&i| matches!(self.sources[i].id, SourceId::Monitor { .. }))
            .collect();
        self.windows = (0..self.sources.len())
            .filter(|&i| matches!(self.sources[i].id, SourceId::Window { .. }))
            .filter(|&i| {
                f.is_empty()
                    || self.sources[i].title.to_lowercase().contains(&f)
                    || self.sources[i].subtitle.to_lowercase().contains(&f)
            })
            .collect();
        self.applied.clone_from(&self.filter);
    }

    fn selected_is_window(&self) -> bool {
        matches!(self.selected, Some(SourceId::Window { .. }))
    }
}

/// Início do compartilhamento em andamento (thread de fundo).
pub struct Starting {
    pub rx: Receiver<anyhow::Result<ShareSession>>,
    title: String,
    source: SourceId,
}

/// Miniatura periódica da fonte compartilhada.
pub struct Preview {
    rx: Receiver<Thumb>,
    _stop: Sender<()>,
    texture: Option<egui::TextureHandle>,
    available: bool,
}

impl Preview {
    pub fn new(ctx: &egui::Context, source: SourceId) -> Preview {
        let (tx, rx) = channel::<Thumb>();
        let (stop, stop_rx) = channel::<()>();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            loop {
                if tx.send(capture::thumbnail(&source, 480)).is_err() {
                    return;
                }
                ctx.request_repaint();
                if let Err(std::sync::mpsc::RecvTimeoutError::Disconnected) =
                    stop_rx.recv_timeout(Duration::from_secs(3))
                {
                    return;
                }
            }
        });
        Preview {
            rx,
            _stop: stop,
            texture: None,
            available: false,
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(t) = self.rx.try_recv() {
            self.available = t.is_some();
            let Some(t) = t else { continue };
            let img = to_image(t);
            match &mut self.texture {
                Some(tex) => tex.set(img, egui::TextureOptions::LINEAR),
                None => {
                    self.texture =
                        Some(ctx.load_texture("share-preview", img, egui::TextureOptions::LINEAR))
                }
            }
        }
    }
}

fn begin_start(app: &mut App, ctx: &egui::Context) {
    let Some(p) = app.picker.as_ref() else { return };
    let Some(info) = p
        .sources
        .iter()
        .find(|s| Some(&s.id) == p.selected.as_ref())
        .cloned()
    else {
        app.error = Some("A fonte selecionada não existe mais".into());
        return;
    };
    let Some(kind) = app.encoder.lock().unwrap().clone() else {
        return;
    };
    let (preset, audio) = (p.preset, p.audio);
    app.prefs.preset = preset;
    app.prefs.share_audio = audio;
    app.save_prefs();
    app.error = None;

    // exclui a janela da captura antes de a thread de início criar a captura
    app.sync_capture_exclusion(true);
    let (tx, rx) = channel();
    let (rt, overrides, ctx2) = (app.rt.clone(), app.overrides, ctx.clone());
    let (title, source) = (info.title.clone(), info.id.clone());
    app.background.push(std::thread::spawn(move || {
        let res = ShareSession::start(&rt, info, preset, audio, kind, overrides);
        ctx2.request_repaint();
        if let Err(std::sync::mpsc::SendError(Ok(s))) = tx.send(res) {
            s.stop();
        }
    }));
    app.starting = Some(Starting { rx, title, source });
}

/// Devolve true se a transição para Sharing ocorreu.
fn poll_starting(app: &mut App, ctx: &egui::Context) {
    let Some(st) = app.starting.as_ref() else {
        return;
    };
    match st.rx.try_recv() {
        Ok(Ok(s)) => {
            let st = app.starting.take().unwrap();
            app.share = Some(s);
            app.share_title = st.title;
            app.share_started = std::time::Instant::now();
            app.share_preview = Some(Preview::new(ctx, st.source.clone()));
            app.share_source = Some(st.source);
            app.known_viewers.clear();
            app.picker = None;
            app.screen = Screen::Sharing;
        }
        Ok(Err(e)) => {
            app.error = Some(format!("Não foi possível iniciar: {e:#}"));
            app.starting = None;
        }
        Err(TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(100)),
        Err(TryRecvError::Disconnected) => {
            app.error = Some("Falha inesperada ao iniciar".into());
            app.starting = None;
        }
    }
}

fn preset_hint(pr: Preset) -> &'static str {
    match pr {
        Preset::P1080p60 => "Melhor imagem; exige mais da rede e da GPU",
        Preset::P720p30 => "Leve; boa para redes lentas",
        Preset::Native60 => "Resolução original da fonte; mais pesado",
    }
}

/// Cartão de fonte; devolve (clique, duplo clique).
fn source_card(
    ui: &mut egui::Ui,
    s: &SourceInfo,
    tex: Option<&egui::TextureHandle>,
    selected: bool,
    card_w: f32,
) -> (bool, bool) {
    let pal = theme::palette(ui);
    let thumb_w = (card_w - 16.0).max(1.0);
    let thumb_h = (thumb_w * 9.0 / 16.0).round();
    let card_h = thumb_h + 65.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
    let resp = resp.on_hover_text(format!("{}\n{}", s.title, s.subtitle));
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    let stroke = if selected {
        Stroke::new(2.0, pal.accent)
    } else if hovered {
        Stroke::new(1.0, pal.accent)
    } else {
        Stroke::new(1.0, pal.border)
    };
    ui.painter()
        .rect(rect, 8, pal.card, stroke, StrokeKind::Inside);

    let thumb = egui::Rect::from_min_size(rect.min + vec2(8.0, 8.0), vec2(thumb_w, thumb_h));
    ui.painter()
        .rect_filled(thumb, 6, ui.visuals().extreme_bg_color);
    let weak = ui.visuals().weak_text_color();
    match tex {
        Some(t) if !s.minimized => {
            let sz = t.size_vec2();
            let scale = (thumb.width() / sz.x.max(1.0)).min(thumb.height() / sz.y.max(1.0));
            let r = egui::Rect::from_center_size(thumb.center(), sz * scale);
            ui.painter().image(
                t.id(),
                r,
                egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        _ => {
            ui.painter().text(
                thumb.center() - vec2(0.0, 10.0),
                Align2::CENTER_CENTER,
                I_WINDOW,
                egui::FontId::proportional(28.0),
                weak,
            );
            let msg = if s.minimized {
                "Minimizada"
            } else {
                "Sem prévia"
            };
            ui.painter().text(
                thumb.center() + vec2(0.0, 18.0),
                Align2::CENTER_CENTER,
                msg,
                egui::FontId::proportional(12.0),
                weak,
            );
        }
    }
    if selected {
        ui.painter().text(
            thumb.right_top() + vec2(-6.0, 6.0),
            Align2::RIGHT_TOP,
            I_CHECK,
            egui::FontId::proportional(18.0),
            pal.accent,
        );
    }
    let text_rect = egui::Rect::from_min_max(
        pos2(rect.min.x + 10.0, thumb.max.y + 6.0),
        pos2(rect.max.x - 10.0, rect.max.y - 4.0),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 2.0;
    child.add(egui::Label::new(RichText::new(&s.title).strong()).truncate());
    child.add(egui::Label::new(RichText::new(&s.subtitle).small().color(weak)).truncate());

    (resp.clicked(), resp.double_clicked())
}

/// Barras do seletor com respiro das bordas.
fn bar_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(16, 10))
}

pub fn picker(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    poll_starting(app, &ctx);
    if !matches!(app.screen, Screen::SharePicker) {
        return;
    }
    let busy = app.starting.is_some();
    let ready = app.encoder.lock().unwrap().is_some();
    let Some(p) = app.picker.as_mut() else { return };
    p.poll(&ctx);
    let mut go_back = false;
    let mut start = false;

    egui::Panel::top("picker_top").frame(bar_frame(ui)).show(ui, |ui| {
        ui.add_enabled_ui(!busy, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                if ui.button(format!("{I_BACK}  Voltar")).clicked() {
                    go_back = true;
                }
                ui.heading("Escolha o que compartilhar");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(I_REFRESH)
                        .on_hover_text("Atualizar lista")
                        .clicked()
                    {
                        let _ = p.refresh.send(());
                    }
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut p.filter)
                            .hint_text(format!("{I_SEARCH}  Filtrar janelas…"))
                            .desired_width(220.0),
                    );
                    if !p.filter_focused {
                        resp.request_focus();
                        p.filter_focused = true;
                    } else if !resp.has_focus() {
                        let mut typed = String::new();
                        ctx.input(|i| {
                            for ev in &i.events {
                                if let egui::Event::Text(t) = ev {
                                    for c in t.chars() {
                                        if !c.is_control() {
                                            typed.push(c);
                                        }
                                    }
                                }
                            }
                        });
                        if !typed.is_empty() {
                            p.filter.push_str(&typed);
                            resp.request_focus();
                        }
                    }
                });
            });
        });
    });

    if p.filter != p.applied {
        p.refresh_filter();
    }

    egui::Panel::bottom("picker_bottom").frame(bar_frame(ui)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.add_enabled_ui(!busy, |ui| {
                ui.label("Qualidade");
                egui::ComboBox::from_id_salt("preset")
                    .selected_text(p.preset.label())
                    .show_ui(ui, |ui| {
                        for pr in Preset::ALL {
                            ui.selectable_value(&mut p.preset, pr, pr.label())
                                .on_hover_text(preset_hint(pr));
                        }
                    })
                    .response
                    .on_hover_text(preset_hint(p.preset));
                let (icon, target) = if p.selected_is_window() {
                    (I_WINDOW, "áudio do app")
                } else {
                    (I_MONITOR, "áudio do sistema")
                };
                let label = format!("{icon}  Compartilhar {target}");
                ui.checkbox(&mut p.audio, label).on_hover_text(
                    "Janela: só o som desse aplicativo. Tela: todo o som do computador.",
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if busy {
                    ui.label("Iniciando…");
                    ui.add(egui::Spinner::new());
                } else {
                    let can = p.selected.is_some() && ready;
                    let r = ui
                        .add_enabled_ui(can, |ui| {
                            theme::primary_button(
                                ui,
                                &format!("{I_PLAY}  Iniciar compartilhamento"),
                            )
                        })
                        .inner;
                    if r.clicked() {
                        start = true;
                    }
                    if !can {
                        let hint = if ready {
                            "Selecione uma tela ou janela"
                        } else {
                            "Detectando encoder…"
                        };
                        r.on_hover_text(hint);
                    }
                    if !ready {
                        ui.label(RichText::new("Detectando encoder…").weak());
                        ui.add(egui::Spinner::new().size(14.0));
                    }
                }
                if let Some(e) = &app.error {
                    ui.label(RichText::new(e).color(ui.visuals().error_fg_color));
                }
            });
        });
    });

    let mut new_sel: Option<usize> = None;
    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_enabled_ui(!busy, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // respiro entre os cartões e das bordas
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 8))
                        .show(ui, |ui| {
                            let spacing = 16.0;
                            ui.spacing_mut().item_spacing = vec2(spacing, spacing);
                            let avail_w = ui.available_width();
                            let cols =
                                (((avail_w + spacing) / (256.0 + spacing)).floor() as usize).max(1);
                            let card_w =
                                ((avail_w - spacing * (cols - 1) as f32) / cols as f32).floor();

                            ui.heading(format!("{I_MONITOR}  Telas"));
                            ui.horizontal_wrapped(|ui| {
                                for &i in &p.monitors {
                                    let sel = p.selected.as_ref() == Some(&p.sources[i].id);
                                    let (click, double) = source_card(
                                        ui,
                                        &p.sources[i],
                                        p.textures.get(&p.sources[i].id),
                                        sel,
                                        card_w,
                                    );
                                    if click || double {
                                        new_sel = Some(i);
                                    }
                                    start |= double;
                                }
                            });
                            ui.add_space(8.0);
                            ui.heading(format!("{I_WINDOW}  Janelas ({})", p.windows.len()));
                            if p.windows.is_empty() {
                                let m = if p.applied.is_empty() {
                                    "Nenhuma janela disponível"
                                } else {
                                    "Nenhuma janela corresponde ao filtro"
                                };
                                ui.label(RichText::new(m).weak());
                            }
                            ui.horizontal_wrapped(|ui| {
                                for &i in &p.windows {
                                    let sel = p.selected.as_ref() == Some(&p.sources[i].id);
                                    let (click, double) = source_card(
                                        ui,
                                        &p.sources[i],
                                        p.textures.get(&p.sources[i].id),
                                        sel,
                                        card_w,
                                    );
                                    if click || double {
                                        new_sel = Some(i);
                                    }
                                    start |= double;
                                }
                            });
                        });
                });
        });
    });
    if let Some(i) = new_sel {
        p.selected = Some(p.sources[i].id.clone());
    }
    ctx.request_repaint_after(Duration::from_millis(500));

    if !busy {
        let can = p.selected.is_some() && ready;
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if !p.filter.is_empty() {
                p.filter.clear();
                p.refresh_filter();
            } else {
                go_back = true;
            }
        }
        if can && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            start = true;
        }
    }
    if go_back {
        app.picker = None;
        app.screen = Screen::Home;
    } else if start && !busy && ready {
        begin_start(app, &ctx);
    }
}

fn format_elapsed(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn invite_card(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    st: &ShareStatus,
    copied_at: &mut Option<Instant>,
) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading("Convide quem vai assistir");
        ui.label(
            RichText::new(
                "Envie este código. Ele só funciona enquanto este compartilhamento estiver ativo.",
            )
            .weak(),
        );
        match &st.ticket {
            None => {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label("Preparando conexão…");
                });
            }
            Some(t) => {
                ui.label(RichText::new("Código pronto").strong());
                let copied = copied_at
                    .as_ref()
                    .map_or(false, |t| t.elapsed() < Duration::from_secs(2));
                let btn_text = if copied {
                    format!("{I_CHECK}  Copiado")
                } else {
                    format!("{I_COPY}  Copiar código")
                };
                let pal = theme::palette(ui);
                let btn = egui::Button::new(RichText::new(btn_text).color(pal.on_accent))
                    .fill(pal.accent)
                    .min_size(vec2(ui.available_width(), 34.0));
                if ui.add(btn).clicked() {
                    ctx.copy_text(t.clone());
                    *copied_at = Some(Instant::now());
                }
                if let Some(inst) = copied_at {
                    if inst.elapsed() < Duration::from_secs(2) {
                        ctx.request_repaint_after(Duration::from_millis(100));
                    } else {
                        *copied_at = None;
                    }
                }
            }
        }
        if st.online_warning {
            ui.label(
                RichText::new(format!(
                    "{I_WARN}  Sem servidor de retransmissão: só conexões diretas devem funcionar."
                ))
                .color(ui.visuals().warn_fg_color),
            );
        }
    });
}

fn viewers_card(
    ui: &mut egui::Ui,
    st: &ShareStatus,
    kick: &mut Option<String>,
    kick_pending: &mut Option<(String, Instant)>,
) {
    if let Some((_, t)) = kick_pending {
        if t.elapsed() >= Duration::from_secs(3) {
            *kick_pending = None;
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading(format!(
            "{I_PEOPLE}  Espectadores ({}/{})",
            st.viewers.len(),
            MAX_VIEWERS
        ));
        if st.viewers.is_empty() {
            ui.label(
                RichText::new(
                    "Ninguém assistindo ainda. Envie o código de convite para até 4 pessoas.",
                )
                .weak(),
            );
        }
        let p = theme::palette(ui);
        for v in &st.viewers {
            ui.horizontal(|ui| {
                ui.label(format!("{I_PERSON}  {}", v.label()));
                ui.label(RichText::new(format!("#{}", v.id_short)).small().weak());
                let (text, color) = match v.path.as_str() {
                    "Direto" => ("Direto", p.success),
                    "Relay" => ("Via servidor", p.warn),
                    _ => ("Verificando", ui.visuals().weak_text_color()),
                };
                theme::badge(ui, text, color).on_hover_text(format!("{} ms", v.rtt_ms));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let is_pending = kick_pending.as_ref().map_or(false, |(id, t)| {
                        id == &v.id_short && t.elapsed() < Duration::from_secs(3)
                    });
                    if is_pending {
                        if theme::danger_button(ui, "Confirmar?").clicked() {
                            *kick = Some(v.id_short.clone());
                            *kick_pending = None;
                        }
                    } else if ui.small_button(format!("{I_REMOVE}  Remover")).clicked() {
                        *kick_pending = Some((v.id_short.clone(), Instant::now()));
                    }
                });
            });
        }
    });
}

fn preview_card(
    ui: &mut egui::Ui,
    st: &ShareStatus,
    source: &Option<SourceId>,
    preview: &Option<Preview>,
) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading("O que está sendo transmitido");
        let is_window = matches!(source, Some(SourceId::Window { .. }));
        match preview
            .as_ref()
            .and_then(|p| p.texture.as_ref().filter(|_| p.available))
        {
            Some(t) => {
                let sz = t.size_vec2();
                let w = ui.available_width();
                ui.add(
                    egui::Image::new((t.id(), vec2(w, w * sz.y / sz.x.max(1.0)))).corner_radius(6),
                );
            }
            None => {
                let msg = if is_window {
                    "Janela minimizada — quem assiste vê o último quadro"
                } else {
                    "Prévia indisponível"
                };
                ui.label(RichText::new(msg).weak());
            }
        }
        match &st.audio {
            AudioState::On => {
                let who = if is_window { "do app" } else { "do sistema" };
                ui.label(format!("{I_VOLUME}  Áudio: {who}"));
            }
            AudioState::Off => {
                ui.label("Áudio: desligado");
            }
            AudioState::Unavailable(m) => {
                ui.label(
                    RichText::new(format!("{I_WARN}  Áudio indisponível: {m}"))
                        .color(ui.visuals().warn_fg_color),
                );
            }
        }
    });
}

pub fn sharing(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let Some(share) = app.share.as_ref() else {
        return;
    };
    let st = share.status.lock().unwrap().clone();
    if let Some(msg) = st.ended.clone() {
        app.end(msg, false);
        return;
    }
    if let Some(pv) = app.share_preview.as_mut() {
        pv.poll(&ctx);
    }

    // entradas e saídas de espectadores (só remonta o mapa quando o conjunto muda)
    let same = st.viewers.len() == app.known_viewers.len()
        && st
            .viewers
            .iter()
            .all(|v| app.known_viewers.contains_key(&v.id_short));
    if !same {
        let current: HashMap<String, String> = st
            .viewers
            .iter()
            .map(|v| (v.id_short.clone(), v.label()))
            .collect();
        let mut joined = false;
        for (id, label) in &current {
            if !app.known_viewers.contains_key(id) {
                app.toasts.push(format!("{label} entrou"), ToastKind::Info);
                joined = true;
            }
        }
        if joined && !ctx.input(|i| i.viewport().focused.unwrap_or(true)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                egui::UserAttentionType::Informational,
            ));
        }
        for (id, label) in &app.known_viewers {
            if !current.contains_key(id) {
                app.toasts.push(format!("{label} saiu"), ToastKind::Info);
            }
        }
        app.known_viewers = current;
    }

    let mut stop = false;
    let mut kick: Option<String> = None;
    egui::Panel::top("share_top").show(ui, |ui| {
        ui.horizontal(|ui| {
            theme::live_pill(ui);
            ui.heading(&app.share_title);
            ui.label(
                RichText::new(format!(
                    "· {}",
                    format_elapsed(app.share_started.elapsed().as_secs())
                ))
                .weak(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::danger_button(ui, &format!("{I_STOP}  Parar")).clicked() {
                    stop = true;
                }
            });
        });
    });
    egui::Panel::bottom("share_status").show(ui, |ui| {
        ui.horizontal(|ui| {
            let status = format!(
                "{} · {} · {}×{} @ {} fps · {} fps · {} kbps",
                st.backend.label(),
                st.encoder,
                st.width,
                st.height,
                st.target_fps,
                st.fps,
                st.kbps
            );
            ui.label(RichText::new(status).weak());
        });
    });
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if ui.available_width() >= 900.0 {
                    ui.columns(2, |cols| {
                        preview_card(&mut cols[0], &st, &app.share_source, &app.share_preview);
                        invite_card(&mut cols[1], &ctx, &st, &mut app.copied_at);
                        viewers_card(&mut cols[1], &st, &mut kick, &mut app.kick_pending);
                    });
                } else {
                    preview_card(ui, &st, &app.share_source, &app.share_preview);
                    invite_card(ui, &ctx, &st, &mut app.copied_at);
                    viewers_card(ui, &st, &mut kick, &mut app.kick_pending);
                }
            });
    });
    if let Some(id) = kick {
        if let Some(share) = app.share.as_ref() {
            share.kick(&id);
        }
        let label = app.known_viewers.remove(&id).unwrap_or_else(|| id.clone());
        app.toasts
            .push(format!("{label} foi removido"), ToastKind::Warn);
    }
    if stop {
        app.end("Compartilhamento encerrado".into(), false);
        return;
    }
    ctx.request_repaint_after(Duration::from_millis(500));
}
