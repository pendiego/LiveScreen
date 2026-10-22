//! Reprodução do áudio recebido por datagram: Opus → buffer compartilhado → cpal.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opus::{Channels, Decoder};
use tracing::{debug, warn};

/// 60 ms de atraso inicial; acima de 200 ms o buffer é cortado.
const PREBUFFER_FLOATS: usize = 5760;
const MAX_BUFFER_FLOATS: usize = 19200;
/// Maior saída que o decodificador é instruído a produzir (60 ms estéreo); o `opus`
/// chama `opus_decode_float` com `len/2` amostras por canal, então nada além disso cabe.
const DECODE_FLOATS: usize = 5760;

pub struct PlaybackHandle {
    stop: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.task.abort();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// `gain`: ganho linear (bits de f32), ajustável durante a reprodução.
pub fn start(
    rt: &tokio::runtime::Handle,
    conn: iroh::endpoint::Connection,
    gain: Arc<AtomicU32>,
) -> anyhow::Result<PlaybackHandle> {
    let stop = Arc::new(AtomicBool::new(false));
    let samples: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));

    let task = rt.spawn(decode_loop(conn, samples.clone(), stop.clone()));

    let thread_stop = stop.clone();
    let thread = std::thread::Builder::new()
        .name("audio-out".into())
        .spawn(move || output_loop(samples, thread_stop, gain))?;

    Ok(PlaybackHandle {
        stop,
        task,
        thread: Some(thread),
    })
}

fn parse_packet(datagram: &[u8]) -> Option<(u32, &[u8])> {
    if datagram.len() < 4 {
        return None;
    }
    let seq = u32::from_le_bytes([datagram[0], datagram[1], datagram[2], datagram[3]]);
    Some((seq, &datagram[4..]))
}

async fn decode_loop(
    conn: iroh::endpoint::Connection,
    samples: Arc<Mutex<VecDeque<f32>>>,
    stop: Arc<AtomicBool>,
) {
    let mut decoder = match Decoder::new(48_000, Channels::Stereo) {
        Ok(decoder) => decoder,
        Err(e) => {
            warn!("criar decoder Opus falhou: {e}");
            return;
        }
    };
    let mut pcm = [0f32; DECODE_FLOATS];
    let mut expected: Option<u32> = None;

    while !stop.load(Ordering::Relaxed) {
        let datagram = match conn.read_datagram().await {
            Ok(datagram) => datagram,
            Err(_) => break,
        };
        let Some((seq, payload)) = parse_packet(&datagram) else {
            continue;
        };

        if let Some(previous) = expected {
            let gap = seq.wrapping_sub(previous);
            if gap >= 0x8000_0000 {
                // Pacote antigo/reordenado: descartar.
                continue;
            }
            // Lacuna curta: ocultação de perda (PLC) para cada pacote perdido.
            for _ in 0..gap.min(3) {
                if let Ok(n) = decoder.decode_float(&[], &mut pcm, false) {
                    push(&samples, &pcm[..n * 2]);
                }
            }
        }
        expected = Some(seq.wrapping_add(1));

        match decoder.decode_float(payload, &mut pcm, false) {
            Ok(n) => push(&samples, &pcm[..n * 2]),
            Err(e) => debug!("decodificar Opus falhou: {e}"),
        }
    }
    debug!("recepção de áudio encerrada");
}

fn push(samples: &Mutex<VecDeque<f32>>, data: &[f32]) {
    let Ok(mut queue) = samples.lock() else {
        return;
    };
    queue.extend(data.iter().copied());
    if queue.len() > MAX_BUFFER_FLOATS {
        let excess = queue.len() - PREBUFFER_FLOATS;
        queue.drain(..excess);
    }
}

fn output_loop(samples: Arc<Mutex<VecDeque<f32>>>, stop: Arc<AtomicBool>, gain: Arc<AtomicU32>) {
    let Some(device) = cpal::default_host().default_output_device() else {
        warn!("sem dispositivo de saída de áudio; seguindo sem som");
        return;
    };
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: 48_000,
        buffer_size: cpal::BufferSize::Default,
    };

    let mut playing = false;
    let stream = device.build_output_stream::<f32, _, _>(
        config,
        move |data: &mut [f32], _| {
            let Ok(mut queue) = samples.lock() else {
                data.fill(0.0);
                return;
            };
            if !playing {
                if queue.len() < PREBUFFER_FLOATS {
                    data.fill(0.0);
                    return;
                }
                playing = true;
            }
            let take = data.len().min(queue.len());
            let g = f32::from_bits(gain.load(Ordering::Relaxed));
            for (dst, src) in data[..take].iter_mut().zip(queue.drain(..take)) {
                *dst = src * g;
            }
            if take < data.len() {
                // Subrun: preenche com silêncio e volta a pré-bufferizar.
                data[take..].fill(0.0);
                playing = false;
            }
        },
        |e| warn!("erro no dispositivo de saída: {e}"),
        None,
    );

    let stream = match stream {
        Ok(stream) => stream,
        Err(e) => {
            warn!("criar saída de áudio falhou: {e}");
            return;
        }
    };
    if let Err(e) = stream.play() {
        warn!("iniciar saída de áudio falhou: {e}");
        return;
    }

    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    drop(stream);
    debug!("saída de áudio encerrada");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_packet_roundtrip() {
        let mut encoder =
            opus::Encoder::new(48_000, Channels::Stereo, opus::Application::Audio).unwrap();
        let mut decoder = Decoder::new(48_000, Channels::Stereo).unwrap();

        let sine: Vec<f32> = (0..960)
            .flat_map(|i| {
                let v = (i as f32 * 2.0 * std::f32::consts::PI * 440.0 / 48_000.0).sin() * 0.5;
                [v, v]
            })
            .collect();

        let mut packet = [0u8; 1500];
        let len = encoder.encode_float(&sine, &mut packet).unwrap();
        let mut framed = Vec::with_capacity(4 + len);
        framed.extend_from_slice(&7u32.to_le_bytes());
        framed.extend_from_slice(&packet[..len]);

        let (seq, payload) = parse_packet(&framed).unwrap();
        assert_eq!(seq, 7);

        let mut pcm = [0f32; DECODE_FLOATS];
        let n = decoder.decode_float(payload, &mut pcm, false).unwrap();
        assert_eq!(n, 960);
        let rms = (pcm[..n * 2].iter().map(|s| s * s).sum::<f32>() / (n * 2) as f32).sqrt();
        assert!(rms > 0.01, "sinal decodificado silencioso: {rms}");
    }
}
