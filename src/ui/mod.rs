mod home;
mod prefs;
mod settings;
mod share_view;
mod theme;
mod toast;
mod watch_view;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::Result;
use tokio::runtime::Handle;

use crate::capture::SourceId;
use crate::config::Overrides;
use crate::share::ShareSession;
use crate::video::encoder::EncoderKind;
use crate::video::present;
use crate::watch::WatchSession;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew, wgpu};
use windows::Win32::Foundation::{HWND, LPARAM, POINT};
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONULL, MonitorFromPoint};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GW_OWNER, GetWindow, IsWindowVisible, SetWindowDisplayAffinity,
    WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
};
use windows::core::BOOL;

// A sessão é parada em outra thread para não travar a UI.
const _: fn() = || {
    fn s<T: Send>() {}
    s::<ShareSession>();
    s::<WatchSession>();
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EndKind {
    Info,
    Warn,
    Error,
}

fn classify(m: &str) -> EndKind {
    match m {
        "Transmissão encerrada" | "Compartilhamento encerrado" | "A janela foi fechada" => {
            EndKind::Info
        }
        "Você foi removido" | "Sala cheia" => EndKind::Warn,
        _ if m.starts_with("Recusado") => EndKind::Warn,
        _ => EndKind::Error,
    }
}

pub enum Screen {
    Home,
    SharePicker,
    Sharing,
    WatchInput,
    Watching,
    Ended {
        message: String,
        kind: EndKind,
        retry: bool,
    },
}

pub struct App {
    pub rt: Handle,
    pub screen: Screen,
    pub overrides: Overrides,
    pub encoder: Arc<Mutex<Option<Vec<EncoderKind>>>>,
    pub picker: Option<share_view::Picker>,
    pub share: Option<ShareSession>,
    pub share_title: String,
    pub watch: Option<WatchSession>,
    pub watch_view: watch_view::WatchView,
    pub display: Option<present::Display>,
    pub ticket_input: String,
    pub error: Option<String>,
    pub prefs: prefs::Prefs,
    saved_prefs: prefs::Prefs,
    background: Vec<std::thread::JoinHandle<()>>,
    pub starting: Option<share_view::Starting>,
    pub share_started: Instant,
    pub share_source: Option<SourceId>,
    pub share_preview: Option<share_view::Preview>,
    pub known_viewers: HashMap<String, String>,
    pub toasts: toast::Toasts,
    pub kick_pending: Option<(String, Instant)>,
    pub copied_at: Option<Instant>,
    confirm_quit: bool,
    quit_confirmed: bool,
    title: String,
    own_hwnd: Option<isize>,
    capture_hidden: bool,
    /// Release mais nova que esta, achada na verificação ao abrir.
    pub update: Arc<Mutex<Option<crate::update::Release>>>,
    /// Download/instalação em andamento.
    pub updating: Option<std::sync::mpsc::Receiver<Result<()>>>,
    /// Popups da tela de transmissão.
    pub quality_open: bool,
    pub confirm_stop: bool,
    /// O código de convite desta transmissão já foi para a área de transferência.
    pub code_auto_copied: bool,
    /// Upload medido de quem transmite (uma vez por execução, ao abrir o seletor).
    pub upload: Arc<Mutex<crate::speedtest::Upload>>,
    /// Modal de Configurações (abre de qualquer tela).
    pub settings_open: bool,
    pub settings_tab: settings::Tab,
    update_check: Option<tokio::task::AbortHandle>,
}

impl App {
    fn new(rt: Handle, prefs: prefs::Prefs, ctx: egui::Context, display: Option<present::Display>) -> Self {
        crate::video::mf::startup();
        let overrides = crate::config::overrides();
        let encoder = Arc::new(Mutex::new(None));
        let update = Arc::new(Mutex::new(None));
        let mut app = App {
            rt,
            screen: Screen::Home,
            overrides,
            encoder,
            picker: None,
            share: None,
            share_title: String::new(),
            watch: None,
            watch_view: Default::default(),
            display,
            ticket_input: String::new(),
            error: None,
            saved_prefs: prefs.clone(),
            prefs,
            background: vec![],
            starting: None,
            share_started: Instant::now(),
            share_source: None,
            share_preview: None,
            known_viewers: HashMap::new(),
            toasts: Default::default(),
            kick_pending: None,
            copied_at: None,
            confirm_quit: false,
            quit_confirmed: false,
            title: "LiveScreen".into(),
            own_hwnd: None,
            capture_hidden: false,
            update,
            updating: None,
            quality_open: false,
            confirm_stop: false,
            code_auto_copied: false,
            upload: Arc::new(Mutex::new(crate::speedtest::Upload::Unknown)),
            settings_open: false,
            settings_tab: settings::Tab::Video,
            update_check: None,
        };
        app.detect_encoder();
        if app.prefs.check_updates {
            app.check_updates(&ctx);
        }
        ctx.set_theme(app.prefs.theme);
        app
    }

    /// `config.json`/env somados às preferências da tela de Configurações.
    pub fn effective_overrides(&self) -> Overrides {
        self.prefs.apply(self.overrides)
    }

    /// Refaz a detecção do encoder em segundo plano; `None` até terminar.
    pub fn detect_encoder(&self) {
        *self.encoder.lock().unwrap() = None;
        let (encoder, overrides) = (self.encoder.clone(), self.effective_overrides());
        std::thread::spawn(move || {
            let k = crate::video::probe::detect(&overrides);
            *encoder.lock().unwrap() = Some(k);
        });
    }

    /// Cancela a verificação em andamento, se houver, para ela não repor `update` depois.
    pub fn cancel_update_check(&mut self) {
        if let Some(t) = self.update_check.take() {
            t.abort();
        }
        *self.update.lock().unwrap() = None;
    }

    pub fn check_updates(&mut self, ctx: &egui::Context) {
        self.cancel_update_check();
        let (update, ctx) = (self.update.clone(), ctx.clone());
        let task = self.rt.spawn(async move {
            match crate::update::check().await {
                Ok(Some(r)) => {
                    tracing::info!("atualização disponível: {}", r.version);
                    *update.lock().unwrap() = Some(r);
                    ctx.request_repaint();
                }
                Ok(None) => {}
                Err(e) => tracing::info!("verificação de atualização falhou: {e:#}"),
            }
        });
        self.update_check = Some(task.abort_handle());
    }

    /// Exclui (ou devolve) a janela da captura; chamado também antes de criar a captura.
    fn sync_capture_exclusion(&mut self, want: bool) {
        if want == self.capture_hidden {
            return;
        }
        if self.own_hwnd.is_none() {
            self.own_hwnd = own_hwnd();
        }
        if let Some(h) = self.own_hwnd {
            let aff = if want {
                WDA_EXCLUDEFROMCAPTURE
            } else {
                WDA_NONE
            };
            if let Err(e) = unsafe { SetWindowDisplayAffinity(HWND(h as *mut _), aff) } {
                tracing::warn!("SetWindowDisplayAffinity: {e}");
            }
            // só marca com a janela achada: sem HWND, tenta de novo no próximo frame
            self.capture_hidden = want;
        }
    }

    pub fn save_prefs(&mut self) {
        if self.prefs != self.saved_prefs {
            self.prefs.save();
            self.saved_prefs = self.prefs.clone();
        }
    }

    /// Para as sessões em threads de fundo; a UI nunca bloqueia.
    pub fn stop_all(&mut self) {
        self.picker = None;
        self.share_preview = None;
        if let Some(s) = self.share.take() {
            self.background.push(std::thread::spawn(move || s.stop()));
        }
        if let Some(w) = self.watch.take() {
            self.background.push(std::thread::spawn(move || w.stop()));
        }
        self.background.retain(|h| !h.is_finished());
        self.watch_view = Default::default();
        if let Some(d) = self.display.as_mut() {
            d.reset();
        }
        self.kick_pending = None;
        self.copied_at = None;
        self.quality_open = false;
        self.confirm_stop = false;
        self.code_auto_copied = false;
        self.save_prefs();
    }

    pub fn end(&mut self, message: String, from_watch: bool) {
        self.stop_all();
        let kind = classify(&message);
        let retry = from_watch && kind != EndKind::Info && message != "Você foi removido";
        self.screen = Screen::Ended {
            message,
            kind,
            retry,
        };
    }

    fn quit_modal(&mut self, ctx: &egui::Context) {
        let mut cancel = false;
        let mut confirm = false;
        let mut focus_in = false;
        let modal = theme::modal(ctx, "quit").show(ctx, |ui| {
            ui.set_width(340.0);
            theme::heading(ui, "Encerrar a transmissão?");
            ui.label("Quem está assistindo será desconectado.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let keep = ui.button("Continuar transmitindo");
                focus_in |= keep.has_focus();
                if keep.clicked() {
                    cancel = true;
                }
                let quit = theme::danger_button(ui, "Encerrar e sair");
                focus_in |= quit.has_focus();
                if quit.clicked() {
                    confirm = true;
                }
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) || modal.should_close() {
            cancel = true;
        }
        // Enter é atalho de "Encerrar e sair" só sem foco num botão; com foco, ativa o botão
        if !focus_in && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            confirm = true;
        }
        if cancel {
            self.confirm_quit = false;
        } else if confirm {
            self.quit_confirmed = true;
            self.confirm_quit = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Mede o upload em segundo plano; não repete enquanto mede.
    pub fn measure_upload(&self, ctx: &egui::Context) {
        {
            let mut u = self.upload.lock().unwrap();
            if *u == crate::speedtest::Upload::Measuring {
                return;
            }
            *u = crate::speedtest::Upload::Measuring;
        }
        let (upload, ctx) = (self.upload.clone(), ctx.clone());
        self.rt.spawn(async move {
            let r = match crate::speedtest::measure_upload().await {
                Ok(bps) => {
                    tracing::info!("upload medido: {} kbps", bps / 1000);
                    crate::speedtest::Upload::Done(bps)
                }
                Err(e) => {
                    tracing::info!("medição de upload falhou: {e:#}");
                    crate::speedtest::Upload::Failed
                }
            };
            *upload.lock().unwrap() = r;
            ctx.request_repaint();
        });
    }

    /// Baixa, confere e instala em segundo plano; a UI só acompanha.
    pub fn start_update(&mut self, rel: crate::update::Release, ctx: &egui::Context) {
        if self.updating.is_some() {
            return;
        }
        self.save_prefs(); // a nova instância lê as preferências antes desta sair
        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = ctx.clone();
        self.rt.spawn(async move {
            let r = match crate::update::download(&rel).await {
                Ok(bytes) => tokio::task::spawn_blocking(move || crate::update::install(&bytes))
                    .await
                    .unwrap_or_else(|e| Err(e.into())),
                Err(e) => Err(e),
            };
            let _ = tx.send(r);
            ctx.request_repaint();
        });
        self.updating = Some(rx);
    }

    fn poll_update(&mut self, ctx: &egui::Context) {
        let Some(r) = self.updating.as_ref().and_then(|rx| rx.try_recv().ok()) else {
            return;
        };
        self.updating = None;
        match r {
            // a nova versão já está aberta; esta sai
            Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Err(e) => {
                tracing::warn!("atualização: {e:#}");
                self.toasts.push(
                    format!("Não foi possível atualizar: {e:#}"),
                    toast::ToastKind::Warn,
                );
            }
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        watch_view::sync_visible(self, ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested())
            && self.share.is_some()
            && !self.quit_confirmed
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_quit = true;
        }
        // Esc com Configurações abertas só fecha o modal: consome a tecla antes das telas
        // (senão "Voltar", sair da tela cheia etc. disparam por baixo).
        if self.settings_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            self.settings_open = false;
        }
        match self.screen {
            Screen::Home => home::home(self, ui),
            Screen::SharePicker => share_view::picker(self, ui),
            Screen::Sharing => share_view::sharing(self, ui),
            Screen::WatchInput => watch_view::input(self, ui),
            Screen::Watching => watch_view::watching(self, ui),
            Screen::Ended { .. } => home::ended(self, ui),
        }
        // único acesso às configurações em tela cheia
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Comma)) {
            self.settings_open = true;
        }
        if self.settings_open {
            settings::show(self, &ctx);
        }
        // some da captura enquanto compartilha (evita o espelho infinito)
        self.sync_capture_exclusion(self.share.is_some() || self.starting.is_some());
        if matches!(self.screen, Screen::Home) && self.encoder.lock().unwrap().is_none() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
        if self.confirm_quit {
            self.quit_modal(&ctx);
        }
        self.poll_update(&ctx);

        // não grava janela maximizada/minimizada/tela cheia: o tamanho normal vale
        let (normal, outer, inner) = ctx.input(|i| {
            let v = i.viewport();
            let normal = !v.fullscreen.unwrap_or(false)
                && !v.maximized.unwrap_or(false)
                && !v.minimized.unwrap_or(false);
            (normal, v.outer_rect, v.inner_rect)
        });
        if let (true, Some(o), Some(r)) = (normal, outer, inner) {
            self.prefs.window = Some([
                o.min.x.round(),
                o.min.y.round(),
                r.width().round(),
                r.height().round(),
            ]);
        }

        let title = match self.screen {
            Screen::Sharing => format!("LiveScreen — Compartilhando: {}", self.share_title),
            Screen::Watching => {
                let t = self
                    .watch
                    .as_ref()
                    .map(|w| w.status.lock().unwrap().title.clone())
                    .filter(|t| !t.is_empty());
                match t {
                    Some(t) => format!("LiveScreen — Assistindo: {t}"),
                    None => "LiveScreen — Assistindo".to_string(),
                }
            }
            _ => "LiveScreen".to_string(),
        };
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
        self.toasts.show(&ctx);
    }

    fn on_exit(&mut self) {
        self.stop_all();
        for h in self.background.drain(..) {
            let _ = h.join();
        }
        if let Some(st) = self.starting.take()
            && let Ok(Ok(s)) = st.rx.try_recv()
        {
            s.stop();
        }
    }
}

/// A barra de título precisa cair num monitor; senão o sistema escolhe a posição.
fn on_screen(x: f32, y: f32) -> bool {
    !unsafe {
        MonitorFromPoint(
            POINT {
                x: x as i32 + 40,
                y: y as i32 + 15,
            },
            MONITOR_DEFAULTTONULL,
        )
    }
    .is_invalid()
}

/// HWND visível, sem dono, da janela do eframe; só vale no thread da UI.
fn own_hwnd() -> Option<isize> {
    unsafe extern "system" fn cb(h: HWND, l: LPARAM) -> BOOL {
        unsafe {
            if IsWindowVisible(h).as_bool() && GetWindow(h, GW_OWNER).is_err() {
                *(l.0 as *mut Option<isize>) = Some(h.0 as isize);
                return BOOL(0);
            }
        }
        BOOL(1)
    }
    let mut out: Option<isize> = None;
    // EnumThreadWindows devolve FALSE quando o callback interrompe; o resultado vem de `out`.
    unsafe {
        let _ = EnumThreadWindows(
            GetCurrentThreadId(),
            Some(cb),
            LPARAM(&mut out as *mut Option<isize> as isize),
        );
    }
    out
}

pub fn run() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let handle = rt.handle().clone();
    let prefs = prefs::Prefs::load();
    let size = prefs
        .window
        .map(|w| [w[2].max(760.0), w[3].max(520.0)])
        .unwrap_or([1040.0, 720.0]);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("LiveScreen")
        .with_inner_size(size)
        .with_min_inner_size([760.0, 520.0])
        .with_icon(Arc::new(theme::app_icon()));
    if let Some(w) = prefs.window
        && on_screen(w[0], w[1])
    {
        viewport = viewport.with_position([w[0], w[1]]);
    }
    // DX12 é o backend do viewer sem cópia (`video::present`); WGPU_BACKEND continua valendo como escape.
    let mut setup = WgpuSetupCreateNew::without_display_handle();
    setup.instance_descriptor.backends = wgpu::Backends::from_env().unwrap_or(wgpu::Backends::DX12);
    let options = eframe::NativeOptions {
        viewport,
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(setup),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "LiveScreen",
        options,
        Box::new(move |cc| {
            theme::install(&cc.egui_ctx);
            let display = cc.wgpu_render_state.as_ref().and_then(crate::video::present::Display::new);
            Ok(Box::new(App::new(handle, prefs, cc.egui_ctx.clone(), display)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
