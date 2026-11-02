package io.github.pendiego.livescreen

import android.media.AudioFormat
import android.media.AudioPlaybackCaptureConfiguration
import android.media.AudioAttributes
import android.media.AudioRecord
import android.media.MediaCodec
import android.media.MediaCodecList
import android.media.MediaFormat
import android.media.projection.MediaProjection
import android.os.Build
import android.util.Log
import kotlin.math.abs
import kotlin.math.max

/** Captura o áudio do sistema (Android 10+) e codifica em Opus de 20 ms. */
class AudioEncoder(private val projection: MediaProjection) {
    private var record: AudioRecord? = null
    private var codec: MediaCodec? = null
    private var thread: Thread? = null
    @Volatile private var running = false

    /** `null` quando deu certo; senão a mensagem de erro. */
    fun start(): String? {
        if (Build.VERSION.SDK_INT < 29) return "áudio exige Android 10 ou mais novo"
        return try {
            startQ()
        } catch (e: SecurityException) {
            release()
            "permissão de áudio negada"
        } catch (e: Exception) {
            Log.w("LiveScreen", "áudio indisponível", e)
            release()
            "falha ao iniciar a captura de áudio: ${e.message}"
        }
    }

    @android.annotation.SuppressLint("MissingPermission")
    @androidx.annotation.RequiresApi(29)
    private fun startQ(): String? {
        val cfg = AudioPlaybackCaptureConfiguration.Builder(projection)
            .addMatchingUsage(AudioAttributes.USAGE_MEDIA)
            .addMatchingUsage(AudioAttributes.USAGE_GAME)
            .addMatchingUsage(AudioAttributes.USAGE_UNKNOWN)
            .build()
        val pcm = AudioFormat.Builder()
            .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
            .setSampleRate(RATE)
            .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
            .build()
        val min = AudioRecord.getMinBufferSize(RATE, AudioFormat.CHANNEL_IN_STEREO, AudioFormat.ENCODING_PCM_16BIT)
        val rec = AudioRecord.Builder()
            .setAudioPlaybackCaptureConfig(cfg)
            .setAudioFormat(pcm)
            .setBufferSizeInBytes(max(min, CHUNK * 8))
            .build()
        record = rec
        if (rec.state != AudioRecord.STATE_INITIALIZED) {
            release()
            return "falha ao iniciar a captura de áudio"
        }

        val fmt = MediaFormat.createAudioFormat(MediaFormat.MIMETYPE_AUDIO_OPUS, RATE, 2).apply {
            setInteger(MediaFormat.KEY_BIT_RATE, 128_000)
        }
        val name = MediaCodecList(MediaCodecList.REGULAR_CODECS).findEncoderForFormat(fmt)
        if (name == null) {
            release()
            return "este aparelho não tem codificador Opus"
        }
        val c = MediaCodec.createByCodecName(name)
        codec = c
        c.configure(fmt, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
        c.start()
        return null
    }

    fun run(handle: Long) {
        val rec = record ?: return
        val c = codec ?: return
        running = true
        thread = Thread({ loop(rec, c, handle) }, "ls-aenc").also { it.start() }
    }

    private fun loop(rec: AudioRecord, c: MediaCodec, h: Long) {
        val pcm = ByteArray(CHUNK)
        val info = MediaCodec.BufferInfo()
        var pts = 0L
        var outs = 0
        var prev = 0L
        try {
            rec.startRecording()
            while (running) {
                val n = rec.read(pcm, 0, CHUNK)
                if (n > 0) {
                    val i = c.dequeueInputBuffer(10_000)
                    if (i >= 0) {
                        val buf = c.getInputBuffer(i)!!
                        buf.clear()
                        buf.put(pcm, 0, n)
                        c.queueInputBuffer(i, 0, n, pts, 0)
                        pts += 20_000
                    }
                } else if (n < 0) {
                    throw IllegalStateException("AudioRecord.read = $n")
                }
                while (running) {
                    val o = c.dequeueOutputBuffer(info, 0)
                    if (o < 0) break
                    val buf = c.getOutputBuffer(o)
                    if (buf != null && info.size > 0 && info.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG == 0) {
                        outs++
                        if (outs == 1) {
                            prev = info.presentationTimeUs
                        } else if (outs == 2 && abs(info.presentationTimeUs - prev - 20_000) > 1_000) {
                            c.releaseOutputBuffer(o, false)
                            running = false
                            Native.hostSetAudio(h, false, "codificador Opus do aparelho não usa quadros de 20 ms")
                            return
                        }
                        val bytes = ByteArray(info.size)
                        buf.position(info.offset)
                        buf.limit(info.offset + info.size)
                        buf.get(bytes)
                        Native.hostPushAudio(h, bytes)
                    }
                    c.releaseOutputBuffer(o, false)
                }
            }
        } catch (e: Exception) {
            if (running) {
                running = false
                Log.w("LiveScreen", "áudio falhou", e)
                Native.hostSetAudio(h, false, "falha no áudio: ${e.message}")
            }
        }
    }

    private fun release() {
        codec?.let {
            runCatching { it.stop() }
            it.release()
        }
        codec = null
        record?.release()
        record = null
    }

    fun stop() {
        running = false
        thread?.join()
        thread = null
        release()
    }

    private companion object {
        const val RATE = 48_000
        const val CHUNK = 3840 // 20 ms de PCM 16 bits estéreo
    }
}
