package io.github.pendiego.livescreen

object Native {
    init {
        System.loadLibrary("livescreen_android")
    }

    @JvmStatic external fun isValidTicket(ticket: String): Boolean
    @JvmStatic external fun start(ticket: String, name: String): Long
    @JvmStatic external fun status(handle: Long): String
    @JvmStatic external fun presets(): String
    @JvmStatic external fun nextVideo(handle: Long, timeoutMs: Int, meta: LongArray): ByteArray?
    @JvmStatic external fun nextAudio(handle: Long, timeoutMs: Int): ByteArray?
    @JvmStatic external fun reportDecoded(handle: Long, frames: Int, busyUs: Long, hardware: Boolean)
    @JvmStatic external fun decoderFailed(handle: Long)
    @JvmStatic external fun requestKeyframe(handle: Long)
    @JvmStatic external fun requestQuality(handle: Long, id: String)
    @JvmStatic external fun stop(handle: Long)
}
