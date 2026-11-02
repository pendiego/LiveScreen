//! Chat de texto da sala. O host é o centro: atribui `sender` (0 é o host) e `role`
//! a cada mensagem; o nome é só rótulo e nunca dá privilégio. Linha única, sem histórico
//! para quem entra depois.

use std::collections::VecDeque;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::protocol::sanitize;

pub const MAX_CHAT_CHARS: usize = 500;
pub const CHAT_HISTORY: usize = 200;
pub const HOST_SENDER: u32 = 0;
const MAX_NAME_CHARS: usize = 32;
/// Teto de uma doação simulada (moedas fictícias).
pub const MAX_DONATE: u32 = 1000;
/// Valores oferecidos pelos clientes; o host aceita qualquer valor em 1..=MAX_DONATE.
pub const DONATE_AMOUNTS: [u32; 4] = [5, 10, 50, 100];

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    Host,
    Viewer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChatLine {
    pub sender: u32,
    pub role: ChatRole,
    pub name: String,
    pub friend: bool,
    pub text: String,
    /// Moedas da doação simulada; 0 = mensagem comum.
    #[serde(default)]
    pub donate: u32,
}

/// Limpa o texto e descarta o que ficar vazio.
pub fn clean_chat(text: &str) -> Option<String> {
    let t = sanitize(text, MAX_CHAT_CHARS);
    (!t.is_empty()).then_some(t)
}

impl ChatLine {
    pub fn from_viewer(sender: u32, name: &str, friend: bool, text: &str) -> Option<ChatLine> {
        Some(ChatLine {
            sender,
            role: ChatRole::Viewer,
            name: sanitize(name, MAX_NAME_CHARS),
            friend,
            text: clean_chat(text)?,
            donate: 0,
        })
    }

    pub fn from_host(name: &str, text: &str) -> Option<ChatLine> {
        Some(ChatLine {
            sender: HOST_SENDER,
            role: ChatRole::Host,
            name: sanitize(name, MAX_NAME_CHARS),
            friend: false,
            text: clean_chat(text)?,
            donate: 0,
        })
    }

    /// Limpa uma linha vinda da rede (fronteira de confiança do viewer).
    pub fn sanitized(self) -> Option<ChatLine> {
        Some(ChatLine {
            name: sanitize(&self.name, MAX_NAME_CHARS),
            text: clean_chat(&self.text)?,
            donate: self.donate.min(MAX_DONATE),
            ..self
        })
    }

    /// Mensagem vinda de um espectador. `donate > 0` só passa com a função ligada no host e
    /// dentro do teto; o texto da doação é montado aqui, para clientes antigos verem a linha.
    pub fn incoming(sender: u32, name: &str, friend: bool, text: &str, donate: u32, donate_enabled: bool) -> Option<ChatLine> {
        if donate == 0 {
            return Self::from_viewer(sender, name, friend, text);
        }
        if !donate_enabled || donate > MAX_DONATE {
            return None;
        }
        let msg = sanitize(text, MAX_CHAT_CHARS);
        let t = if msg.is_empty() { format!("doou {donate} moedas") } else { format!("doou {donate} moedas: {msg}") };
        Some(ChatLine { donate, ..Self::from_viewer(sender, name, friend, &t)? })
    }

    pub fn label(&self) -> String {
        match self.role {
            ChatRole::Host if self.name.is_empty() => "Anfitrião".into(),
            ChatRole::Host => self.name.clone(),
            ChatRole::Viewer => {
                let n = if self.name.is_empty() { "Anônimo" } else { &self.name };
                format!("{n} #{}", self.sender)
            }
        }
    }
}

#[derive(Default)]
pub struct ChatLog {
    lines: VecDeque<(u64, ChatLine)>,
    next_seq: u64,
}

impl ChatLog {
    pub fn push(&mut self, l: ChatLine) -> u64 {
        self.next_seq += 1;
        self.lines.push_back((self.next_seq, l));
        if self.lines.len() > CHAT_HISTORY {
            self.lines.pop_front();
        }
        self.next_seq
    }

    pub fn last_seq(&self) -> u64 {
        self.next_seq
    }

    pub fn since(&self, after: u64) -> impl Iterator<Item = (u64, &ChatLine)> {
        self.lines.iter().filter(move |(s, _)| *s > after).map(|(s, l)| (*s, l))
    }

    pub fn lines(&self) -> impl Iterator<Item = &ChatLine> {
        self.lines.iter().map(|(_, l)| l)
    }
}

/// Balde de fichas: rajada de 5, reposição de 2 por segundo.
pub struct ChatLimiter {
    tokens: f32,
    last: Option<Instant>,
}

impl Default for ChatLimiter {
    fn default() -> Self {
        ChatLimiter { tokens: 5.0, last: None }
    }
}

impl ChatLimiter {
    pub fn allow(&mut self, now: Instant) -> bool {
        if let Some(last) = self.last {
            let dt = now.saturating_duration_since(last).as_secs_f32();
            self.tokens = (self.tokens + dt * 2.0).min(5.0);
        }
        self.last = Some(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn limiter_burst_and_refill() {
        let mut l = ChatLimiter::default();
        let t = Instant::now();
        for _ in 0..5 {
            assert!(l.allow(t));
        }
        assert!(!l.allow(t));
        let t2 = t + Duration::from_millis(500);
        assert!(l.allow(t2));
        assert!(!l.allow(t2));
        let t3 = t2 + Duration::from_secs(10);
        for _ in 0..5 {
            assert!(l.allow(t3));
        }
        assert!(!l.allow(t3));
    }

    #[test]
    fn clean_chat_rules() {
        assert_eq!(clean_chat("  \u{7} "), None);
        assert_eq!(clean_chat(&"x".repeat(600)).unwrap().chars().count(), 500);
    }

    #[test]
    fn labels_never_impersonate_host() {
        let v = ChatLine::from_viewer(7, "Diego", false, "oi").unwrap();
        assert_eq!(v.role, ChatRole::Viewer);
        assert_eq!(v.label(), "Diego #7");
        assert_eq!(ChatLine::from_viewer(7, "", false, "oi").unwrap().label(), "Anônimo #7");
        assert_eq!(ChatLine::from_host("", "oi").unwrap().label(), "Anfitrião");
    }

    #[test]
    fn log_caps_and_since() {
        let mut log = ChatLog::default();
        for i in 0..205 {
            log.push(ChatLine::from_host("h", &format!("m{i}")).unwrap());
        }
        assert_eq!(log.lines().count(), 200);
        assert_eq!(log.last_seq(), 205);
        let s: Vec<u64> = log.since(203).map(|(s, _)| s).collect();
        assert_eq!(s, vec![204, 205]);
    }

    #[test]
    fn incoming_donation_rules() {
        let c = ChatLine::incoming(3, "Ana", false, "oi", 0, false).unwrap();
        assert_eq!(c.donate, 0);
        let d = ChatLine::incoming(3, "Ana", false, "", 50, true).unwrap();
        assert_eq!((d.text.as_str(), d.donate, d.role), ("doou 50 moedas", 50, ChatRole::Viewer));
        assert_eq!(ChatLine::incoming(3, "Ana", false, "valeu", 50, true).unwrap().text, "doou 50 moedas: valeu");
        assert!(ChatLine::incoming(3, "Ana", false, "", 50, false).is_none());
        assert!(ChatLine::incoming(3, "Ana", false, "", 1001, true).is_none());
        let mut l = ChatLine::from_host("h", "x").unwrap();
        l.donate = 5000;
        assert_eq!(l.sanitized().unwrap().donate, 1000);
    }
}
