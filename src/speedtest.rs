//! Mede o upload de quem transmite, para recomendar quantas pessoas a internet aguenta.
//! Envia dados para o endpoint de teste da Cloudflare (o mesmo do speed.cloudflare.com).
//!
//! Só conta envios confirmados: cada POST tem tamanho fixo e vale quando o servidor responde
//! 2xx, ou seja, depois de receber o corpo inteiro. Contar o que o cliente HTTP puxa para os
//! buffers inflaria links lentos (os buffers do sistema engolem MBs) e aceitaria envios que
//! falharam no meio.
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use bytes::Bytes;

const URL: &str = "https://speed.cloudflare.com/__up";
/// Uma conexão só não enche links rápidos (51 Mbps sozinha contra 88 Mbps com 4, medido).
const STREAMS: usize = 4;
/// Envios que começam antes disso não contam: partida lenta do TCP.
const WARMUP: Duration = Duration::from_secs(1);
/// Depois de WARMUP + WINDOW nenhum envio novo começa.
const WINDOW: Duration = Duration::from_secs(3);
/// Tempo extra para os envios em curso terminarem; o que não terminar não conta.
const TAIL: Duration = Duration::from_secs(4);
/// Envios começam pequenos (link lento ainda confirma algum) e dobram até levar ~0,5 s.
const MIN_CHUNK: usize = 64 * 1024;
const MAX_CHUNK: usize = 16 * 1024 * 1024;
const TARGET: Duration = Duration::from_millis(500);
/// Teto de dados: em link de 1 Gbps o teste inteiro gastaria ~500 MB.
const MAX_BYTES: u64 = 150 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Upload {
    Unknown,
    Measuring,
    /// bits por segundo
    Done(u64),
    Failed,
}

/// Upload em bits por segundo. Leva ~4–5 s; qualquer envio com erro ou status não-2xx
/// derruba a medição inteira.
pub async fn measure_upload() -> Result<u64> {
    measure(URL).await
}

async fn measure(url: &str) -> Result<u64> {
    let client = crate::update::client()?;
    let zeros = Bytes::from(vec![0u8; MAX_CHUNK]);
    let budget = Arc::new(AtomicU64::new(MAX_BYTES));
    let start = Instant::now();
    // JoinSet aborta as outras conexões quando sai por erro
    let mut set = tokio::task::JoinSet::new();
    for _ in 0..STREAMS {
        set.spawn(worker(
            client.clone(),
            url.to_string(),
            zeros.clone(),
            budget.clone(),
            start,
        ));
    }
    // um intervalo só, do primeiro envio contado à última confirmação: somar a taxa de cada
    // conexão no próprio intervalo inflava (quem termina antes libera banda para as outras)
    let (mut bytes, mut span) = (0u64, None::<(Instant, Instant)>);
    while let Some(r) = set.join_next().await {
        let (b, s) = r??;
        bytes += b;
        if let Some((f, l)) = s {
            span = Some(span.map_or((f, l), |(f0, l0)| (f0.min(f), l0.max(l))));
        }
    }
    match span {
        Some((first, last)) if last > first && bytes > 0 => {
            Ok((bytes as f64 * 8.0 / (last - first).as_secs_f64()) as u64)
        }
        _ => bail!("nenhum envio de teste terminou a tempo"),
    }
}

/// Uma conexão enviando em sequência; devolve os bytes confirmados e o intervalo em que foram
/// enviados (do início do primeiro envio contado à resposta do último).
async fn worker(
    client: reqwest::Client,
    url: String,
    zeros: Bytes,
    budget: Arc<AtomicU64>,
    start: Instant,
) -> Result<(u64, Option<(Instant, Instant)>)> {
    let (count_from, stop_new) = (start + WARMUP, start + WARMUP + WINDOW);
    let hard = tokio::time::Instant::from_std(stop_new + TAIL);
    let (mut size, mut bytes) = (MIN_CHUNK, 0u64);
    let mut span: Option<(Instant, Instant)> = None;
    while Instant::now() < stop_new {
        let reserved = budget.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |b| {
            b.checked_sub(size as u64)
        });
        if reserved.is_err() {
            break;
        }
        let t0 = Instant::now();
        let req = client.post(&url).body(zeros.slice(..size)).send();
        let Ok(resp) = tokio::time::timeout_at(hard, req).await else {
            break; // não terminou a tempo: fica com o que já foi confirmado
        };
        resp?.error_for_status()?;
        let t1 = Instant::now();
        if t0 >= count_from {
            let first = span.map_or(t0, |(f, _)| f);
            span = Some((first, t1));
            bytes += size as u64;
        }
        if t1 - t0 < TARGET {
            size = (size * 2).min(MAX_CHUNK);
        }
    }
    Ok((bytes, span))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    #[derive(Clone, Copy)]
    enum Mode {
        /// lê o corpo a `n` bytes/s por conexão e responde 200
        Throttle(u64),
        /// responde com este status depois de ler o corpo
        Status(u16),
        /// fecha a conexão no meio do corpo
        CloseMidBody,
    }

    /// Servidor HTTP/1.1 mínimo (keep-alive, Content-Length) em 127.0.0.1.
    async fn serve(mode: Mode) -> String {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/__up", l.local_addr().unwrap());
        tokio::spawn(async move {
            while let Ok((s, _)) = l.accept().await {
                tokio::spawn(conn(s, mode));
            }
        });
        url
    }

    async fn conn(mut s: TcpStream, mode: Mode) {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let mut head = Vec::new();
            while !head.ends_with(b"\r\n\r\n") {
                let mut b = [0u8; 1];
                if s.read_exact(&mut b).await.is_err() {
                    return;
                }
                head.push(b[0]);
            }
            let head = String::from_utf8_lossy(&head).to_ascii_lowercase();
            let len: usize = head
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0);
            let mut left = len;
            while left > 0 {
                let cap = left.min(buf.len());
                let n = s.read(&mut buf[..cap]).await.unwrap_or(0);
                if n == 0 {
                    return;
                }
                left -= n;
                match mode {
                    Mode::CloseMidBody if len - left > 1024 => return,
                    Mode::Throttle(bps) => {
                        tokio::time::sleep(Duration::from_secs_f64(n as f64 / bps as f64)).await
                    }
                    _ => {}
                }
            }
            let status = match mode {
                Mode::Status(c) => c,
                _ => 200,
            };
            let resp = format!("HTTP/1.1 {status} X\r\ncontent-length: 0\r\n\r\n");
            if s.write_all(resp.as_bytes()).await.is_err() {
                return;
            }
        }
    }

    #[tokio::test]
    async fn http_error_fails_measurement() {
        let url = serve(Mode::Status(500)).await;
        assert!(measure(&url).await.is_err());
    }

    #[tokio::test]
    async fn connection_dropped_mid_upload_fails_measurement() {
        let url = serve(Mode::CloseMidBody).await;
        assert!(measure(&url).await.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn throttled_link_is_not_inflated_by_buffers() {
        // 4 conexões a 500 kB/s = 16 Mbps; contar bytes puxados para os buffers dava bem mais
        let url = serve(Mode::Throttle(500_000)).await;
        let bps = measure(&url).await.unwrap() as f64;
        let real = 4.0 * 500_000.0 * 8.0;
        assert!(bps > real * 0.6 && bps < real * 1.15, "{bps} vs {real}");
    }
}
