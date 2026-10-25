//! Configurações do app: modal aberto de qualquer tela; tudo vai para ui.json.
use std::time::Duration;

use egui::RichText;

use super::App;
use super::prefs::ThemeChoice;
use super::share_view::{MAX_ROOM, preset_hint};
use super::theme::{self, I_SETTINGS};
use super::toast::ToastKind;
use crate::preset::Preset;

const FORCED: &str = "Desligado por config.json ou LIVESCREEN_CAPTURE_OVERRIDES";

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Video,
    Share,
    Look,
    Updates,
    Diagnostics,
}

const TABS: [(Tab, &str); 5] = [
    (Tab::Video, "Vídeo"),
    (Tab::Share, "Transmissão"),
    (Tab::Look, "Aparência"),
    (Tab::Updates, "Atualizações"),
    (Tab::Diagnostics, "Diagnóstico"),
];

/// Engrenagem; devolve true no clique (para telas que seguram `app` emprestado).
pub fn icon_button(ui: &mut egui::Ui) -> bool {
    theme::icon_button(ui, I_SETTINGS, "Configurações (Ctrl+,)").clicked()
}

pub fn button(app: &mut App, ui: &mut egui::Ui) {
    if icon_button(ui) {
        app.settings_open = true;
    }
}

pub(super) fn preset_name(p: Preset) -> &'static str {
    match p {
        Preset::P480p30 => "480p 30 fps",
        Preset::P720p30 => "720p 30 fps",
        Preset::P720p60 => "720p 60 fps",
        Preset::P1080p30 => "1080p 30 fps",
        Preset::P1080p60 => "1080p 60 fps",
        Preset::P1440p60 => "1440p 60 fps",
        Preset::P2160p60 => "4K 60 fps",
        Preset::Native60 => "Nativa 60 fps",
        Preset::NativeHigh => "Nativa na taxa do monitor",
    }
}

/// Checkbox; `forced` (config.json/env) mostra desmarcado e travado. Devolve true se mudou.
fn toggle(ui: &mut egui::Ui, value: &mut bool, label: &str, forced: bool, enabled: bool) -> bool {
    let mut shown = *value && !forced;
    let r = ui
        .add_enabled_ui(enabled && !forced, |ui| theme::checkbox(ui, &mut shown, label))
        .inner;
    let r = if forced {
        r.on_disabled_hover_text(FORCED)
    } else {
        r
    };
    if r.changed() {
        *value = shown;
        true
    } else {
        false
    }
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).small().weak());
}

/// Dica de um checkbox: recuada para alinhar com o rótulo, não com a caixa.
fn toggle_hint(ui: &mut egui::Ui, text: &str) {
    ui.indent(text, |ui| hint(ui, text));
}

pub fn show(app: &mut App, ctx: &egui::Context) {
    let mut close = false;
    let modal = theme::modal(ctx, "settings").show(ctx, |ui| {
        ui.set_width(560.0);
        theme::heading(ui, "Configurações");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            for (t, label) in TABS {
                ui.selectable_value(&mut app.settings_tab, t, label);
            }
        });
        ui.separator();
        match app.settings_tab {
            Tab::Video => video(app, ui),
            Tab::Share => share(app, ui),
            Tab::Look => look(app, ui, ctx),
            Tab::Updates => updates(app, ui, ctx),
            Tab::Diagnostics => diagnostics(app, ui),
        }
        ui.add_space(8.0);
        ui.separator();
        if ui.button("Fechar").clicked() {
            close = true;
        }
    });
    if close || modal.should_close() {
        app.settings_open = false;
    }
    app.save_prefs();
    if app.encoder.lock().unwrap().is_none() {
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn video(app: &mut App, ui: &mut egui::Ui) {
    let forced = app.overrides;
    toggle(
        ui,
        &mut app.prefs.hardware_decode,
        "Decodificar vídeo pela placa de vídeo",
        forced.deny_hardware_decoder,
        true,
    );
    toggle_hint(
        ui,
        "Vale na próxima vez que você assistir. Se falhar, o LiveScreen usa o processador.",
    );
    ui.add_space(8.0);

    let encoder = app.encoder.lock().unwrap().clone();
    if toggle(
        ui,
        &mut app.prefs.hardware_encoder,
        "Codificar vídeo pela placa de vídeo",
        forced.deny_hardware_encoder,
        encoder.is_some(),
    ) {
        app.detect_encoder();
    }
    toggle_hint(ui, "Vale na próxima transmissão.");
    toggle_hint(
        ui,
        &match &encoder {
            Some(kinds) => format!(
                "Em uso: {}",
                kinds.iter().map(|k| k.label()).collect::<Vec<_>>().join(", ")
            ),
            None => "Detectando…".to_string(),
        },
    );
    ui.add_space(8.0);

    theme::checkbox(ui, &mut app.prefs.show_stats,
        "Mostrar estatísticas de vídeo ao assistir",
    );
}

fn share(app: &mut App, ui: &mut egui::Ui) {
    let mut pr = app.prefs.preset;
    egui::ComboBox::from_label("Qualidade padrão")
        .selected_text(preset_name(pr))
        .show_ui(ui, |ui| {
            for p in Preset::ALL {
                ui.selectable_value(&mut pr, p, preset_name(p))
                    .on_hover_text(preset_hint(p));
            }
        });
    if pr != app.prefs.preset {
        app.prefs.preset = pr;
        if let Some(p) = app.picker.as_mut() {
            p.preset = pr;
        }
    }

    if theme::checkbox(ui, &mut app.prefs.share_audio, "Transmitir áudio")
        .changed()
        && let Some(p) = app.picker.as_mut()
    {
        p.audio = app.prefs.share_audio;
    }

    if theme::checkbox(ui, &mut app.prefs.share_mic, "Transmitir microfone")
        .changed()
        && let Some(p) = app.picker.as_mut()
    {
        p.mic = app.prefs.share_mic;
    }

    ui.horizontal(|ui| {
        ui.label("Limite da sala");
        let r = ui.add(egui::DragValue::new(&mut app.prefs.room_limit).range(1..=MAX_ROOM));
        if r.changed()
            && let Some(s) = &app.share
        {
            s.set_max_viewers(app.prefs.room_limit);
        }
    });

    ui.label("Seu nome ao assistir");
    ui.add(theme::field(
        ui,
        egui::TextEdit::singleline(&mut app.prefs.name)
            .char_limit(32)
            .hint_text("Como você aparece para quem compartilha")
            .desired_width(f32::INFINITY),
    ));
    hint(
        ui,
        "A qualidade e o áudio também mudam quando você escolhe ao iniciar uma transmissão.",
    );
}

fn look(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    let before = app.prefs.theme;
    ui.radio_value(
        &mut app.prefs.theme,
        ThemeChoice::System,
        "Seguir o Windows",
    );
    ui.radio_value(&mut app.prefs.theme, ThemeChoice::Light, "Claro");
    ui.radio_value(&mut app.prefs.theme, ThemeChoice::Dark, "Escuro");
    if app.prefs.theme != before {
        ctx.set_theme(app.prefs.theme);
    }
}

fn updates(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    if theme::checkbox(ui, 
            &mut app.prefs.check_updates,
            "Procurar atualizações ao abrir o LiveScreen",
        )
        .changed()
    {
        if app.prefs.check_updates {
            app.check_updates(ctx);
        } else {
            app.cancel_update_check();
        }
    }
    hint(
        ui,
        &format!("Versão instalada: {}", env!("CARGO_PKG_VERSION")),
    );
}

fn diagnostics(app: &mut App, ui: &mut egui::Ui) {
    if ui.button("Abrir pasta de logs").clicked()
        && std::process::Command::new("explorer.exe")
            .arg(crate::config::app_dir())
            .spawn()
            .is_err()
    {
        app.toasts
            .push("Não foi possível abrir a pasta de logs", ToastKind::Warn);
    }
    ui.add_space(8.0);

    // nunca fica sem backend: só desmarca um se o outro está efetivamente ligado
    let forced = app.overrides;
    let eff = app.effective_overrides();
    let gfx_ok = !app.prefs.graphics_capture || !eff.deny_dxgi;
    let dxgi_ok = !app.prefs.dxgi_capture || !eff.deny_graphics_capture;
    toggle(
        ui,
        &mut app.prefs.graphics_capture,
        "Usar Windows Graphics Capture (necessário para compartilhar janelas)",
        forced.deny_graphics_capture,
        gfx_ok,
    );
    toggle(
        ui,
        &mut app.prefs.dxgi_capture,
        "Usar Desktop Duplication (DXGI) como alternativa para telas",
        forced.deny_dxgi,
        dxgi_ok,
    );
    toggle_hint(ui, "Vale na próxima transmissão.");
}
