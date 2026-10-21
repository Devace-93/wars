//! JNI entry points for the Android shell (`net.m4.wars.NativeBridge`).

use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jfloat, jint};
use jni::JNIEnv;
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::ptr::NonNull;

static WINDOW: std::sync::Mutex<Option<ndk::native_window::NativeWindow>> =
    std::sync::Mutex::new(None);

fn init_logging() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Warn)
                .with_tag("wars"),
        );
        std::panic::set_hook(Box::new(|info| {
            log::error!("panic: {info}");
        }));
    });
}

#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_create(
    mut env: JNIEnv,
    _c: JClass,
    width: jint,
    height: jint,
    scale: jfloat,
    data_dir: JString,
) {
    init_logging();
    let dir: String = env
        .get_string(&data_dir)
        .map(|s| s.into())
        .unwrap_or_default();
    log::warn!("create {width}x{height} scale {scale} dir {dir}");
    crate::create(width.max(1) as u32, height.max(1) as u32, scale, Some(&dir));
}

#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_surfaceCreated(
    env: JNIEnv,
    _c: JClass,
    surface: JObject,
    width: jint,
    height: jint,
) -> jboolean {
    init_logging();
    let window =
        unsafe { ndk::native_window::NativeWindow::from_surface(env.get_raw(), surface.as_raw()) };
    let Some(window) = window else {
        log::error!("no native window");
        return 0;
    };
    let ptr = NonNull::new(window.ptr().as_ptr() as *mut std::ffi::c_void).unwrap();
    let raw = RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(ptr));
    let display = RawDisplayHandle::Android(AndroidDisplayHandle::new());
    *WINDOW.lock().unwrap() = Some(window);
    let target = wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: display,
        raw_window_handle: raw,
    };
    match unsafe { crate::attach_surface(target, width.max(1) as u32, height.max(1) as u32) } {
        Ok(()) => 1,
        Err(e) => {
            log::error!("attach surface: {e}");
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_surfaceChanged(
    _env: JNIEnv,
    _c: JClass,
    width: jint,
    height: jint,
) {
    crate::resize(width.max(1) as u32, height.max(1) as u32);
}

#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_surfaceDestroyed(_env: JNIEnv, _c: JClass) {
    crate::detach_surface();
    *WINDOW.lock().unwrap() = None;
}

#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_drawFrame(
    _env: JNIEnv,
    _c: JClass,
) -> jboolean {
    crate::frame() as jboolean
}

#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_touch(
    _env: JNIEnv,
    _c: JClass,
    kind: jint,
    x: jfloat,
    y: jfloat,
) {
    crate::touch(kind, x, y);
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_longPress(
    _env: JNIEnv,
    _c: JClass,
    x: jfloat,
    y: jfloat,
) {
    crate::long_press(x, y);
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_doubleTap(
    _env: JNIEnv,
    _c: JClass,
    x: jfloat,
    y: jfloat,
) {
    crate::double_tap(x, y);
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_pinch(
    _env: JNIEnv,
    _c: JClass,
    scale: jfloat,
    x: jfloat,
    y: jfloat,
) {
    crate::pinch(scale, x, y);
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_pan(
    _env: JNIEnv,
    _c: JClass,
    dx: jfloat,
    dy: jfloat,
) {
    crate::pan(dx, dy);
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_key(
    _env: JNIEnv,
    _c: JClass,
    code: jint,
    down: jboolean,
) {
    crate::key(code, down != 0);
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_text(mut env: JNIEnv, _c: JClass, s: JString) {
    if let Ok(t) = env.get_string(&s) {
        let t: String = t.into();
        crate::text(&t);
    }
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_wantsKeyboard(
    _env: JNIEnv,
    _c: JClass,
) -> jboolean {
    crate::wants_keyboard() as jboolean
}
#[no_mangle]
pub extern "system" fn Java_net_m4_wars_NativeBridge_back(_env: JNIEnv, _c: JClass) -> jboolean {
    crate::back() as jboolean
}
