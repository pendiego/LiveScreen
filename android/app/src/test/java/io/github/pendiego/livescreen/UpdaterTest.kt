package io.github.pendiego.livescreen

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import java.io.File
import java.io.IOException
import java.security.MessageDigest

class UpdaterTest {
    private fun apk(digest: String?) = GhAsset(Updater.ASSET, "https://x/${Updater.ASSET}", digest)

    @Test
    fun picksOnlyNewerReleaseWithHashedApk() {
        assertEquals(
            UpdateInfo("0.2.0", "https://x/livescreen-android.apk", "abcd"),
            Updater.pick("v0.2.0", listOf(apk("sha256:ABCD")), "0.1.0"),
        )
        assertNotNull(Updater.pick("v0.10.0", listOf(apk("sha256:ab")), "0.9.0"))
        assertNull(Updater.pick("v0.1.0", listOf(apk("sha256:ab")), "0.1.0"))
        assertNull(Updater.pick("v0.0.9", listOf(apk("sha256:ab")), "0.1.0"))
        assertNull(Updater.pick("nightly", listOf(apk("sha256:ab")), "0.1.0"))
        assertNull(
            Updater.pick("v0.2.0", listOf(GhAsset("livescreen-windows-x64.zip", "https://x/z", "sha256:ab")), "0.1.0"),
        )
        assertNull(Updater.pick("v0.2.0", listOf(apk(null)), "0.1.0"))
    }

    @Test
    fun downloadRejectsWrongHash() {
        val tmp = java.nio.file.Files.createTempDirectory("upd").toFile()
        val src = File(tmp, "src.bin").apply { writeBytes("apk".toByteArray()) }
        val sha = MessageDigest.getInstance("SHA-256").digest("apk".toByteArray())
            .joinToString("") { "%02x".format(it) }
        val cache = File(tmp, "cache")

        val ok = Updater.download(cache, UpdateInfo("1.0.0", src.toURI().toString(), sha)) {}
        assertEquals("apk", ok.readText())

        try {
            Updater.download(cache, UpdateInfo("1.0.0", src.toURI().toString(), "00")) {}
            fail("deveria rejeitar hash errado")
        } catch (_: IOException) {
        }
        assertTrue(File(cache, "updates").listFiles { f -> f.extension == "apk" }!!.isEmpty())
        assertFalse(File(cache, "updates/livescreen-1.0.0.apk").exists())
        tmp.deleteRecursively()
    }
}
