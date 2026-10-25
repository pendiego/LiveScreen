package io.github.pendiego.livescreen

import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.core.content.FileProvider
import org.json.JSONObject
import java.io.File
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLConnection
import java.security.MessageDigest

/**
 * Espelha `src/update.rs` do desktop: mesma release `latest` do GitHub, mesmo critério de versão
 * (3 números, comparação numérica) e o sha256 vem do campo `digest` do asset. O Android não deixa
 * instalar em silêncio, então aqui só baixamos e conferimos; quem instala é o instalador do sistema.
 * Fica em Kotlin porque a lib Rust do Android não tem cliente HTTP.
 */
data class UpdateInfo(val version: String, val url: String, val sha256: String)
data class GhAsset(val name: String, val url: String, val digest: String?)

object Updater {
    const val ASSET = "livescreen-android.apk"
    private val API = "https://api.github.com/repos/${BuildConfig.UPDATE_REPO}/releases/latest"

    internal fun version(v: String): List<Long>? {
        val parts = v.removePrefix("v").split('-', '+').first().split('.')
        if (parts.size != 3) return null
        val nums = parts.map { it.toLongOrNull() ?: return null }
        return nums
    }

    private fun isNewer(a: List<Long>, b: List<Long>): Boolean {
        for (i in a.indices) if (a[i] != b[i]) return a[i] > b[i]
        return false
    }

    internal fun pick(tag: String, assets: List<GhAsset>, current: String): UpdateInfo? {
        val latest = version(tag) ?: return null
        val cur = version(current) ?: return null
        if (!isNewer(latest, cur)) return null
        val asset = assets.firstOrNull { it.name == ASSET }
        val digest = asset?.digest
        if (asset == null || digest == null || !digest.startsWith("sha256:")) {
            Log.w("LiveScreen", "release $tag sem $ASSET com sha256; ignorada")
            return null
        }
        return UpdateInfo(tag.removePrefix("v"), asset.url, digest.removePrefix("sha256:").lowercase())
    }

    /** Bloqueante: chamar em Dispatchers.IO. */
    fun check(): UpdateInfo? {
        val conn = URL(API).openConnection() as HttpURLConnection
        try {
            conn.setRequestProperty("Accept", "application/vnd.github+json")
            conn.setRequestProperty("User-Agent", "LiveScreen-Android/${BuildConfig.VERSION_NAME}")
            conn.connectTimeout = 15000
            conn.readTimeout = 30000
            if (conn.responseCode != 200) throw IOException("GitHub respondeu HTTP ${conn.responseCode}")
            val json = JSONObject(conn.inputStream.bufferedReader().readText())
            val arr = json.getJSONArray("assets")
            val assets = (0 until arr.length()).map {
                val a = arr.getJSONObject(it)
                GhAsset(a.optString("name"), a.optString("browser_download_url"), a.optString("digest").ifEmpty { null })
            }
            return pick(json.getString("tag_name"), assets, BuildConfig.VERSION_NAME)
        } finally {
            conn.disconnect()
        }
    }

    /** Bloqueante. `URLConnection` puro (sem cast) para o teste poder usar URL `file:`. */
    fun download(cacheDir: File, info: UpdateInfo, onProgress: (Float) -> Unit): File {
        val dir = File(cacheDir, "updates").apply { mkdirs() }
        dir.listFiles { f -> f.extension == "apk" }?.forEach { it.delete() }
        val out = File(dir, "livescreen-${info.version}.apk")
        val md = MessageDigest.getInstance("SHA-256")
        val conn: URLConnection = URL(info.url).openConnection()
        conn.connectTimeout = 15000
        conn.readTimeout = 30000
        try {
            val total = conn.contentLengthLong
            var read = 0L
            conn.getInputStream().use { input ->
                out.outputStream().use { os ->
                    val buf = ByteArray(64 * 1024)
                    while (true) {
                        val n = input.read(buf)
                        if (n < 0) break
                        md.update(buf, 0, n)
                        os.write(buf, 0, n)
                        read += n
                        if (total > 0) onProgress(read.toFloat() / total)
                    }
                }
            }
        } catch (e: Exception) {
            out.delete()
            throw e
        } finally {
            (conn as? HttpURLConnection)?.disconnect()
        }
        val got = md.digest().joinToString("") { "%02x".format(it) }
        if (got != info.sha256) {
            out.delete()
            throw IOException("download corrompido (sha256 $got, esperado ${info.sha256})")
        }
        return out
    }

    fun installIntent(context: Context, apk: File): Intent =
        Intent(Intent.ACTION_VIEW)
            .setDataAndType(
                FileProvider.getUriForFile(context, "${context.packageName}.updates", apk),
                "application/vnd.android.package-archive",
            )
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
}
