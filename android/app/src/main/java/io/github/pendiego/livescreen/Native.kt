package io.github.pendiego.livescreen

object Native {
    init {
        System.loadLibrary("livescreen_android")
    }

    @JvmStatic external fun init(context: android.content.Context)
    @JvmStatic external fun isValidTicket(ticket: String): Boolean
    @JvmStatic external fun isLegacyInvite(ticket: String): Boolean
    @JvmStatic external fun findInvite(text: String): String?
    @JvmStatic external fun start(ticket: String, name: String, identity: String): Long
    @JvmStatic external fun status(handle: Long): String
    @JvmStatic external fun presets(): String
    @JvmStatic external fun nextVideo(handle: Long, timeoutMs: Int, meta: LongArray): ByteArray?
    @JvmStatic external fun nextAudio(handle: Long, timeoutMs: Int): ByteArray?
    @JvmStatic external fun reportDecoded(handle: Long, frames: Int, busyUs: Long, hardware: Boolean)
    @JvmStatic external fun decoderFailed(handle: Long)
    @JvmStatic external fun requestKeyframe(handle: Long)
    @JvmStatic external fun requestQuality(handle: Long, id: String)
    @JvmStatic external fun sendChat(handle: Long, text: String)
    @JvmStatic external fun chatSince(handle: Long, after: Long): String
    @JvmStatic external fun setVisible(handle: Long, visible: Boolean)
    @JvmStatic external fun stop(handle: Long)

    @JvmStatic external fun friendCode(identity: String): String?
    @JvmStatic external fun isValidFriendCode(code: String): Boolean
    @JvmStatic external fun presenceStart(identity: String, codes: String): Long
    @JvmStatic external fun presenceSetIds(handle: Long, codes: String)
    @JvmStatic external fun presenceStatus(handle: Long): String
    @JvmStatic external fun presenceStop(handle: Long)

    @JvmStatic external fun encodeParams(presetId: String, srcW: Int, srcH: Int, hz: Int): String
    @JvmStatic external fun hostStart(
        identity: String, name: String, title: String, codes: String, maxViewers: Int,
        presetId: String, srcW: Int, srcH: Int, hz: Int, width: Int, height: Int, fps: Int,
        bitrate: Int, encoderName: String, audioOn: Boolean, audioError: String
    ): Long
    @JvmStatic external fun hostPushVideo(handle: Long, data: ByteArray, ptsUs: Long, keyframe: Boolean)
    @JvmStatic external fun hostPushAudio(handle: Long, opus: ByteArray)
    @JvmStatic external fun hostTakeKeyframe(handle: Long): Boolean
    @JvmStatic external fun hostTargetBps(handle: Long): Int
    @JvmStatic external fun hostTakeQualityRequest(handle: Long): String?
    @JvmStatic external fun hostSetStream(handle: Long, presetId: String, width: Int, height: Int, fps: Int, bitrate: Int)
    @JvmStatic external fun hostSetAudio(handle: Long, on: Boolean, error: String)
    @JvmStatic external fun hostStatus(handle: Long): String
    @JvmStatic external fun hostWebLink(handle: Long): String?
    @JvmStatic external fun hostNewInvite(handle: Long)
    @JvmStatic external fun hostKick(handle: Long, id: String)
    @JvmStatic external fun hostSetFriends(handle: Long, codes: String)
    @JvmStatic external fun hostSendChat(handle: Long, text: String)
    @JvmStatic external fun hostChatSince(handle: Long, after: Long): String
    @JvmStatic external fun hostStop(handle: Long)
}
