package io.github.pendiego.livescreen

import android.media.MediaCodec
import android.media.MediaCodecList
import android.media.MediaFormat
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.view.Surface
import android.util.Log
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit

class VideoDecoder(
    private val handle: Long,
    private val surface: Surface,
    private val width: Int,
    private val height: Int,
    private val onSize: (Int, Int) -> Unit
) {
    @Volatile
    private var running = false
    private var thread: Thread? = null
    private val callbackThread = HandlerThread("video-codec")

    fun start() {
        running = true
        callbackThread.start()
        thread = Thread({ runLoop() }, "video-decoder").apply { start() }
    }

    // O join do runLoop libera o codec antes de a thread de callbacks sair:
    // nenhum callback chama o JNI depois do Native.stop.
    fun stop() {
        running = false
        thread?.join()
        callbackThread.quitSafely()
        callbackThread.join()
    }

    private fun isHardware(codec: MediaCodec): Boolean {
        return if (Build.VERSION.SDK_INT >= 29) {
            codec.codecInfo.isHardwareAccelerated
        } else {
            val name = codec.codecInfo.name
            !name.startsWith("c2.android.") && !name.startsWith("OMX.google.")
        }
    }

    private fun findSoftwareDecoder(): String? {
        val list = MediaCodecList(MediaCodecList.ALL_CODECS)
        for (info in list.codecInfos) {
            if (info.isEncoder) continue
            val types = info.supportedTypes
            if (types.none { it.equals("video/avc", ignoreCase = true) }) continue
            if (Build.VERSION.SDK_INT >= 29 && info.isSoftwareOnly) {
                return info.name
            }
            val name = info.name
            if (name.startsWith("c2.android.") || name.startsWith("OMX.google.")) {
                return name
            }
        }
        return null
    }

    // Estado por codec: ao recriar o codec, nenhum índice ou erro vaza para o novo.
    private inner class CodecState(val software: Boolean) : MediaCodec.Callback() {
        val inputs = LinkedBlockingQueue<Int>()
        val queuedAt = ConcurrentHashMap<Long, Long>() // pts -> nanoTime
        @Volatile var failed = false
        @Volatile var hardware = false

        override fun onInputBufferAvailable(c: MediaCodec, index: Int) {
            inputs.offer(index)
        }

        override fun onOutputBufferAvailable(c: MediaCodec, index: Int, info: MediaCodec.BufferInfo) {
            try {
                c.releaseOutputBuffer(index, true)
            } catch (_: IllegalStateException) {
                return
            }
            val busyUs = queuedAt.remove(info.presentationTimeUs)
                ?.let { (System.nanoTime() - it) / 1000L } ?: 0L
            Native.reportDecoded(handle, 1, busyUs, hardware)
        }

        override fun onError(c: MediaCodec, e: MediaCodec.CodecException) {
            failed = true
        }

        override fun onOutputFormatChanged(c: MediaCodec, format: MediaFormat) {
            val cropW = if (format.containsKey("crop-left") && format.containsKey("crop-right")) {
                format.getInteger("crop-right") - format.getInteger("crop-left") + 1
            } else {
                format.getInteger(MediaFormat.KEY_WIDTH)
            }
            val cropH = if (format.containsKey("crop-top") && format.containsKey("crop-bottom")) {
                format.getInteger("crop-bottom") - format.getInteger("crop-top") + 1
            } else {
                format.getInteger(MediaFormat.KEY_HEIGHT)
            }
            Log.i("LiveScreen", "VideoDecoder: formato alterado para ${cropW}x${cropH}")
            if (cropW > 0 && cropH > 0) {
                onSize(cropW, cropH)
            }
        }
    }

    private fun createCodec(software: Boolean): Pair<MediaCodec, CodecState> {
        val codec = if (software) {
            val name = findSoftwareDecoder()
                ?: throw IllegalStateException("Decodificador de software video/avc não encontrado")
            MediaCodec.createByCodecName(name)
        } else {
            MediaCodec.createDecoderByType("video/avc")
        }

        try {
            val st = CodecState(software)
            val handler = Handler(callbackThread.looper)
            val w = if (width > 0) width else 1920
            val h = if (height > 0) height else 1080
            fun makeFormat(lowLatency: Boolean): MediaFormat {
                return MediaFormat.createVideoFormat("video/avc", w, h).apply {
                    setInteger(MediaFormat.KEY_MAX_WIDTH, 3840)
                    setInteger(MediaFormat.KEY_MAX_HEIGHT, 2160)
                    setInteger(MediaFormat.KEY_MAX_INPUT_SIZE, 4 * 1024 * 1024)
                    if (lowLatency && Build.VERSION.SDK_INT >= 30) {
                        setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
                    }
                }
            }

            codec.setCallback(st, handler) // antes do configure
            try {
                codec.configure(makeFormat(true), surface, null, 0)
            } catch (_: Exception) {
                codec.reset() // apaga o callback
                codec.setCallback(st, handler)
                codec.configure(makeFormat(false), surface, null, 0)
            }
            codec.start()
            st.hardware = isHardware(codec)
            return codec to st
        } catch (e: Exception) {
            try { codec.release() } catch (_: Exception) {}
            throw e
        }
    }

    private fun runLoop() {
        var software = false
        var (codec, st) = try {
            createCodec(false)
        } catch (_: Exception) {
            try {
                software = true
                createCodec(true)
            } catch (_: Exception) {
                Native.decoderFailed(handle)
                return
            }
        }
        software = st.software || !st.hardware

        var discarding = true
        var failures = 0
        val meta = LongArray(2)

        try {
            while (running) {
                try {
                    if (st.failed) throw IllegalStateException("codec falhou")
                    val data = Native.nextVideo(handle, 100, meta) ?: continue
                    val pts = meta[0]
                    val keyframe = meta[1] != 0L
                    if (discarding && !keyframe) continue

                    val idx = st.inputs.poll(10, TimeUnit.MILLISECONDS)
                    if (idx == null) {
                        discarding = true
                        Native.requestKeyframe(handle)
                        continue
                    }
                    val buf = codec.getInputBuffer(idx)
                    if (buf == null || data.size > buf.capacity()) {
                        st.inputs.offer(idx) // o índice continua nosso
                        discarding = true
                        Native.requestKeyframe(handle)
                        continue
                    }
                    buf.clear()
                    buf.put(data)
                    if (st.queuedAt.size > 64) st.queuedAt.clear() // quadros que o decoder descartou
                    st.queuedAt[pts] = System.nanoTime()
                    codec.queueInputBuffer(
                        idx, 0, data.size, pts,
                        if (keyframe) MediaCodec.BUFFER_FLAG_KEY_FRAME else 0
                    )
                    if (keyframe) discarding = false
                    failures = 0
                } catch (e: Exception) {
                    if (e !is IllegalStateException && e !is MediaCodec.CodecException) throw e
                    discarding = true
                    Native.requestKeyframe(handle)
                    failures++
                    if (failures >= 3 && !software) {
                        software = true
                        failures = 0
                    }
                    try { codec.stop() } catch (_: Exception) {}
                    try { codec.release() } catch (_: Exception) {}
                    try {
                        val n = createCodec(software)
                        codec = n.first
                        st = n.second
                    } catch (_: Exception) {
                        Native.decoderFailed(handle)
                        return
                    }
                }
            }
        } finally {
            try { codec.stop() } catch (_: Exception) {}
            try { codec.release() } catch (_: Exception) {}
        }
    }
}
