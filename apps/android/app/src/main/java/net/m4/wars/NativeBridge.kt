package net.m4.wars

import android.view.Surface

/** JNI surface of the Rust engine (crates/ffi/src/android.rs). */
object NativeBridge {
    init {
        System.loadLibrary("wars_ffi")
    }

    external fun create(width: Int, height: Int, scale: Float, dataDir: String)
    external fun surfaceCreated(surface: Surface, width: Int, height: Int): Boolean
    external fun surfaceChanged(width: Int, height: Int)
    external fun surfaceDestroyed()
    external fun drawFrame(): Boolean
    external fun touch(kind: Int, x: Float, y: Float)
    external fun longPress(x: Float, y: Float)
    external fun doubleTap(x: Float, y: Float)
    external fun pinch(scale: Float, x: Float, y: Float)
    external fun pan(dx: Float, dy: Float)
    external fun key(code: Int, down: Boolean)
    external fun text(s: String)
    external fun wantsKeyboard(): Boolean
    external fun back(): Boolean
}
