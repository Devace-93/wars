//! wars-ffi: the mobile bridge. One global engine (app + renderer) driven by the
//! platform shell through a small C ABI (iOS) and JNI functions (Android).

use std::ffi::{c_char, c_void, CStr};
use std::sync::Mutex;
use std::time::Instant;
use wars_app::App;
use wars_render::Renderer;
use wars_ui::{Input, Key, Mods};

#[cfg(target_os = "android")]
pub mod android;

pub struct Engine {
    pub app: App,
    pub renderer: Option<Renderer>,
    pub last: Instant,
    pub scale: f32,
    pub phys: (u32, u32),
}

pub static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

pub fn with_engine<R>(f: impl FnOnce(&mut Engine) -> R) -> Option<R> {
    let mut g = ENGINE.lock().ok()?;
    g.as_mut().map(f)
}

/// Create the app for a physical surface size and UI scale (logical = physical / scale).
pub fn create(width: u32, height: u32, scale: f32, data_dir: Option<&str>) {
    if let Some(d) = data_dir {
        wars_client::saves::set_data_dir(std::path::PathBuf::from(d));
    }
    let scale = scale.max(0.5);
    let mut app = App::new(width as f32 / scale, height as f32 / scale);
    app.touch = true;
    let mut g = ENGINE.lock().unwrap();
    *g = Some(Engine {
        app,
        renderer: None,
        last: Instant::now(),
        scale,
        phys: (width, height),
    });
}

/// Attach a surface from raw window/display handles.
///
/// # Safety
/// Handles must be valid for as long as the renderer lives (until `detach_surface`).
pub unsafe fn attach_surface(
    target: wgpu::SurfaceTargetUnsafe,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let mut g = ENGINE.lock().unwrap();
    let Some(e) = g.as_mut() else {
        return Err("engine not created".into());
    };
    let mut r = unsafe { Renderer::from_raw(target, width, height) }?;
    r.scale = e.scale;
    e.phys = (width, height);
    e.app.input(Input::Resize {
        w: width as f32 / e.scale,
        h: height as f32 / e.scale,
    });
    e.renderer = Some(r);
    Ok(())
}

pub fn detach_surface() {
    if let Some(e) = ENGINE.lock().unwrap().as_mut() {
        e.renderer = None;
    }
}

pub fn resize(width: u32, height: u32) {
    with_engine(|e| {
        e.phys = (width, height);
        if let Some(r) = e.renderer.as_mut() {
            r.resize(width, height);
        }
        let s = e.scale;
        e.app.input(Input::Resize {
            w: width as f32 / s,
            h: height as f32 / s,
        });
    });
}

/// Advance and draw one frame. Returns false when the app asked to quit.
pub fn frame() -> bool {
    with_engine(|e| {
        let now = Instant::now();
        let dt = (now - e.last).as_secs_f32().min(0.25);
        e.last = now;
        e.app.frame(dt);
        if let Some(r) = e.renderer.as_mut() {
            match r.render(&e.app.dl, &mut e.app.text, [0.02, 0.015, 0.01, 1.0]) {
                Ok(()) => {}
                Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                    let (w, h) = e.phys;
                    r.resize(w, h);
                }
                Err(err) => log::warn!("render: {err}"),
            }
        }
        !e.app.quit
    })
    .unwrap_or(false)
}

fn logical(e: &Engine, x: f32, y: f32) -> (f32, f32) {
    (x / e.scale, y / e.scale)
}

/// Touch primitives from the shell's gesture detectors.
/// kind: 0 = down, 1 = move, 2 = up (one finger: acts as the left mouse button).
pub fn touch(kind: i32, x: f32, y: f32) {
    with_engine(|e| {
        let (lx, ly) = logical(e, x, y);
        match kind {
            0 => {
                e.app.input(Input::MouseMove { x: lx, y: ly });
                e.app.input(Input::MouseDown {
                    button: 0,
                    x: lx,
                    y: ly,
                });
            }
            1 => e.app.input(Input::MouseMove { x: lx, y: ly }),
            _ => {
                e.app.input(Input::MouseMove { x: lx, y: ly });
                e.app.input(Input::MouseUp {
                    button: 0,
                    x: lx,
                    y: ly,
                });
            }
        }
    });
}

pub fn long_press(x: f32, y: f32) {
    with_engine(|e| {
        let (lx, ly) = logical(e, x, y);
        e.app.input(Input::MouseMove { x: lx, y: ly });
        e.app.input(Input::LongPress { x: lx, y: ly });
    });
}

pub fn double_tap(x: f32, y: f32) {
    with_engine(|e| {
        let (lx, ly) = logical(e, x, y);
        e.app.input(Input::DoubleTap { x: lx, y: ly });
    });
}

pub fn pinch(scale: f32, x: f32, y: f32) {
    with_engine(|e| {
        let (lx, ly) = logical(e, x, y);
        e.app.input(Input::Pinch {
            scale,
            x: lx,
            y: ly,
        });
    });
}

pub fn pan(dx: f32, dy: f32) {
    with_engine(|e| {
        let s = e.scale;
        e.app.input(Input::Pan {
            dx: dx / s,
            dy: dy / s,
        });
    });
}

/// Hardware/soft keyboard. `code` follows the C ABI key table in `key_from_code`.
pub fn key(code: i32, down: bool) {
    with_engine(|e| {
        if let Some(k) = key_from_code(code) {
            e.app.input(Input::Key {
                key: k,
                down,
                mods: Mods::default(),
            });
        }
    });
}

pub fn text(s: &str) {
    with_engine(|e| {
        for c in s.chars() {
            if c == '\n' {
                e.app.input(Input::Key {
                    key: Key::Enter,
                    down: true,
                    mods: Mods::default(),
                });
            } else if c == '\u{8}' {
                e.app.input(Input::Key {
                    key: Key::Backspace,
                    down: true,
                    mods: Mods::default(),
                });
            } else {
                e.app.input(Input::Char(c));
            }
        }
    });
}

/// The shell shows the soft keyboard while a text field is focused.
pub fn wants_keyboard() -> bool {
    with_engine(|e| e.app.focus.is_some()).unwrap_or(false)
}

/// Android back button: returns true when consumed (not at the main menu).
pub fn back() -> bool {
    with_engine(|e| {
        let at_main = e.app.screen == wars_app::Screen::Main;
        e.app.input(Input::Back);
        !at_main
    })
    .unwrap_or(false)
}

pub fn key_from_code(code: i32) -> Option<Key> {
    Some(match code {
        1 => Key::Escape,
        2 => Key::Enter,
        3 => Key::Backspace,
        4 => Key::Tab,
        5 => Key::Space,
        6 => Key::Up,
        7 => Key::Down,
        8 => Key::Left,
        9 => Key::Right,
        10 => Key::Delete,
        c if (32..127).contains(&c) => Key::Char((c as u8 as char).to_ascii_lowercase()),
        _ => return None,
    })
}

// ---------------- C ABI (iOS and anything else) ----------------

/// # Safety
/// `data_dir` must be a valid C string or null.
#[no_mangle]
pub unsafe extern "C" fn wars_create(width: u32, height: u32, scale: f32, data_dir: *const c_char) {
    let dir = if data_dir.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(data_dir) }.to_str().ok()
    };
    create(width, height, scale, dir);
}

/// Attach a CAMetalLayer (iOS/macOS).
///
/// # Safety
/// `layer` must be a valid CAMetalLayer pointer that outlives the surface.
#[no_mangle]
pub unsafe extern "C" fn wars_attach_metal_layer(
    layer: *mut c_void,
    width: u32,
    height: u32,
) -> bool {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    {
        let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer);
        return unsafe { attach_surface(target, width, height) }
            .map_err(|e| log::error!("{e}"))
            .is_ok();
    }
    #[cfg(not(any(target_os = "ios", target_os = "macos")))]
    {
        let _ = (layer, width, height);
        false
    }
}

#[no_mangle]
pub extern "C" fn wars_detach_surface() {
    detach_surface();
}
#[no_mangle]
pub extern "C" fn wars_resize(width: u32, height: u32) {
    resize(width, height);
}
#[no_mangle]
pub extern "C" fn wars_frame() -> bool {
    frame()
}
#[no_mangle]
pub extern "C" fn wars_touch(kind: i32, x: f32, y: f32) {
    touch(kind, x, y);
}
#[no_mangle]
pub extern "C" fn wars_long_press(x: f32, y: f32) {
    long_press(x, y);
}
#[no_mangle]
pub extern "C" fn wars_double_tap(x: f32, y: f32) {
    double_tap(x, y);
}
#[no_mangle]
pub extern "C" fn wars_pinch(scale: f32, x: f32, y: f32) {
    pinch(scale, x, y);
}
#[no_mangle]
pub extern "C" fn wars_pan(dx: f32, dy: f32) {
    pan(dx, dy);
}
#[no_mangle]
pub extern "C" fn wars_key(code: i32, down: bool) {
    key(code, down);
}
/// # Safety
/// `s` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn wars_text(s: *const c_char) {
    if s.is_null() {
        return;
    }
    if let Ok(t) = unsafe { CStr::from_ptr(s) }.to_str() {
        text(t);
    }
}
#[no_mangle]
pub extern "C" fn wars_wants_keyboard() -> bool {
    wants_keyboard()
}
#[no_mangle]
pub extern "C" fn wars_back() -> bool {
    back()
}
