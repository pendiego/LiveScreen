use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{
    Align, Align2, Color32, CursorIcon, Layout, RichText, Sense, Stroke, UiBuilder,
    pos2, vec2,
};

use super::theme::{
    self, I_ADD_FRIEND, I_BACK, I_CHECK, I_CIRCLE_CHECK, I_COPY, I_HANGUP, I_MIC, I_MIC_OFF,
    I_MONITOR, I_PEOPLE, I_PERSON, I_PLAY, I_QUALITY, I_REFRESH, I_REMOVE, I_SEARCH, I_VOLUME,
    I_WARN, I_WINDOW,
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
    let prober = app.take_prober();
    let pending = std::mem::take(&mut app.background);
    let identity = app.identity.clone();
    let host_name = livescreen_core::protocol::sanitize(&app.prefs.name, 32);
    let friends: std::collections::HashSet<iroh::EndpointId> =
        app.friend_ids().into_iter().collect();
    app.background.push(std::thread::spawn(move || {
        if let Some(p) = prober {
            p.stop();
        }
        for h in pending {
            let _ = h.join();
        }
        let res = ShareSession::start(
            &rt, info, preset, audio, mic, encoders, overrides, room, identity, host_name,
            friends,
        );
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
            app.share = Some(Arc::new(s));
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
        Preset::P240p30 => "Mínima; PC ou internet muito fracos",
        Preset::P360p30 => "Bem leve; PC ou internet fracos",
        Preset::P480p15 => "Muito leve; telas paradas e slides",
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
    let pal = theme::P;
    let thumb_w = (card_w - 32.0).max(1.0);
    let thumb_h = (thumb_w * 9.0 / 16.0).round();
    let card_h = 16.0 + thumb_h + 12.0 + 38.0 + 16.0;
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

    let thumb = egui::Rect::from_min_size(rect.min + vec2(16.0, 16.0), vec2(thumb_w, thumb_h));
    ui.painter().rect_filled(thumb, theme::R_MD, pal.bg);
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
                pal.text_muted,
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
                pal.text_muted,
            );
        }
    }
    let title_right = if selected {
        ui.painter().text(
            pos2(rect.max.x - 16.0, thumb.max.y + 12.0 + 9.0),
            Align2::RIGHT_CENTER,
            I_CIRCLE_CHECK,
            egui::FontId::proportional(20.0),
            pal.accent,
        );
        rect.max.x - 16.0 - 24.0
    } else {
        rect.max.x - 16.0
    };
    let text_rect = egui::Rect::from_min_max(
        pos2(rect.min.x + 16.0, thumb.max.y + 12.0),
        pos2(title_right, thumb.max.y + 12.0 + 38.0),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 2.0;
    child.add(
        egui::Label::new(
            RichText::new(&s.title)
                .family(egui::FontFamily::Name("semibold".into()))
                .size(14.0)
                .color(pal.text),
        )
        .truncate(),
    );
    child.add(
        egui::Label::new(
            RichText::new(&s.subtitle)
                .small()
                .color(pal.text_muted),
        )
        .truncate(),
    );

    (resp.clicked(), resp.double_clicked())
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
        .frame(egui::Frame::new().fill(theme::P.surface).inner_margin(egui::Margin::symmetric(24, 16)))
        .show(ui, |ui| {
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    // Altura do bloco de título antes de tudo: botões dos dois lados centram nela.
                    ui.set_min_height(44.0);
                    if theme::button(ui, theme::Kind::Ghost, format!("{I_BACK}  Voltar")).clicked() {
                        go_back = true;
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new("Passo 1 de 2").small().color(theme::P.text_muted));
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
                                .hint_text("Filtrar janelas…")
                                .prefix(RichText::new(I_SEARCH).color(theme::P.text_muted))
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
                        .inner_margin(egui::Margin::symmetric(24, 16))
                        .show(ui, |ui| {
                            let spacing = 16.0;
                            ui.spacing_mut().item_spacing = vec2(spacing, spacing);
                            let avail_w = ui.available_width();
                            let cols =
                                (((avail_w + spacing) / (300.0 + spacing)).floor() as usize).max(1);
                            let card_w =
                                ((avail_w - spacing * (cols - 1) as f32) / cols as f32).floor();

                            theme::section_label(ui, format!("{I_MONITOR}  Telas"));
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
                            theme::section_label(
                                ui,
                                format!("{I_WINDOW}  Janelas ({})", p.windows.len()),
                            );
                            if p.windows.is_empty() {
                                let m = if p.applied.is_empty() {
                                    "Nenhuma janela disponível"
                                } else {
                                    "Nenhuma janela corresponde ao filtro"
                                };
                                ui.label(RichText::new(m).color(theme::P.text_muted));
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
    let warn = theme::P.warn;
    let mut again = false;
    match (upload, people_now) {
        (Upload::Measuring, _) => {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(12.0));
                ui.label(RichText::new("Medindo sua internet…").small().color(theme::P.text_muted));
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
                ui.label(if n == 0 { rt.color(warn) } else { rt.color(theme::P.text_muted) });
                if allow_remeasure && theme::small_button(ui, theme::Kind::Ghost, "Medir de novo").clicked() {
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
                        .color(theme::P.text_muted),
                );
                if allow_remeasure && theme::small_button(ui, theme::Kind::Ghost, "Tentar de novo").clicked() {
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
    let p = theme::P;
    egui::Frame::new()
        .fill(theme::soft(p.warn))
        .stroke(Stroke::new(1.0, p.warn))
        .corner_radius(theme::R_MD)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let msg = if has_lighter {
                    "O computador não está dando conta desta qualidade e quem assiste vê travadas."
                } else {
                    "O computador não está dando conta nem da qualidade mais leve. Feche programas pesados."
                };
                ui.label(RichText::new(I_WARN).color(p.warn));
                ui.label(RichText::new(msg).color(p.text));
                if has_lighter {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::small_button(ui, theme::Kind::Secondary, "Reduzir qualidade").clicked() {
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
        .max_height((ui.ctx().content_rect().height() - 480.0).clamp(120.0, 300.0))
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            for &(pr, _) in &choices.list {
                let detail = match choices.people(pr) {
                    Some(0) => "ninguém".to_string(),
                    Some(1) => "até 1 pessoa".to_string(),
                    Some(n) => format!("até {n} pessoas"),
                    None => String::new(),
                };
                let r = theme::option_row(
                    ui,
                    Some(pr) == current,
                    &choices.label(pr),
                    &detail,
                    Some(pr) == recommended,
                )
                .on_hover_text(preset_hint(pr));
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
        ui.label(RichText::new("Passo 2 de 2").small().color(theme::P.text_muted));
        theme::heading(ui, "Confirmar transmissão");
        ui.add_space(8.0);
        ui.add_enabled_ui(!busy, |ui| {
            egui::Frame::new()
                .fill(theme::P.surface)
                .stroke(Stroke::new(1.0, theme::P.border))
                .corner_radius(theme::R_MD)
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
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
                                        .corner_radius(theme::R_SM),
                                );
                            }
                            None => {
                                ui.label(RichText::new(icon).size(40.0).color(theme::P.text_muted));
                            }
                        }
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(format!("{icon}  {kind}"))
                                    .small()
                                    .color(theme::P.text_muted),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&info.title)
                                        .family(egui::FontFamily::Name("semibold".into()))
                                        .size(14.0)
                                        .color(theme::P.text),
                                )
                                .truncate(),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&info.subtitle)
                                        .small()
                                        .color(theme::P.text_muted),
                                )
                                .truncate(),
                            );
                        });
                    });
                });
            ui.add_space(12.0);
            theme::section_label(ui, "Qualidade");
            if software {
                ui.label(
                    RichText::new(format!(
                        "{I_WARN}  Sem encoder de vídeo na placa: o processador faz o trabalho, prefira a qualidade leve."
                    ))
                    .small()
                    .color(theme::P.warn),
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
                        .icon(theme::chevron_icon)
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
            ui.label(RichText::new(e).color(theme::P.danger));
            ui.add_space(4.0);
        }
        ui.horizontal(|ui| {
            if busy {
                ui.add(egui::Spinner::new());
                ui.label("Iniciando…");
                return;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let r = ui
                    .add_enabled_ui(ready, |ui| {
                        theme::button(ui, theme::Kind::Primary, &format!("{I_PLAY}  Iniciar transmissão"))
                    })
                    .inner;
                focus_in |= r.has_focus();
                if r.clicked() {
                    start = true;
                }
                let back = theme::button(ui, theme::Kind::Secondary, format!("{I_BACK}  Trocar"));
                focus_in |= back.has_focus();
                if back.clicked() {
                    close = true;
                }
                if !ready {
                    ui.add(egui::Spinner::new().size(14.0));
                    ui.label(RichText::new("Detectando encoder…").small().color(theme::P.text_muted));
                }
            });
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
            .small()
            .color(theme::P.text_muted),
        );
        match &st.ticket {
            None => {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label("Preparando conexão…");
                });
            }
            Some(t) => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(I_CIRCLE_CHECK).small().color(theme::P.success));
                    ui.label(RichText::new("Código pronto e já copiado").small().color(theme::P.text));
                });
                let copied = copied_at
                    .as_ref()
                    .map_or(false, |t| t.elapsed() < Duration::from_secs(2));
                let btn_text = if copied {
                    format!("{I_CHECK}  Copiado")
                } else {
                    format!("{I_COPY}  Copiar código")
                };
                ui.scope(|ui| {
                    ui.spacing_mut().interact_size.y = 36.0;
                    if theme::button_full(ui, theme::Kind::Primary, &btn_text).clicked() {
                        ctx.copy_text(t.clone());
                        *copied_at = Some(Instant::now());
                    }
                    if theme::button_full(ui, theme::Kind::Secondary, "Novo convite")
                        .on_hover_text("O código anterior deixa de funcionar; quem já está assistindo continua")
                        .clicked()
                    {
                        *new_invite = true;
                    }
                });
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
                .color(theme::P.warn),
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
    add_friend: &mut Option<(String, iroh::EndpointId)>,
    can_add_friend: bool,
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
            ui.label(RichText::new("Limite da sala").small().color(theme::P.text_muted));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                theme::stepper(ui, room, 1, MAX_ROOM, "Quem já está assistindo continua");
            });
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
                .small()
                .color(theme::P.text_muted),
            );
        }
        let p = theme::P;
        for v in &st.viewers {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{I_PERSON}  {}", v.label()))
                        .family(egui::FontFamily::Name("semibold".into()))
                        .size(14.0)
                        .color(p.text),
                );
                ui.label(
                    RichText::new(format!("#{}", v.id_short))
                        .family(egui::FontFamily::Monospace)
                        .size(12.0)
                        .color(p.text_muted),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let is_pending = kick_pending.as_ref().map_or(false, |(id, t)| {
                        id == &v.id_short && t.elapsed() < Duration::from_secs(3)
                    });
                    if is_pending {
                        if theme::small_button(ui, theme::Kind::Danger, "Confirmar?").clicked() {
                            *kick = Some(v.id_short.clone());
                            *kick_pending = None;
                        }
                    } else if theme::small_button(
                        ui,
                        theme::Kind::Ghost,
                        RichText::new(format!("{I_REMOVE}  Remover"))
                            .small()
                            .color(p.text_muted),
                    )
                    .clicked()
                    {
                        *kick_pending = Some((v.id_short.clone(), Instant::now()));
                    }
                });
            });
            // segunda linha: com ids longos, nome + id + selos + botões não cabem numa só
            ui.horizontal(|ui| {
                if v.friend {
                    theme::badge(ui, "Amigo", p.accent);
                } else if can_add_friend
                    && theme::small_button(ui, theme::Kind::Secondary, format!("{I_ADD_FRIEND}  Amigo"))
                        .on_hover_text("Adicionar aos amigos")
                        .clicked()
                {
                    *add_friend = Some((v.name.clone(), v.id));
                }
                let (text, color) = match v.path.as_str() {
                    "Direto" => ("Direto", p.success),
                    "Relay" => ("Via servidor", p.warn),
                    _ => ("Verificando", p.text_muted),
                };
                let tip = if v.path == "Relay" {
                    format!("{} ms\nServidor público compartilhado, com limite de banda: se travar, baixe a qualidade.", v.rtt_ms)
                } else {
                    format!("{} ms", v.rtt_ms)
                };
                theme::badge(ui, text, color).on_hover_text(tip);
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
        ui.label(RichText::new("O que está sendo transmitido").small().color(theme::P.text_muted));
        let is_window = matches!(source, Some(SourceId::Window { .. }));
        match preview
            .as_ref()
            .and_then(|p| p.texture.as_ref().filter(|_| p.available))
        {
            Some(t) => {
                let sz = t.size_vec2();
                let w = ui.available_width();
                ui.add(
                    egui::Image::new((t.id(), vec2(w, w * sz.y / sz.x.max(1.0)))).corner_radius(theme::R_SM),
                );
            }
            None => {
                let msg = if is_window {
                    "Janela minimizada — quem assiste vê o último quadro"
                } else {
                    "Prévia indisponível"
                };
                ui.label(RichText::new(msg).small().color(theme::P.text_muted));
            }
        }
        match &st.audio {
            AudioState::On => {
                let who = if is_window { "do app" } else { "do sistema" };
                ui.label(RichText::new(format!("{I_VOLUME}  Áudio: {who}")).small().color(theme::P.text));
            }
            AudioState::Off => {
                ui.label(RichText::new("Áudio: desligado").small().color(theme::P.text_muted));
            }
            AudioState::Unavailable(m) => {
                ui.label(
                    RichText::new(format!("{I_WARN}  Áudio indisponível: {m}"))
                        .small()
                        .color(theme::P.warn),
                );
            }
        }
        match &st.mic {
            AudioState::On if st.mic_muted => {
                ui.label(RichText::new(format!("{I_MIC_OFF}  Microfone: mudo")).small().color(theme::P.text_muted));
            }
            AudioState::On => {
                ui.label(RichText::new(format!("{I_MIC}  Microfone: ligado")).small().color(theme::P.text));
            }
            AudioState::Unavailable(m) => {
                ui.label(
                    RichText::new(format!("{I_WARN}  Microfone indisponível: {m}"))
                        .small()
                        .color(theme::P.warn),
                );
            }
            AudioState::Off => {}
        }
    });
}

/// O aviso de sobrecarga só some com 5 s limpos: sem esta espera, uma só sobrecarga baixaria várias faixas seguidas.
const AUTO_QUALITY_COOLDOWN: Duration = Duration::from_secs(10);

/// Troca de encoder fora da UI: subir o encoder novo pode levar até 15 s.
fn spawn_share_job(
    app: &mut App,
    ctx: &egui::Context,
    share: Arc<ShareSession>,
    job: impl FnOnce(&ShareSession) -> Option<(String, ToastKind)> + Send + 'static,
) {
    let (tx, rx) = channel();
    let ctx = ctx.clone();
    app.background.push(std::thread::spawn(move || {
        let _ = tx.send(job(&share));
        ctx.request_repaint();
    }));
    app.share_job = Some(rx);
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
    let mut add_friend: Option<(String, iroh::EndpointId)> = None;
    let busy = app.share_job.is_some();
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
    egui::Panel::top("share_top")
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(theme::P.bg)
                .inner_margin(egui::Margin::symmetric(32, 20)),
        )
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                theme::live_pill(ui);
                theme::heading(ui, &app.share_title).on_hover_text(&status);
                ui.label(
                    RichText::new(format!(
                        "· {}",
                        format_elapsed(app.share_started.elapsed().as_secs())
                    ))
                    .color(theme::P.text_muted),
                )
                .on_hover_text(&status);
            });
        });
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(32, 0))
                    .show(ui, |ui| {
                        if st.overloaded {
                            overload_banner(
                                ui,
                                choices.lighter_than(&applied).is_some(),
                                &mut app.quality_open,
                            );
                        }
                        if ui.available_width() >= 900.0 {
                            let total_w = ui.available_width();
                            let right_w = 360.0;
                            let gap = 24.0;
                            let left_w = (total_w - right_w - gap).max(100.0);
                            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                let sp = ui.spacing().item_spacing;
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.allocate_ui_with_layout(
                                    vec2(left_w, 0.0),
                                    Layout::top_down(Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing = sp;
                                        ui.set_width(left_w);
                                        preview_card(ui, &st, &app.share_source, &app.share_preview);
                                    },
                                );
                                ui.add_space(gap);
                                ui.allocate_ui_with_layout(
                                    vec2(right_w, 0.0),
                                    Layout::top_down(Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing = sp;
                                        ui.set_width(right_w);
                                        invite_card(ui, &ctx, &st, &mut app.copied_at, &mut new_invite);
                                        ui.add_space(16.0);
                                        viewers_card(
                                            ui,
                                            &st,
                                            &mut kick,
                                            &mut app.kick_pending,
                                            &mut room,
                                            upload,
                                            people_now,
                                            &mut add_friend,
                                            !app.identity_temporary,
                                        );
                                    },
                                );
                            });
                            ui.add_space(theme::CALL_BAR_CLEARANCE);
                        } else {
                            preview_card(ui, &st, &app.share_source, &app.share_preview);
                            ui.add_space(16.0);
                            invite_card(ui, &ctx, &st, &mut app.copied_at, &mut new_invite);
                            ui.add_space(16.0);
                            viewers_card(
                                ui,
                                &st,
                                &mut kick,
                                &mut app.kick_pending,
                                &mut room,
                                upload,
                                people_now,
                                &mut add_friend,
                                !app.identity_temporary,
                            );
                            ui.add_space(theme::CALL_BAR_CLEARANCE);
                        }
                    });
            });
    });
    theme::call_bar(&ctx, "share_bar", |ui| {
        if matches!(st.mic, AudioState::On) {
            let (icon, tip, tone) = if st.mic_muted {
                (I_MIC_OFF, "Ativar microfone", theme::CallTone::Alert)
            } else {
                (I_MIC, "Silenciar microfone", theme::CallTone::Neutral)
            };
            if theme::call_button(ui, icon, tip, tone).clicked() {
                toggle_mic = true;
            }
        }
        ui.add_enabled_ui(!busy, |ui| {
            let q_tip = if busy {
                "Trocando qualidade…".to_string()
            } else {
                format!("Qualidade: {cur_label} (trocar sem parar a transmissão)")
            };
            let q_tone = if app.quality_open {
                theme::CallTone::Active
            } else {
                theme::CallTone::Neutral
            };
            if theme::call_button(ui, I_QUALITY, &q_tip, q_tone).clicked() {
                app.quality_open = true;
            }
        });
        if theme::call_button(
            ui,
            theme::I_SETTINGS,
            super::settings::SETTINGS_LABEL,
            if app.settings_open {
                theme::CallTone::Active
            } else {
                theme::CallTone::Neutral
            },
        )
        .clicked()
        {
            app.settings_open = true;
        }
        if theme::hangup_button(ui, I_HANGUP, "Parar a transmissão").clicked() {
            if st.viewers.is_empty() {
                stop = true;
            } else {
                app.confirm_stop = true;
            }
        }
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
                RichText::new("Vale na hora; quem está assistindo vê uma pausa curta.")
                    .small()
                    .color(theme::P.text_muted),
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
            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::button(ui, theme::Kind::Secondary, "Cancelar").clicked() {
                    close = true;
                }
            });
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
            ui.set_width(440.0);
            ui.vertical_centered(|ui| {
                theme::icon_ring(ui, I_HANGUP, theme::P.danger);
                ui.add_space(12.0);
                theme::heading(ui, "Encerrar a transmissão?");
                ui.add_space(4.0);
                let who = if n == 1 {
                    "A pessoa que está assistindo será desconectada.".to_string()
                } else {
                    format!("As {n} pessoas que estão assistindo serão desconectadas.")
                };
                ui.label(
                    RichText::new(who)
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
                        if theme::button_full(ui, theme::Kind::Secondary, "Continuar transmitindo").clicked() {
                            cancel = true;
                        }
                    });
                    ui.scope(|ui| {
                        ui.set_width(btn_w);
                        if theme::button_full(ui, theme::Kind::Danger, format!("{I_HANGUP}  Encerrar")).clicked() {
                            confirm = true;
                        }
                    });
                });
            });
        });
        if confirm {
            stop = true;
        } else if cancel || modal.should_close() {
            app.confirm_stop = false;
        }
    }

    if let Some(rx) = &app.share_job {
        match rx.try_recv() {
            Ok(msg) => {
                app.share_job = None;
                if let Some((text, kind)) = msg {
                    app.toasts.push(text, kind);
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                app.share_job = None;
                app.toasts.push("Não foi possível trocar a qualidade", ToastKind::Warn);
            }
        }
    }

    if picked.is_some() && picked != cur_choice {
        if app.share_job.is_some() {
            app.toasts.push(
                "Aguarde: a troca de qualidade anterior ainda está em andamento",
                ToastKind::Info,
            );
        } else if let Some(share) = app.share.clone()
            && let Some(p) = picked
        {
            let label = choices.label(p);
            spawn_share_job(app, &ctx, share, move |s| match s.set_preset(p) {
                Ok(()) => Some((format!("Qualidade alterada para {label}"), ToastKind::Info)),
                Err(e) => Some((format!("Não foi possível trocar a qualidade: {e}"), ToastKind::Warn)),
            });
        }
    }

    if app.share_job.is_none()
        && let Some(share) = app.share.clone()
    {
        if share.codec_pending() {
            spawn_share_job(app, &ctx, share, |s| match s.apply_codec() {
                Some(Ok(c)) => Some((format!("Codec da sala: {}", c.label()), ToastKind::Info)),
                Some(Err(e)) => Some((format!("Não foi possível trocar o codec: {e}"), ToastKind::Warn)),
                None => None,
            });
        } else {
            let mut check_auto = true;
            if let Some((preset, who)) = share.take_quality_request() {
                let name = preset.name();
                let who = if who.is_empty() { "Alguém".to_string() } else { who };
                if !app.prefs.accept_quality_requests {
                    app.toasts.push(
                        format!("{who} pediu {name}; pedidos de quem assiste estão desligados nas Configurações"),
                        ToastKind::Info,
                    );
                } else {
                    check_auto = false;
                    spawn_share_job(app, &ctx, share.clone(), move |s| match s.apply_quality_request(preset) {
                        Ok(()) => Some((format!("{who} pediu {name}: qualidade alterada para todos"), ToastKind::Info)),
                        Err(e) => Some((format!("{who} pediu {name}, mas não foi possível trocar: {e}"), ToastKind::Warn)),
                    });
                }
            }
            if check_auto
                && app.prefs.auto_quality
                && st.overloaded
                && app.auto_quality_at.is_none_or(|t| t.elapsed() >= AUTO_QUALITY_COOLDOWN)
                && let Some(lighter) = choices.lighter_than(&applied)
            {
                app.auto_quality_at = Some(Instant::now());
                let label = choices.label(lighter);
                spawn_share_job(app, &ctx, share, move |s| match s.set_preset(lighter) {
                    Ok(()) => Some((format!("PC sobrecarregado: qualidade reduzida para {label}"), ToastKind::Warn)),
                    Err(e) => Some((format!("PC sobrecarregado, mas não foi possível reduzir a qualidade: {e}"), ToastKind::Warn)),
                });
            }
        }
    }
    if let Some(id) = kick {
        if let Some(share) = app.share.as_ref() {
            share.kick(&id);
        }
        let label = app.known_viewers.remove(&id).unwrap_or_else(|| id.clone());
        app.toasts.push(
            format!("{label} foi removido e não pode voltar nesta transmissão."),
            ToastKind::Warn,
        );
    }
    if let Some((name, id)) = add_friend {
        app.add_friend = Some(super::friends::Draft {
            name,
            code: livescreen_core::friends::friend_code(&id),
            error: None,
        });
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
