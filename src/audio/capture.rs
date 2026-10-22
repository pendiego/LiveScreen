//! Captura de áudio: loopback do processo compartilhado ou do sistema, codificado em Opus.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, anyhow};
use bytes::Bytes;
use opus::{Application, Bitrate, Channels, Encoder};
use tracing::{debug, info, warn};
use wasapi::{AudioClient, DeviceEnumerator, Direction, SampleType, StreamMode, WaveFormat};

/// 20 ms a 48 kHz estéreo.
const FRAME_SAMPLES: usize = 960;
const FRAME_FLOATS: usize = FRAME_SAMPLES * 2;
const FRAME_BYTES: usize = FRAME_FLOATS * 4;
const OPUS_MAX_PACKET: usize = 1500;

pub enum AudioTarget {
    /// Loopback do processo (e de sua árvore de processos).
    Process(u32),
    /// Loopback do sistema inteiro (dispositivo de render padrão).
    System,
}

pub struct AudioHandle {
    stop: Arc<AtomicBool>,
    join: Option<std::thread::JoinHandle<()>>,
}

impl AudioHandle {
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for AudioHandle {
    /// Um handle abandonado (ex.: falha depois de iniciar a captura) não pode deixar a
    /// thread capturando e codificando para sempre.
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Inicia a captura numa thread própria. Bloqueia até a inicialização terminar,
/// para que erros (ex.: loopback de processo indisponível) cheguem ao chamador.
pub fn start(
    target: AudioTarget,
    out: tokio::sync::broadcast::Sender<Bytes>,
) -> anyhow::Result<AudioHandle> {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let (init_tx, init_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let join = std::thread::Builder::new()
        .name("audio-capture".into())
        .spawn(move || run(target, out, thread_stop, init_tx))
        .context("criar thread de captura de áudio")?;

    match init_rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(Ok(())) => Ok(AudioHandle {
            stop,
            join: Some(join),
        }),
        Ok(Err(e)) => {
            let _ = join.join();
            Err(anyhow!("{e}"))
        }
        // A inicialização demorou demais; o laço sai assim que ela terminar.
        Err(e) => {
            stop.store(true, Ordering::Relaxed);
            Err(anyhow!("inicialização do áudio sem resposta: {e}"))
        }
    }
}

fn run(
    target: AudioTarget,
    out: tokio::sync::broadcast::Sender<Bytes>,
    stop: Arc<AtomicBool>,
    init: std::sync::mpsc::Sender<Result<(), String>>,
) {
    // Vários threads chamam isto; apartamento já inicializado não é erro.
    let _ = wasapi::initialize_mta();
    let mut ready = false;
    let result = capture_loop(&target, &out, &stop, &init, &mut ready);
    if let Err(e) = result {
        if !ready {
            let _ = init.send(Err(format!("{e:#}")));
        } else {
            warn!("captura de áudio encerrada: {e:#}");
        }
    }
}

fn capture_loop(
    target: &AudioTarget,
    out: &tokio::sync::broadcast::Sender<Bytes>,
    stop: &Arc<AtomicBool>,
    init: &std::sync::mpsc::Sender<Result<(), String>>,
    ready: &mut bool,
) -> anyhow::Result<()> {
    let mut client = match target {
        AudioTarget::Process(pid) => AudioClient::new_application_loopback_client(*pid, true)
            .context("criar cliente de loopback do processo")?,
        AudioTarget::System => DeviceEnumerator::new()
            .context("criar enumerador de dispositivos")?
            .get_default_device(&Direction::Render)
            .context("obter dispositivo de reprodução padrão")?
            .get_iaudioclient()
            .context("abrir cliente de áudio do dispositivo")?,
    };

    let format = WaveFormat::new(32, 32, &SampleType::Float, 48_000, 2, None);
    // `buffer_duration_hns: 0` deixa o motor escolher; neste modo não se consulta
    // mixformat/buffer_size/padding.
    client
        .initialize_client(
            &format,
            &Direction::Capture,
            &StreamMode::EventsShared {
                autoconvert: true,
                buffer_duration_hns: 0,
            },
        )
        .context("inicializar cliente de captura")?;

    let event = client.set_get_eventhandle().context("criar evento")?;
    let capture = client
        .get_audiocaptureclient()
        .context("obter cliente de captura")?;

    let mut encoder = Encoder::new(48_000, Channels::Stereo, Application::Audio)
        .map_err(|e| anyhow!("criar encoder Opus: {e}"))?;
    encoder
        .set_bitrate(Bitrate::Bits(128_000))
        .map_err(|e| anyhow!("configurar bitrate do Opus: {e}"))?;

    client.start_stream().context("iniciar captura")?;
    *ready = true;
    let _ = init.send(Ok(()));
    info!("captura de áudio iniciada (Opus 48 kHz estéreo)");

    let mut raw_bytes: VecDeque<u8> = VecDeque::new();
    let mut frame = [0f32; FRAME_FLOATS];
    let mut packet = [0u8; OPUS_MAX_PACKET];
    let mut seq: u32 = 0;
    let mut buf = [0u8; FRAME_BYTES];
    let mut read_errors = 0u32;

    while !stop.load(Ordering::Relaxed) {
        // Timeout é normal com áudio silencioso.
        let _ = event.wait_for_event(200);
        if let Err(e) = capture.read_from_device_to_deque(&mut raw_bytes) {
            // Dispositivo inválido faz o erro repetir para sempre (e rápido); desistir é
            // melhor que girar em silêncio eternamente. 20 seguidas ≈ 4 s de falha real,
            // para um erro isolado não derrubar o áudio.
            read_errors += 1;
            if read_errors >= 20 {
                let _ = client.stop_stream();
                return Err(anyhow!("leitura de áudio falhou repetidamente: {e}"));
            }
            debug!("leitura de áudio falhou: {e}");
            continue;
        }
        read_errors = 0;

        while raw_bytes.len() >= FRAME_BYTES {
            let (a, b) = raw_bytes.as_slices();
            let n = a.len().min(FRAME_BYTES);
            buf[..n].copy_from_slice(&a[..n]);
            if n < FRAME_BYTES {
                buf[n..].copy_from_slice(&b[..FRAME_BYTES - n]);
            }
            raw_bytes.drain(..FRAME_BYTES);

            for (dst, src) in frame.iter_mut().zip(buf.chunks_exact(4)) {
                *dst = f32::from_le_bytes([src[0], src[1], src[2], src[3]]);
            }

            let len = match encoder.encode_float(&frame, &mut packet) {
                Ok(len) => len,
                Err(e) => {
                    warn!("encodar Opus falhou: {e}");
                    continue;
                }
            };
            let mut msg = Vec::with_capacity(4 + len);
            msg.extend_from_slice(&seq.to_le_bytes());
            msg.extend_from_slice(&packet[..len]);
            // Sem receptores ainda: o pacote só é descartado.
            let _ = out.send(Bytes::from(msg));
            seq = seq.wrapping_add(1);
        }
    }

    let _ = client.stop_stream();
    debug!("captura de áudio encerrada");
    Ok(())
}
