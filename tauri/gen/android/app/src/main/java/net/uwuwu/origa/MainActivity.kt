package net.uwuwu.origa

import android.os.Bundle
import android.view.View
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
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
