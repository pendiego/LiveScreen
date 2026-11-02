//! Avisos temporários no centro inferior, acima da barra de status ou da barra de chamada.

use std::time::{Duration, Instant};

use egui::{Align2, Margin, Order, RichText};

use super::theme::{self, I_CIRCLE_CHECK, I_INFO, I_WARN};

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

    pub fn show(&mut self, ctx: &egui::Context, above_call_bar: bool) {
        self.items.retain(|(_, _, t)| t.elapsed() < LIFETIME);
        let Some(oldest) = self.items.first().map(|i| i.2) else {
            return;
        };
        let skip = self.items.len().saturating_sub(MAX_VISIBLE);
        egui::Area::new(egui::Id::unique("toasts"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, -(if above_call_bar { theme::CALL_BAR_CLEARANCE } else { 56.0 })])
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let p = theme::P;
                ui.vertical_centered(|ui| {
                    for (text, kind, _) in &self.items[skip..] {
                        let (icon, color) = match kind {
                            ToastKind::Info => (I_INFO, p.accent),
                            ToastKind::Success => (I_CIRCLE_CHECK, p.success),
                            ToastKind::Warn => (I_WARN, p.warn),
                        };
                        egui::Frame::new()
                            .fill(p.raised)
                            .stroke(egui::Stroke::new(1.0, p.border))
                            .corner_radius(theme::R_MD)
                            .inner_margin(Margin::symmetric(14, 10))
                            .shadow(ui.visuals().popup_shadow)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;
                                    ui.label(RichText::new(icon).size(18.0).color(color));
                                    ui.label(RichText::new(text).text_style(egui::TextStyle::Body).color(p.text));
                                });
                            });
                    }
                });
            });
        ctx.request_repaint_after(LIFETIME.saturating_sub(oldest.elapsed()));
    }
}
