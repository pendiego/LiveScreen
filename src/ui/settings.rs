//! Configurações do app: modal aberto de qualquer tela; tudo vai para ui.json.
use std::time::Duration;

use egui::{Align, Layout, RichText};

use super::App;
use super::share_view::{MAX_ROOM, preset_hint};
use super::theme::{self, I_MIC, I_SETTINGS, I_VOLUME};
use super::toast::ToastKind;
use livescreen_core::preset::Preset;

const FORCED: &str = "Desligado por config.json ou LIVESCREEN_CAPTURE_OVERRIDES";

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Video,
    Share,
    Updates,
    Diagnostics,
}

const TABS: [(Tab, &str); 4] = [
    (Tab::Video, "Vídeo"),
    (Tab::Share, "Transmissão"),
    (Tab::Updates, "Atualizações"),
    (Tab::Diagnostics, "Diagnóstico"),
];

pub const SETTINGS_LABEL: &str = "Configurações (Ctrl+,)";

/// Engrenagem; devolve true no clique (para telas que seguram `app` emprestado).
pub fn icon_button(ui: &mut egui::Ui) -> bool {
    theme::icon_button(ui, I_SETTINGS, SETTINGS_LABEL).clicked()
}

pub fn button(app: &mut App, ui: &mut egui::Ui) {
    if theme::outline_icon_button(ui, I_SETTINGS, SETTINGS_LABEL).clicked() {
        app.settings_open = true;
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

const TAB_HEIGHT: f32 = 340.0;

pub fn show(app: &mut App, ctx: &egui::Context) {
    let mut close = false;
    let modal = theme::modal(ctx, "settings").show(ctx, |ui| {
        ui.set_width(560.0);
        theme::heading(ui, "Configurações");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (t, label) in TABS {
                if theme::tab(ui, app.settings_tab == t, label).clicked() {
                    app.settings_tab = t;
                }
            }
        });
        ui.add_space(16.0);
        // Viewport de altura fixa (limitada pela janela): o modal nunca muda de tamanho, e conteúdo maior rola.
        let h = TAB_HEIGHT.min(ctx.content_rect().height() - 260.0).max(120.0);
        egui::ScrollArea::vertical()
            .id_salt(("settings_tab", app.settings_tab as u8))
            .auto_shrink([false, false])
            .min_scrolled_height(h)
            .max_height(h)
            .show(ui, |ui| match app.settings_tab {
                Tab::Video => video(app, ui),
                Tab::Share => share(app, ui),
                Tab::Updates => updates(app, ui, ctx),
                Tab::Diagnostics => diagnostics(app, ui),
            });
        ui.add_space(16.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if theme::button(ui, theme::Kind::Secondary, "Fechar").clicked() {
                close = true;
            }
        });
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
    ui.add_space(4.0);
    toggle(
        ui,
        &mut app.prefs.zero_copy,
        "Mostrar o vídeo direto da placa de vídeo, sem cópia",
        forced.deny_zero_copy,
        app.prefs.hardware_decode && !forced.deny_hardware_decoder,
    );
    toggle_hint(
        ui,
        "Vale na próxima vez que você assistir. Desligue se a imagem ficar preta ou piscando: o quadro passa a ser copiado pelo processador.",
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
    ui.horizontal(|ui| {
        ui.label("Qualidade padrão");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            egui::ComboBox::from_id_salt("default_quality")
                .width(220.0)
                .icon(theme::chevron_icon)
                .selected_text(pr.name())
                .show_ui(ui, |ui| {
                    for p in Preset::ALL.into_iter().rev() {
                        ui.selectable_value(&mut pr, p, p.name())
                            .on_hover_text(preset_hint(p));
                    }
                });
        });
    });
    if pr != app.prefs.preset {
        app.prefs.preset = pr;
        if let Some(p) = app.picker.as_mut() {
            p.preset = pr;
        }
    }
    ui.add_space(8.0);

    if theme::checkbox(
        ui,
        &mut app.prefs.share_audio,
        format!("{I_VOLUME}  Transmitir áudio"),
    )
    .changed()
        && let Some(p) = app.picker.as_mut()
    {
        p.audio = app.prefs.share_audio;
    }

    if theme::checkbox(
        ui,
        &mut app.prefs.share_mic,
        format!("{I_MIC}  Transmitir microfone"),
    )
    .changed()
        && let Some(p) = app.picker.as_mut()
    {
        p.mic = app.prefs.share_mic;
    }

    ui.add_space(4.0);
    theme::checkbox(
        ui,
        &mut app.prefs.auto_quality,
        "Baixar a qualidade sozinho quando o PC não der conta",
    );
    toggle_hint(
        ui,
        "Vale na hora, inclusive com a transmissão no ar. Só reduz; para subir, escolha a qualidade.",
    );
    ui.add_space(4.0);
    theme::checkbox(
        ui,
        &mut app.prefs.accept_quality_requests,
        "Aceitar pedidos de qualidade mais leve de quem assiste",
    );
    toggle_hint(ui, "Desligado, os pedidos só aparecem como aviso.");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Limite da sala");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if theme::stepper(ui, &mut app.prefs.room_limit, 1, MAX_ROOM, "")
                && let Some(s) = &app.share
            {
                s.set_max_viewers(app.prefs.room_limit);
            }
        });
    });
    ui.add_space(8.0);

    theme::section_label(ui, "Seu nome");
    ui.add_space(4.0);
    ui.add(theme::field(
        ui,
        egui::TextEdit::singleline(&mut app.prefs.name)
            .char_limit(32)
            .hint_text("Como você aparece para quem compartilha")
            .desired_width(f32::INFINITY),
    ));
    ui.add_space(4.0);
    hint(
        ui,
        "A qualidade e o áudio também mudam quando você escolhe ao iniciar uma transmissão.",
    );
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
    if theme::button(ui, theme::Kind::Secondary, "Abrir pasta de logs").clicked() {
        open_logs(app);
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

pub fn open_logs(app: &mut App) {
    if std::process::Command::new("explorer.exe")
        .arg(crate::config::app_dir())
        .spawn()
        .is_err()
    {
        app.toasts
            .push("Não foi possível abrir a pasta de logs", ToastKind::Warn);
    }
}

pub fn open_help(app: &mut App) {
    if std::process::Command::new("explorer.exe")
        .arg("https://github.com/pendiego/LiveScreen#readme")
        .spawn()
        .is_err()
    {
        app.toasts
            .push("Não foi possível abrir a ajuda", ToastKind::Warn);
    }
}
