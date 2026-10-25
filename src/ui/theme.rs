//! Tema no estilo Vercel/Geist: fonte Geist embutida, claro/escuro monocromáticos, sombra-como-borda, anel de foco azul e helpers de widgets.

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Margin, RichText, Shadow, Stroke,
    TextStyle, Theme,
};
use tracing::debug;
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};

const GEIST: &[u8] = include_bytes!("../../assets/fonts/Geist-Regular.ttf");
const GEIST_SB: &[u8] = include_bytes!("../../assets/fonts/Geist-SemiBold.ttf");
const GEIST_MD: &[u8] = include_bytes!("../../assets/fonts/Geist-Medium.ttf");
const GEIST_MONO: &[u8] = include_bytes!("../../assets/fonts/GeistMono-Regular.ttf");

pub const I_SHARE: &str = "\u{E72D}";
pub const I_WATCH: &str = "\u{E714}";
pub const I_MONITOR: &str = "\u{E7F4}";
pub const I_WINDOW: &str = "\u{E8A7}";
pub const I_BACK: &str = "\u{E72B}";
pub const I_REFRESH: &str = "\u{E72C}";
pub const I_PLAY: &str = "\u{E768}";
pub const I_STOP: &str = "\u{E71A}";
pub const I_COPY: &str = "\u{E8C8}";
pub const I_PASTE: &str = "\u{E77F}";
pub const I_PERSON: &str = "\u{E77B}";
pub const I_PEOPLE: &str = "\u{E716}";
pub const I_REMOVE: &str = "\u{E711}";
pub const I_FULLSCREEN: &str = "\u{E740}";
pub const I_EXIT_FULLSCREEN: &str = "\u{E73F}";
pub const I_VOLUME: &str = "\u{E767}";
pub const I_MUTE: &str = "\u{E74F}";
pub const I_MIC: &str = "\u{E720}";
pub const I_MIC_OFF: &str = "\u{EC54}";
pub const I_CHECK: &str = "\u{E73E}";
pub const I_WARN: &str = "\u{E7BA}";
pub const I_ERROR: &str = "\u{E783}";
pub const I_INFO: &str = "\u{E946}";
pub const I_SEARCH: &str = "\u{E721}";
pub const I_UPDATE: &str = "\u{E896}";
pub const I_SETTINGS: &str = "\u{E713}";

#[derive(Clone, Copy)]
pub struct Palette {
    pub card: Color32,
    /// Anel de 1 px semitransparente (sombra-como-borda).
    pub border: Color32,
    /// Anel no hover: mais contraste que o repouso.
    pub border_strong: Color32,
    /// Azul de foco, seleção, links e realces.
    pub accent: Color32,
    pub primary: Color32,
    pub on_primary: Color32,
    pub danger: Color32,
    pub on_danger: Color32,
    pub success: Color32,
    pub warn: Color32,
    /// Camada ambiente dos cartões.
    pub shadow: Shadow,
}

const DARK: Palette = Palette {
    card: Color32::from_rgb(0x0A, 0x0A, 0x0A),
    border: Color32::from_rgba_premultiplied(37, 37, 37, 37),
    border_strong: Color32::from_rgba_premultiplied(61, 61, 61, 61),
    accent: Color32::from_rgb(0x00, 0x72, 0xF5),
    primary: Color32::from_rgb(0xED, 0xED, 0xED),
    on_primary: Color32::from_rgb(0x0A, 0x0A, 0x0A),
    danger: Color32::from_rgb(0xFF, 0x5B, 0x4F),
    on_danger: Color32::WHITE,
    success: Color32::from_rgb(0x6C, 0xCB, 0x5F),
    warn: Color32::from_rgb(0xF2, 0xC7, 0x44),
    shadow: Shadow {
        offset: [0, 2],
        blur: 2,
        spread: 0,
        color: Color32::from_black_alpha(64),
    },
};
const LIGHT: Palette = Palette {
    card: Color32::WHITE,
    border: Color32::from_black_alpha(20),
    border_strong: Color32::from_black_alpha(36),
    accent: Color32::from_rgb(0x00, 0x72, 0xF5),
    primary: Color32::from_rgb(0x17, 0x17, 0x17),
    on_primary: Color32::WHITE,
    danger: Color32::from_rgb(0xFF, 0x5B, 0x4F),
    on_danger: Color32::WHITE,
    success: Color32::from_rgb(0x0F, 0x7B, 0x0F),
    warn: Color32::from_rgb(0x9D, 0x5D, 0x00),
    shadow: Shadow {
        offset: [0, 2],
        blur: 2,
        spread: 0,
        color: Color32::from_black_alpha(10),
    },
};

pub fn palette(ui: &egui::Ui) -> Palette {
    if ui.visuals().dark_mode { DARK } else { LIGHT }
}

fn read_font(name: &str) -> Option<Vec<u8>> {
    let dir = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "C:\\Windows".into());
    match std::fs::read(dir.join("Fonts").join(name)) {
        Ok(b) => Some(b),
        Err(e) => {
            debug!("fonte {name} indisponível: {e}");
            None
        }
    }
}

pub fn install(ctx: &egui::Context) {
    let mut defs = FontDefinitions::default();
    // Carrega o primeiro arquivo que existir; ausência não é erro.
    let load = |defs: &mut FontDefinitions, key: &str, files: &[&str]| {
        let Some(bytes) = files.iter().find_map(|f| read_font(f)) else {
            return false;
        };
        defs.font_data
            .insert(key.to_string(), Arc::new(FontData::from_owned(bytes)));
        true
    };
    defs.font_data
        .insert("geist".into(), Arc::new(FontData::from_static(GEIST)));
    defs.font_data
        .insert("geist-sb".into(), Arc::new(FontData::from_static(GEIST_SB)));
    defs.font_data
        .insert("geist-md".into(), Arc::new(FontData::from_static(GEIST_MD)));
    defs.font_data.insert(
        "geist-mono".into(),
        Arc::new(FontData::from_static(GEIST_MONO)),
    );
    let segoe = load(&mut defs, "segoe", &["segoeui.ttf"]);
    let semibold = load(&mut defs, "segoe-sb", &["seguisb.ttf"]);
    let mono = load(&mut defs, "mono", &["CascadiaMono.ttf", "consola.ttf"]);
    let icons = load(&mut defs, "icons", &["SegoeIcons.ttf", "segmdl2.ttf"]);

    let prop = defs.families.entry(FontFamily::Proportional).or_default();
    if segoe {
        prop.insert(0, "segoe".into());
    }
    prop.insert(0, "geist".into());
    let mut semi: Vec<String> = vec!["geist-sb".into()];
    if semibold {
        semi.push("segoe-sb".into());
    }
    semi.extend(prop.iter().cloned());
    // Peso médio só nos botões, como no Geist; sem Segoe Medium, cai na regular.
    let mut med: Vec<String> = vec!["geist-md".into()];
    med.extend(prop.iter().cloned());
    let monof = defs.families.entry(FontFamily::Monospace).or_default();
    if mono {
        monof.insert(0, "mono".into());
    }
    monof.insert(0, "geist-mono".into());
    if icons {
        for f in [FontFamily::Proportional, FontFamily::Monospace] {
            defs.families.entry(f).or_default().push("icons".into());
        }
        semi.push("icons".into());
        med.push("icons".into());
    }
    defs.families
        .insert(FontFamily::Name("semibold".into()), semi);
    defs.families
        .insert(FontFamily::Name("medium".into()), med);
    ctx.set_fonts(defs);

    let semibold = FontFamily::Name("semibold".into());
    let medium = FontFamily::Name("medium".into());
    let mut anim = windows::core::BOOL(1);
    let _ = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some(&mut anim as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    let reduce_motion = !anim.as_bool();
    ctx.all_styles_mut(|s| {
        use egui::FontId;
        s.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
        s.text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        s.text_styles
            .insert(TextStyle::Button, FontId::new(14.0, medium.clone()));
        s.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        );
        s.text_styles
            .insert(TextStyle::Heading, FontId::new(22.0, semibold.clone()));
        s.text_styles.insert(
            TextStyle::Name("title".into()),
            FontId::new(28.0, semibold.clone()),
        );
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size.y = 36.0;
        s.spacing.icon_width = 16.0;
        s.spacing.icon_spacing = 8.0;
        // Recuo = caixa + espaço: texto recuado alinha com o rótulo do checkbox/radio.
        s.spacing.indent = 24.0;
        if reduce_motion {
            s.animation_time = 0.0;
            s.scroll_animation = egui::style::ScrollAnimation::none();
        }
    });

    for (theme, pal, panel, window, extreme, text, weak, hover, pressed) in [
        (Theme::Dark, DARK, 0x0Au8, 0x0Au8, 0x1Au8, 0xEDu8, 0xA1u8, 0x1Fu8, 0x2Eu8),
        (Theme::Light, LIGHT, 0xFF, 0xFF, 0xFA, 0x17, 0x66, 0xF2, 0xEB),
    ] {
        let dark = theme == Theme::Dark;
        let mut v = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        let text = Color32::from_gray(text);
        v.panel_fill = Color32::from_gray(panel);
        v.window_fill = Color32::from_gray(window);
        v.extreme_bg_color = Color32::from_gray(extreme);
        v.text_edit_bg_color = Some(pal.card);
        v.weak_text_color = Some(Color32::from_gray(weak));
        v.window_stroke = Stroke::new(1.0, pal.border);
        v.window_shadow = Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(if dark { 128 } else { 31 }),
        };
        v.popup_shadow = Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(if dark { 96 } else { 20 }),
        };
        let hover = Color32::from_gray(hover);
        let pressed = Color32::from_gray(pressed);
        let w = &mut v.widgets;
        let ring = Stroke::new(1.0, pal.border);
        let ring_strong = Stroke::new(1.0, pal.border_strong);
        for (x, fill, stroke) in [
            (&mut w.noninteractive, pal.card, ring),
            (&mut w.inactive, pal.card, ring_strong),
            (&mut w.hovered, hover, ring_strong),
            (&mut w.open, hover, ring_strong),
            (&mut w.active, pressed, Stroke::new(2.0, pal.accent)),
        ] {
            x.corner_radius = CornerRadius::same(6);
            x.expansion = 0.0;
            x.fg_stroke.color = text;
            x.weak_bg_fill = fill;
            x.bg_fill = fill;
            x.bg_stroke = stroke;
        }
        v.indent_has_left_vline = false;
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(8);
        v.selection.bg_fill = pal.accent.linear_multiply(0.18);
        v.selection.stroke = Stroke::new(1.0, pal.accent);
        v.hyperlink_color = pal.accent;
        v.warn_fg_color = pal.warn;
        v.error_fg_color = pal.danger;
        ctx.set_visuals_of(theme, v);
    }
}

pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    let p = palette(ui);
    egui::Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(1.0, p.border))
        .shadow(p.shadow)
        .corner_radius(12)
        .inner_margin(Margin::same(16))
        .show(ui, add)
}

/// Fundo de cartão pintado à mão: sombra ambiente, preenchimento e anel.
pub fn paint_surface(ui: &egui::Ui, rect: egui::Rect, ring: Stroke) {
    let p = palette(ui);
    ui.painter().add(p.shadow.as_shape(rect, 12));
    ui.painter()
        .rect(rect, 12, p.card, ring, egui::StrokeKind::Inside);
}

/// Título de seção em Geist com tracking negativo, como no Vercel.
pub fn heading(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.label(RichText::new(text).heading().extra_letter_spacing(-0.5))
}

/// Botão só com ícone: dica no hover e nome acessível (o leitor de tela não lê o glifo).
pub fn icon_button(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let enabled = ui.is_enabled();
    let side = ui.spacing().interact_size.y;
    // Sem padding: o mínimo quadrado vira o tamanho exato, qualquer que seja o glifo.
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            ui.add(egui::Button::new(icon).min_size(egui::vec2(side, side)))
        })
        .inner
        .on_hover_text(label);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));
    r
}

/// Checkbox no estilo Geist: marcado vira caixa cheia na cor primária com o visto em contraste.
pub fn checkbox(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: impl Into<egui::WidgetText>,
) -> egui::Response {
    let mut on = *checked;
    let r = ui.add(egui::Checkbox::new(&mut on, label.into()));
    *checked = on;
    if !on || !ui.is_rect_visible(r.rect) {
        return r;
    }
    let p = palette(ui);
    let size = ui.spacing().icon_width;
    let b = egui::Rect::from_center_size(
        egui::pos2(r.rect.left() + size / 2.0, r.rect.center().y),
        egui::vec2(size, size),
    );
    let (fill, fg) = if ui.is_enabled() {
        (p.primary, p.on_primary)
    } else {
        (p.primary.gamma_multiply(0.4), p.on_primary)
    };
    let ring = if r.has_focus() {
        Stroke::new(2.0, p.accent)
    } else {
        Stroke::NONE
    };
    let painter = ui.painter();
    painter.rect(b, 4, fill, ring, egui::StrokeKind::Outside);
    let c = b.center();
    painter.line(
        vec![
            c + egui::vec2(-3.5, 0.0),
            c + egui::vec2(-1.0, 2.5),
            c + egui::vec2(3.5, -2.5),
        ],
        Stroke::new(1.75, fg),
    );
    r
}

fn filled_button(
    ui: &mut egui::Ui,
    text: &str,
    fill: Color32,
    fg: Color32,
    min_w: f32,
) -> egui::Response {
    ui.scope(|ui| {
        let w = &mut ui.visuals_mut().widgets;
        for (s, f) in [
            (&mut w.inactive, fill),
            (&mut w.hovered, fill.lerp_to_gamma(fg, 0.15)),
            (&mut w.active, fill.lerp_to_gamma(fg, 0.25)),
        ] {
            s.weak_bg_fill = f;
            s.fg_stroke.color = fg;
        }
        w.inactive.bg_stroke = Stroke::NONE;
        w.hovered.bg_stroke = Stroke::NONE;
        let h = ui.spacing().interact_size.y;
        ui.add(egui::Button::new(text).min_size(egui::vec2(min_w, h)))
    })
    .inner
}

pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette(ui);
    filled_button(ui, text, p.primary, p.on_primary, 0.0)
}

pub fn primary_button_full(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette(ui);
    let w = ui.available_width();
    filled_button(ui, text, p.primary, p.on_primary, w)
}

pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette(ui);
    filled_button(ui, text, p.danger, p.on_danger, 0.0)
}

fn chip(
    ui: &mut egui::Ui,
    text: &str,
    font: egui::FontId,
    fg: Color32,
    bg: Color32,
) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, fg);
    let (rect, resp) =
        ui.allocate_exact_size(galley.size() + egui::vec2(12.0, 4.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4, bg);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, fg);
    resp
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) -> egui::Response {
    chip(
        ui,
        text,
        egui::FontId::proportional(12.0),
        color,
        color.linear_multiply(0.18),
    )
}

pub fn live_pill(ui: &mut egui::Ui) {
    let p = palette(ui);
    chip(
        ui,
        "AO VIVO",
        egui::FontId::new(12.0, egui::FontFamily::Name("semibold".into())),
        p.on_danger,
        p.danger,
    );
}

/// Ícone da janela; o mesmo desenho vai embutido no .exe pelo `build.rs`.
pub fn app_icon() -> egui::IconData {
    egui::IconData {
        rgba: crate::icon::rgba(64),
        width: 64,
        height: 64,
    }
}

/// Modal com margem interna de 20 px; só os modais, sem mexer em menus e popups.
pub fn modal(ctx: &egui::Context, name: &'static str) -> egui::Modal {
    let frame = egui::Frame::popup(&ctx.global_style()).inner_margin(Margin::same(20));
    egui::Modal::new(egui::Id::unique(name)).frame(frame)
}

/// Botão compacto para linhas de texto e listas: mesmo padrão dos outros, só mais baixo.
pub fn small_button(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(10.0, 0.0);
        ui.add(egui::Button::new(text.into()).min_size(egui::vec2(0.0, 28.0)))
    })
    .inner
}

/// Campo de texto de uma linha na mesma altura dos botões, texto centrado na vertical.
pub fn field<'t>(ui: &egui::Ui, edit: egui::TextEdit<'t>) -> egui::TextEdit<'t> {
    edit.min_size(egui::vec2(0.0, ui.spacing().interact_size.y))
        .margin(Margin::symmetric(10, 0))
        .align(egui::Align2::LEFT_CENTER)
}
