#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod audio;
mod capture;
mod config;
mod gpu;
mod icon;
mod preset;
mod protocol;
mod share;
mod speedtest;
mod ui;
mod video;
mod update;
mod watch;

fn init_logging() {
    let path = config::app_dir().join("livescreen.log");
    if let Ok(f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = tracing_subscriber::fmt()
            .with_writer(std::sync::Mutex::new(f))
            .with_ansi(false)
            .try_init();
    }
}

fn main() -> anyhow::Result<()> {
    init_logging();
    match std::env::args().nth(1).as_deref() {
        Some("--probe-encoder") => video::probe::run_child(),
        Some("--selftest-capture") => capture::selftest(),
        _ => {
            update::cleanup();
            ui::run()
        }
    }
}

#[cfg(test)]
mod e2e_window;
