//! Tema "Sinal" (só escuro): fontes Inter, Space Grotesk e JetBrains Mono embutidas, ícones Lucide como fonte.
//!
//! Subconjunto dos glifos Lucide gerado com:
//! python -m fontTools.subset <tmp>/lucide.ttf --unicodes=U+E14F,U+E195,U+E11D,U+E426,U+E048,U+E145,U+E13C,U+E09E,U+E3E8,U+E19F,U+E1A4,U+E1A2,U+E1B2,U+E18E,U+E112,U+E11A,U+E1AB,U+E1AC,U+E118,U+E119,U+E06C,U+E226,U+E193,U+E084,U+E0F9,U+E151,U+E0B2,U+E154,U+E06F,U+E06D,U+E070,U+E138,U+E29A,U+E11C,U+E13D --no-hinting --layout-features='' --output-file=assets/fonts/lucide.ttf
//! Regra: ícone novo = acrescentar o codepoint de https://unpkg.com/lucide-static@1.62.0/font/codepoints.json ao comando e regenerar lucide.ttf.
//! Depois do subset, o cmap é remapeado para o plano 15 (U+F0000 + codepoint − U+E000, tabela formato 12 via fontTools),
//! porque Inter e JetBrains Mono já ocupam os codepoints U+E000.. e sombreariam os ícones; as constantes `I_*` usam esse plano.

use std::sync::Arc;

use egui::{
    Align2, Area, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak,
    Id, InnerResponse, Margin, Order, Response, RichText, Sense, Shadow, Stroke, StrokeKind,
    TextEdit, TextStyle, Theme, WidgetInfo, vec2,
};
use tracing::debug;
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};

const INTER: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const INTER_MD: &[u8] = include_bytes!("../../assets/fonts/Inter-Medium.ttf");
const INTER_SB: &[u8] = include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");
const SG_MD: &[u8] = include_bytes!("../../assets/fonts/SpaceGrotesk-Medium.ttf");
const SG_BOLD: &[u8] = include_bytes!("../../assets/fonts/SpaceGrotesk-Bold.ttf");
const JBM: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
const LUCIDE: &[u8] = include_bytes!("../../assets/fonts/lucide.ttf");

pub const I_SHARE: &str = "\u{F014F}";
pub const I_WATCH: &str = "\u{F0195}";
pub const I_MONITOR: &str = "\u{F011D}";
pub const I_WINDOW: &str = "\u{F0426}";
pub const I_BACK: &str = "\u{F0048}";
pub const I_REFRESH: &str = "\u{F0145}";
pub const I_PLAY: &str = "\u{F013C}";
pub const I_COPY: &str = "\u{F009E}";
pub const I_PASTE: &str = "\u{F03E8}";
pub const I_PERSON: &str = "\u{F019F}";
pub const I_PEOPLE: &str = "\u{F01A4}";
pub const I_ADD_FRIEND: &str = "\u{F01A2}";
pub const I_REMOVE: &str = "\u{F01B2}";
pub const I_FULLSCREEN: &str = "\u{F0112}";
pub const I_EXIT_FULLSCREEN: &str = "\u{F011A}";
pub const I_VOLUME: &str = "\u{F01AB}";
pub const I_MUTE: &str = "\u{F01AC}";
pub const I_MIC: &str = "\u{F0118}";
pub const I_MIC_OFF: &str = "\u{F0119}";
pub const I_CHECK: &str = "\u{F006C}";
pub const I_WARN: &str = "\u{F0193}";
pub const I_ERROR: &str = "\u{F0084}";
pub const I_INFO: &str = "\u{F00F9}";
pub const I_SEARCH: &str = "\u{F0151}";
pub const I_UPDATE: &str = "\u{F00B2}";
pub const I_SETTINGS: &str = "\u{F0154}";
pub const I_CHEVRON: &str = "\u{F006F}";
pub const I_HANGUP: &str = "\u{F0138}";

pub const I_CIRCLE_CHECK: &str = "\u{F0226}";
pub const I_TRASH: &str = "\u{F018E}";
pub const I_CHEVRON_DOWN: &str = "\u{F006D}";
pub const I_CHEVRON_UP: &str = "\u{F0070}";
pub const I_QUALITY: &str = "\u{F029A}";
pub const I_MINUS: &str = "\u{F011C}";
pub const I_PLUS: &str = "\u{F013D}";

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_faint: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub success: Color32,
    pub warn: Color32,
    pub danger: Color32,
    pub on_danger: Color32,
}

pub const P: Palette = Palette {
    bg: Color32::from_rgb(0x0B, 0x0D, 0x10),
    surface: Color32::from_rgb(0x14, 0x17, 0x1C),
    raised: Color32::from_rgb(0x1C, 0x20, 0x27),
    border: Color32::from_rgb(0x26, 0x2B, 0x33),
    border_strong: Color32::from_rgb(0x39, 0x40, 0x4B),
    text: Color32::from_rgb(0xF3, 0xF5, 0xF7),
    text_muted: Color32::from_rgb(0x9A, 0xA3, 0xAE),
    text_faint: Color32::from_rgb(0x5F, 0x67, 0x73),
    accent: Color32::from_rgb(0xC8, 0xF1, 0x35),
    on_accent: Color32::from_rgb(0x0B, 0x0D, 0x10),
    success: Color32::from_rgb(0x3D, 0xDC, 0x97),
    warn: Color32::from_rgb(0xFF, 0xB5, 0x47),
    danger: Color32::from_rgb(0xFF, 0x5A, 0x4E),
    on_danger: Color32::from_rgb(0x1A, 0x07, 0x05),
};

pub const SCRIM: Color32 = Color32::from_black_alpha(153);

/// Cor a 16%: accent-soft, fundos de badge e do botão de alerta.
pub fn soft(c: Color32) -> Color32 {
    c.gamma_multiply(0.16)
}

pub const R_SM: u8 = 8;
pub const R_MD: u8 = 12;
pub const R_LG: u8 = 20;
pub const R_PILL: u8 = 255;

const SHADOW_E: Shadow = Shadow {
    offset: [0, 8],
    blur: 24,
    spread: 0,
    color: Color32::from_black_alpha(115),
};

const SHADOW_M: Shadow = Shadow {
    offset: [0, 16],
    blur: 48,
    spread: 0,
    color: Color32::from_black_alpha(153),
};

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

    defs.font_data
        .insert("inter".into(), Arc::new(FontData::from_static(INTER)));
    defs.font_data
        .insert("inter-md".into(), Arc::new(FontData::from_static(INTER_MD)));
    defs.font_data
        .insert("inter-sb".into(), Arc::new(FontData::from_static(INTER_SB)));
    defs.font_data
        .insert("sg-md".into(), Arc::new(FontData::from_static(SG_MD)));
    defs.font_data
        .insert("sg-bold".into(), Arc::new(FontData::from_static(SG_BOLD)));
    defs.font_data
        .insert("jbm".into(), Arc::new(FontData::from_static(JBM)));
    defs.font_data.insert(
        "lucide".into(),
        Arc::new(
            FontData::from_static(LUCIDE).tweak(FontTweak {
                y_offset_factor: 0.0,
                ..Default::default()
            }),
        ),
    );

    let segoe = read_font("segoeui.ttf").map(|b| {
        defs.font_data
            .insert("segoe".into(), Arc::new(FontData::from_owned(b)));
        "segoe".to_string()
    });


    let mut prop = vec!["inter".to_string()];
    if let Some(s) = &segoe {
        prop.push(s.clone());
    }
    prop.push("lucide".to_string());

    let mut med = vec!["inter-md".to_string(), "inter".to_string()];
    if let Some(s) = &segoe {
        med.push(s.clone());
    }
    med.push("lucide".to_string());

    let mut semi = vec!["inter-sb".to_string(), "inter".to_string()];
    if let Some(s) = &segoe {
        semi.push(s.clone());
    }
    semi.push("lucide".to_string());

    let display = vec!["sg-bold".to_string(), "inter".to_string(), "lucide".to_string()];
    let display_md = vec!["sg-md".to_string(), "inter".to_string(), "lucide".to_string()];
    let mono = vec!["jbm".to_string(), "inter".to_string(), "lucide".to_string()];

    defs.families.insert(FontFamily::Proportional, prop);
    defs.families.insert(FontFamily::Name("medium".into()), med);
    defs.families.insert(FontFamily::Name("semibold".into()), semi);
    defs.families.insert(FontFamily::Name("display".into()), display);
    defs.families.insert(FontFamily::Name("display-medium".into()), display_md);
    defs.families.insert(FontFamily::Monospace, mono);

    ctx.set_fonts(defs);

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
        s.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
        s.text_styles.insert(
            TextStyle::Body,
            FontId::new(14.0, FontFamily::Proportional),
        );
        s.text_styles.insert(
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Name("semibold".into())),
        );
        s.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        );
        s.text_styles.insert(
            TextStyle::Heading,
            FontId::new(20.0, FontFamily::Name("display-medium".into())),
        );
        s.text_styles.insert(
            TextStyle::Name("title".into()),
            FontId::new(32.0, FontFamily::Name("display".into())),
        );

        s.spacing.item_spacing = vec2(8.0, 8.0);
        s.spacing.button_padding = vec2(20.0, 10.0);
        s.spacing.interact_size.y = 40.0;
        s.spacing.icon_width = 20.0;
        s.spacing.icon_width_inner = 14.0;
        s.spacing.icon_spacing = 10.0;
        s.spacing.indent = 30.0;

        if reduce_motion {
            s.animation_time = 0.0;
            s.scroll_animation = egui::style::ScrollAnimation::none();
        }
    });

    let mut v = egui::Visuals::dark();
    v.panel_fill = P.bg;
    v.window_fill = P.raised;
    v.extreme_bg_color = P.bg;
    v.faint_bg_color = P.surface;
    v.text_edit_bg_color = Some(P.bg);
    v.weak_text_color = Some(P.text_muted);
    v.window_stroke = Stroke::new(1.0, P.border);
    v.window_shadow = SHADOW_M;
    v.popup_shadow = SHADOW_E;
    v.window_corner_radius = CornerRadius::same(R_LG);
    v.menu_corner_radius = CornerRadius::same(R_SM);
    v.disabled_alpha = 0.4;
    v.selection.bg_fill = soft(P.accent);
    v.selection.stroke = Stroke::new(1.0, P.accent);
    v.hyperlink_color = P.accent;
    v.warn_fg_color = P.warn;
    v.error_fg_color = P.danger;
    v.indent_has_left_vline = false;

    let w = &mut v.widgets;
    for x in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.open,
        &mut w.active,
    ] {
        x.corner_radius = CornerRadius::same(R_MD);
        x.expansion = 0.0;
        x.fg_stroke.color = P.text;
    }

    w.noninteractive.weak_bg_fill = P.surface;
    w.noninteractive.bg_fill = P.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, P.border);

    w.inactive.weak_bg_fill = P.raised;
    w.inactive.bg_fill = Color32::TRANSPARENT;
    w.inactive.bg_stroke = Stroke::new(1.0, P.border);

    let hovered_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
    w.hovered.weak_bg_fill = hovered_fill;
    w.hovered.bg_fill = Color32::TRANSPARENT;
    w.hovered.bg_stroke = Stroke::new(1.0, P.border_strong);

    w.open.weak_bg_fill = hovered_fill;
    w.open.bg_fill = Color32::TRANSPARENT;
    w.open.bg_stroke = Stroke::new(1.0, P.border_strong);

    let active_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.10);
    w.active.weak_bg_fill = active_fill;
    w.active.bg_fill = Color32::TRANSPARENT;
    w.active.bg_stroke = Stroke::new(2.0, P.accent);

    ctx.set_visuals_of(Theme::Dark, v);
}

pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
    egui::Frame::new()
        .fill(P.surface)
        .stroke(Stroke::new(1.0, P.border))
        .corner_radius(R_LG)
        .inner_margin(16)
        .show(ui, add)
}

pub fn paint_surface(ui: &egui::Ui, rect: egui::Rect, ring: Stroke) {
    ui.painter()
        .rect(rect, R_LG, P.surface, ring, StrokeKind::Inside);
}

pub fn heading(ui: &mut egui::Ui, text: impl Into<String>) -> Response {
    ui.label(RichText::new(text.into()).heading().color(P.text))
}

pub fn section_label(ui: &mut egui::Ui, text: impl Into<String>) -> Response {
    ui.label(
        RichText::new(text.into())
            .size(13.0)
            .family(FontFamily::Name("medium".into()))
            .color(P.text_muted),
    )
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

fn draw_button(
    ui: &mut egui::Ui,
    kind: Kind,
    text: egui::WidgetText,
    width: Option<f32>,
    height: f32,
    padding: egui::Vec2,
) -> Response {
    let (fill, text_color, stroke, hover_fill, press_fill, focus_stroke) = match kind {
        Kind::Primary => (
            P.accent,
            P.on_accent,
            Stroke::NONE,
            P.accent.lerp_to_gamma(Color32::WHITE, 0.08),
            P.accent.lerp_to_gamma(Color32::WHITE, 0.14),
            Stroke::new(2.0, P.text),
        ),
        Kind::Secondary => (
            P.raised,
            P.text,
            Stroke::new(1.0, P.border),
            P.raised.lerp_to_gamma(Color32::WHITE, 0.08),
            P.raised.lerp_to_gamma(Color32::WHITE, 0.14),
            Stroke::new(2.0, P.accent),
        ),
        Kind::Ghost => (
            Color32::TRANSPARENT,
            P.text,
            Stroke::NONE,
            P.raised,
            P.raised.lerp_to_gamma(Color32::WHITE, 0.06),
            Stroke::new(2.0, P.accent),
        ),
        Kind::Danger => (
            P.danger,
            P.on_danger,
            Stroke::NONE,
            P.danger.lerp_to_gamma(Color32::WHITE, 0.08),
            P.danger.lerp_to_gamma(Color32::WHITE, 0.14),
            Stroke::new(2.0, P.accent),
        ),
    };

    let resp = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = padding;
            let w = &mut ui.visuals_mut().widgets;
            w.inactive.weak_bg_fill = fill;
            w.inactive.bg_fill = fill;
            w.inactive.fg_stroke.color = text_color;
            w.inactive.bg_stroke = stroke;

            w.hovered.weak_bg_fill = hover_fill;
            w.hovered.bg_fill = hover_fill;
            w.hovered.fg_stroke.color = text_color;
            w.hovered.bg_stroke = stroke;

            w.active.weak_bg_fill = press_fill;
            w.active.bg_fill = press_fill;
            w.active.fg_stroke.color = text_color;
            w.active.bg_stroke = stroke;

            let btn = egui::Button::new(text)
                .corner_radius(R_PILL)
                .min_size(vec2(width.unwrap_or(0.0), height));
            match width {
                Some(w) => ui
                    .allocate_ui_with_layout(vec2(w, height), egui::Layout::top_down(egui::Align::Center), |ui| ui.add(btn))
                    .inner,
                None => ui.add(btn),
            }
        })
        .inner;

    if resp.has_focus() {
        ui.painter()
            .rect_stroke(resp.rect, R_PILL, focus_stroke, StrokeKind::Outside);
    }
    resp
}

pub fn button(ui: &mut egui::Ui, kind: Kind, text: impl Into<egui::WidgetText>) -> Response {
    draw_button(ui, kind, text.into(), None, 40.0, vec2(20.0, 10.0))
}

pub fn button_full(ui: &mut egui::Ui, kind: Kind, text: impl Into<egui::WidgetText>) -> Response {
    draw_button(
        ui,
        kind,
        text.into(),
        Some(ui.available_width()),
        40.0,
        vec2(20.0, 10.0),
    )
}

pub fn small_button(ui: &mut egui::Ui, kind: Kind, text: impl Into<egui::WidgetText>) -> Response {
    draw_button(ui, kind, text.into(), None, 32.0, vec2(12.0, 6.0))
}

pub fn tab(ui: &mut egui::Ui, selected: bool, text: &str) -> Response {
    let font = FontId::new(13.0, FontFamily::Name("medium".into()));
    let (fill, stroke, text_color) = if selected {
        (P.raised, Stroke::new(1.0, P.border), P.text)
    } else {
        (Color32::TRANSPARENT, Stroke::NONE, P.text_muted)
    };
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = vec2(14.0, 0.0);
        let w = &mut ui.visuals_mut().widgets;
        w.inactive.weak_bg_fill = fill;
        w.inactive.bg_fill = fill;
        w.inactive.fg_stroke.color = text_color;
        w.inactive.bg_stroke = stroke;

        w.hovered.weak_bg_fill = fill;
        w.hovered.bg_fill = fill;
        w.hovered.fg_stroke.color = P.text;
        w.hovered.bg_stroke = stroke;

        w.active.weak_bg_fill = fill;
        w.active.bg_fill = fill;
        w.active.fg_stroke.color = P.text;
        w.active.bg_stroke = stroke;

        let btn = egui::Button::new(RichText::new(text).font(font))
            .corner_radius(R_PILL)
            .min_size(vec2(0.0, 32.0));
        ui.add(btn)
    })
    .inner
}

pub fn icon_button(ui: &mut egui::Ui, icon: &str, label: &str) -> Response {
    let enabled = ui.is_enabled();
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            let w = &mut ui.visuals_mut().widgets;
            w.inactive.weak_bg_fill = P.raised;
            w.inactive.bg_fill = P.raised;
            w.inactive.bg_stroke = Stroke::new(1.0, P.border);
            w.inactive.fg_stroke.color = P.text;

            w.hovered.weak_bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
            w.hovered.bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
            w.hovered.bg_stroke = Stroke::new(1.0, P.border_strong);
            w.hovered.fg_stroke.color = P.text;

            w.active.weak_bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.10);
            w.active.bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.10);
            w.active.bg_stroke = Stroke::new(2.0, P.accent);
            w.active.fg_stroke.color = P.text;

            let icon_text = RichText::new(icon).size(20.0);
            ui.add(
                egui::Button::new(icon_text)
                    .corner_radius(R_PILL)
                    .min_size(vec2(40.0, 40.0)),
            )
        })
        .inner
        .on_hover_text(label);
    r.widget_info(|| WidgetInfo::labeled(egui::Role::Button, enabled, label));
    r
}

pub fn outline_icon_button(ui: &mut egui::Ui, icon: &str, label: &str) -> Response {
    let enabled = ui.is_enabled();
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            let w = &mut ui.visuals_mut().widgets;
            w.inactive.weak_bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_stroke = Stroke::new(1.0, P.border);
            w.inactive.fg_stroke.color = P.text_muted;

            w.hovered.weak_bg_fill = P.raised;
            w.hovered.bg_fill = P.raised;
            w.hovered.bg_stroke = Stroke::new(1.0, P.border_strong);
            w.hovered.fg_stroke.color = P.text;

            w.active.weak_bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
            w.active.bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
            w.active.bg_stroke = Stroke::new(2.0, P.accent);
            w.active.fg_stroke.color = P.text;

            let icon_text = RichText::new(icon).size(18.0);
            ui.add(
                egui::Button::new(icon_text)
                    .corner_radius(R_MD)
                    .min_size(vec2(32.0, 32.0)),
            )
        })
        .inner
        .on_hover_text(label);
    r.widget_info(|| WidgetInfo::labeled(egui::Role::Button, enabled, label));
    r
}

pub fn ghost_icon_button(ui: &mut egui::Ui, icon: &str, label: &str) -> Response {
    let enabled = ui.is_enabled();
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            let w = &mut ui.visuals_mut().widgets;
            w.inactive.weak_bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_stroke = Stroke::NONE;
            w.inactive.fg_stroke.color = P.text_muted;

            w.hovered.weak_bg_fill = P.raised;
            w.hovered.bg_fill = P.raised;
            w.hovered.bg_stroke = Stroke::NONE;
            w.hovered.fg_stroke.color = P.text;

            w.active.weak_bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
            w.active.bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
            w.active.bg_stroke = Stroke::new(2.0, P.accent);
            w.active.fg_stroke.color = P.text;

            let icon_text = RichText::new(icon).size(18.0);
            ui.add(
                egui::Button::new(icon_text)
                    .corner_radius(R_PILL)
                    .min_size(vec2(32.0, 32.0)),
            )
        })
        .inner
        .on_hover_text(label);
    r.widget_info(|| WidgetInfo::labeled(egui::Role::Button, enabled, label));
    r
}

#[derive(Clone, Copy, PartialEq)]
pub enum CallTone {
    Plain,
    Neutral,
    Active,
    Alert,
}

pub const CALL_BAR_BOTTOM: f32 = 24.0;
pub const CALL_BAR_CLEARANCE: f32 = 104.0;

fn paint_call_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    icon: &str,
    label: &str,
    tone: CallTone,
    icon_size: f32,
) -> Response {
    let enabled = ui.is_enabled();
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let (fill, stroke, fg, hover_fill) = match tone {
        CallTone::Neutral => (
            P.raised,
            Stroke::new(1.0, P.border),
            P.text,
            P.raised.lerp_to_gamma(Color32::WHITE, 0.08),
        ),
        CallTone::Active => (
            P.accent,
            Stroke::NONE,
            P.on_accent,
            P.accent.lerp_to_gamma(Color32::WHITE, 0.08),
        ),
        CallTone::Alert => (
            soft(P.danger),
            Stroke::NONE,
            P.danger,
            soft(P.danger).lerp_to_gamma(Color32::WHITE, 0.08),
        ),
        CallTone::Plain => (
            Color32::TRANSPARENT,
            Stroke::NONE,
            P.text,
            P.raised,
        ),
    };

    let mut actual_fill = fill;
    let mut actual_fg = fg;
    if !enabled {
        actual_fill = actual_fill.gamma_multiply(0.4);
        actual_fg = actual_fg.gamma_multiply(0.4);
    } else if resp.is_pointer_button_down_on() {
        actual_fill = actual_fill.lerp_to_gamma(Color32::WHITE, 0.14);
    } else if resp.hovered() {
        actual_fill = hover_fill;
    }

    let ring = if resp.has_focus() {
        Stroke::new(2.0, P.accent)
    } else {
        stroke
    };

    ui.painter().rect(
        rect,
        R_PILL,
        actual_fill,
        ring,
        StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(icon_size),
        actual_fg,
    );
    let resp = resp.on_hover_text(label);
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, enabled, label));
    resp
}

pub fn call_button(ui: &mut egui::Ui, icon: &str, label: &str, tone: CallTone) -> Response {
    let (size, icon_size) = match tone {
        CallTone::Plain => (vec2(32.0, 48.0), 20.0),
        _ => (vec2(48.0, 48.0), 22.0),
    };
    paint_call_button(ui, size, icon, label, tone, icon_size)
}

pub fn hangup_button(ui: &mut egui::Ui, icon: &str, label: &str) -> Response {
    let enabled = ui.is_enabled();
    let (rect, resp) = ui.allocate_exact_size(vec2(72.0, 48.0), Sense::click());
    let mut fill = P.danger;
    let mut fg = P.on_danger;
    if !enabled {
        fill = fill.gamma_multiply(0.4);
        fg = fg.gamma_multiply(0.4);
    } else if resp.is_pointer_button_down_on() {
        fill = fill.lerp_to_gamma(Color32::WHITE, 0.14);
    } else if resp.hovered() {
        fill = fill.lerp_to_gamma(Color32::WHITE, 0.08);
    }
    let ring = if resp.has_focus() {
        Stroke::new(2.0, P.accent)
    } else {
        Stroke::NONE
    };
    ui.painter().rect(rect, R_PILL, fill, ring, StrokeKind::Outside);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(22.0),
        fg,
    );
    let resp = resp.on_hover_text(label);
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, enabled, label));
    resp
}

pub fn call_bar<R>(
    ctx: &egui::Context,
    id: &'static str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> InnerResponse<R> {
    Area::new(Id::unique(id))
        .anchor(Align2::CENTER_BOTTOM, [0.0, -CALL_BAR_BOTTOM])
        .order(Order::Middle)
        .show(ctx, |ui| {
            let frame = egui::Frame::new()
                .fill(P.raised)
                .stroke(Stroke::new(1.0, P.border))
                .corner_radius(R_PILL)
                .inner_margin(Margin::same(8))
                .shadow(SHADOW_E);
            frame
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        add(ui)
                    })
                    .inner
                })
                .inner
        })
}

pub fn call_title(ctx: &egui::Context, text: &str) {
    Area::new(Id::unique("call_title"))
        .anchor(Align2::CENTER_TOP, [0.0, 16.0])
        .order(Order::Middle)
        .interactable(false)
        .show(ctx, |ui| {
            let frame = egui::Frame::new()
                .fill(P.raised)
                .stroke(Stroke::new(1.0, P.border))
                .corner_radius(R_PILL)
                .inner_margin(Margin::symmetric(16, 8));
            frame.show(ui, |ui| {
                ui.set_max_width(480.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(text)
                            .size(14.0)
                            .family(FontFamily::Name("semibold".into()))
                            .color(P.text),
                    )
                    .truncate(),
                );
            });
        });
}

pub fn checkbox(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: impl Into<egui::WidgetText>,
) -> Response {
    let mut on = *checked;
    let r = ui
        .scope(|ui| {
            let w = &mut ui.visuals_mut().widgets;
            w.inactive.bg_stroke = Stroke::new(1.0, P.border_strong);
            w.hovered.bg_stroke = Stroke::new(1.0, P.border_strong);
            ui.add(egui::Checkbox::new(&mut on, label.into()))
        })
        .inner;
    *checked = on;
    if !ui.is_rect_visible(r.rect) {
        return r;
    }
    let size = 20.0;
    let b = egui::Rect::from_center_size(
        egui::pos2(r.rect.left() + size / 2.0, r.rect.center().y),
        vec2(size, size),
    );
    if on {
        let (fill, fg) = if ui.is_enabled() {
            (P.accent, P.on_accent)
        } else {
            (P.accent.gamma_multiply(0.4), P.on_accent)
        };
        let ring = if r.has_focus() {
            Stroke::new(2.0, P.accent)
        } else {
            Stroke::NONE
        };
        let painter = ui.painter();
        painter.rect(b, 6, fill, ring, StrokeKind::Outside);
        let c = b.center();
        painter.line(
            vec![
                c + vec2(-5.0, 0.0),
                c + vec2(-1.5, 3.5),
                c + vec2(5.0, -3.5),
            ],
            Stroke::new(2.0, fg),
        );
    } else if r.has_focus() {
        ui.painter()
            .rect_stroke(b, 6, Stroke::new(2.0, P.accent), StrokeKind::Outside);
    }
    r
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) -> Response {
    let font = FontId::new(12.0, FontFamily::Name("medium".into()));
    let fill = if color == P.text_muted || color == P.text_faint {
        P.raised
    } else {
        soft(color)
    };
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, color);
    let (rect, resp) =
        ui.allocate_exact_size(galley.size() + vec2(16.0, 4.0), Sense::hover());
    ui.painter().rect_filled(rect, R_PILL, fill);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, color);
    let enabled = ui.is_enabled();
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Label, enabled, text));
    resp
}

pub fn live_pill(ui: &mut egui::Ui) {
    let font = FontId::new(12.0, FontFamily::Name("semibold".into()));
    let text = "AO VIVO";
    let fg = P.on_danger;
    let bg = P.danger;
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, fg);
    let content_w = 6.0 + 6.0 + galley.size().x;
    let content_h = galley.size().y.max(6.0);
    let (rect, resp) =
        ui.allocate_exact_size(vec2(content_w + 20.0, content_h + 8.0), Sense::hover());
    ui.painter().rect_filled(rect, R_PILL, bg);
    let dot_center = egui::pos2(rect.left() + 10.0 + 3.0, rect.center().y);
    ui.painter().circle_filled(dot_center, 3.0, fg);
    let text_pos = egui::pos2(
        rect.left() + 10.0 + 6.0 + 6.0,
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, galley, fg);
    let enabled = ui.is_enabled();
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Label, enabled, text));
}

pub fn icon_ring(ui: &mut egui::Ui, icon: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(56.0, 56.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 28.0, soft(color));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(26.0),
        color,
    );
}

pub fn modal(_ctx: &egui::Context, name: &'static str) -> egui::Modal {
    egui::Modal::new(Id::unique(name))
        .backdrop_color(SCRIM)
        .frame(
            egui::Frame::new()
                .fill(P.raised)
                .stroke(Stroke::new(1.0, P.border))
                .corner_radius(R_LG)
                .inner_margin(24)
                .shadow(SHADOW_M),
        )
}

pub fn field<'t>(ui: &egui::Ui, edit: TextEdit<'t>) -> TextEdit<'t> {
    let _ = ui;
    edit.min_size(vec2(0.0, 40.0))
        .margin(Margin::symmetric(12, 0))
        .align(Align2::LEFT_CENTER)
}

pub fn stepper(
    ui: &mut egui::Ui,
    value: &mut usize,
    min: usize,
    max: usize,
    hint: &str,
) -> bool {
    let mut changed = false;
    let cur = *value;
    ui.allocate_ui_with_layout(vec2(28.0 * 2.0 + 20.0 + 4.0 * 2.0, 28.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let btn = |ui: &mut egui::Ui, icon: &str, enabled: bool| {
            ui.add_enabled_ui(enabled, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().button_padding = egui::Vec2::ZERO;
                    let w = &mut ui.visuals_mut().widgets;
                    w.inactive.weak_bg_fill = P.raised;
                    w.inactive.bg_fill = P.raised;
                    w.inactive.bg_stroke = Stroke::new(1.0, P.border);
                    w.inactive.fg_stroke.color = P.text;

                    w.hovered.weak_bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
                    w.hovered.bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.06);
                    w.hovered.bg_stroke = Stroke::new(1.0, P.border_strong);
                    w.hovered.fg_stroke.color = P.text;

                    w.active.weak_bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.10);
                    w.active.bg_fill = P.raised.lerp_to_gamma(Color32::WHITE, 0.10);
                    w.active.bg_stroke = Stroke::new(2.0, P.accent);
                    w.active.fg_stroke.color = P.text;

                    let icon_text = RichText::new(icon).size(14.0);
                    ui.add(
                        egui::Button::new(icon_text)
                            .corner_radius(R_SM)
                            .min_size(vec2(28.0, 28.0)),
                    )
                })
                .inner
            })
            .inner
        };

        let minus = btn(ui, I_MINUS, cur > min);
        let minus = if !hint.is_empty() {
            minus.on_hover_text(hint)
        } else {
            minus
        };
        if minus.clicked() {
            *value = cur.saturating_sub(1).max(min);
            changed = true;
        }

        ui.add_sized(
            [20.0, 28.0],
            egui::Label::new(
                RichText::new(cur.to_string())
                    .size(14.0)
                    .family(FontFamily::Name("semibold".into()))
                    .color(P.text),
            )
            .selectable(false),
        );

        let plus = btn(ui, I_PLUS, cur < max);
        let plus = if !hint.is_empty() {
            plus.on_hover_text(hint)
        } else {
            plus
        };
        if plus.clicked() {
            *value = (cur + 1).min(max);
            changed = true;
        }
    });
    changed
}

pub fn option_row(
    ui: &mut egui::Ui,
    selected: bool,
    title: &str,
    detail: &str,
    recommended: bool,
) -> Response {
    let enabled = ui.is_enabled();
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
    let (fill, stroke) = if selected {
        (soft(P.accent), Stroke::new(2.0, P.accent))
    } else if resp.hovered() {
        (P.bg, Stroke::new(1.0, P.border_strong))
    } else {
        (P.bg, Stroke::new(1.0, P.border))
    };
    ui.painter()
        .rect(rect, R_MD, fill, stroke, StrokeKind::Inside);

    let radio_center = egui::pos2(rect.left() + 14.0 + 8.0, rect.center().y);
    if selected {
        ui.painter()
            .circle_stroke(radio_center, 8.0, Stroke::new(2.0, P.accent));
        ui.painter().circle_filled(radio_center, 4.0, P.accent);
    } else {
        ui.painter()
            .circle_stroke(radio_center, 8.0, Stroke::new(1.0, P.border_strong));
    }

    let title_font = FontId::new(14.0, FontFamily::Name("semibold".into()));
    let detail_font = FontId::new(12.0, FontFamily::Proportional);

    let title_galley = ui.painter().layout_no_wrap(title.to_string(), title_font, P.text);
    let title_x = rect.left() + 14.0 + 16.0 + 12.0;
    let title_pos = egui::pos2(title_x, rect.center().y - title_galley.size().y / 2.0);
    ui.painter().galley(title_pos, title_galley.clone(), P.text);

    if !detail.is_empty() {
        let text = format!("  ·  {detail}");
        let detail_galley = ui.painter().layout_no_wrap(text, detail_font, P.text_muted);
        let detail_x = title_x + title_galley.size().x;
        let detail_pos = egui::pos2(detail_x, rect.center().y - detail_galley.size().y / 2.0);
        ui.painter().galley(detail_pos, detail_galley, P.text_muted);
    }

    if recommended {
        let badge_font = FontId::new(12.0, FontFamily::Name("medium".into()));
        let badge_text = "recomendado";
        let b_galley = ui.painter().layout_no_wrap(badge_text.to_string(), badge_font, P.accent);
        let b_rect = egui::Rect::from_center_size(
            egui::pos2(
                rect.right() - 14.0 - (b_galley.size().x + 16.0) / 2.0,
                rect.center().y,
            ),
            vec2(b_galley.size().x + 16.0, b_galley.size().y + 4.0),
        );
        ui.painter().rect_filled(b_rect, R_PILL, soft(P.accent));
        ui.painter()
            .galley(b_rect.center() - b_galley.size() / 2.0, b_galley, P.accent);
    }

    resp.widget_info(|| WidgetInfo::selected(egui::Role::RadioButton, enabled, selected, title));
    resp
}

pub fn chevron_icon(ui: &egui::Ui, rect: egui::Rect, _v: &egui::style::WidgetVisuals, open: bool) {
    let icon = if open {
        I_CHEVRON_UP
    } else {
        I_CHEVRON_DOWN
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(16.0),
        P.text_muted,
    );
}

pub fn app_icon() -> egui::IconData {
    egui::IconData {
        rgba: crate::icon::rgba(64),
        width: 64,
        height: 64,
    }
}
