//! Lista de amigos locais, código de amigo e prober de presença.

use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{Align, Layout, RichText};
use livescreen_core::friends::{PresenceState, friend_code, parse_friend_code};
use livescreen_core::protocol::sanitize;
use serde::{Deserialize, Serialize};

use super::theme::{self, I_ADD_FRIEND, I_COPY, I_PEOPLE, I_PERSON, I_REMOVE};
use super::toast::ToastKind;
use super::{App, Screen};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Friend {
    pub name: String,
    pub code: String,
}

impl Friend {
    pub fn id(&self) -> Option<iroh::EndpointId> {
        parse_friend_code(&self.code)
    }
}

#[derive(Default, Clone, Debug)]
pub struct Draft {
    pub name: String,
    pub code: String,
    pub error: Option<String>,
}

pub fn load() -> Vec<Friend> {
    let path = crate::config::app_dir().join("friends.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            tracing::warn!("friends.json ilegível: {e}");
            let bad_path = crate::config::app_dir().join("friends.json.bad");
            let _ = std::fs::rename(&path, &bad_path);
            return Vec::new();
        }
    };
    let list: Vec<Friend> = match serde_json::from_str(&content) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("friends.json inválido: {e}");
            let bad_path = crate::config::app_dir().join("friends.json.bad");
            let _ = std::fs::rename(&path, &bad_path);
            return Vec::new();
        }
    };
    list.into_iter()
        .filter(|f| {
            if f.id().is_some() {
                true
            } else {
                tracing::warn!("amigo com código inválido descartado: {:?}", f.name);
                false
            }
        })
        .collect()
}

pub fn save(list: &[Friend]) -> Result<(), String> {
    let dir = crate::config::app_dir();
    let tmp = dir.join("friends.json.tmp");
    let target = dir.join("friends.json");
    serde_json::to_string_pretty(list)
        .map_err(|e| e.to_string())
        .and_then(|j| std::fs::write(&tmp, j).map_err(|e| e.to_string()))
        .and_then(|_| std::fs::rename(&tmp, target).map_err(|e| e.to_string()))
}

impl App {
    pub fn add_friend(&mut self, name: &str, id: iroh::EndpointId) {
        if self.identity_temporary {
            return;
        }
        if id == self.identity.public() {
            self.toasts
                .push("Esse é o seu próprio código de amigo", ToastKind::Warn);
            return;
        }
        let code = friend_code(&id);
        if let Some(existing) = self.friends.iter().find(|f| f.code == code) {
            self.toasts.push(
                format!("{} já está nos seus amigos", existing.name),
                ToastKind::Info,
            );
            return;
        }
        let s_name = sanitize(name.trim(), 32);
        let final_name = if s_name.is_empty() {
            "Amigo".to_string()
        } else {
            s_name
        };
        self.friends.push(Friend {
            name: final_name.clone(),
            code,
        });
        if let Err(e) = save(&self.friends) {
            self.toasts.push(
                format!("Não foi possível salvar a lista de amigos: {e}"),
                ToastKind::Warn,
            );
        }
        self.friends_changed();
        self.toasts.push(
            format!("{final_name} adicionado aos amigos. Peça para ele adicionar você também."),
            ToastKind::Success,
        );
    }

    pub fn remove_friend(&mut self, code: &str) {
        self.friends.retain(|f| f.code != code);
        if let Err(e) = save(&self.friends) {
            self.toasts.push(
                format!("Não foi possível salvar a lista de amigos: {e}"),
                ToastKind::Warn,
            );
        }
        self.friends_changed();
    }

    /// Ids dos amigos; vazio numa janela com identidade temporária.
    pub fn friend_ids(&self) -> Vec<iroh::EndpointId> {
        if self.identity_temporary {
            return Vec::new();
        }
        self.friends.iter().filter_map(Friend::id).collect()
    }

    fn friends_changed(&mut self) {
        let ids = self.friend_ids();
        if let Some(p) = self.prober.as_ref() {
            p.set_ids(ids.clone());
        }
        if let Some(s) = self.share.as_ref() {
            s.set_friends(ids.into_iter().collect());
        }
    }

    /// Único ponto que tira o prober do `App`; guarda o último estado conhecido.
    pub fn take_prober(&mut self) -> Option<livescreen_core::friends::Prober> {
        let p = self.prober.take()?;
        self.presence_cache = p.snapshot();
        Some(p)
    }

    /// Bloqueia até ~3,5 s: o endpoint anterior precisa fechar antes de abrir outro com a mesma chave.
    pub fn join_background(&mut self) {
        for h in std::mem::take(&mut self.background) {
            let _ = h.join();
        }
    }

    pub fn sync_prober(&mut self, ctx: &egui::Context) {
        self.background.retain(|h| !h.is_finished());
        let want = self.share.is_none()
            && self.watch.is_none()
            && self.starting.is_none()
            && !self.friends.is_empty()
            && !self.identity_temporary;

        if want {
            if self.prober.is_none() {
                if !self.background.is_empty() {
                    // endpoint antigo da mesma identidade ainda está fechando
                    ctx.request_repaint_after(Duration::from_millis(100));
                    return;
                }
                let recent = self
                    .prober_retry
                    .is_some_and(|t| t.elapsed() < Duration::from_secs(30));
                if !recent {
                    let ctx_clone = ctx.clone();
                    match livescreen_core::friends::Prober::start(
                        &self.rt,
                        self.identity.clone(),
                        self.friend_ids(),
                        Arc::new(move || ctx_clone.request_repaint()),
                    ) {
                        Ok(p) => {
                            self.prober = Some(p);
                            self.prober_retry = None;
                        }
                        Err(e) => {
                            tracing::warn!("não foi possível iniciar prober de presença: {e}");
                            self.prober_retry = Some(Instant::now());
                            ctx.request_repaint_after(Duration::from_secs(30));
                        }
                    }
                }
            }
        } else if let Some(p) = self.take_prober() {
            self.background.push(std::thread::spawn(move || p.stop()));
        }
    }
}

/// Estado a mostrar: o do prober; senão o último conhecido, para não voltar a "Verificando…".
fn presence_of(app: &App, id: &iroh::EndpointId) -> PresenceState {
    match app.prober.as_ref().map(|p| p.state(id)) {
        Some(s) if s != PresenceState::Unknown => s,
        _ => match app.presence_cache.get(id) {
            Some(s) => s.clone(),
            None if app.prober.is_none() && app.prober_retry.is_some() => PresenceState::Offline,
            None => PresenceState::Unknown,
        },
    }
}

/// Lista longa rola dentro do cartão, até `MAX_LIST` sem altura alvo.
const MAX_LIST: f32 = 240.0;

/// Cartão de amigos. `height`: altura total desejada (a lista estica ou rola para caber).
/// Devolve a altura natural, com a lista inteira, para quem alinha colunas.
pub fn card(app: &mut App, ui: &mut egui::Ui, height: Option<f32>) -> f32 {
    if app.identity_temporary {
        let r = theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            // 16 de margem e 1 de borda em cada lado
            ui.set_min_height(height.unwrap_or(0.0) - 34.0);
            theme::section_title(ui, format!("{I_PEOPLE}  Amigos"));
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "Outra janela do LiveScreen está usando seus amigos. Nesta janela, use códigos de convite.",
                )
                .weak(),
            );
        });
        return r.response.rect.height();
    }
    if let Some((_, t)) = app.friend_remove_pending {
        if t.elapsed() >= Duration::from_secs(3) {
            app.friend_remove_pending = None;
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }

    let mut remove_code: Option<String> = None;
    let mut watch_code: Option<String> = None;
    let mut open_add_modal = false;
    let p = theme::palette(ui);
    let my_code = friend_code(&app.identity.public());

    let rest_id = ui.make_persistent_id("friends_rest");
    let rest: f32 = ui.data(|d| d.get_temp(rest_id)).unwrap_or(0.0);
    let (mut list_area, mut list_content) = (0.0, 0.0);
    let frame = theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        {
            ui.horizontal(|ui| {
                theme::section_title(ui, format!("{I_PEOPLE}  Amigos"));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::small_button(ui, format!("{I_ADD_FRIEND}  Adicionar")).clicked() {
                        open_add_modal = true;
                    }
                });
            });
            ui.add_space(4.0);
            if app.friends.is_empty() {
                ui.label(
                    RichText::new(
                        "Amigos veem quando você transmite e entram sem convite. Troque códigos de amigo ou use o botão de amigo durante uma transmissão.",
                    )
                    .weak(),
                );
            }
        }
        // lista rola dentro do cartão; com altura alvo, o espaço que sobra fica antes do rodapé
        let max = height.map_or(MAX_LIST, |h| (h - rest).max(0.0));
        let list = egui::ScrollArea::vertical()
            .id_salt("friends_list")
            .auto_shrink([false, true])
            .max_height(max)
            // sem isto a área encolhe para a altura livre do pai (0 na coluna da home)
            .min_scrolled_height(max)
            .show(ui, |ui| {
            for friend in &app.friends {
                let presence = friend
                    .id()
                    .map_or(PresenceState::Unknown, |id| presence_of(app, &id));

                ui.horizontal(|ui| {
                    ui.label(format!("{I_PERSON}  {}", friend.name));
                    match &presence {
                        PresenceState::Live { .. } => {
                            theme::badge(ui, "Transmitindo", p.success);
                        }
                        PresenceState::Offline => {
                            theme::badge(ui, "Offline", ui.visuals().weak_text_color());
                        }
                        PresenceState::NotFriend => {
                            theme::badge(ui, "Ainda não te adicionou", p.warn).on_hover_text(
                                format!(
                                    "Peça para {} adicionar o seu código de amigo",
                                    friend.name
                                ),
                            );
                        }
                        PresenceState::Removed => {
                            theme::badge(ui, "Você foi removido", p.danger).on_hover_text(
                                format!(
                                    "{} removeu você desta transmissão. Na próxima você entra de novo.",
                                    friend.name
                                ),
                            );
                        }
                        PresenceState::Unknown => {
                            theme::badge(ui, "Verificando…", ui.visuals().weak_text_color());
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // só quem está ao vivo ganha ação; offline o badge já diz tudo
                        if matches!(presence, PresenceState::Live { .. })
                            && theme::primary_button(ui, "Assistir").clicked()
                        {
                            watch_code = Some(friend.code.clone());
                        }
                        let is_pending = app.friend_remove_pending.as_ref().map_or(false, |(c, t)| {
                            c == &friend.code && t.elapsed() < Duration::from_secs(3)
                        });
                        if is_pending {
                            if theme::danger_button(ui, "Confirmar?").clicked() {
                                remove_code = Some(friend.code.clone());
                                app.friend_remove_pending = None;
                            }
                        } else if theme::ghost_icon_button(ui, I_REMOVE, &format!("Remover {}", friend.name)).clicked() {
                            app.friend_remove_pending = Some((friend.code.clone(), Instant::now()));
                        }
                        if let PresenceState::Live { title } = &presence
                            && !title.is_empty()
                        {
                            ui.add(egui::Label::new(RichText::new(title).weak()).truncate());
                        }
                    });
                });
            }
        });
        let fill = if height.is_some() { (max - list.inner_rect.height()).max(0.0) } else { 0.0 };
        ui.add_space(fill);
        list_area = list.inner_rect.height() + fill;
        list_content = list.content_size.y;
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(RichText::new("Seu código de amigo").small().weak());
        let prefix = format!(
            "{}…{}",
            &my_code[..12.min(my_code.len())],
            &my_code[my_code.len().saturating_sub(4)..]
        );
        ui.horizontal(|ui| {
            let mut shown = prefix.as_str();
            ui.add(theme::field(
                ui,
                egui::TextEdit::singleline(&mut shown)
                    .font(egui::TextStyle::Monospace)
                    .interactive(false)
                    .desired_width(ui.available_width() - 44.0),
            ))
            .on_hover_text(&my_code);
            if theme::icon_button(ui, I_COPY, "Copiar código de amigo").clicked() {
                ui.ctx().copy_text(my_code.clone());
                app.toasts.push("Código de amigo copiado", ToastKind::Info);
            }
        });
    });
    // altura sem a área da lista + conteúdo da lista = altura natural do cartão
    let new_rest = frame.response.rect.height() - list_area;
    ui.data_mut(|d| d.insert_temp(rest_id, new_rest));
    if (new_rest - rest).abs() > 0.5 {
        ui.ctx().request_repaint();
    }
    let natural = new_rest + list_content;

    if open_add_modal {
        app.add_friend = Some(Draft::default());
    }
    if let Some(code) = remove_code {
        app.remove_friend(&code);
    }
    if let Some(code) = watch_code {
        let ctx = ui.ctx().clone();
        super::watch_view::connect(app, &ctx, code);
        if matches!(app.screen, Screen::Home) {
            if let Some(err) = app.error.take() {
                app.toasts.push(err, ToastKind::Warn);
            }
        }
    }
    natural
}

pub fn add_friend_modal(app: &mut App, ctx: &egui::Context) {
    let Some(draft) = app.add_friend.as_mut() else {
        return;
    };
    let mut cancel = false;
    let mut confirm = false;
    let mut focus_in = false;
    let modal = theme::modal(ctx, "add_friend").show(ctx, |ui| {
        ui.set_width(420.0);
        theme::heading(ui, "Adicionar amigo");
        ui.add_space(8.0);
        ui.label("Nome");
        ui.add(theme::field(
            ui,
            egui::TextEdit::singleline(&mut draft.name)
                .char_limit(32)
                .hint_text("Como você quer reconhecer o amigo")
                .desired_width(f32::INFINITY),
        ));
        ui.add_space(4.0);
        ui.label("Código de amigo");
        ui.add(theme::field(
            ui,
            egui::TextEdit::singleline(&mut draft.code)
                .font(egui::TextStyle::Monospace)
                .hint_text("Cole o código de 64 caracteres")
                .desired_width(f32::INFINITY),
        ));
        if let Some(err) = &draft.error {
            ui.add_space(4.0);
            ui.label(RichText::new(err).color(ui.visuals().error_fg_color));
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let cancel_btn = ui.button("Cancelar");
            focus_in |= cancel_btn.has_focus();
            if cancel_btn.clicked() {
                cancel = true;
            }
            let add_btn = theme::primary_button(ui, "Adicionar");
            focus_in |= add_btn.has_focus();
            if add_btn.clicked() {
                confirm = true;
            }
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) || modal.should_close() {
        cancel = true;
    }
    if !focus_in && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        confirm = true;
    }
    if cancel {
        app.add_friend = None;
    } else if confirm {
        let draft = app.add_friend.as_mut().unwrap();
        let name_trimmed = draft.name.trim();
        if name_trimmed.is_empty() {
            draft.error = Some("Dê um nome para reconhecer o amigo.".to_string());
        } else if let Some(id) = parse_friend_code(&draft.code) {
            let name = draft.name.clone();
            app.add_friend = None;
            app.add_friend(&name, id);
        } else {
            draft.error =
                Some("Código de amigo inválido: confira se copiou os 64 caracteres.".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friend_serialization_roundtrip() {
        let list = vec![Friend {
            name: "Ana".to_string(),
            code: "a".repeat(64),
        }];
        let json = serde_json::to_string(&list).unwrap();
        let parsed: Vec<Friend> = serde_json::from_str(&json).unwrap();
        assert_eq!(list, parsed);
    }

    #[test]
    fn friend_id_rejects_invalid_code() {
        let f = Friend {
            name: "Ruim".to_string(),
            code: "invalido".to_string(),
        };
        assert!(f.id().is_none());
    }
}
