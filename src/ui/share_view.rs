use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::time::{Duration, Instant};

use egui::{
    Align, Align2, Color32, CursorIcon, Layout, RichText, Sense, Stroke, UiBuilder,
    pos2, vec2,
};

use super::theme::{
    self, I_BACK, I_CHECK, I_COPY, I_MONITOR, I_PEOPLE, I_PERSON, I_PLAY, I_REFRESH, I_REMOVE,
    I_MIC, I_MIC_OFF, I_SEARCH, I_STOP, I_VOLUME, I_WARN, I_WINDOW,
};
use super::toast::ToastKind;
use super::{App, Screen};
use crate::audio::capture::Mic;
use crate::capture::{self, SourceId, SourceInfo};
use livescreen_core::preset::{self, EncodeParams, Preset, viewers_for};
use crate::share::{AudioState, ShareSession, ShareStatus};
use crate::speedtest::Upload;

type Thumb = Option<(u32, u32, Vec<u8>)>;
struct Update {
    sources: Vec<SourceInfo>,
    thumbs: Vec<(SourceId, Thumb)>,
    mics: Vec<(String, String)>,
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
    pub mic: bool,
    pub mic_device: Option<String>,
    mics: Vec<(String, String)>,
    /// Popup de confirmação (passo 2) aberto.
    pub confirm: bool,
    opened: Instant,
}

impl Picker {
    pub fn new(
        ctx: egui::Context,
        preset: Preset,
        audio: bool,
        mic: bool,
        mic_device: Option<String>,
    ) -> Picker {
        let (tx, rx) = channel::<Update>();
        let (refresh, refresh_rx) = channel::<()>();
        std::thread::spawn(move || {
            loop {
                let sources = capture::enumerate();
                let thumbs = sources
                    .iter()
                    .map(|s| (s.id.clone(), capture::thumbnail(&s.id, 240)))
                    .collect();
                let mics = crate::audio::capture::microphones();
                if tx.send(Update { sources, thumbs, mics }).is_err() {
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
            mic,
            mic_device,
            mics: vec![],
            confirm: false,
            opened: Instant::now(),
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        let mut changed = false;
        while let Ok(u) = self.rx.try_recv() {
            self.sources = u.sources;
            self.mics = u.mics;
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
    let Some(encoders) = app.encoder.lock().unwrap().clone() else {
        return;
    };
    let (preset, audio) = (p.preset, p.audio);
    let mic = p
        .mic
        .then(|| p.mic_device.clone().map_or(Mic::Default, Mic::Device));
    app.prefs.preset = preset;
    app.prefs.share_audio = audio;
    app.prefs.share_mic = p.mic;
    app.prefs.mic_device = p.mic_device.clone();
    app.save_prefs();
    app.error = None;

    // exclui a janela da captura antes de a thread de início criar a captura
    app.sync_capture_exclusion(true);
    let (tx, rx) = channel();
    let (rt, overrides, ctx2) = (app.rt.clone(), app.effective_overrides(), ctx.clone());
    let room = app.prefs.room_limit;
    let (title, source) = (info.title.clone(), info.id.clone());
    app.background.push(std::thread::spawn(move || {
        let res = ShareSession::start(&rt, info, preset, audio, mic, encoders, overrides, room);
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

pub(super) fn preset_hint(pr: Preset) -> &'static str {
    match pr {
        Preset::P480p30 => "Muito leve; internet fraca",
        Preset::P720p30 => "Leve; redes lentas",
        Preset::P720p60 => "Leve e fluida; jogos",
        Preset::P1080p30 => "Nítida; textos e slides",
        Preset::P1080p60 => "Nítida e fluida",
        Preset::P1440p60 => "Muito nítida; telas QHD",
        Preset::P2160p60 => "4K; rede e GPU fortes",
        Preset::Native60 => "Resolução da fonte",
        Preset::NativeHigh => "Taxa do seu monitor; a mais fluida",
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
    let stroke = if selected || resp.has_focus() {
        Stroke::new(2.0, pal.accent)
    } else if hovered {
        Stroke::new(1.0, pal.border_strong)
    } else {
        Stroke::new(1.0, pal.border)
    };
    theme::paint_surface(ui, rect, stroke);

    let thumb = egui::Rect::from_min_size(rect.min + vec2(8.0, 8.0), vec2(thumb_w, thumb_h));
    ui.painter()
        .rect_filled(thumb, 4, ui.visuals().extreme_bg_color);
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
        pos2(rect.min.x + 8.0, thumb.max.y + 6.0),
        pos2(rect.max.x - 8.0, rect.max.y - 4.0),
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
    egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(24, 12))
}

pub fn picker(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    poll_starting(app, &ctx);
    if !matches!(app.screen, Screen::SharePicker) {
        return;
    }
    let busy = app.starting.is_some();
    let (ready, software) = match &*app.encoder.lock().unwrap() {
        Some(kinds) => (
            true,
            kinds
                .iter()
                .all(|k| matches!(k, crate::video::encoder::EncoderKind::Software)),
        ),
        None => (false, false),
    };
    let upload = *app.upload.lock().unwrap();
    let room = app.prefs.room_limit;
    let Some(p) = app.picker.as_mut() else { return };
    p.poll(&ctx);
    let mut go_back = false;
    let mut open_settings = false;
    let mut start = false;
    let mut remeasure = false;

    egui::Panel::top("picker_top")
        .frame(bar_frame(ui))
        .show(ui, |ui| {
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    // Altura do bloco de título antes de tudo: botões dos dois lados centram nela.
                    ui.set_min_height(44.0);
                    if ui.button(format!("{I_BACK}  Voltar")).clicked() {
                        go_back = true;
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new("Passo 1 de 2").small().weak());
                        theme::heading(ui, "Clique no que você quer compartilhar");
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        open_settings |= super::settings::icon_button(ui);
                        if theme::icon_button(ui, I_REFRESH, "Atualizar lista").clicked() {
                            let _ = p.refresh.send(());
                        }
                        let resp = ui.add(theme::field(
                            ui,
                            egui::TextEdit::singleline(&mut p.filter)
                                .hint_text(format!("{I_SEARCH}  Filtrar janelas…"))
                                .desired_width(220.0),
                        ));
                        if !p.filter_focused {
                            resp.request_focus();
                            p.filter_focused = true;
                        } else if !resp.has_focus() && !p.confirm {
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

    let modal_open = p.confirm;

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

                            theme::heading(ui, format!("{I_MONITOR}  Telas"));
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
                                }
                            });
                            ui.add_space(8.0);
                            theme::heading(ui, format!("{I_WINDOW}  Janelas ({})", p.windows.len()));
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
                                }
                            });
                        });
                });
        });
    });
    // o 2º clique de um duplo clique no "Compartilhar" da home cairia num cartão e já abriria
    // a confirmação de uma fonte que a pessoa nem viu
    if let Some(i) = new_sel
        && p.opened.elapsed() >= Duration::from_millis(400)
    {
        p.selected = Some(p.sources[i].id.clone());
        p.confirm = true;
        // o filtro de janelas, atrás do popup, não pode continuar recebendo as teclas
        ctx.memory_mut(|m| m.stop_text_input());
        app.error = None;
    }
    ctx.request_repaint_after(Duration::from_millis(500));

    if modal_open {
        let (s, close, again) = confirm_modal(
            p,
            &ctx,
            busy,
            ready,
            app.error.as_deref(),
            software,
            upload,
            room,
        );
        remeasure = again;
        start |= s;
        if close && !busy {
            p.confirm = false;
            app.error = None;
        }
    } else if !busy {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if !p.filter.is_empty() {
                p.filter.clear();
                p.refresh_filter();
            } else {
                go_back = true;
            }
        }
        if p.selected.is_some() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            p.confirm = true;
            ctx.memory_mut(|m| m.stop_text_input());
        }
    }
    if open_settings {
        app.settings_open = true;
    }
    if go_back {
        app.picker = None;
        app.error = None;
        app.screen = Screen::Home;
    } else if start && !busy && ready {
        begin_start(app, &ctx);
    }
    if remeasure {
        app.measure_upload(&ctx);
    }
}

/// Opções de qualidade desta fonte (tamanho e Hz do monitor) e o que cada uma custa.
struct Choices {
    list: Vec<(Preset, EncodeParams)>,
    src: (u32, u32),
    upload: Upload,
}

/// Pixels por segundo: mede o peso de uma opção para o encoder.
fn pixel_rate(p: &EncodeParams) -> u64 {
    p.width as u64 * p.height as u64 * p.fps as u64
}

impl Choices {
    fn new(sw: u32, sh: u32, hz: u32, upload: Upload) -> Self {
        Choices {
            list: preset::options(sw, sh, hz),
            src: (sw, sh),
            upload,
        }
    }

    fn params(&self, pr: Preset) -> Option<&EncodeParams> {
        self.list.iter().find(|(p, _)| *p == pr).map(|(_, e)| e)
    }

    fn label(&self, pr: Preset) -> String {
        match self.params(pr) {
            Some(e) => preset::label(pr, e, self.src.0, self.src.1),
            None => format!("{pr:?}"),
        }
    }

    /// Pessoas que o upload medido aguenta nesta opção; `None` sem medição.
    fn people(&self, pr: Preset) -> Option<usize> {
        let Upload::Done(bps) = self.upload else {
            return None;
        };
        self.params(pr).map(|e| viewers_for(bps, e.bitrate_bps))
    }

    /// A mais pesada que este PC e esta internet sustentam, até o peso de 1080p60: sem
    /// encoder na placa o processador só aguenta até 720p30, e a internet precisa dar conta
    /// da sala inteira.
    fn recommended(&self, software: bool, room: usize) -> Preset {
        let cap = if software {
            1280 * 720 * 30
        } else {
            1920 * 1080 * 60
        };
        self.list
            .iter()
            .filter(|(pr, e)| pixel_rate(e) <= cap && self.people(*pr).is_none_or(|n| n >= room))
            .max_by_key(|(_, e)| pixel_rate(e))
            .or(self.list.first())
            .map_or(Preset::P720p30, |(pr, _)| *pr)
    }

    /// Próxima opção mais leve para o encoder que a configuração em uso, para quem está
    /// perdendo quadros.
    fn lighter_than(&self, applied: &EncodeParams) -> Option<Preset> {
        let cur = pixel_rate(applied);
        self.list
            .iter()
            .filter(|(_, e)| pixel_rate(e) < cur)
            .max_by_key(|(_, e)| pixel_rate(e))
            .map(|(pr, _)| *pr)
    }
}

/// Teto do seletor de limite da sala; o que manda de verdade é o upload.
pub(super) const MAX_ROOM: usize = 99;

/// Linha "Sua internet…" e o aviso quando a sala passa do que o upload aguenta.
/// Devolve true se pediram para medir de novo (só oferecido fora da transmissão).
fn upload_line(
    ui: &mut egui::Ui,
    upload: Upload,
    people_now: Option<usize>,
    room: usize,
    allow_remeasure: bool,
) -> bool {
    let warn = ui.visuals().warn_fg_color;
    let mut again = false;
    match (upload, people_now) {
        (Upload::Measuring, _) => {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(12.0));
                ui.label(RichText::new("Medindo sua internet…").small().weak());
            });
        }
        (Upload::Done(bps), Some(n)) => {
            let mbps = bps as f64 / 1e6;
            ui.horizontal_wrapped(|ui| {
                let text = match n {
                    0 => format!(
                        "{I_WARN}  Sua internet envia ~{mbps:.0} Mbps: não aguenta nem 1 pessoa nesta qualidade."
                    ),
                    1 => format!("Sua internet envia ~{mbps:.0} Mbps: aguenta 1 pessoa nesta qualidade."),
                    _ => format!(
                        "Sua internet envia ~{mbps:.0} Mbps: aguenta até {n} pessoas nesta qualidade."
                    ),
                };
                let rt = RichText::new(text).small();
                ui.label(if n == 0 { rt.color(warn) } else { rt.weak() });
                if allow_remeasure && theme::small_button(ui, "Medir de novo").clicked() {
                    again = true;
                }
            });
            if n > 0 && room > n {
                ui.label(
                    RichText::new(format!(
                        "{I_WARN}  A sala aceita {room}: com mais de {n} pessoas assistindo, a imagem pode travar para todos. Reduza a qualidade ou o limite."
                    ))
                    .small()
                    .color(warn),
                );
            }
        }
        (Upload::Failed, _) => {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Não foi possível medir sua internet.")
                        .small()
                        .weak(),
                );
                if allow_remeasure && theme::small_button(ui, "Tentar de novo").clicked() {
                    again = true;
                }
            });
        }
        _ => {}
    }
    again
}

/// Aviso de PC sem fôlego, com atalho para o popup de qualidade.
fn overload_banner(ui: &mut egui::Ui, has_lighter: bool, quality_open: &mut bool) {
    let p = theme::palette(ui);
    egui::Frame::new()
        .fill(p.warn.linear_multiply(0.15))
        .stroke(Stroke::new(1.0, p.warn))
        .corner_radius(8)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let msg = if has_lighter {
                    "O computador não está dando conta desta qualidade e quem assiste vê travadas."
                } else {
                    "O computador não está dando conta nem da qualidade mais leve. Feche programas pesados."
                };
                ui.label(RichText::new(format!("{I_WARN}  {msg}")).color(p.warn));
                if has_lighter {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Reduzir qualidade").clicked() {
                            *quality_open = true;
                        }
                    });
                }
            });
        });
    ui.add_space(8.0);
}

/// Opções de qualidade como botões; devolve (escolhida no clique, alguma tem foco).
fn quality_options(
    ui: &mut egui::Ui,
    current: Option<Preset>,
    recommended: Option<Preset>,
    choices: &Choices,
) -> (Option<Preset>, bool) {
    let (mut picked, mut focus) = (None, false);
    egui::ScrollArea::vertical()
        .max_height(300.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let w = ui.available_width();
            for &(pr, _) in &choices.list {
                let mut label = choices.label(pr);
                match choices.people(pr) {
                    Some(0) => label.push_str("  · ninguém"),
                    Some(1) => label.push_str("  · até 1 pessoa"),
                    Some(n) => label.push_str(&format!("  · até {n} pessoas")),
                    None => {}
                }
                if Some(pr) == recommended {
                    label.push_str("  · recomendado");
                }
                let b =
                    egui::Button::selectable(Some(pr) == current, RichText::new(label).strong())
                        .right_text(RichText::new(preset_hint(pr)).small().weak())
                        .min_size(vec2(w, 34.0));
                let r = ui.add(b);
                focus |= r.has_focus();
                if r.clicked() {
                    picked = Some(pr);
                }
            }
        });
    (picked, focus)
}

/// Passo 2: confirma fonte, qualidade e áudio. Devolve (iniciar, fechar, medir de novo).
fn confirm_modal(
    p: &mut Picker,
    ctx: &egui::Context,
    busy: bool,
    ready: bool,
    error: Option<&str>,
    software: bool,
    upload: Upload,
    room: usize,
) -> (bool, bool, bool) {
    let Some(info) = p
        .selected
        .as_ref()
        .and_then(|id| p.sources.iter().find(|s| &s.id == id))
        .cloned()
    else {
        return (false, true, false);
    };
    let is_window = matches!(info.id, SourceId::Window { .. });
    let (mut start, mut close, mut focus_in, mut remeasure) = (false, false, false, false);
    let (sw, sh) = capture::source_size(&info.id).unwrap_or((1920, 1080));
    let hz = capture::refresh_rate(&info.id);
    let choices = Choices::new(sw, sh, hz, upload);
    // preferência salva em outro monitor: a opção daqui com a mesma imagem
    p.preset = preset::normalize(p.preset, sw, sh, hz);
    let rec = ready.then(|| choices.recommended(software, room));
    let modal = theme::modal(ctx, "confirm_share").show(ctx, |ui| {
        ui.set_width(640.0);
        ui.label(RichText::new("Passo 2 de 2").small().weak());
        theme::heading(ui, "Confirmar transmissão");
        ui.add_space(8.0);
        ui.add_enabled_ui(!busy, |ui| {
            ui.horizontal(|ui| {
                let (icon, kind) = if is_window {
                    (I_WINDOW, "Janela")
                } else {
                    (I_MONITOR, "Tela")
                };
                match p.textures.get(&info.id).filter(|_| !info.minimized) {
                    Some(t) => {
                        let sz = t.size_vec2();
                        let h = 72.0;
                        ui.add(
                            egui::Image::new((t.id(), vec2(h * sz.x / sz.y.max(1.0), h)))
                                .corner_radius(4),
                        );
                    }
                    None => {
                        ui.label(RichText::new(icon).size(40.0).weak());
                    }
                }
                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("{icon}  {kind}")).small().weak());
                    ui.add(egui::Label::new(RichText::new(&info.title).strong()).truncate());
                    ui.add(egui::Label::new(RichText::new(&info.subtitle).weak()).truncate());
                });
            });
            ui.add_space(12.0);
            ui.label(RichText::new("Qualidade").strong());
            if software {
                ui.label(
                    RichText::new(format!(
                        "{I_WARN}  Sem encoder de vídeo na placa: o processador faz o trabalho, prefira a qualidade leve."
                    ))
                    .small()
                    .color(ui.visuals().warn_fg_color),
                );
            }
            let (picked, f) = quality_options(ui, Some(p.preset), rec, &choices);
            focus_in |= f;
            if let Some(pr) = picked {
                p.preset = pr;
            }
            let people_now = choices.people(p.preset);
            remeasure = upload_line(ui, upload, people_now, room, true);
            ui.add_space(8.0);
            let target = if is_window {
                "o áudio deste app"
            } else {
                "todo o áudio do computador"
            };
            focus_in |= theme::checkbox(ui, &mut p.audio, format!("{I_VOLUME}  Transmitir {target}"))
                .has_focus();
            focus_in |= theme::checkbox(ui, &mut p.mic, format!("{I_MIC}  Transmitir microfone"))
                .has_focus();
            if p.mic {
                let name = p
                    .mic_device
                    .as_ref()
                    .and_then(|id| p.mics.iter().find(|(i, _)| i == id))
                    .map_or("Padrão do Windows", |(_, n)| n.as_str())
                    .to_owned();
                // Recuado sob o checkbox do microfone, alinhado ao rótulo.
                let r = ui.indent("mic_device", |ui| {
                    egui::ComboBox::from_id_salt("mic_device")
                        .selected_text(name)
                        .width(320.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut p.mic_device, None, "Padrão do Windows");
                            for (id, n) in &p.mics {
                                ui.selectable_value(&mut p.mic_device, Some(id.clone()), n);
                            }
                        })
                });
                focus_in |= r.inner.response.has_focus();
            }
        });
        ui.add_space(12.0);
        if let Some(e) = error {
            ui.label(RichText::new(e).color(ui.visuals().error_fg_color));
            ui.add_space(4.0);
        }
        ui.horizontal(|ui| {
            if busy {
                ui.add(egui::Spinner::new());
                ui.label("Iniciando…");
                return;
            }
            let back = ui.button(format!("{I_BACK}  Trocar"));
            focus_in |= back.has_focus();
            if back.clicked() {
                close = true;
            }
            let r = ui
                .add_enabled_ui(ready, |ui| {
                    theme::primary_button(ui, &format!("{I_PLAY}  Iniciar transmissão"))
                })
                .inner;
            focus_in |= r.has_focus();
            if r.clicked() {
                start = true;
            }
            if !ready {
                ui.add(egui::Spinner::new().size(14.0));
                ui.label(RichText::new("Detectando encoder…").weak());
            }
        });
    });
    // Enter é atalho de "Iniciar" só sem foco em um controle do popup; com foco, o Enter
    // ativa o próprio controle (opção, áudio, Trocar) e não pode iniciar por tabela.
    if !busy && ready && !focus_in && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        start = true;
    }
    let close = close || modal.should_close();
    (start && !close, close, remeasure)
}

fn format_elapsed(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn invite_card(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    st: &ShareStatus,
    copied_at: &mut Option<Instant>,
    new_invite: &mut bool,
) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::heading(ui, "Convide quem vai assistir");
        ui.label(
            RichText::new(
                "Cole (Ctrl+V) para quem vai assistir. O código só funciona enquanto esta transmissão estiver ativa.",
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
                ui.label(RichText::new("Código pronto e já copiado").strong());
                let copied = copied_at
                    .as_ref()
                    .map_or(false, |t| t.elapsed() < Duration::from_secs(2));
                let btn_text = if copied {
                    format!("{I_CHECK}  Copiado")
                } else {
                    format!("{I_COPY}  Copiar código")
                };
                if theme::primary_button_full(ui, &btn_text).clicked() {
                    ctx.copy_text(t.clone());
                    *copied_at = Some(Instant::now());
                }
                if ui
                    .add_sized([ui.available_width(), 0.0], egui::Button::new("Novo convite"))
                    .on_hover_text("O código anterior deixa de funcionar; quem já está assistindo continua")
                    .clicked()
                {
                    *new_invite = true;
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
    room: &mut usize,
    upload: Upload,
    people_now: Option<usize>,
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
        theme::heading(ui, format!(
            "{I_PEOPLE}  Espectadores ({} de {})",
            st.viewers.len(),
            room
        ));
        ui.horizontal(|ui| {
            ui.label("Limite da sala");
            if ui
                .add_enabled(*room > 1, egui::Button::new("−").min_size(vec2(36.0, 36.0)))
                .on_hover_text("Quem já está assistindo continua")
                .clicked()
            {
                *room -= 1;
            }
            ui.label(RichText::new(room.to_string()).strong());
            if ui
                .add_enabled(*room < MAX_ROOM, egui::Button::new("+").min_size(vec2(36.0, 36.0)))
                .clicked()
            {
                *room += 1;
            }
        });
        upload_line(ui, upload, people_now, *room, false);
        if st.viewers.is_empty() {
            let who = if *room == 1 {
                "1 pessoa".to_string()
            } else {
                format!("até {room} pessoas")
            };
            ui.label(
                RichText::new(format!(
                    "Ninguém assistindo ainda. Envie o código de convite para {who}."
                ))
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
                    } else if theme::small_button(ui, format!("{I_REMOVE}  Remover")).clicked() {
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
        theme::heading(ui, "O que está sendo transmitido");
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
        match &st.mic {
            AudioState::On if st.mic_muted => {
                ui.label(RichText::new(format!("{I_MIC_OFF}  Microfone: mudo")).weak());
            }
            AudioState::On => {
                ui.label(format!("{I_MIC}  Microfone: ligado"));
            }
            AudioState::Unavailable(m) => {
                ui.label(
                    RichText::new(format!("{I_WARN}  Microfone indisponível: {m}"))
                        .color(ui.visuals().warn_fg_color),
                );
            }
            AudioState::Off => {}
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

    // código copiado sozinho assim que fica pronto: falta só colar para quem vai assistir
    if let Some(t) = &st.ticket
        && !app.code_auto_copied
    {
        ctx.copy_text(t.clone());
        app.copied_at = Some(Instant::now());
        app.code_auto_copied = true;
        app.toasts.push(
            "Código de convite copiado: é só colar para quem vai assistir",
            ToastKind::Info,
        );
    }

    let (sw, sh, hz) = share.source_shape();
    let upload = *app.upload.lock().unwrap();
    // opções para o tamanho atual da fonte (o que a pessoa pode escolher agora)
    let choices = Choices::new(sw, sh, hz, upload);
    // transmissão atual: o que o encoder usa de fato; a janela pode ter mudado de tamanho e
    // a mesma imagem pode ter outro bitrate ("Nativa" pequena tem piso de 6 Mbps)
    let (applied_preset, applied, applied_src) = share.current();
    let cur_label = preset::label(applied_preset, &applied, applied_src.0, applied_src.1);
    let people_now = match upload {
        Upload::Done(bps) => Some(viewers_for(bps, applied.bitrate_bps)),
        _ => None,
    };
    // marcada na lista só se for exatamente a configuração em uso
    let cur_choice = choices
        .list
        .iter()
        .find(|(_, e)| *e == applied)
        .map(|(pr, _)| *pr);
    let mut room = app.prefs.room_limit;
    let mut picked: Option<Preset> = None;
    let mut stop = false;
    let mut kick: Option<String> = None;
    let mut toggle_mic = false;
    let mut new_invite = false;
    egui::Panel::top("share_top").show(ui, |ui| {
        ui.horizontal(|ui| {
            theme::live_pill(ui);
            theme::heading(ui, &app.share_title);
            ui.label(
                RichText::new(format!(
                    "· {}",
                    format_elapsed(app.share_started.elapsed().as_secs())
                ))
                .weak(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::danger_button(ui, &format!("{I_STOP}  Parar")).clicked() {
                    // sem ninguém assistindo não há o que perder: para direto
                    if st.viewers.is_empty() {
                        stop = true;
                    } else {
                        app.confirm_stop = true;
                    }
                }
                if matches!(st.mic, AudioState::On) {
                    let label = if st.mic_muted {
                        format!("{I_MIC_OFF}  Ativar microfone")
                    } else {
                        format!("{I_MIC}  Silenciar microfone")
                    };
                    if ui.button(label).clicked() {
                        toggle_mic = true;
                    }
                }
                if ui
                    .button(format!("{I_MONITOR}  Qualidade: {cur_label}"))
                    .on_hover_text("Trocar a qualidade sem parar a transmissão")
                    .clicked()
                {
                    app.quality_open = true;
                }
                super::settings::button(app, ui);
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
                if st.overloaded {
                    overload_banner(
                        ui,
                        choices.lighter_than(&applied).is_some(),
                        &mut app.quality_open,
                    );
                }
                if ui.available_width() >= 900.0 {
                    ui.columns(2, |cols| {
                        preview_card(&mut cols[0], &st, &app.share_source, &app.share_preview);
                        invite_card(&mut cols[1], &ctx, &st, &mut app.copied_at, &mut new_invite);
                        viewers_card(
                            &mut cols[1],
                            &st,
                            &mut kick,
                            &mut app.kick_pending,
                            &mut room,
                            upload,
                            people_now,
                        );
                    });
                } else {
                    preview_card(ui, &st, &app.share_source, &app.share_preview);
                    invite_card(ui, &ctx, &st, &mut app.copied_at, &mut new_invite);
                    viewers_card(
                        ui,
                        &st,
                        &mut kick,
                        &mut app.kick_pending,
                        &mut room,
                        upload,
                        people_now,
                    );
                }
            });
    });

    if toggle_mic && let Some(s) = &app.share {
        s.set_mic_muted(!st.mic_muted);
    }
    // popup de qualidade: a escolha aplica na hora e fecha (abrir + escolher = 2 cliques)
    if app.quality_open {
        let mut close = false;
        let modal = theme::modal(&ctx, "live_quality").show(&ctx, |ui| {
            ui.set_width(640.0);
            theme::heading(ui, "Qualidade da transmissão");
            ui.label(
                RichText::new("Vale na hora; quem está assistindo vê uma pausa curta.").weak(),
            );
            ui.add_space(8.0);
            let rec = if st.overloaded {
                choices.lighter_than(&applied)
            } else {
                Some(choices.recommended(st.encoder.starts_with("software"), room))
            };
            if let (Some(pr), _) = quality_options(ui, cur_choice, rec, &choices) {
                picked = Some(pr);
                close = true;
            }
            ui.add_space(8.0);
            if ui.button("Cancelar").clicked() {
                close = true;
            }
        });
        if close || modal.should_close() {
            app.quality_open = false;
        }
    }
    if room != app.prefs.room_limit {
        app.prefs.room_limit = room;
        if let Some(s) = app.share.as_ref() {
            s.set_max_viewers(room);
        }
        app.save_prefs();
    }
    if app.confirm_stop {
        let (mut cancel, mut confirm) = (false, false);
        let n = st.viewers.len();
        let modal = theme::modal(&ctx, "confirm_stop").show(&ctx, |ui| {
            ui.set_width(360.0);
            theme::heading(ui, "Encerrar a transmissão?");
            let who = if n == 1 {
                "A pessoa que está assistindo será desconectada.".to_string()
            } else {
                format!("As {n} pessoas que estão assistindo serão desconectadas.")
            };
            ui.label(who);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Continuar transmitindo").clicked() {
                    cancel = true;
                }
                if theme::danger_button(ui, &format!("{I_STOP}  Encerrar")).clicked() {
                    confirm = true;
                }
            });
        });
        if confirm {
            stop = true;
        } else if cancel || modal.should_close() {
            app.confirm_stop = false;
        }
    }

    if let Some(new_preset) = picked
        && picked != cur_choice
        && let Some(share) = app.share.as_ref()
    {
        let new_label = choices.label(new_preset);
        match share.set_preset(new_preset) {
            Ok(()) => app.toasts.push(
                format!("Qualidade alterada para {new_label}"),
                ToastKind::Info,
            ),
            Err(e) => app.toasts.push(
                format!("Não foi possível trocar a qualidade: {e}"),
                ToastKind::Warn,
            ),
        }
    }
    if let Some(share) = app.share.as_ref()
        && let Some((who, preset, res)) = share.apply_quality_request()
    {
        let who = if who.is_empty() { "Alguém".to_string() } else { who };
        let name = preset.name();
        match res {
            Ok(()) => app.toasts.push(
                format!("{who} pediu {name}: qualidade alterada para todos"),
                ToastKind::Info,
            ),
            Err(e) => app.toasts.push(
                format!("{who} pediu {name}, mas não foi possível trocar: {e}"),
                ToastKind::Warn,
            ),
        }
    }
    if let Some(share) = app.share.as_ref()
        && let Some(res) = share.apply_codec()
    {
        match res {
            Ok(c) => app.toasts.push(
                format!("Codec da sala: {}", c.label()),
                ToastKind::Info,
            ),
            Err(e) => app.toasts.push(
                format!("Não foi possível trocar o codec: {e}"),
                ToastKind::Warn,
            ),
        }
    }
    if let Some(id) = kick {
        if let Some(share) = app.share.as_ref() {
            share.kick(&id);
        }
        let label = app.known_viewers.remove(&id).unwrap_or_else(|| id.clone());
        app.toasts.push(
            format!("{label} foi removido. Para impedir a volta, gere um novo convite."),
            ToastKind::Warn,
        );
    }
    if new_invite {
        if let Some(share) = app.share.as_ref() {
            share.new_invite();
            let ticket = share.status.lock().unwrap().ticket.clone();
            if let Some(t) = ticket {
                ctx.copy_text(t);
                let now = Instant::now();
                app.copied_at = Some(now);
                app.toasts.push(
                    "Novo código copiado; o anterior não funciona mais",
                    ToastKind::Info,
                );
            }
        }
    }
    if stop {
        app.end("Compartilhamento encerrado".into(), false);
        return;
    }
    ctx.request_repaint_after(Duration::from_millis(500));
}
