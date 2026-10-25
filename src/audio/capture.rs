//! Captura de áudio: loopback do processo compartilhado ou do sistema e/ou microfone,
//! mixados num único stream Opus.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, anyhow};
use bytes::Bytes;
use opus::{Application, Bitrate, Channels, Encoder};
use tracing::{debug, info, warn};
use wasapi::{
    AudioCaptureClient, AudioClient, DeviceEnumerator, Direction, Handle, SampleType, StreamMode,
    WaveFormat,
};

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

/// Fontes pedidas; pelo menos uma precisa abrir.
pub struct AudioRequest {
    /// Loopback do app (janela) ou do sistema (tela); `None` = sem loopback.
    pub loopback: Option<AudioTarget>,
    pub mic: Option<Mic>,
    /// `true` zera as amostras do microfone sem parar a captura.
    pub mic_muted: Arc<AtomicBool>,
}

/// Qual microfone abrir.
#[derive(Clone, Debug, PartialEq)]
pub enum Mic {
    Default,
    Device(String),
}

/// Resultado de cada fonte pedida; `None` = não pedida.
#[derive(Debug, Default)]
pub struct Opened {
    pub loopback: Option<Result<(), String>>,
    pub mic: Option<Result<(), String>>,
}

/// Tolerância de atraso da fonte que não é o relógio (100 ms).
const MAX_LAG_BYTES: usize = FRAME_BYTES * 5;

/// Inicia a captura numa thread própria. Bloqueia até a inicialização terminar,
/// para que erros (ex.: loopback de processo indisponível) cheguem ao chamador.
pub fn start(
    req: AudioRequest,
    out: tokio::sync::broadcast::Sender<Bytes>,
) -> anyhow::Result<(AudioHandle, Opened)> {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let (init_tx, init_rx) = std::sync::mpsc::channel::<Result<Opened, String>>();

    let join = std::thread::Builder::new()
        .name("audio-capture".into())
        .spawn(move || run(req, out, thread_stop, init_tx))
        .context("criar thread de captura de áudio")?;

    match init_rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(Ok(opened)) => Ok((
            AudioHandle {
                stop,
                join: Some(join),
            },
            opened,
        )),
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

/// Lista os microfones ativos como `(id, nome)`.
pub fn microphones() -> Vec<(String, String)> {
    let _ = wasapi::initialize_mta();
    let list = || -> anyhow::Result<Vec<(String, String)>> {
        let en = DeviceEnumerator::new()?;
        let col = en.get_device_collection(&Direction::Capture)?;
        Ok((&col)
            .into_iter()
            .filter_map(|d| {
                let d = d.ok()?;
                Some((d.get_id().ok()?, d.get_friendlyname().ok()?))
            })
            .collect())
    };
    list().unwrap_or_else(|e| {
        debug!("listar microfones falhou: {e:#}");
        vec![]
    })
}

fn run(
    req: AudioRequest,
    out: tokio::sync::broadcast::Sender<Bytes>,
    stop: Arc<AtomicBool>,
    init: std::sync::mpsc::Sender<Result<Opened, String>>,
) {
    // Vários threads chamam isto; apartamento já inicializado não é erro.
    let _ = wasapi::initialize_mta();
    let mut ready = false;
    let result = capture_loop(&req, &out, &stop, &init, &mut ready);
    if let Err(e) = result {
        if !ready {
            let _ = init.send(Err(format!("{e:#}")));
        } else {
            warn!("captura de áudio encerrada: {e:#}");
        }
    }
}

/// Uma fonte aberta: cliente WASAPI e a fila de bytes ainda não consumidos.
struct Source {
    client: AudioClient,
    event: Handle,
    capture: AudioCaptureClient,
    q: VecDeque<u8>,
    errors: u32,
}

fn open(mut client: AudioClient) -> anyhow::Result<Source> {
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
    Ok(Source {
        client,
        event,
        capture,
        q: VecDeque::new(),
        errors: 0,
    })
}

fn open_loopback(target: &AudioTarget) -> anyhow::Result<Source> {
    open(match target {
        AudioTarget::Process(pid) => AudioClient::new_application_loopback_client(*pid, true)
            .context("criar cliente de loopback do processo")?,
        AudioTarget::System => DeviceEnumerator::new()
            .context("criar enumerador de dispositivos")?
            .get_default_device(&Direction::Render)
            .context("obter dispositivo de reprodução padrão")?
            .get_iaudioclient()
            .context("abrir cliente de áudio do dispositivo")?,
    })
}

fn open_mic(mic: &Mic) -> anyhow::Result<Source> {
    let en = DeviceEnumerator::new().context("criar enumerador de dispositivos")?;
    let device = match mic {
        Mic::Device(id) => match en.get_device(id) {
            Ok(d) => d,
            Err(e) => {
                warn!("microfone escolhido indisponível ({e}); usando o padrão");
                en.get_default_device(&Direction::Capture)
                    .context("obter microfone padrão")?
            }
        },
        Mic::Default => en
            .get_default_device(&Direction::Capture)
            .context("obter microfone padrão")?,
    };
    open(device.get_iaudioclient().context("abrir microfone")?)
}

fn mic_error_text(e: &anyhow::Error) -> String {
    let denied = e.chain().any(|c| {
        matches!(
            c.downcast_ref::<wasapi::WasapiError>(),
            Some(wasapi::WasapiError::Windows(w)) if w.code().0 == 0x8007_0005_u32 as i32
        )
    });
    if denied {
        "acesso ao microfone negado em Configurações > Privacidade > Microfone".into()
    } else {
        format!("{e:#}")
    }
}

/// Lê o que o dispositivo tem. Depois de 20 falhas seguidas (≈ 4 s de falha real; um erro
/// isolado não deve derrubar o áudio) a fonte é desligada.
fn pump(slot: &mut Option<Source>, name: &str, last_err: &mut String) {
    let Some(s) = slot else { return };
    match s.capture.read_from_device_to_deque(&mut s.q) {
        Ok(_) => s.errors = 0,
        Err(e) => {
            s.errors += 1;
            if s.errors >= 20 {
                let _ = s.client.stop_stream();
                warn!("{name} desligado: {e}");
                *last_err = e.to_string();
                *slot = None;
            } else {
                debug!("leitura de áudio falhou ({name}): {e}");
            }
        }
    }
}

/// Copia até um quadro da fila, completando com silêncio.
fn take_frame(q: &mut VecDeque<u8>, out: &mut [f32; FRAME_FLOATS]) {
    let n = q.len().min(FRAME_BYTES);
    out.fill(0.0);
    for (dst, src) in out.iter_mut().zip(q.drain(..n).collect::<Vec<u8>>().chunks_exact(4)) {
        *dst = f32::from_le_bytes([src[0], src[1], src[2], src[3]]);
    }
}

fn mix(a: &mut [f32], b: &[f32]) {
    for (x, y) in a.iter_mut().zip(b) {
        *x = (*x + y).clamp(-1.0, 1.0);
    }
}

/// Descarta o mais antigo para a fila não passar de `max` bytes.
fn trim_lag(q: &mut VecDeque<u8>, max: usize) {
    if q.len() > max {
        q.drain(..q.len() - max);
    }
}

fn capture_loop(
    req: &AudioRequest,
    out: &tokio::sync::broadcast::Sender<Bytes>,
    stop: &Arc<AtomicBool>,
    init: &std::sync::mpsc::Sender<Result<Opened, String>>,
    ready: &mut bool,
) -> anyhow::Result<()> {
    let mut opened = Opened::default();
    let mut errors: Vec<String> = Vec::new();
    let mut loopback = None;
    let mut mic = None;
    if let Some(t) = &req.loopback {
        match open_loopback(t) {
            Ok(s) => {
                loopback = Some(s);
                opened.loopback = Some(Ok(()));
            }
            Err(e) => {
                let m = format!("{e:#}");
                errors.push(m.clone());
                opened.loopback = Some(Err(m));
            }
        }
    }
    if let Some(m) = &req.mic {
        match open_mic(m) {
            Ok(s) => {
                mic = Some(s);
                opened.mic = Some(Ok(()));
            }
            Err(e) => {
                let m = mic_error_text(&e);
                errors.push(m.clone());
                opened.mic = Some(Err(m));
            }
        }
    }
    if loopback.is_none() && mic.is_none() {
        return Err(anyhow!("{}", errors.join("; ")));
    }

    let mut encoder = Encoder::new(48_000, Channels::Stereo, Application::Audio)
        .map_err(|e| anyhow!("criar encoder Opus: {e}"))?;
    encoder
        .set_bitrate(Bitrate::Bits(128_000))
        .map_err(|e| anyhow!("configurar bitrate do Opus: {e}"))?;

    for s in loopback.iter().chain(mic.iter()) {
        s.client.start_stream().context("iniciar captura")?;
    }
    info!(
        "captura de áudio iniciada (Opus 48 kHz estéreo): loopback={} microfone={}",
        loopback.is_some(),
        mic.is_some()
    );
    *ready = true;
    let _ = init.send(Ok(opened));

    let mut frame = [0f32; FRAME_FLOATS];
    let mut extra = [0f32; FRAME_FLOATS];
    let mut packet = [0u8; OPUS_MAX_PACKET];
    let mut seq: u32 = 0;
    let mut last_err = String::new();

    while !stop.load(Ordering::Relaxed) {
        // O microfone entrega dados continuamente; o loopback para em silêncio.
        // Timeout é normal com áudio silencioso.
        if let Some(c) = mic.as_ref().or(loopback.as_ref()) {
            let _ = c.event.wait_for_event(200);
        }
        pump(&mut loopback, "loopback", &mut last_err);
        pump(&mut mic, "microfone", &mut last_err);
        if loopback.is_none() && mic.is_none() {
            return Err(anyhow!("leitura de áudio falhou repetidamente: {last_err}"));
        }

        let mic_is_clock = mic.is_some();
        let (clock, other) = if mic_is_clock {
            (&mut mic, &mut loopback)
        } else {
            (&mut loopback, &mut mic)
        };
        let Some(clock) = clock else { continue };
        if let Some(o) = other {
            trim_lag(&mut o.q, MAX_LAG_BYTES);
        }
        let muted = req.mic_muted.load(Ordering::Relaxed);

        while clock.q.len() >= FRAME_BYTES {
            take_frame(&mut clock.q, &mut frame);
            if mic_is_clock && muted {
                frame.fill(0.0);
            }
            if let Some(o) = other {
                take_frame(&mut o.q, &mut extra);
                // `other` é o microfone só quando o relógio é o loopback.
                if mic_is_clock || !muted {
                    mix(&mut frame, &extra);
                }
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

    for s in loopback.iter().chain(mic.iter()) {
        let _ = s.client.stop_stream();
    }
    debug!("captura de áudio encerrada");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_frame_pads_with_silence() {
        let mut q: VecDeque<u8> = [0.5f32, -0.5].iter().flat_map(|f| f.to_le_bytes()).collect();
        let mut out = [1f32; FRAME_FLOATS];
        take_frame(&mut q, &mut out);
        assert_eq!(out[0], 0.5);
        assert_eq!(out[1], -0.5);
        assert!(out[2..].iter().all(|&x| x == 0.0));
        assert!(q.is_empty());
    }

    #[test]
    fn mix_clamps() {
        let mut a = [0.8f32, -0.8];
        mix(&mut a, &[0.5, -0.5]);
        assert_eq!(a, [1.0, -1.0]);
    }

    #[test]
    fn trim_lag_keeps_newest() {
        let mut q: VecDeque<u8> = (0..48).collect();
        trim_lag(&mut q, 16);
        assert_eq!(q.iter().copied().collect::<Vec<_>>(), (32..48).collect::<Vec<u8>>());
    }

    #[test]
    #[ignore = "usa o microfone real: cargo test --release -- --ignored mic_capture"]
    fn mic_capture_produces_packets() {
        println!("microfones: {:?}", microphones());
        let mic_muted = Arc::new(AtomicBool::new(false));
        let (tx, mut rx) = tokio::sync::broadcast::channel(64);
        let (h, opened) = start(
            AudioRequest {
                loopback: None,
                mic: Some(Mic::Default),
                mic_muted: mic_muted.clone(),
            },
            tx,
        )
        .expect("abrir microfone padrão");
        assert_eq!(opened.mic, Some(Ok(())));
        assert!(opened.loopback.is_none());
        let drain = |rx: &mut tokio::sync::broadcast::Receiver<Bytes>| {
            let mut n = 0;
            while rx.try_recv().is_ok() {
                n += 1;
            }
            n
        };
        std::thread::sleep(std::time::Duration::from_secs(1));
        let n = drain(&mut rx);
        assert!(n >= 40, "pacotes sem mudo: {n}");
        mic_muted.store(true, Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_secs(1));
        let n = drain(&mut rx);
        assert!(n >= 40, "pacotes com mudo: {n}");
        h.stop();
    }
}
