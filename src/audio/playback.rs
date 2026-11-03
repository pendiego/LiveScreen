//! Reprodução do áudio recebido por datagram: Opus → buffer compartilhado → cpal.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, anyhow};

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

/// `device`: id de `outputs()`; `None` ou ausente = padrão do Windows.
/// `gain`: ganho linear (bits de f32), ajustável durante a reprodução.
pub fn start(
    rt: &tokio::runtime::Handle,
    conn: iroh::endpoint::Connection,
    gain: Arc<AtomicU32>,
    device: Option<String>,
) -> anyhow::Result<PlaybackHandle> {
    let stop = Arc::new(AtomicBool::new(false));
    let samples: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));

    let task = rt.spawn(decode_loop(conn, samples.clone(), stop.clone()));

    let thread_stop = stop.clone();
    let thread = std::thread::Builder::new()
        .name("audio-out".into())
        .spawn(move || output_loop(samples, thread_stop, gain, device))?;

    Ok(PlaybackHandle {
        stop,
        task,
        thread: Some(thread),
    })
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
    let mut frame = 960usize; // amostras por canal do último pacote (capture.rs: FRAME_SAMPLES)

    while !stop.load(Ordering::Relaxed) {
        let datagram = match conn.read_datagram().await {
            Ok(datagram) => datagram,
            Err(_) => break,
        };
        let Some((seq, payload)) = livescreen_core::protocol::parse_audio_datagram(&datagram) else {
            continue;
        };
        let Some(lost) = livescreen_core::protocol::audio_gap(&mut expected, seq) else {
            continue;
        };
        conceal(&mut decoder, &mut pcm, frame, lost, |s| push(&samples, s));

        match decoder.decode_float(payload, &mut pcm, false) {
            Ok(n) => {
                if n > 0 {
                    frame = n;
                }
                push(&samples, &pcm[..n * 2]);
            }
            Err(e) => debug!("decodificar Opus falhou: {e}"),
        }
    }
    debug!("recepção de áudio encerrada");
}

/// Compensa até 3 pacotes perdidos, cada um com a duração do último pacote bom: o Opus gera o
/// tamanho do buffer recebido, e o buffer inteiro (60 ms) inseriria 40 ms a mais por perda de 20 ms.
fn conceal(decoder: &mut Decoder, pcm: &mut [f32], frame: usize, lost: u32, mut emit: impl FnMut(&[f32])) {
    for _ in 0..lost.min(3) {
        if let Ok(n) = decoder.decode_float(&[], &mut pcm[..frame * 2], false) {
            emit(&pcm[..n * 2]);
        }
    }
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

fn open_output(
    device: &cpal::Device,
    samples: Arc<Mutex<VecDeque<f32>>>,
    gain: Arc<AtomicU32>,
    lost: Arc<AtomicBool>,
) -> anyhow::Result<cpal::Stream> {
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: 48_000,
        buffer_size: cpal::BufferSize::Default,
    };

    let mut playing = false;
    let stream = device
        .build_output_stream::<f32, _, _>(
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
            move |e: cpal::Error| match e.kind() {
                cpal::ErrorKind::DeviceNotAvailable | cpal::ErrorKind::StreamInvalidated => {
                    lost.store(true, Ordering::Relaxed)
                }
                cpal::ErrorKind::DeviceChanged => {
                    debug!("saída de áudio seguiu o novo dispositivo padrão")
                }
                _ => warn!("erro no dispositivo de saída: {e}"),
            },
            None,
        )
        .context("criar saída de áudio")?;
    stream.play().context("iniciar saída de áudio")?;
    Ok(stream)
}

/// Saídas de áudio como `(id, nome)`.
pub fn outputs() -> Vec<(String, String)> {
    let Ok(devices) = cpal::default_host().output_devices() else {
        return vec![];
    };
    devices
        .filter_map(|d| Some((d.id().ok()?.to_string(), d.description().ok()?.name().to_owned())))
        .collect()
}

/// A saída escolhida, ou a padrão quando não há escolha ou o dispositivo sumiu.
fn pick_output(device: &Option<String>) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if let Some(id) = device {
        match id.parse::<cpal::DeviceId>().ok().and_then(|id| host.device_by_id(&id)) {
            Some(d) => return Some(d),
            None => debug!("saída de áudio escolhida indisponível; usando a padrão"),
        }
    }
    host.default_output_device()
}

fn output_loop(
    samples: Arc<Mutex<VecDeque<f32>>>,
    stop: Arc<AtomicBool>,
    gain: Arc<AtomicU32>,
    device: Option<String>,
) {
    let mut warned = false; // um aviso por sequência de falhas, não um por segundo
    while !stop.load(Ordering::Relaxed) {
        let lost = Arc::new(AtomicBool::new(false));
        let opened = pick_output(&device)
            .ok_or_else(|| anyhow!("sem dispositivo de saída de áudio"))
            .and_then(|d| open_output(&d, samples.clone(), gain.clone(), lost.clone()));
        match opened {
            Ok(stream) => {
                warned = false;
                while !stop.load(Ordering::Relaxed) && !lost.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(50));
                }
                drop(stream);
                if lost.load(Ordering::Relaxed) {
                    warn!("saída de áudio perdida; reabrindo");
                }
            }
            Err(e) => {
                if !warned {
                    warn!("{e:#}; tentando de novo a cada 1 s");
                    warned = true;
                }
                for _ in 0..20 {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }
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

        let (seq, payload) = livescreen_core::protocol::parse_audio_datagram(&framed).unwrap();
        assert_eq!(seq, 7);

        let mut pcm = [0f32; DECODE_FLOATS];
        let n = decoder.decode_float(payload, &mut pcm, false).unwrap();
        assert_eq!(n, 960);
        let rms = (pcm[..n * 2].iter().map(|s| s * s).sum::<f32>() / (n * 2) as f32).sqrt();
        assert!(rms > 0.01, "sinal decodificado silencioso: {rms}");
    }

    #[test]
    fn conceal_matches_lost_duration() {
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
        let mut pcm = [0f32; DECODE_FLOATS];
        let n = decoder.decode_float(&packet[..len], &mut pcm, false).unwrap();
        assert_eq!(n, 960);

        for lost in [1, 2, 3] {
            let mut total = 0;
            conceal(&mut decoder, &mut pcm, 960, lost, |s| total += s.len());
            assert_eq!(total, lost as usize * 960 * 2);
        }

        let mut total = 0;
        conceal(&mut decoder, &mut pcm, 960, 5, |s| total += s.len());
        assert_eq!(total, 3 * 960 * 2);
    }
}
