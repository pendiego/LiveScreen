package io.github.pendiego.livescreen

import android.media.MediaCodec
import android.media.MediaCodecList
import android.media.MediaFormat
import android.os.Build
import android.view.Surface
import android.util.Log

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

    fun start() {
        running = true
        thread = Thread({ runLoop() }, "video-decoder").apply { start() }
    }

    fun stop() {
        running = false
        thread?.join()
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

    private fun createCodec(software: Boolean): MediaCodec {
        val codec = if (software) {
            val name = findSoftwareDecoder()
                ?: throw IllegalStateException("Decodificador de software video/avc não encontrado")
            MediaCodec.createByCodecName(name)
        } else {
            MediaCodec.createDecoderByType("video/avc")
        }

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

        try {
            codec.configure(makeFormat(true), surface, null, 0)
        } catch (_: Exception) {
            codec.reset()
            codec.configure(makeFormat(false), surface, null, 0)
        }
        codec.start()
        return codec
    }

    private fun drain(codec: MediaCodec, info: MediaCodec.BufferInfo): Int {
        var rendered = 0
        while (running) {
            val idx = codec.dequeueOutputBuffer(info, 0)
            if (idx >= 0) {
                codec.releaseOutputBuffer(idx, true)
                rendered++
            } else if (idx == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED) {
                val fmt = codec.outputFormat
                val cropW = if (fmt.containsKey("crop-left") && fmt.containsKey("crop-right")) {
                    fmt.getInteger("crop-right") - fmt.getInteger("crop-left") + 1
                } else {
                    fmt.getInteger(MediaFormat.KEY_WIDTH)
                }
                val cropH = if (fmt.containsKey("crop-top") && fmt.containsKey("crop-bottom")) {
                    fmt.getInteger("crop-bottom") - fmt.getInteger("crop-top") + 1
                } else {
                    fmt.getInteger(MediaFormat.KEY_HEIGHT)
                }
                Log.i("LiveScreen", "VideoDecoder: formato alterado para ${cropW}x${cropH}")
                if (cropW > 0 && cropH > 0) {
                    onSize(cropW, cropH)
                }
            } else {
                break
            }
        }
        return rendered
    }

    private fun runLoop() {
        var software = false
        var codec: MediaCodec = try {
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
        software = !isHardware(codec)

        var discarding = true
        var failures = 0
        val meta = LongArray(2)
        val info = MediaCodec.BufferInfo()

        try {
            while (running) {
                try {
                    val drainStart = System.nanoTime()
                    val rendered = drain(codec, info)
                    val drainUs = (System.nanoTime() - drainStart) / 1000L

                    val data = Native.nextVideo(handle, 5, meta)
                    if (data == null) {
                        if (rendered > 0) {
                            failures = 0
                            discarding = false
                            Native.reportDecoded(handle, rendered, drainUs, isHardware(codec))
                        }
                        continue
                    }

                    val pts = meta[0]
                    val keyframe = meta[1] != 0L

                    if (discarding && !keyframe) {
                        if (rendered > 0) {
                            failures = 0
                            discarding = false
                            Native.reportDecoded(handle, rendered, drainUs, isHardware(codec))
                        }
                        continue
                    }

                    val queueStart = System.nanoTime()
                    val inIdx = codec.dequeueInputBuffer(10_000)
                    var queued = false
                    if (inIdx < 0) {
                        discarding = true
                        Native.requestKeyframe(handle)
                    } else {
                        val buf = codec.getInputBuffer(inIdx)
                        if (buf == null || data.size > buf.capacity()) {
                            discarding = true
                            Native.requestKeyframe(handle)
                        } else {
                            buf.clear()
                            buf.put(data)
                            codec.queueInputBuffer(
                                inIdx,
                                0,
                                data.size,
                                pts,
                                if (keyframe) MediaCodec.BUFFER_FLAG_KEY_FRAME else 0
                            )
                            queued = true
                        }
                    }
                    val queueUs = (System.nanoTime() - queueStart) / 1000L
                    val totalUs = drainUs + queueUs

                    if (queued || rendered > 0) {
                        failures = 0
                        discarding = false
                        Native.reportDecoded(handle, rendered, totalUs, isHardware(codec))
                    }
                } catch (e: Exception) {
                    if (e is IllegalStateException || e is MediaCodec.CodecException) {
                        discarding = true
                        Native.requestKeyframe(handle)
                        failures++
                        if (failures >= 3 && !software) {
                            software = true
                            failures = 0
                        }
                        try {
                            codec.stop()
                        } catch (_: Exception) {}
                        try {
                            codec.release()
                        } catch (_: Exception) {}
                        try {
                            codec = createCodec(software)
                        } catch (_: Exception) {
                            Native.decoderFailed(handle)
                            return
                        }
                    } else {
                        throw e
                    }
                }
            }
        } finally {
            try {
                codec.stop()
            } catch (_: Exception) {}
            try {
                codec.release()
            } catch (_: Exception) {}
        }
    }
}
