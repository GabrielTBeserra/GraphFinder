// Reveal the native window as soon as the splash has painted —
// the user never sees a blank webview.
function gfShowWindow() {
  try {
    window.__TAURI__ && window.__TAURI__.window.getCurrentWindow().show()
  } catch (_) {}
}
if (document.readyState === 'loading') {
  addEventListener('DOMContentLoaded', gfShowWindow)
} else {
  gfShowWindow()
}
