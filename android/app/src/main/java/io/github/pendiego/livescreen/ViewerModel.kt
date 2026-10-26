package io.github.pendiego.livescreen

import android.app.Application
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import android.util.Log
import java.io.File
import android.view.Surface
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
data class Friend(val name: String, val code: String)
data class Presence(val state: String, val title: String = "")

data class Status(
    val connecting: Boolean = false,
    val ended: String? = null,
    val path: String? = null,
    val rttMs: Int? = null,
    val fps: Int = 0,
    val width: Int = 0,
    val height: Int = 0,
    val audio: Boolean = false,
    val title: String = "",
    val decoderHw: Boolean = false,
    val decodeFps: Int = 0,
    val decodeMs: Double = 0.0,
    val hostId: String = "",
    val hostName: String = ""
)

sealed interface UpdateState {
    data object Idle : UpdateState
    data class Available(val info: UpdateInfo) : UpdateState
    data class Downloading(val info: UpdateInfo, val progress: Float?) : UpdateState
    data class Ready(val info: UpdateInfo, val apk: File) : UpdateState
    data class Failed(val info: UpdateInfo, val message: String) : UpdateState
}

class ViewerModel(app: Application) : AndroidViewModel(app) {
    private val prefs = app.getSharedPreferences("livescreen", Context.MODE_PRIVATE)

    var name: String by mutableStateOf(prefs.getString("name", "") ?: "")
    var ticket: String by mutableStateOf("")
    var lastEnd: String? by mutableStateOf(null)
    var status: Status? by mutableStateOf(null)
    var videoSize: Pair<Int, Int>? by mutableStateOf(null)


    val identity: String = run {
        val existing = prefs.getString("identity_key", null)
        if (existing != null && Native.friendCode(existing) != null) {
            existing
        } else {
            val bytes = ByteArray(32)
            java.security.SecureRandom().nextBytes(bytes)
            val hex = bytes.joinToString("") { "%02x".format(it) }
            prefs.edit().putString("identity_key", hex).commit()
            hex
        }
    }
    val myCode: String = Native.friendCode(identity) ?: ""

    var friends: List<Friend> by mutableStateOf(run {
        val raw = prefs.getString("friends", "[]") ?: "[]"
        try {
            val arr = JSONArray(raw)
            (0 until arr.length()).mapNotNull { i ->
                val obj = arr.getJSONObject(i)
                val code = obj.optString("code", "")
                val fName = obj.optString("name", "")
                if (Native.isValidFriendCode(code)) {
                    Friend(fName, code)
                } else null
            }
        } catch (_: Exception) {
            emptyList()
        }
    })

    private fun saveFriends() {
        val arr = JSONArray()
        for (f in friends) {
            val obj = JSONObject()
            obj.put("name", f.name)
            obj.put("code", f.code)
            arr.put(obj)
        }
        prefs.edit().putString("friends", arr.toString()).apply()
    }

    fun addFriend(name: String, code: String): String? {
        val cleanName = name.trim()
        if (cleanName.isEmpty()) {
            return "Dê um nome para reconhecer o amigo."
        }
        val cleanCode = code.filterNot(Char::isWhitespace).lowercase()
        if (!Native.isValidFriendCode(cleanCode)) {
            return "Código de amigo inválido: confira se copiou os 64 caracteres."
        }
        if (cleanCode == myCode) {
            return "Esse é o seu próprio código de amigo"
        }
        val existing = friends.find { it.code == cleanCode }
        if (existing != null) {
            return "${existing.name} já está nos seus amigos"
        }
        val newFriend = Friend(name = cleanName.take(32), code = cleanCode)
        friends = friends + newFriend
        saveFriends()
        syncPresenceIds()
        return null
    }

    fun removeFriend(code: String) {
        friends = friends.filterNot { it.code == code }
        saveFriends()
        syncPresenceIds()
    }

    var presence: Map<String, Presence> by mutableStateOf(emptyMap())
    private var presenceJob: Job? = null
    @Volatile private var viewerStop: Thread? = null
    var starting: Boolean by mutableStateOf(false)

    fun startPresence() {
        if (friends.isEmpty() || handle != 0L || pollingJob?.isActive == true || presenceJob?.isActive == true) return
        val prev = presenceJob
        presenceJob = viewModelScope.launch(Dispatchers.IO) {
            prev?.join()
            viewerStop?.join()
            val codes = friends.joinToString(",") { it.code }
            val ph = try {
                Native.presenceStart(identity, codes)
            } catch (e: IllegalArgumentException) {
                Log.w("LiveScreen", "falha ao iniciar presença", e)
                return@launch
            }
            if (ph == 0L) return@launch
            try {
                var sent = codes
                while (isActive) {
                    val now = friends.joinToString(",") { it.code }
                    if (now != sent) {
                        Native.presenceSetIds(ph, now)
                        sent = now
                    }
                    val jsonStr = Native.presenceStatus(ph)
                    val map = try {
                        val root = JSONObject(jsonStr)
                        val m = mutableMapOf<String, Presence>()
                        val keys = root.keys()
                        while (keys.hasNext()) {
                            val k = keys.next()
                            val obj = root.getJSONObject(k)
                            m[k] = Presence(
                                state = obj.optString("state", "unknown"),
                                title = obj.optString("title", "")
                            )
                        }
                        m
                    } catch (e: JSONException) {
                        Log.w("LiveScreen", "erro ao ler presença", e)
                        null
                    }
                    if (map != null) {
                        withContext(Dispatchers.Main) {
                            presence = presence + map
                        }
                    }
                    delay(1000)
                }
            } finally {
                withContext(NonCancellable) {
                    Native.presenceStop(ph)
                }
            }
        }
    }

    fun stopPresence() {
        presenceJob?.cancel()
    }

    fun syncPresenceIds() {
        if (friends.isEmpty()) {
            stopPresence()
        } else {
            startPresence()
        }
    }

    fun watchFriend(code: String) = connect(code)
    private val _muted = mutableStateOf(false)
    var muted: Boolean
        get() = _muted.value
        set(value) {
            _muted.value = value
            player?.muted = value
        }

    val presets: List<Pair<String, String>> = try {
        val arr = JSONArray(Native.presets())
        (0 until arr.length()).map { i ->
            val obj = arr.getJSONObject(i)
            Pair(obj.getString("id"), obj.getString("name"))
        }
    } catch (_: Exception) {
        emptyList()
    }

    private var handle: Long = 0L
    private var pollingJob: Job? = null
    private var decoder: VideoDecoder? = null
    private var player: AudioPlayer? = null
    private var currentSurface: Surface? = null
    private var hidden = false

    var update: UpdateState by mutableStateOf(UpdateState.Idle)

    init {
        // Debug é assinado com outra chave e não aceitaria o APK de release; também poupa o e2e_android.
        if (!BuildConfig.DEBUG) {
            viewModelScope.launch {
                val res = withContext(Dispatchers.IO) { runCatching { Updater.check() } }
                res.onSuccess { info -> if (info != null) update = UpdateState.Available(info) }
                    .onFailure { Log.w("LiveScreen", "verificação de atualização falhou", it) }
            }
        }
    }

    fun startUpdate() {
        when (val s = update) {
            is UpdateState.Ready -> install(s.apk)
            is UpdateState.Available -> download(s.info)
            is UpdateState.Failed -> download(s.info)
            else -> {}
        }
    }

    private fun download(info: UpdateInfo) {
        update = UpdateState.Downloading(info, null)
        viewModelScope.launch {
            try {
                val apk = withContext(Dispatchers.IO) {
                    Updater.download(getApplication<Application>().cacheDir, info) { p ->
                        viewModelScope.launch { update = UpdateState.Downloading(info, p) }
                    }
                }
                update = UpdateState.Ready(info, apk)
                install(apk)
            } catch (e: Exception) {
                update = UpdateState.Failed(info, e.message ?: "falha no download")
            }
        }
    }

    private fun install(apk: File) {
        val ctx = getApplication<Application>()
        if (!ctx.packageManager.canRequestPackageInstalls()) {
            ctx.startActivity(
                Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:${ctx.packageName}"))
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            )
            return
        }
        ctx.startActivity(Updater.installIntent(ctx, apk))
    }

    fun connect(target: String = ticket.trim()) {
        if (handle != 0L || pollingJob?.isActive == true) return
        starting = true
        prefs.edit().putString("name", name).apply()
        lastEnd = null
        hidden = false

        val pj = presenceJob
        pj?.cancel()

        pollingJob = viewModelScope.launch {
            pj?.join()
            val h = try {
                withContext(Dispatchers.IO + NonCancellable) {
                    viewerStop?.join()
                    Native.start(target, name, identity)
                }
            } catch (e: IllegalArgumentException) {
                starting = false
                lastEnd = e.message
                return@launch
            }

            if (!isActive) {
                if (h != 0L) {
                    viewerStop = Thread { Native.stop(h) }.also { it.start() }
                }
                starting = false
                return@launch
            }

            if (h == 0L) {
                starting = false
                lastEnd = "Falha ao iniciar conexão"
                return@launch
            }
            handle = h
            starting = false
            startVideo()

            while (isActive) {
                delay(if (status?.connecting != false) 500 else 1000)
                val s = withContext(Dispatchers.IO) {
                    Native.status(h)
                }
                val parsed = parseStatus(s)
                status = parsed

                if (parsed.audio && !parsed.connecting && player == null) {
                    player = AudioPlayer(h).apply {
                        this.paused = hidden
                        this.muted = this@ViewerModel.muted
                        start()
                    }
                }

                if (parsed.ended != null) {
                    leave(parsed.ended)
                    break
                }

                if (decoder == null && currentSurface != null && currentSurface!!.isValid) {
                    startVideo()
                }
            }
        }
    }

    // A SurfaceView destrói a surface quando a janela some (Home, tela apagada): é o sinal de visibilidade.
    private fun setVisible(v: Boolean) {
        if (hidden == !v) return
        hidden = !v
        player?.paused = !v
        if (handle != 0L) Native.setVisible(handle, v)
        Log.i("LiveScreen", if (v) "viewer: visível" else "viewer: em segundo plano")
    }

    fun attachSurface(surface: Surface) {
        currentSurface = surface
        setVisible(true) // esvazia o canal de vídeo: o decoder não pode estar rodando
        if (handle != 0L && decoder == null) {
            startVideo()
        }
    }

    fun detachSurface() {
        currentSurface = null
        decoder?.stop()
        decoder = null
        setVisible(false)
    }

    private fun startVideo() {
        val surf = currentSurface ?: return
        if (!surf.isValid || handle == 0L || decoder != null) return

        val currentStatus = status
        val w = currentStatus?.width ?: 0
        val h = currentStatus?.height ?: 0

        decoder = VideoDecoder(handle, surf, w, h) { nw, nh ->
            viewModelScope.launch(Dispatchers.Main) {
                videoSize = Pair(nw, nh)
            }
        }.apply {
            start()
        }
    }

    fun requestQuality(id: String) {
        val h = handle
        if (h != 0L) {
            Native.requestQuality(h, id)
        }
    }

    fun leave(reason: String? = null) {
        pollingJob?.cancel()
        pollingJob = null
        starting = false

        decoder?.stop()
        decoder = null

        player?.stop()
        player = null

        val h = handle
        handle = 0L
        if (h != 0L) {
            viewerStop = Thread { Native.stop(h) }.also { it.start() }
        }

        status = null
        videoSize = null
        lastEnd = reason
    }

    override fun onCleared() {
        super.onCleared()
        stopPresence()
        if (handle != 0L) {
            leave()
        }
    }

    private fun parseStatus(jsonStr: String): Status {
        val obj = JSONObject(jsonStr)
        var connecting = false
        var ended: String? = null

        val phaseObj = obj.optJSONObject("phase")
        if (phaseObj != null) {
            ended = if (phaseObj.has("Ended")) phaseObj.getString("Ended") else null
        } else {
            val phaseStr = obj.optString("phase")
            if (phaseStr == "Connecting") {
                connecting = true
            }
        }

        return Status(
            connecting = connecting,
            ended = ended,
            path = if (obj.has("path") && !obj.isNull("path")) obj.getString("path") else null,
            rttMs = if (obj.has("rtt_ms") && !obj.isNull("rtt_ms")) obj.getInt("rtt_ms") else null,
            fps = obj.optInt("fps", 0),
            width = obj.optInt("width", 0),
            height = obj.optInt("height", 0),
            audio = obj.optBoolean("audio", false),
            title = obj.optString("title", ""),
            decoderHw = obj.optBoolean("decoder_hw", false),
            decodeFps = obj.optInt("decode_fps", 0),
            decodeMs = obj.optDouble("decode_ms", 0.0),
            hostId = obj.optString("host_id", ""),
            hostName = obj.optString("host_name", "")
        )
    }
}
