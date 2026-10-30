package io.github.pendiego.livescreen

import android.hardware.display.DisplayManager
import android.hardware.display.VirtualDisplay
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.media.projection.MediaProjection
import android.os.Build
import android.os.Bundle
import android.util.Log

/** Captura a tela (MediaProjection) e codifica em H.264 Annex B, entregando os quadros ao núcleo. */
class ScreenEncoder(private val projection: MediaProjection, private val dpi: Int) {
    var width = 0
        private set
    var height = 0
        private set
    var onError: ((String) -> Unit)? = null

    private var codec: MediaCodec? = null
    private var display: VirtualDisplay? = null
    private var thread: Thread? = null
    @Volatile private var running = false
    private var stopped = false
    private var bitrate = 0
    private var firstPts = -1L

    // shortcut: rotação mantém o tamanho inicial (imagem com barras), reconfigurar ao girar se incomodar
    private fun make(w: Int, h: Int, fps: Int, bitrate: Int): MediaCodec {
        val c = MediaCodec.createEncoderByType(MediaFormat.MIMETYPE_VIDEO_AVC)
        try {
            val caps = c.codecInfo.getCapabilitiesForType(MediaFormat.MIMETYPE_VIDEO_AVC)
            val fmt = MediaFormat.createVideoFormat(MediaFormat.MIMETYPE_VIDEO_AVC, w, h).apply {
                setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface)
                setInteger(MediaFormat.KEY_BIT_RATE, bitrate)
                setInteger(MediaFormat.KEY_FRAME_RATE, fps)
                setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 30) // mesmo GOP do desktop
                setLong(MediaFormat.KEY_REPEAT_PREVIOUS_FRAME_AFTER, 500_000) // tela parada ainda atende pedido de keyframe
                if (caps.profileLevels.any { it.profile == MediaCodecInfo.CodecProfileLevel.AVCProfileHigh }) {
                    setInteger(MediaFormat.KEY_PROFILE, MediaCodecInfo.CodecProfileLevel.AVCProfileHigh)
                }
                if (caps.encoderCapabilities.isBitrateModeSupported(MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR)) {
                    setInteger(MediaFormat.KEY_BITRATE_MODE, MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR)
                }
                if (Build.VERSION.SDK_INT >= 30 &&
                    caps.isFeatureSupported(MediaCodecInfo.CodecCapabilities.FEATURE_LowLatency)
                ) setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
            }
            c.configure(fmt, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
            return c
        } catch (e: Exception) {
            c.release()
            throw e
        }
    }

    /** Devolve o nome do codec. As dimensões efetivas ficam em [width]/[height]. */
    @Synchronized
    fun start(w: Int, h: Int, fps: Int, bitrate: Int): String = startLocked(w, h, fps, bitrate)

    private fun startLocked(w: Int, h: Int, fps: Int, bitrate: Int): String {
        var cw = w
        var ch = h
        val c = try {
            make(cw, ch, fps, bitrate)
        } catch (e: Exception) {
            cw = w / 16 * 16
            ch = h / 16 * 16
            make(cw, ch, fps, bitrate)
        }
        val surface = c.createInputSurface()
        c.start()
        val d = display
        if (d == null) {
            display = projection.createVirtualDisplay(
                "LiveScreen", cw, ch, dpi, DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR, surface, null, null
            )
        } else {
            d.resize(cw, ch, dpi)
            d.surface = surface
        }
        codec = c
        width = cw
        height = ch
        this.bitrate = bitrate
        return c.name
    }

    @Synchronized
    fun run(handle: Long) {
        val c = codec ?: return
        running = true
        thread = Thread({ drain(c, handle) }, "ls-venc").also { it.start() }
    }

    private fun drain(c: MediaCodec, h: Long) {
        val info = MediaCodec.BufferInfo()
        var csd: ByteArray? = null
        var current = bitrate
        var lastCheck = System.nanoTime()
        try {
            while (running) {
                if (Native.hostTakeKeyframe(h)) {
                    c.setParameters(Bundle().apply { putInt(MediaCodec.PARAMETER_KEY_REQUEST_SYNC_FRAME, 0) })
                }
                val i = c.dequeueOutputBuffer(info, 100_000)
                if (i >= 0) {
                    val buf = c.getOutputBuffer(i)
                    if (buf != null && info.size > 0) {
                        val bytes = ByteArray(info.size)
                        buf.position(info.offset)
                        buf.limit(info.offset + info.size)
                        buf.get(bytes)
                        if (info.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG != 0) {
                            csd = bytes
                        } else {
                            val key = info.flags and MediaCodec.BUFFER_FLAG_KEY_FRAME != 0
                            val data = if (key && csd != null) csd + bytes else bytes
                            if (firstPts < 0) firstPts = info.presentationTimeUs
                            Native.hostPushVideo(h, data, info.presentationTimeUs - firstPts, key)
                        }
                    }
                    c.releaseOutputBuffer(i, false)
                }
                val now = System.nanoTime()
                if (now - lastCheck >= 1_000_000_000L) {
                    lastCheck = now
                    val t = Native.hostTargetBps(h)
                    val want = if (t == 0) bitrate else t
                    if (want != current) {
                        c.setParameters(Bundle().apply { putInt(MediaCodec.PARAMETER_KEY_VIDEO_BITRATE, want) })
                        current = want
                    }
                }
            }
        } catch (e: Exception) {
            if (running) {
                Log.w("LiveScreen", "encoder de vídeo falhou", e)
                onError?.invoke("falha no encoder de vídeo: ${e.message}")
            }
        }
    }

    private fun joinThread() {
        running = false
        thread?.join()
        thread = null
    }

    private fun releaseCodec() {
        codec?.let {
            runCatching { it.stop() }
            it.release()
        }
        codec = null
    }

    @Synchronized
    fun reconfigure(handle: Long, w: Int, h: Int, fps: Int, bitrate: Int) {
        if (stopped) return
        joinThread()
        releaseCodec()
        startLocked(w, h, fps, bitrate)
        run(handle)
    }

    @Synchronized
    fun stop() {
        stopped = true
        joinThread()
        releaseCodec()
        display?.release()
        display = null
    }
}
