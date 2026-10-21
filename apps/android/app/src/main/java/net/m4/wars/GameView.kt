package net.m4.wars

import android.content.Context
import android.view.Choreographer
import android.view.GestureDetector
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import android.view.inputmethod.BaseInputConnection
import kotlin.math.abs

/**
 * The game surface. Gestures become the same semantic inputs the desktop uses:
 * tap = click, drag = box select, long press = context action (right click),
 * double tap = select all of a type, two-finger drag = pan, pinch = zoom.
 */
class GameView(context: Context, private val onQuit: () -> Unit) : SurfaceView(context), SurfaceHolder.Callback {
    private var created = false
    private var running = false
    private var keyboardShown = false
    private var dragging = false
    private var multiTouch = false
    private var lastPanX = 0f
    private var lastPanY = 0f
    private var downX = 0f
    private var downY = 0f

    private val frameCallback = object : Choreographer.FrameCallback {
        override fun doFrame(frameTimeNanos: Long) {
            if (!running) return
            if (!NativeBridge.drawFrame()) {
                onQuit()
                return
            }
            syncKeyboard()
            Choreographer.getInstance().postFrameCallback(this)
        }
    }

    private val gestures = GestureDetector(context, object : GestureDetector.SimpleOnGestureListener() {
        override fun onDown(e: MotionEvent): Boolean = true
        override fun onSingleTapConfirmed(e: MotionEvent): Boolean {
            NativeBridge.touch(0, e.x, e.y)
            NativeBridge.touch(2, e.x, e.y)
            return true
        }
        override fun onDoubleTap(e: MotionEvent): Boolean {
            NativeBridge.doubleTap(e.x, e.y)
            return true
        }
        override fun onLongPress(e: MotionEvent) {
            if (!multiTouch && !dragging) {
                performHapticFeedback(android.view.HapticFeedbackConstants.LONG_PRESS)
                NativeBridge.longPress(e.x, e.y)
            }
        }
    })

    private val scaler = ScaleGestureDetector(context, object : ScaleGestureDetector.SimpleOnScaleGestureListener() {
        override fun onScale(d: ScaleGestureDetector): Boolean {
            NativeBridge.pinch(d.scaleFactor, d.focusX, d.focusY)
            return true
        }
    })

    init {
        holder.addCallback(this)
        isFocusable = true
        isFocusableInTouchMode = true
        keepScreenOn = true
    }

    private fun scale(): Float {
        // UI scale: ~2.0 on a typical phone so the 52 px command cells become ~40 dp.
        val dpi = resources.displayMetrics.densityDpi
        return (dpi / 200f).coerceIn(1.2f, 3.0f)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        val w = width.coerceAtLeast(1)
        val h = height.coerceAtLeast(1)
        if (!created) {
            NativeBridge.create(w, h, scale(), context.filesDir.absolutePath)
            created = true
        }
        NativeBridge.surfaceCreated(holder.surface, w, h)
        running = true
        Choreographer.getInstance().postFrameCallback(frameCallback)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        NativeBridge.surfaceChanged(width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        running = false
        Choreographer.getInstance().removeFrameCallback(frameCallback)
        NativeBridge.surfaceDestroyed()
    }

    override fun onTouchEvent(e: MotionEvent): Boolean {
        scaler.onTouchEvent(e)
        if (e.pointerCount >= 2) {
            multiTouch = true
            if (dragging) {
                // Cancel the box selection that a second finger interrupted.
                NativeBridge.touch(2, downX, downY)
                dragging = false
            }
            val cx = (e.getX(0) + e.getX(1)) / 2f
            val cy = (e.getY(0) + e.getY(1)) / 2f
            if (e.actionMasked == MotionEvent.ACTION_POINTER_DOWN || e.actionMasked == MotionEvent.ACTION_DOWN) {
                lastPanX = cx
                lastPanY = cy
            } else if (e.actionMasked == MotionEvent.ACTION_MOVE && !scaler.isInProgress) {
                NativeBridge.pan(cx - lastPanX, cy - lastPanY)
                lastPanX = cx
                lastPanY = cy
            }
            if (e.actionMasked == MotionEvent.ACTION_MOVE) {
                lastPanX = cx
                lastPanY = cy
            }
            return true
        }
        val handledByGestures = gestures.onTouchEvent(e)
        when (e.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                multiTouch = false
                dragging = false
                downX = e.x
                downY = e.y
            }
            MotionEvent.ACTION_MOVE -> {
                if (!multiTouch) {
                    val moved = abs(e.x - downX) > 24f || abs(e.y - downY) > 24f
                    if (!dragging && moved) {
                        dragging = true
                        NativeBridge.touch(0, downX, downY)
                    }
                    if (dragging) NativeBridge.touch(1, e.x, e.y)
                }
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                if (dragging) {
                    NativeBridge.touch(2, e.x, e.y)
                    dragging = false
                }
                multiTouch = false
            }
        }
        return handledByGestures || true
    }

    private fun syncKeyboard() {
        val wants = NativeBridge.wantsKeyboard()
        if (wants != keyboardShown) {
            keyboardShown = wants
            val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
            if (wants) {
                requestFocus()
                imm.showSoftInput(this, InputMethodManager.SHOW_IMPLICIT)
            } else {
                imm.hideSoftInputFromWindow(windowToken, 0)
            }
        }
    }

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        outAttrs.inputType = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_FLAG_NO_SUGGESTIONS
        outAttrs.imeOptions = EditorInfo.IME_ACTION_DONE or EditorInfo.IME_FLAG_NO_EXTRACT_UI
        return object : BaseInputConnection(this, false) {
            override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
                if (text != null) NativeBridge.text(text.toString())
                return true
            }
            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
                repeat(beforeLength) { NativeBridge.key(3, true) }
                return true
            }
            override fun performEditorAction(actionCode: Int): Boolean {
                NativeBridge.key(2, true)
                return true
            }
        }
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val code = when (keyCode) {
            KeyEvent.KEYCODE_ENTER -> 2
            KeyEvent.KEYCODE_DEL -> 3
            KeyEvent.KEYCODE_TAB -> 4
            KeyEvent.KEYCODE_SPACE -> 5
            KeyEvent.KEYCODE_DPAD_UP -> 6
            KeyEvent.KEYCODE_DPAD_DOWN -> 7
            KeyEvent.KEYCODE_DPAD_LEFT -> 8
            KeyEvent.KEYCODE_DPAD_RIGHT -> 9
            KeyEvent.KEYCODE_FORWARD_DEL -> 10
            else -> {
                val c = event.unicodeChar
                if (c in 32..126) c else -1
            }
        }
        if (code > 0) {
            NativeBridge.key(code, true)
            if (code in 32..126 && !NativeBridge.wantsKeyboard()) return true
            if (code in 32..126) NativeBridge.text(code.toChar().toString())
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        val c = event.unicodeChar
        if (c in 32..126) NativeBridge.key(c, false)
        return super.onKeyUp(keyCode, event)
    }
}
