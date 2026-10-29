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
//! `presenceStop` só é chamado pela mesma corrotina que usa o handle; nada mais o usa depois.

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
use livescreen_core::friends::{Prober, friend_code, parse_friend_code, parse_secret_key};
use livescreen_core::preset::Preset;
use livescreen_core::protocol::{EncodedFrame, audio_gap, find_invite, parse_audio_datagram, parse_invite};
use livescreen_core::viewer::{DecodeStats, StartOutputs, ViewerSession, is_valid_invite};
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
    if is_valid_invite(&s) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_isLegacyInvite(
    mut env: JNIEnv,
    _class: JClass,
    ticket: JString,
) -> jboolean {
    let Ok(ticket) = env.get_string(&ticket) else {
        return JNI_FALSE;
    };
    let s: String = ticket.into();
    if matches!(parse_invite(&s), Some((_, None))) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_findInvite(
    mut env: JNIEnv,
    _class: JClass,
    text: JString,
) -> jstring {
    let Ok(text) = env.get_string(&text) else {
        return std::ptr::null_mut();
    };
    let s: String = text.into();
    let Some(inv) = find_invite(&s) else {
        return std::ptr::null_mut();
    };
    match env.new_string(inv) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_start(
    mut env: JNIEnv,
    _class: JClass,
    ticket: JString,
    name: JString,
    identity: JString,
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
    let identity_str: String = match env.get_string(&identity) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            return 0;
        }
    };
    let secret_key = match parse_secret_key(&identity_str) {
        Some(k) => k,
        None => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "Identidade inválida");
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
        secret_key,
        &name_str,
        env!("CARGO_PKG_VERSION"),
        // o MediaCodec do app só decodifica video/avc
        vec![livescreen_core::protocol::Codec::H264],
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
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_friendCode(
    mut env: JNIEnv,
    _class: JClass,
    identity: JString,
) -> jstring {
    let Ok(identity) = env.get_string(&identity) else {
        return std::ptr::null_mut();
    };
    let identity_str: String = identity.into();
    match parse_secret_key(&identity_str) {
        Some(key) => {
            let code = friend_code(&key.public());
            match env.new_string(code) {
                Ok(js) => js.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_isValidFriendCode(
    mut env: JNIEnv,
    _class: JClass,
    code: JString,
) -> jboolean {
    let Ok(code) = env.get_string(&code) else {
        return JNI_FALSE;
    };
    let s: String = code.into();
    if parse_friend_code(&s).is_some() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_init(
    env: JNIEnv,
    _class: JClass,
    context: jni::objects::JObject,
) {
    init_logging();
    let vm = match env.get_java_vm() {
        Ok(vm) => vm,
        Err(_) => return,
    };
    let vm_ptr = vm.get_java_vm_pointer() as *mut std::ffi::c_void;
    let ctx_ref = match env.new_global_ref(context) {
        Ok(r) => r,
        Err(_) => return,
    };
    let ctx_ptr = ctx_ref.as_raw() as *mut std::ffi::c_void;
    std::mem::forget(ctx_ref);
    unsafe {
        ndk_context::initialize_android_context(vm_ptr, ctx_ptr);
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_presenceStart(
    mut env: JNIEnv,
    _class: JClass,
    identity: JString,
    codes: JString,
) -> jlong {
    init_logging();
    let identity_str: String = match env.get_string(&identity) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            return 0;
        }
    };
    let key = match parse_secret_key(&identity_str) {
        Some(k) => k,
        None => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "Identidade inválida");
            return 0;
        }
    };
    let codes_str: String = match env.get_string(&codes) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            return 0;
        }
    };
    let ids: Vec<_> = codes_str
        .split(',')
        .filter_map(parse_friend_code)
        .collect();

    match Prober::start(RT.handle(), key, ids, Arc::new(|| {})) {
        Ok(prober) => Box::into_raw(Box::new(prober)) as jlong,
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_presenceSetIds(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    codes: JString,
) {
    if handle == 0 {
        return;
    }
    let Ok(codes) = env.get_string(&codes) else {
        return;
    };
    let codes_str: String = codes.into();
    let ids: Vec<_> = codes_str
        .split(',')
        .filter_map(parse_friend_code)
        .collect();
    unsafe {
        let prober = &*(handle as *const Prober);
        prober.set_ids(ids);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_presenceStatus(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    if handle == 0 {
        return match env.new_string("{}") {
            Ok(js) => js.into_raw(),
            Err(_) => std::ptr::null_mut(),
        };
    }
    let json = unsafe {
        let prober = &*(handle as *const Prober);
        prober.states_json()
    };
    match env.new_string(json) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_presenceStop(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    unsafe {
        let prober = Box::from_raw(handle as *mut Prober);
        drop(prober);
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
        .rev()
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
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_setVisible(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    visible: jboolean,
) {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return;
    };
    let v = visible != 0;
    // quadros enfileirados ficaram velhos; o decoder já parou (o Kotlin dá join antes)
    if let Ok(rx) = session.video.lock() {
        while rx.try_recv().is_ok() {}
    }
    // Escondido não pede keyframe: o key_loop pediria um por segundo à sala toda.
    // Ao voltar, o viewer descarta até o próximo keyframe e pede um.
    session.need_key.store(v, Ordering::SeqCst);
    session.viewer.set_visible(v);
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
