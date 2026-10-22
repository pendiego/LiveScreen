//! Tema nativo do Windows 11: fontes do sistema, cores e helpers de widgets.

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Margin, RichText, Stroke,
    TextStyle, Theme,
};
use tracing::debug;

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
    pub border: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
    pub on_danger: Color32,
    pub success: Color32,
    pub warn: Color32,
}

const DARK: Palette = Palette {
    card: Color32::from_rgb(0x2B, 0x2B, 0x2B),
    border: Color32::from_rgb(0x3D, 0x3D, 0x3D),
    accent: Color32::from_rgb(0x4C, 0xC2, 0xFF),
    on_accent: Color32::BLACK,
    danger: Color32::from_rgb(0xE5, 0x48, 0x4D),
    on_danger: Color32::WHITE,
    success: Color32::from_rgb(0x6C, 0xCB, 0x5F),
    warn: Color32::from_rgb(0xF2, 0xC7, 0x44),
};
const LIGHT: Palette = Palette {
    card: Color32::WHITE,
    border: Color32::from_rgb(0xE5, 0xE5, 0xE5),
    accent: Color32::from_rgb(0x00, 0x67, 0xC0),
    on_accent: Color32::WHITE,
    danger: Color32::from_rgb(0xC4, 0x2B, 0x1C),
    on_danger: Color32::WHITE,
    success: Color32::from_rgb(0x0F, 0x7B, 0x0F),
    warn: Color32::from_rgb(0x9D, 0x5D, 0x00),
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
    let segoe = load(&mut defs, "segoe", &["segoeui.ttf"]);
    let semibold = load(&mut defs, "segoe-sb", &["seguisb.ttf"]);
    let mono = load(&mut defs, "mono", &["CascadiaMono.ttf", "consola.ttf"]);
    let icons = load(&mut defs, "icons", &["SegoeIcons.ttf", "segmdl2.ttf"]);

    let prop = defs.families.entry(FontFamily::Proportional).or_default();
    if segoe {
        prop.insert(0, "segoe".into());
    }
    let mut semi: Vec<String> = Vec::new();
    if semibold {
        semi.push("segoe-sb".into());
    }
    semi.extend(prop.iter().cloned());
    if mono {
        defs.families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "mono".into());
    }
    if icons {
        for f in [FontFamily::Proportional, FontFamily::Monospace] {
            defs.families.entry(f).or_default().push("icons".into());
        }
        semi.push("icons".into());
    }
    defs.families
        .insert(FontFamily::Name("semibold".into()), semi);
    ctx.set_fonts(defs);

    let semibold = FontFamily::Name("semibold".into());
    ctx.all_styles_mut(|s| {
        use egui::FontId;
        s.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
        s.text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        s.text_styles.insert(
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        );
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
        s.spacing.button_padding = egui::vec2(14.0, 7.0);
        s.spacing.interact_size.y = 32.0;
    });

    for (theme, pal, panel, window, extreme) in [
        (Theme::Dark, DARK, 0x20u8, 0x2Bu8, 0x1Cu8),
        (Theme::Light, LIGHT, 0xF3, 0xFB, 0xFF),
    ] {
        let mut v = if theme == Theme::Dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.panel_fill = Color32::from_gray(panel);
        v.window_fill = Color32::from_gray(window);
        v.extreme_bg_color = Color32::from_gray(extreme);
        let r = CornerRadius::same(6);
        let w = &mut v.widgets;
        for x in [
            &mut w.noninteractive,
            &mut w.inactive,
            &mut w.hovered,
            &mut w.active,
            &mut w.open,
        ] {
            x.corner_radius = r;
        }
        let fg = Color32::from_gray(if theme == Theme::Dark { 232 } else { 26 });
        v.widgets.noninteractive.fg_stroke.color = fg;
        v.widgets.inactive.fg_stroke.color = fg;
        v.window_corner_radius = CornerRadius::same(8);
        v.menu_corner_radius = CornerRadius::same(6);
        v.selection.bg_fill = pal.accent.linear_multiply(0.4);
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
        .corner_radius(8)
        .inner_margin(Margin::same(16))
        .show(ui, add)
}

fn filled_button(ui: &mut egui::Ui, text: &str, fill: Color32, fg: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).color(fg))
            .fill(fill)
            .min_size(egui::vec2(0.0, 34.0)),
    )
}

pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette(ui);
    filled_button(ui, text, p.accent, p.on_accent)
}

pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette(ui);
    filled_button(ui, text, p.danger, p.on_danger)
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
