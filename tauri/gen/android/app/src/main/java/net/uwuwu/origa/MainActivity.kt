package net.uwuwu.origa

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import android.view.View
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import java.io.File

private const val MAX_SHARE_BYTES: Long = 200L * 1024 * 1024

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        captureShare(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        captureShare(intent)
    }

    private fun captureShare(intent: Intent?) {
        if (intent == null || intent.action != Intent.ACTION_SEND) {
            return
        }
        val mimeType = intent.type ?: return
        when {
            mimeType == "text/plain" && intent.hasExtra(Intent.EXTRA_TEXT) -> {
                val text = intent.getStringExtra(Intent.EXTRA_TEXT) ?: return
                ShareBuffer.setPending(text)
            }
            mimeType.startsWith("image/") || mimeType.startsWith("audio/") -> {
                @Suppress("DEPRECATION")
                val uri = intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM) ?: return
                captureSharedUri(uri, mimeType)
            }
        }
    }

    private fun captureSharedUri(uri: Uri, mimeType: String) {
        try {
            val resolver = contentResolver ?: return
            val displayName = queryDisplayName(uri)
            val size = querySize(uri)
            if (size > MAX_SHARE_BYTES) {
                ShareBuffer.setError("The shared file exceeds the size limit")
                return
            }
            val stream = resolver.openInputStream(uri) ?: return
            val buffer = java.io.ByteArrayOutputStream()
            val chunk = ByteArray(64 * 1024)
            var total = 0L
            var oversized = false
            stream.use { input ->
                while (true) {
                    val read = input.read(chunk)
                    if (read < 0) break
                    total += read
                    if (total > MAX_SHARE_BYTES) {
                        oversized = true
                        break
                    }
                    buffer.write(chunk, 0, read)
                }
            }
            if (oversized || total > MAX_SHARE_BYTES) {
                ShareBuffer.setError("The shared file exceeds the size limit")
                return
            }
            val bytes = buffer.toByteArray()
            val extension = displayName.substringAfterLast('.', "").lowercase()
            val dir = File(cacheDir, "share-intake")
            dir.mkdirs()
            val fileName = "${java.util.UUID.randomUUID()}" +
                if (extension.isNotEmpty()) ".$extension" else ""
            val target = File(dir, fileName)
            target.writeBytes(bytes)
            ShareBuffer.setPending(displayName, mimeType, target.absolutePath)
        } catch (e: Exception) {
            ShareBuffer.setError("Shared file read failed: ${e.message}")
        }
    }

    private fun querySize(uri: Uri): Long {
        try {
            contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                if (sizeIndex >= 0 && cursor.moveToFirst() && !cursor.isNull(sizeIndex)) {
                    return cursor.getLong(sizeIndex)
                }
            }
        } catch (_: Exception) {
        }
        return -1L // unknown → skip the pre-check, readBytes cap below
    }

    private fun queryDisplayName(uri: Uri): String {
        try {
            contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                if (nameIndex >= 0 && cursor.moveToFirst()) {
                    cursor.getString(nameIndex)?.let { return it }
                }
            }
        } catch (_: Exception) {
        }
        return uri.lastPathSegment ?: "file"
    }

    override fun onStart() {
        super.onStart()
        configureWebView()
    }

    private fun configureWebView() {
        val decorView = window?.decorView ?: return
        decorView.post {
            applyNativeScrollSettings(decorView.rootView)
        }
    }

    private fun applyNativeScrollSettings(view: View) {
        if (view is WebView) {
            with(view.settings) {
                setSupportZoom(false)
                builtInZoomControls = false
                displayZoomControls = false
            }
            view.overScrollMode = View.OVER_SCROLL_NEVER
        }
        if (view is android.view.ViewGroup) {
            for (i in 0 until view.childCount) {
                applyNativeScrollSettings(view.getChildAt(i))
            }
        }
    }
}

// In-memory buffer for pending shares: Kotlin writes, the Rust
// `get_pending_share` command drains it over JNI. Take-once semantics —
// a newer share overwrites an unconsumed older one.
object ShareBuffer {
    private var pendingJson: String? = null

    @Synchronized
    fun setPending(text: String) {
        pendingJson = buildString {
            append("{\"kind\":\"text\",\"text\":")
            append(jsonEscape(text))
            append("}")
        }
    }

    @Synchronized
    fun setPending(fileName: String, mime: String, cachePath: String) {
        pendingJson = buildString {
            append("{\"kind\":\"file\",\"fileName\":")
            append(jsonEscape(fileName))
            append(",\"mime\":")
            append(jsonEscape(mime))
            append(",\"cachePath\":")
            append(jsonEscape(cachePath))
            append("}")
        }
    }

    @Synchronized
    fun setError(message: String) {
        pendingJson = buildString {
            append("{\"kind\":\"error\",\"message\":")
            append(jsonEscape(message))
            append("}")
        }
    }

    /// Drains and clears the pending share (null when empty).
    @JvmStatic
    fun takePending(): String? = pendingJson.also { pendingJson = null }

    private fun jsonEscape(value: String): String {
        val sb = StringBuilder("\"")
        for (ch in value) {
            when (ch) {
                '"' -> sb.append("\\\"")
                '\\' -> sb.append("\\\\")
                '\n' -> sb.append("\\n")
                '\r' -> sb.append("\\r")
                '\t' -> sb.append("\\t")
                else -> if (ch < ' ') sb.append("\\u%04x".format(ch.code)) else sb.append(ch)
            }
        }
        sb.append("\"")
        return sb.toString()
    }
}
