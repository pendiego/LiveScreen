package io.github.pendiego.livescreen

import android.content.Context
import android.content.Intent
import android.graphics.Rect
import android.hardware.display.DisplayManager
import android.media.projection.MediaProjection
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.util.DisplayMetrics
import android.util.Log
import android.view.Display
import android.view.WindowManager
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.delay
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.roundToInt

data class HostRequest(val identity: String, val name: String, val title: String, val codes: String)

data class HostViewer(
    val id: String,
    val label: String,
    val friend: Boolean,
    val path: String?,
    val rttMs: Int?
)

data class HostStatus(
    val ticket: String?,
    val onlineWarning: Boolean,
    val web: String,
    val webError: String,
    val viewers: List<HostViewer>,
    val fps: Int,
    val kbps: Int,
    val width: Int,
    val height: Int,
    val targetFps: Int,
    val overloaded: Boolean,
    val audio: String,
    val audioError: String,
    val ended: String?
)

private fun JSONObject.strOrNull(k: String): String? = if (isNull(k)) null else optString(k)

private fun parseHostStatus(raw: String): HostStatus {
    val o = JSONObject(raw)
    val arr = o.optJSONArray("viewers") ?: JSONArray()
    val ow = o.opt("online_warning")
    return HostStatus(
        ticket = o.strOrNull("ticket"),
        onlineWarning = ow == true || (ow is String && ow.isNotEmpty()),
        web = o.optString("web", "off"),
        webError = o.strOrNull("web_error") ?: "",
        viewers = (0 until arr.length()).map { i ->
            val v = arr.getJSONObject(i)
            HostViewer(
                id = v.optString("id"),
                label = v.optString("label"),
                friend = v.optBoolean("friend", false),
                path = v.strOrNull("path"),
                rttMs = if (v.isNull("rtt_ms")) null else v.optInt("rtt_ms")
            )
        },
        fps = o.optInt("fps"),
        kbps = o.optInt("kbps"),
        width = o.optInt("width"),
        height = o.optInt("height"),
        targetFps = o.optInt("target_fps"),
        overloaded = o.optBoolean("overloaded", false),
        audio = o.optString("audio", "off"),
        audioError = o.strOrNull("audio_error") ?: "",
        ended = o.strOrNull("ended")
    )
}

/** Estado do host do processo todo: sobrevive à Activity, vive no HostService. */
object Hosting {
    var active by mutableStateOf(false)
    var status by mutableStateOf<HostStatus?>(null)
    var lastError by mutableStateOf<String?>(null)
    var notice by mutableStateOf<String?>(null)
    val chat = mutableStateListOf<ChatItem>()
    var chatOpen by mutableStateOf(false)
    var presetId by mutableStateOf("P720p30")
    @Volatile var stopThread: Thread? = null
    var request: HostRequest? = null

    private class Run(
        val h: Long,
        val video: ScreenEncoder,
        val audio: AudioEncoder?,
        val projection: MediaProjection,
        val scope: CoroutineScope,
        val job: Job
    )

    private val lock = Any()
    private var session: Run? = null
    private var chatSeq = 0L
    private var src = intArrayOf(0, 0, 0, 0) // largura, altura, dpi, hz

    /** Handle do núcleo, 0 fora da transmissão. */
    val handle: Long get() = session?.h ?: 0L

    /** Chamadas ao núcleo seguram o lado de leitura; [teardown] toma o de escrita antes do hostStop. */
    private val handleLock = java.util.concurrent.locks.ReentrantReadWriteLock()
    private var handleClosed = false

    /** Executa [block] com o handle vivo, ou devolve null se a transmissão já fechou. */
    fun <T> withHandle(h: Long, block: (Long) -> T): T? {
        val l = handleLock.readLock()
        l.lock()
        try {
            if (handleClosed || session?.h != h) return null
            return block(h)
        } finally {
            l.unlock()
        }
    }

    private fun params(id: String): JSONObject =
        JSONObject(Native.encodeParams(id, src[0], src[1], src[3]))

    private fun presetName(id: String): String = try {
        val arr = JSONArray(Native.presets())
        (0 until arr.length()).map { arr.getJSONObject(it) }.firstOrNull { it.getString("id") == id }
            ?.getString("name") ?: id
    } catch (_: Exception) {
        id
    }

    private fun readScreen(ctx: Context) {
        val dm = ctx.getSystemService(DisplayManager::class.java)
        val display = dm.getDisplay(Display.DEFAULT_DISPLAY)
        val bounds: Rect
        if (Build.VERSION.SDK_INT >= 30) {
            bounds = ctx.getSystemService(WindowManager::class.java).currentWindowMetrics.bounds
        } else {
            val m = DisplayMetrics()
            @Suppress("DEPRECATION") display.getRealMetrics(m)
            bounds = Rect(0, 0, m.widthPixels, m.heightPixels)
        }
        src = intArrayOf(bounds.width(), bounds.height(), ctx.resources.displayMetrics.densityDpi, display.refreshRate.roundToInt())
    }

    /** Roda numa thread de fundo, chamada pelo HostService. */
    fun start(ctx: Context, projection: MediaProjection) {
        var video: ScreenEncoder? = null
        var audio: AudioEncoder? = null
        var h = 0L
        try {
            val req = request ?: error("pedido de transmissão ausente")
            readScreen(ctx)
            // Obrigatório no API 34+, antes do createVirtualDisplay.
            projection.registerCallback(object : MediaProjection.Callback() {
                override fun onStop() = stop(ctx)
            }, Handler(Looper.getMainLooper()))
            val p = params(presetId)
            val fps = p.getInt("fps")
            val bitrate = p.getInt("bitrate")
            video = ScreenEncoder(projection, src[2])
            val encoderName = video.start(p.getInt("width"), p.getInt("height"), fps, bitrate)
            audio = AudioEncoder(projection)
            val audioError = audio.start()
            h = Native.hostStart(
                req.identity, req.name, req.title, req.codes, MAX_VIEWERS, presetId,
                src[0], src[1], src[3], video.width, video.height, fps, bitrate, encoderName,
                audioError == null, audioError ?: ""
            )
            if (h == 0L) error("falha ao iniciar o host")
            video.onError = { fail(ctx, it) }
            video.run(h)
            if (audioError == null) audio.run(h)
            val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
            val job = scope.launch { poll(ctx, h, video) }
            synchronized(lock) {
                if (!active) {
                    // stop() chegou durante a partida: ninguém mais conhece estes recursos.
                    job.cancel()
                    teardown(Run(h, video, audio, projection, scope, job))
                    return
                }
                handleClosed = false
                session = Run(h, video, audio, projection, scope, job)
            }
        } catch (e: Exception) {
            Log.w("LiveScreen", "falha ao iniciar a transmissão", e)
            video?.stop()
            audio?.stop()
            if (h != 0L) Native.hostStop(h)
            projection.stop()
            fail(ctx, e.message ?: "falha ao iniciar a transmissão")
        }
    }

    private suspend fun poll(ctx: Context, h: Long, video: ScreenEncoder) {
        while (true) {
            currentCoroutineContext().ensureActive()
            delay(1000)
            try {
                val (st, items, req) = withContext(Dispatchers.IO) {
                    Triple(
                        parseHostStatus(Native.hostStatus(h)),
                        parseChatItems(Native.hostChatSince(h, chatSeq)),
                        Native.hostTakeQualityRequest(h)
                    )
                }
                status = st
                if (items.isNotEmpty()) {
                    chat.addAll(items)
                    while (chat.size > 200) chat.removeAt(0)
                    chatSeq = items.maxOf { it.seq }
                }
                if (req != null) {
                    val q = JSONObject(req)
                    withContext(Dispatchers.IO) {
                        video.reconfigure(h, q.getInt("width"), q.getInt("height"), q.getInt("fps"), q.getInt("bitrate"))
                        Native.hostSetStream(h, q.getString("preset"), video.width, video.height, q.getInt("fps"), q.getInt("bitrate"))
                    }
                    presetId = q.getString("preset")
                    notice = "Qualidade reduzida para ${presetName(presetId)} a pedido de ${q.optString("who").ifBlank { "um espectador" }}"
                }
                if (st.ended != null) {
                    fail(ctx, st.ended)
                    return
                }
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w("LiveScreen", "falha na transmissão", e)
                fail(ctx, e.message ?: "falha na transmissão")
                return
            }
        }
    }

    /** O próprio host escolhe a qualidade: sem checar se é redução. */
    fun setPreset(ctx: Context, id: String) {
        val r = session ?: return
        Thread {
            try {
                val p = params(id)
                r.video.reconfigure(r.h, p.getInt("width"), p.getInt("height"), p.getInt("fps"), p.getInt("bitrate"))
                withHandle(r.h) {
                    Native.hostSetStream(it, id, r.video.width, r.video.height, p.getInt("fps"), p.getInt("bitrate"))
                } ?: return@Thread
                presetId = id
                notice = null
            } catch (e: Exception) {
                Log.w("LiveScreen", "falha ao trocar a qualidade", e)
                fail(ctx, e.message ?: "falha ao trocar a qualidade")
            }
        }.start()
    }

    fun fail(ctx: Context, msg: String) {
        lastError = msg
        stop(ctx)
    }

    private fun teardown(r: Run) {
        runBlocking { r.job.cancelAndJoin() }
        r.scope.coroutineContext[Job]?.cancel()
        r.video.stop()
        r.audio?.stop()
        r.projection.stop()
        // Espera as chamadas em curso (UI, setPreset) e barra as novas antes de liberar o handle.
        val w = handleLock.writeLock()
        w.lock()
        try {
            handleClosed = true
        } finally {
            w.unlock()
        }
        Native.hostStop(r.h)
    }

    /** Idempotente. O desmonte (que bloqueia) roda em [stopThread]. */
    fun stop(ctx: Context) {
        val r: Run?
        synchronized(lock) {
            if (!active) return
            active = false
            r = session
            session = null
        }
        status = null
        chat.clear()
        chatSeq = 0
        chatOpen = false
        notice = null
        val app = ctx.applicationContext
        stopThread = Thread {
            r?.let { teardown(it) }
            app.stopService(Intent(app, HostService::class.java))
        }.also { it.start() }
    }

    private const val MAX_VIEWERS = 4
}
