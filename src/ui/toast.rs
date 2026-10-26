//! Avisos temporários no centro inferior, acima da barra de status.

use std::time::{Duration, Instant};

use egui::{Align2, Margin, Order, RichText};

use super::theme::{self, I_CHECK, I_INFO, I_WARN};

const LIFETIME: Duration = Duration::from_secs(3);
const MAX_VISIBLE: usize = 4;

#[derive(Clone, Copy)]
pub enum ToastKind {
    Info,
    Success,
    Warn,
}

#[derive(Default)]
pub struct Toasts {
    items: Vec<(String, ToastKind, Instant)>,
}

impl Toasts {
    pub fn push(&mut self, text: impl Into<String>, kind: ToastKind) {
        self.items.push((text.into(), kind, Instant::now()));
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.items.retain(|(_, _, t)| t.elapsed() < LIFETIME);
        let Some(oldest) = self.items.first().map(|i| i.2) else {
            return;
        };
        let skip = self.items.len().saturating_sub(MAX_VISIBLE);
        egui::Area::new(egui::Id::unique("toasts"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, -56.0])
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let p = theme::palette(ui);
                ui.vertical_centered(|ui| {
                for (text, kind, _) in &self.items[skip..] {
                    let (icon, color) = match kind {
                        ToastKind::Info => (I_INFO, p.accent),
                        ToastKind::Success => (I_CHECK, p.success),
                        ToastKind::Warn => (I_WARN, p.warn),
                    };
                    egui::Frame::new()
                        .fill(ui.visuals().window_fill)
                        .stroke(egui::Stroke::new(1.0, p.border))
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(12, 8))
                        .shadow(ui.visuals().popup_shadow)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(icon).color(color));
                                ui.label(text);
                            });
                        });
                }
                });
            });
        ctx.request_repaint_after(LIFETIME.saturating_sub(oldest.elapsed()));
    }
}
