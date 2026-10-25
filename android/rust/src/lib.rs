#![cfg(target_os = "android")]

//! Núcleo do LiveScreen para Android via JNI.
//!
//! Divisão de responsabilidades:
//! - Rust cuida da rede, do iroh, do protocolo e dos pedidos de keyframe/qualidade.
//! - Kotlin decodifica vídeo com o MediaCodec e desenha na SurfaceView, e decodifica/reproduz
//!   áudio com o MediaCodec/AudioTrack.
//!
//! Contrato: o Kotlin só chama `stop` depois de dar join nas threads de vídeo e áudio,
//! e não chama nada após `stop`.

use std::ffi::{CString, c_char, c_int};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, LazyLock, Mutex, Once};
use std::time::Duration;

use bytes::Bytes;
use iroh::endpoint::Connection;
use jni::JNIEnv;
use jni::objects::{JClass, JLongArray, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jbyteArray, jint, jlong, jstring};
use livescreen_core::preset::Preset;
use livescreen_core::protocol::{EncodedFrame, audio_gap, parse_audio_datagram};
use livescreen_core::viewer::{DecodeStats, StartOutputs, ViewerSession, is_valid_ticket};
use tokio::runtime::Runtime;
use tracing::warn;

#[link(name = "log")]
unsafe extern "C" {
    fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

struct Logcat(Vec<u8>);

impl Write for Logcat {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for Logcat {
    fn drop(&mut self) {
        if self.0.is_empty() {
            return;
        }
        while let Some(&b'\n') | Some(&b'\r') | Some(&0) = self.0.last() {
            self.0.pop();
        }
        self.0.retain(|&b| b != 0);
        if let Ok(c) = CString::new(self.0.clone()) {
            unsafe {
                __android_log_write(4, c"LiveScreen".as_ptr(), c.as_ptr());
            }
        }
    }
}

static INIT_LOGGING: Once = Once::new();
static RT: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().expect("runtime"));

fn init_logging() {
    INIT_LOGGING.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_writer(|| Logcat(Vec::new()))
            .with_ansi(false)
            .without_time()
            .try_init();
    });
}

struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct Session {
    viewer: ViewerSession,
    video: Mutex<Receiver<EncodedFrame>>,
    audio: Mutex<Receiver<Bytes>>,
    need_key: Arc<AtomicBool>,
    stats: Arc<DecodeStats>,
}

unsafe fn get_session<'a>(handle: jlong) -> Option<&'a Session> {
    if handle == 0 {
        None
    } else {
        unsafe { Some(&*(handle as *const Session)) }
    }
}

async fn audio_loop(conn: Connection, tx: SyncSender<Bytes>) {
    tracing::info!("audio_loop iniciou");
    let mut expected = None;
    let mut count = 0u64;
    while let Ok(d) = conn.read_datagram().await {
        let Some((seq, _)) = parse_audio_datagram(&d) else {
            continue;
        };
        if audio_gap(&mut expected, seq).is_none() {
            continue;
        }
        let _ = tx.try_send(d.slice(4..));
        count += 1;
        if count % 250 == 0 {
            tracing::info!("audio_loop: {count} datagramas de áudio entregues");
        }
    }
    tracing::info!("audio_loop encerrou, total: {count} datagramas");
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_isValidTicket(
    mut env: JNIEnv,
    _class: JClass,
    ticket: JString,
) -> jboolean {
    let Ok(ticket) = env.get_string(&ticket) else {
        return JNI_FALSE;
    };
    let s: String = ticket.into();
    if is_valid_ticket(&s) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_start(
    mut env: JNIEnv,
    _class: JClass,
    ticket: JString,
    name: JString,
) -> jlong {
    init_logging();
    let ticket_str: String = match env.get_string(&ticket) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            return 0;
        }
    };
    let name_str: String = match env.get_string(&name) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            return 0;
        }
    };

    let (vtx, vrx) = sync_channel::<EncodedFrame>(8);
    let (atx, arx) = sync_channel::<Bytes>(32);
    let need_key = Arc::new(AtomicBool::new(true));
    let stats = Arc::new(DecodeStats::default());

    let outputs: StartOutputs = Box::new(move |conn, audio| {
        let audio_guard = if audio {
            let conn = conn.clone();
            let task = RT.handle().spawn(audio_loop(conn, atx));
            Some(Box::new(AbortOnDrop(task)) as Box<dyn Send>)
        } else {
            None
        };
        (vtx, audio_guard)
    });

    let notify = Arc::new(|| {});
    match ViewerSession::start(
        RT.handle(),
        &ticket_str,
        &name_str,
        env!("CARGO_PKG_VERSION"),
        need_key.clone(),
        stats.clone(),
        notify,
        outputs,
    ) {
        Ok(viewer) => {
            let session = Box::new(Session {
                viewer,
                video: Mutex::new(vrx),
                audio: Mutex::new(arx),
                need_key,
                stats,
            });
            Box::into_raw(session) as jlong
        }
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_status(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return std::ptr::null_mut();
    };
    let json = match session.viewer.status.lock() {
        Ok(s) => serde_json::to_string(&*s).unwrap_or_default(),
        Err(_) => return std::ptr::null_mut(),
    };
    match env.new_string(json) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_presets(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let list: Vec<serde_json::Value> = Preset::ALL
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": serde_json::to_value(p).unwrap_or_default(),
                "name": p.name(),
            })
        })
        .collect();
    let json = serde_json::to_string(&list).unwrap_or_default();
    match env.new_string(json) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_nextVideo(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    timeout_ms: jint,
    meta: JLongArray,
) -> jbyteArray {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return std::ptr::null_mut();
    };
    let rx = match session.video.lock() {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };
    let timeout = Duration::from_millis(timeout_ms.max(0) as u64);
    match rx.recv_timeout(timeout) {
        Ok(frame) => {
            let meta_vals = [frame.pts_us as i64, frame.keyframe as i64];
            let _ = env.set_long_array_region(&meta, 0, &meta_vals);
            match env.byte_array_from_slice(&frame.data) {
                Ok(arr) => arr.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => std::ptr::null_mut(),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            std::thread::sleep(timeout);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_nextAudio(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    timeout_ms: jint,
) -> jbyteArray {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return std::ptr::null_mut();
    };
    let rx = match session.audio.lock() {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };
    let timeout = Duration::from_millis(timeout_ms.max(0) as u64);
    match rx.recv_timeout(timeout) {
        Ok(bytes) => match env.byte_array_from_slice(&bytes) {
            Ok(arr) => arr.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => std::ptr::null_mut(),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            std::thread::sleep(timeout);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_reportDecoded(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    frames: jint,
    busy_us: jlong,
    hardware: jboolean,
) {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return;
    };
    if frames > 0 {
        session.stats.frames.fetch_add(frames as u64, Ordering::Relaxed);
    }
    if busy_us > 0 {
        session.stats.busy_us.fetch_add(busy_us as u64, Ordering::Relaxed);
    }
    session.stats.hardware.store(hardware != 0, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_decoderFailed(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return;
    };
    session.stats.dead.store(true, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_requestKeyframe(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return;
    };
    session.need_key.store(true, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_requestQuality(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    id: JString,
) {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return;
    };
    let id_str: String = match env.get_string(&id) {
        Ok(s) => s.into(),
        Err(_) => return,
    };
    match serde_json::from_value::<Preset>(serde_json::Value::String(id_str)) {
        Ok(preset) => session.viewer.request_quality(preset),
        Err(e) => warn!("solicitação de qualidade inválida: {e}"),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_stop(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            let session = Box::from_raw(handle as *mut Session);
            drop(session);
        }
    }
}
