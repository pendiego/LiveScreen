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
//! No modo host, o Kotlin entrega H.264 Annex B e Opus já codificados. `hostPush*`, `hostTakeKeyframe`
//! e `hostTargetBps` só são chamados pelas threads dos encoders. `hostStop` só depois de dar join
//! nessas threads e de cancelar e esperar a corrotina de status; nada é chamado depois dele.

use std::ffi::{CString, c_char, c_int};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, LazyLock, Mutex, Once};
use std::time::Duration;

use bytes::Bytes;
use iroh::endpoint::Connection;
use jni::JNIEnv;
use jni::objects::{JClass, JLongArray, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jbyteArray, jint, jlong, jstring};
use livescreen_core::friends::{Prober, friend_code, parse_friend_code, parse_secret_key};
use livescreen_core::chat::ChatRole;
use livescreen_core::host::{AudioState, Feeds, Host, HostConfig, ShareStatus, WebState};
use livescreen_core::preset::{EncodeParams, Preset, encode_params, is_reduction};
use livescreen_core::protocol::{
    Codec, EncodedFrame, MAX_FRAME, audio_gap, find_invite, parse_audio_datagram, parse_invite,
};
use livescreen_core::viewer::{DecodeStats, StartOutputs, ViewerSession, is_valid_invite};
use tokio::runtime::Runtime;
use tracing::{info, warn};

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
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_sendChat(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    text: JString,
) {
    let Some(session) = (unsafe { get_session(handle) }) else {
        return;
    };
    let text: String = match env.get_string(&text) {
        Ok(s) => s.into(),
        Err(_) => return,
    };
    session.viewer.send_chat(&text);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_chatSince(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    after: jlong,
) -> jstring {
    let json = match unsafe { get_session(handle) } {
        None => "[]".to_string(),
        Some(session) => {
            let chat_id = session.viewer.status.lock().map(|s| s.chat_id).unwrap_or(0);
            let items: Vec<serde_json::Value> = match session.viewer.chat.lock() {
                Ok(log) => log
                    .since(after.max(0) as u64)
                    .map(|(seq, l)| {
                        serde_json::json!({
                            "seq": seq,
                            "label": l.label(),
                            "host": l.role == ChatRole::Host,
                            "friend": l.friend,
                            "mine": l.role == ChatRole::Viewer && l.sender == chat_id,
                            "text": l.text,
                        })
                    })
                    .collect(),
                Err(_) => Vec::new(),
            };
            serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())
        }
    };
    match env.new_string(json) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
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

// ---- Modo host: o Kotlin captura e codifica, o núcleo transmite ----

struct HostSession {
    host: Host,
    feeds: Feeds,
    video_seq: AtomicU64,
    audio_seq: AtomicU32,
    current: Mutex<(Preset, EncodeParams)>,
    src: (u32, u32, u32),
    audio_on: AtomicBool,
}

unsafe fn get_host<'a>(handle: jlong) -> Option<&'a HostSession> {
    if handle == 0 { None } else { Some(unsafe { &*(handle as *const HostSession) }) }
}

fn jstr(env: &mut JNIEnv, s: &JString) -> Option<String> {
    env.get_string(s).ok().map(Into::into)
}

fn parse_codes(codes: &str) -> std::collections::HashSet<iroh::EndpointId> {
    codes.split(',').filter_map(parse_friend_code).collect()
}

fn parse_preset(id: &str) -> Option<Preset> {
    serde_json::from_value(serde_json::Value::String(id.to_string())).ok()
}

fn preset_id(p: Preset) -> serde_json::Value {
    serde_json::to_value(p).unwrap_or_default()
}

fn new_jstring(env: &JNIEnv, s: String) -> jstring {
    env.new_string(s).map(|j| j.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_encodeParams(
    mut env: JNIEnv,
    _class: JClass,
    preset: JString,
    src_w: jint,
    src_h: jint,
    hz: jint,
) -> jstring {
    let Some(preset) = jstr(&mut env, &preset).as_deref().and_then(parse_preset) else {
        let _ = env.throw_new("java/lang/IllegalArgumentException", "Preset inválido");
        return std::ptr::null_mut();
    };
    let p = encode_params(preset, src_w.max(0) as u32, src_h.max(0) as u32, hz.max(0) as u32);
    new_jstring(
        &env,
        serde_json::json!({"width": p.width, "height": p.height, "fps": p.fps, "bitrate": p.bitrate_bps})
            .to_string(),
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostStart(
    mut env: JNIEnv,
    _class: JClass,
    identity: JString,
    name: JString,
    title: JString,
    codes: JString,
    max_viewers: jint,
    preset: JString,
    src_w: jint,
    src_h: jint,
    hz: jint,
    width: jint,
    height: jint,
    fps: jint,
    bitrate: jint,
    encoder_name: JString,
    audio_on: jboolean,
    audio_error: JString,
) -> jlong {
    init_logging();
    let (Some(identity), Some(name), Some(title), Some(codes), Some(preset), Some(encoder), Some(audio_error)) = (
        jstr(&mut env, &identity),
        jstr(&mut env, &name),
        jstr(&mut env, &title),
        jstr(&mut env, &codes),
        jstr(&mut env, &preset),
        jstr(&mut env, &encoder_name),
        jstr(&mut env, &audio_error),
    ) else {
        let _ = env.throw_new("java/lang/IllegalArgumentException", "Texto inválido");
        return 0;
    };
    let Some(key) = parse_secret_key(&identity) else {
        let _ = env.throw_new("java/lang/IllegalArgumentException", "Identidade inválida");
        return 0;
    };
    let Some(preset) = parse_preset(&preset) else {
        let _ = env.throw_new("java/lang/IllegalArgumentException", "Preset inválido");
        return 0;
    };
    let audio_on = audio_on != 0;
    let (w, h, f) = (width.max(0) as u32, height.max(0) as u32, fps.max(0) as u32);
    let status = Arc::new(Mutex::new(ShareStatus {
        ticket: None,
        online_warning: false,
        web: WebState::Off,
        web_vpn: None,
        backend: "MediaProjection",
        encoder,
        viewers: Vec::new(),
        fps: 0.0,
        kbps: 0,
        audio: if audio_on {
            AudioState::On
        } else if audio_error.is_empty() {
            AudioState::Off
        } else {
            AudioState::Unavailable(audio_error)
        },
        mic: AudioState::Off,
        mic_muted: false,
        ended: None,
        width: w,
        height: h,
        target_fps: f,
        overloaded: false,
    }));
    let feeds = Feeds::new();
    feeds.stats.ceiling_bps.store(bitrate.max(0) as u32, Ordering::Relaxed);
    let cfg = HostConfig {
        identity: key,
        host_name: name,
        title,
        max_viewers: max_viewers.max(1) as usize,
        friends: parse_codes(&codes),
        width: w,
        height: h,
        fps: f,
        audio: audio_on,
        donate: false,
    };
    match Host::start(RT.handle(), cfg, &feeds, status) {
        Ok(host) => {
            let (sw, sh, hz) = (src_w.max(0) as u32, src_h.max(0) as u32, hz.max(0) as u32);
            Box::into_raw(Box::new(HostSession {
                host,
                feeds,
                video_seq: AtomicU64::new(0),
                audio_seq: AtomicU32::new(0),
                current: Mutex::new((
                    preset,
                    EncodeParams { width: w, height: h, fps: f, bitrate_bps: bitrate.max(0) as u32 },
                )),
                src: (sw, sh, hz),
                audio_on: AtomicBool::new(audio_on),
            })) as jlong
        }
        Err(e) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", e.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostPushVideo(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    data: jni::objects::JByteArray,
    pts_us: jlong,
    keyframe: jboolean,
) {
    let Some(s) = (unsafe { get_host(handle) }) else { return };
    let Ok(bytes) = env.convert_byte_array(&data) else { return };
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        warn!("quadro do encoder ignorado: {} bytes", bytes.len());
        return;
    }
    let n = bytes.len() as u64;
    let frame = EncodedFrame {
        seq: s.video_seq.fetch_add(1, Ordering::Relaxed),
        keyframe: keyframe != 0,
        pts_us: pts_us.max(0) as u64,
        codec: Codec::H264,
        data: Bytes::from(bytes),
    };
    let _ = s.feeds.video.send(Arc::new(frame));
    s.feeds.stats.frames.fetch_add(1, Ordering::Relaxed);
    s.feeds.stats.bytes.fetch_add(n, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostPushAudio(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    opus: jni::objects::JByteArray,
) {
    let Some(s) = (unsafe { get_host(handle) }) else { return };
    let Ok(bytes) = env.convert_byte_array(&opus) else { return };
    let mut d = Vec::with_capacity(4 + bytes.len());
    d.extend_from_slice(&s.audio_seq.fetch_add(1, Ordering::Relaxed).to_le_bytes());
    d.extend_from_slice(&bytes);
    let _ = s.feeds.audio.send(Bytes::from(d));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostTakeKeyframe(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    match unsafe { get_host(handle) } {
        Some(s) if s.feeds.force_key.swap(false, Ordering::SeqCst) => JNI_TRUE,
        _ => JNI_FALSE,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostTargetBps(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    unsafe { get_host(handle) }
        .map(|s| s.feeds.stats.target_bps.load(Ordering::Relaxed).min(i32::MAX as u32) as jint)
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostTakeQualityRequest(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let Some(s) = (unsafe { get_host(handle) }) else { return std::ptr::null_mut() };
    let Some((preset, who)) = s.host.take_quality_request() else { return std::ptr::null_mut() };
    let new = encode_params(preset, s.src.0, s.src.1, s.src.2);
    let cur = s.current.lock().map(|c| c.1).unwrap_or(new);
    if !is_reduction(cur, new) {
        info!("pedido de qualidade recusado: {} de {who}", preset.name());
        return std::ptr::null_mut();
    }
    new_jstring(
        &env,
        serde_json::json!({
            "preset": preset_id(preset), "who": who,
            "width": new.width, "height": new.height, "fps": new.fps, "bitrate": new.bitrate_bps,
        })
        .to_string(),
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostSetStream(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    preset: JString,
    width: jint,
    height: jint,
    fps: jint,
    bitrate: jint,
) {
    let Some(s) = (unsafe { get_host(handle) }) else { return };
    let Some(preset) = jstr(&mut env, &preset).as_deref().and_then(parse_preset) else {
        warn!("preset inválido em hostSetStream");
        return;
    };
    let p = EncodeParams {
        width: width.max(0) as u32,
        height: height.max(0) as u32,
        fps: fps.max(0) as u32,
        bitrate_bps: bitrate.max(0) as u32,
    };
    if let Ok(mut c) = s.current.lock() {
        *c = (preset, p);
    }
    s.feeds.stats.ceiling_bps.store(p.bitrate_bps, Ordering::Relaxed);
    s.feeds.stats.target_bps.store(0, Ordering::Relaxed);
    s.feeds.force_key.store(true, Ordering::SeqCst);
    s.host.set_stream(p.width, p.height, p.fps, s.audio_on.load(Ordering::Relaxed));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostSetAudio(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    on: jboolean,
    error: JString,
) {
    let Some(s) = (unsafe { get_host(handle) }) else { return };
    let error = jstr(&mut env, &error).unwrap_or_default();
    let on = on != 0;
    if let Ok(mut st) = s.host.status.lock() {
        st.audio = if on {
            AudioState::On
        } else if error.is_empty() {
            AudioState::Off
        } else {
            AudioState::Unavailable(error)
        };
    }
    s.audio_on.store(on, Ordering::Relaxed);
    let p = s.current.lock().map(|c| c.1).unwrap_or(EncodeParams { width: 0, height: 0, fps: 0, bitrate_bps: 0 });
    s.host.set_stream(p.width, p.height, p.fps, on);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostStatus(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let Some(s) = (unsafe { get_host(handle) }) else { return std::ptr::null_mut() };
    let Ok(st) = s.host.status.lock() else { return std::ptr::null_mut() };
    let (web, web_error) = match &st.web {
        WebState::Off => ("off", None),
        WebState::Connecting => ("connecting", None),
        WebState::Online => ("online", None),
        WebState::Retrying(e) => ("retrying", Some(e.clone())),
    };
    let (audio, audio_error) = match &st.audio {
        AudioState::Off => ("off", None),
        AudioState::On => ("on", None),
        AudioState::Unavailable(e) => ("unavailable", Some(e.clone())),
    };
    let viewers: Vec<_> = st
        .viewers
        .iter()
        .map(|v| {
            serde_json::json!({
                "id": v.id_short, "label": v.label(), "friend": v.friend,
                "path": v.path, "rtt_ms": v.rtt_ms,
            })
        })
        .collect();
    let json = serde_json::json!({
        "ticket": st.ticket, "online_warning": st.online_warning,
        "web": web, "web_error": web_error, "viewers": viewers,
        "fps": st.fps, "kbps": st.kbps, "width": st.width, "height": st.height,
        "target_fps": st.target_fps, "overloaded": st.overloaded,
        "audio": audio, "audio_error": audio_error, "ended": st.ended,
    })
    .to_string();
    drop(st);
    new_jstring(&env, json)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostWebLink(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    match unsafe { get_host(handle) } {
        Some(s) => new_jstring(&env, s.host.web_link()),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostNewInvite(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if let Some(s) = unsafe { get_host(handle) } {
        s.host.new_invite();
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostKick(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    id: JString,
) {
    if let (Some(s), Some(id)) = (unsafe { get_host(handle) }, jstr(&mut env, &id)) {
        s.host.kick(&id);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostSetFriends(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    codes: JString,
) {
    if let (Some(s), Some(codes)) = (unsafe { get_host(handle) }, jstr(&mut env, &codes)) {
        s.host.set_friends(parse_codes(&codes));
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostSendChat(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    text: JString,
) {
    if let (Some(s), Some(text)) = (unsafe { get_host(handle) }, jstr(&mut env, &text)) {
        s.host.send_chat(&text);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostChatSince(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    after: jlong,
) -> jstring {
    let items: Vec<serde_json::Value> = match unsafe { get_host(handle) } {
        None => Vec::new(),
        Some(s) => match s.host.chat.lock() {
            Ok(log) => log
                .since(after.max(0) as u64)
                .map(|(seq, l)| {
                    serde_json::json!({
                        "seq": seq,
                        "label": l.label(),
                        "host": l.role == ChatRole::Host,
                        "friend": l.friend,
                        "mine": l.role == ChatRole::Host,
                        "text": l.text,
                    })
                })
                .collect(),
            Err(_) => Vec::new(),
        },
    };
    new_jstring(&env, serde_json::to_string(&items).unwrap_or_else(|_| "[]".into()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_pendiego_livescreen_Native_hostStop(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        let s = unsafe { Box::from_raw(handle as *mut HostSession) };
        s.host.stop();
    }
}
