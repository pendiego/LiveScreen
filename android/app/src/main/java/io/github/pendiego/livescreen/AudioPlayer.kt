package io.github.pendiego.livescreen

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import android.media.MediaCodec
import android.media.MediaFormat
import java.nio.ByteBuffer
import java.nio.ByteOrder

import android.util.Log
class AudioPlayer(private val handle: Long) {
    @Volatile
    var muted: Boolean = false
        set(value) {
            field = value
            try {
                track?.setVolume(if (value) 0f else 1f)
            } catch (_: Exception) {}
        }

    @Volatile
    private var running = false
    private var thread: Thread? = null

    @Volatile
    private var track: AudioTrack? = null

    fun start() {
        running = true
        thread = Thread({ runLoop() }, "audio").apply { start() }
    }

    fun stop() {
        running = false
        thread?.join()
    }

    private fun runLoop() {
        val csd0 = ByteBuffer.wrap(
            byteArrayOf(
                'O'.code.toByte(), 'p'.code.toByte(), 'u'.code.toByte(), 's'.code.toByte(),
                'H'.code.toByte(), 'e'.code.toByte(), 'a'.code.toByte(), 'd'.code.toByte(),
                1, // Versão 1
                2, // 2 canais
                0, 0, // Pre-skip 0 (u16 LE)
                0x80.toByte(), 0xbb.toByte(), 0, 0, // 48000 Hz (u32 LE: 0x0000BB80)
                0, 0, // Ganho 0 (i16 LE)
                0 // Mapping family 0
            )
        )
        val csd1 = ByteBuffer.allocate(8).order(ByteOrder.LITTLE_ENDIAN).putLong(0L).apply { flip() }
        val csd2 = ByteBuffer.allocate(8).order(ByteOrder.LITTLE_ENDIAN).putLong(80_000_000L).apply { flip() }

        val format = MediaFormat.createAudioFormat("audio/opus", 48000, 2).apply {
            setByteBuffer("csd-0", csd0)
            setByteBuffer("csd-1", csd1)
            setByteBuffer("csd-2", csd2)
        }

        val codec = try {
            MediaCodec.createDecoderByType("audio/opus").apply {
                configure(format, null, null, 0)
                start()
            }
        } catch (e: Exception) {
            Log.e("LiveScreen", "AudioPlayer: falha ao criar/iniciar codec audio/opus", e)
            return
        }

        val minBuf = AudioTrack.getMinBufferSize(
            48000,
            AudioFormat.CHANNEL_OUT_STEREO,
            AudioFormat.ENCODING_PCM_16BIT
        )
        val bufSize = maxOf(minBuf, 38400) // 200 ms: 48000 * 2 * 2 * 0.2

        val audioTrack = try {
            try {
                AudioTrack.Builder()
                    .setAudioAttributes(
                        AudioAttributes.Builder()
                            .setUsage(AudioAttributes.USAGE_MEDIA)
                            .setContentType(AudioAttributes.CONTENT_TYPE_MOVIE)
                            .build()
                    )
                    .setAudioFormat(
                        AudioFormat.Builder()
                            .setSampleRate(48000)
                            .setChannelMask(AudioFormat.CHANNEL_OUT_STEREO)
                            .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                            .build()
                    )
                    .setBufferSizeInBytes(bufSize)
                    .setPerformanceMode(AudioTrack.PERFORMANCE_MODE_LOW_LATENCY)
                    .setTransferMode(AudioTrack.MODE_STREAM)
                    .build()
            } catch (e: Exception) {
                Log.w("LiveScreen", "AudioPlayer: AudioTrack com PERFORMANCE_MODE_LOW_LATENCY falhou, tentando padrão: ${e.message}")
                AudioTrack.Builder()
                    .setAudioAttributes(
                        AudioAttributes.Builder()
                            .setUsage(AudioAttributes.USAGE_MEDIA)
                            .setContentType(AudioAttributes.CONTENT_TYPE_MOVIE)
                            .build()
                    )
                    .setAudioFormat(
                        AudioFormat.Builder()
                            .setSampleRate(48000)
                            .setChannelMask(AudioFormat.CHANNEL_OUT_STEREO)
                            .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                            .build()
                    )
                    .setBufferSizeInBytes(bufSize)
                    .setTransferMode(AudioTrack.MODE_STREAM)
                    .build()
            }
        } catch (e: Exception) {
            Log.e("LiveScreen", "AudioPlayer: falha ao criar AudioTrack", e)
            try { codec.stop() } catch (_: Exception) {}
            try { codec.release() } catch (_: Exception) {}
            return
        }

        track = audioTrack
        audioTrack.setVolume(if (muted) 0f else 1f)
        audioTrack.play()
        Log.i("LiveScreen", "AudioPlayer: áudio iniciado (bufSize=$bufSize, minBuf=$minBuf)")
        var packetsReceived = 0L
        var totalBytesWritten = 0L
        var lastLogMs = System.currentTimeMillis()

        var pts = 0L
        var written = 0L
        val info = MediaCodec.BufferInfo()

        try {
            while (running) {
                val data = Native.nextAudio(handle, 50)
                if (data != null) {
                    val inIdx = codec.dequeueInputBuffer(10_000)
                    if (inIdx >= 0) {
                        val buf = codec.getInputBuffer(inIdx)
                        if (buf != null) {
                            buf.clear()
                            buf.put(data)
                            codec.queueInputBuffer(inIdx, 0, data.size, pts, 0)
                            pts += 20_000
                        }
                    }
                    packetsReceived++
                }

                while (running) {
                    val outIdx = codec.dequeueOutputBuffer(info, 0)
                    if (outIdx >= 0) {
                        val outBuf = codec.getOutputBuffer(outIdx)
                        if (outBuf != null && info.size > 0) {
                            outBuf.position(info.offset)
                            outBuf.limit(info.offset + info.size)

                            val headPos = audioTrack.playbackHeadPosition.toLong() and 0xFFFFFFFFL
                            val delayFrames = written - headPos
                            if (delayFrames > 9600) {
                                // Descarta o bloco (atraso > 200 ms)
                            } else {
                                audioTrack.write(outBuf, info.size, AudioTrack.WRITE_BLOCKING)
                                written += (info.size / 4)
                                totalBytesWritten += info.size
                                val now = System.currentTimeMillis()
                                if (now - lastLogMs >= 5000) {
                                    lastLogMs = now
                                    Log.i(
                                        "LiveScreen",
                                        "AudioPlayer: $packetsReceived pacotes recebidos, $written quadros escritos ($totalBytesWritten bytes PCM), headPos=$headPos"
                                    )
                                }
                            }
                        }
                        codec.releaseOutputBuffer(outIdx, false)
                    } else {
                        break
                    }
                }
            }
        } catch (e: Exception) {
            Log.w("LiveScreen", "AudioPlayer: erro no loop de áudio: ${e.message}", e)
        } finally {
            Log.i("LiveScreen", "AudioPlayer: finalizado, total de quadros escritos=$written ($totalBytesWritten bytes)")
            track = null
            try {
                audioTrack.stop()
            } catch (_: Exception) {}
            try {
                audioTrack.release()
            } catch (_: Exception) {}
            try {
                codec.stop()
            } catch (_: Exception) {}
            try {
                codec.release()
            } catch (_: Exception) {}
        }
    }
}
