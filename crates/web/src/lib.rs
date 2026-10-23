#![cfg(target_arch = "wasm32")]
//! wars-web: the browser shell. A full-window canvas, wgpu (WebGPU or WebGL2), the
//! shared app, and browser events mapped to the same inputs the desktop uses.

use std::cell::RefCell;
use std::rc::Rc;
use wars_app::App;
use wars_render::Renderer;
use wars_ui::{Input, Key, Mods};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

struct Engine {
    app: App,
    renderer: Renderer,
    canvas: web_sys::HtmlCanvasElement,
    scale: f32,
    last: f64,
    // touch state
    touch_start: Option<(f32, f32, f64)>,
    dragging: bool,
    last_tap: f64,
    pinch_dist: Option<f32>,
    pan_last: Option<(f32, f32)>,
    long_press_fired: bool,
}

thread_local! {
    static ENGINE: RefCell<Option<Rc<RefCell<Engine>>>> = const { RefCell::new(None) };
}

fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

fn now_ms() -> f64 {
    window().performance().map(|p| p.now()).unwrap_or(0.0)
}

fn key_of(code: &str, key: &str) -> Option<Key> {
    Some(match code {
        "Escape" => Key::Escape,
        "Enter" | "NumpadEnter" => Key::Enter,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Tab" => Key::Tab,
        "Space" => Key::Space,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "Minus" | "NumpadSubtract" => Key::Char('-'),
        "Equal" | "NumpadAdd" => Key::Char('='),
        "Period" => Key::Char('.'),
        "Comma" => Key::Char(','),
        "Slash" => Key::Char('/'),
        _ => {
            if let Some(rest) = code.strip_prefix("Key") {
                if rest.len() == 1 {
                    return Some(Key::Char(rest.to_ascii_lowercase().chars().next().unwrap()));
                }
            }
            if let Some(rest) = code.strip_prefix("Digit") {
                return Some(Key::Char(rest.chars().next().unwrap()));
            }
            if let Some(n) = code.strip_prefix('F') {
                if let Ok(n) = n.parse::<u8>() {
                    return Some(Key::F(n));
                }
            }
            let c = key.chars().next()?;
            if key.chars().count() != 1 || c.is_control() {
                return None;
            }
            Key::Char(c)
        }
    })
}

fn mods(e: &web_sys::KeyboardEvent) -> Mods {
    Mods {
        shift: e.shift_key(),
        ctrl: e.ctrl_key() || e.meta_key(),
        alt: e.alt_key(),
    }
}

fn with_engine(f: impl FnOnce(&mut Engine)) {
    ENGINE.with(|e| {
        if let Some(rc) = e.borrow().as_ref() {
            if let Ok(mut g) = rc.try_borrow_mut() {
                f(&mut g);
            }
        }
    });
}

fn canvas_pos(canvas: &web_sys::HtmlCanvasElement, cx: f32, cy: f32, scale: f32) -> (f32, f32) {
    let r = canvas.get_bounding_client_rect();
    let dpr = window().device_pixel_ratio() as f32;
    // Logical pixels = CSS pixels * dpr / scale (scale == dpr by default, so CSS px).
    (
        (cx - r.left() as f32) * dpr / scale,
        (cy - r.top() as f32) * dpr / scale,
    )
}

/// Entry point called from the page after the wasm module loads.
#[wasm_bindgen]
pub async fn start(canvas_id: String, webgpu: bool) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Warn);
    let document = window().document().ok_or("no document")?;
    let canvas: web_sys::HtmlCanvasElement = document
        .get_element_by_id(&canvas_id)
        .ok_or("no canvas")?
        .dyn_into()?;
    let dpr = window().device_pixel_ratio() as f32;
    let css_w = window().inner_width()?.as_f64().unwrap_or(960.0) as f32;
    let css_h = window().inner_height()?.as_f64().unwrap_or(600.0) as f32;
    let (pw, ph) = ((css_w * dpr) as u32, (css_h * dpr) as u32);
    canvas.set_width(pw.max(1));
    canvas.set_height(ph.max(1));
    // The page probed `navigator.gpu.requestAdapter()` already: a canvas can hold only
    // one context type, so the backend must be decided before the surface exists.
    let backends = if webgpu {
        wgpu::Backends::BROWSER_WEBGPU
    } else {
        wgpu::Backends::GL
    };
    let mut renderer = Renderer::new_async(
        wgpu::SurfaceTarget::Canvas(canvas.clone()),
        pw.max(1),
        ph.max(1),
        backends,
    )
    .await
    .map_err(|e| JsValue::from_str(&e))?;
    // UI scale: CSS pixels on desktop browsers; a little larger on small touch screens.
    let scale = if css_w < 700.0 { dpr * 1.15 } else { dpr };
    renderer.scale = scale;
    let mut app = App::new(pw as f32 / scale, ph as f32 / scale);
    if css_w < 900.0 {
        app.touch = true;
    }
    let engine = Rc::new(RefCell::new(Engine {
        app,
        renderer,
        canvas: canvas.clone(),
        scale,
        last: now_ms(),
        touch_start: None,
        dragging: false,
        last_tap: 0.0,
        pinch_dist: None,
        pan_last: None,
        long_press_fired: false,
    }));
    ENGINE.with(|e| *e.borrow_mut() = Some(engine.clone()));
    install_events(&canvas)?;
    // Frame loop.
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        with_engine(|e| {
            let now = now_ms();
            let dt = ((now - e.last) / 1000.0).clamp(0.0, 0.25) as f32;
            e.last = now;
            // Follow window resizes.
            let dpr = window().device_pixel_ratio() as f32;
            let css_w = window()
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(960.0) as f32;
            let css_h = window()
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(600.0) as f32;
            let (pw, ph) = ((css_w * dpr) as u32, (css_h * dpr) as u32);
            if pw != e.canvas.width() || ph != e.canvas.height() {
                e.canvas.set_width(pw.max(1));
                e.canvas.set_height(ph.max(1));
                e.renderer.resize(pw.max(1), ph.max(1));
                let s = e.scale;
                e.app.input(Input::Resize {
                    w: pw as f32 / s,
                    h: ph as f32 / s,
                });
            }
            e.app.frame(dt);
            // Touch long press: fire after 450 ms without movement.
            if let Some((x, y, t)) = e.touch_start {
                if !e.dragging && !e.long_press_fired && now - t > 450.0 && e.pinch_dist.is_none() {
                    e.long_press_fired = true;
                    e.app.input(Input::LongPress { x, y });
                }
            }
            if let Err(err) =
                e.renderer
                    .render(&e.app.dl, &mut e.app.text, [0.02, 0.015, 0.01, 1.0])
            {
                web_sys::console::warn_1(&JsValue::from_str(&format!("render: {err}")));
            }
            // Keyboard focus hint for mobile: toggle a hidden input when a text field is focused.
            let wants = e.app.focus.is_some();
            if let Some(doc) = window().document() {
                if let Some(el) = doc.get_element_by_id("kb") {
                    let el: web_sys::HtmlElement = el.dyn_into().unwrap();
                    let focused = doc
                        .active_element()
                        .map(|a| a.id() == "kb")
                        .unwrap_or(false);
                    if wants && !focused && e.app.touch {
                        let _ = el.focus();
                    } else if !wants && focused {
                        let _ = el.blur();
                    }
                }
            }
        });
        request_frame(&f);
    }) as Box<dyn FnMut()>));
    request_frame(&g);
    Ok(())
}

fn request_frame(f: &Rc<RefCell<Option<Closure<dyn FnMut()>>>>) {
    if let Some(cb) = f.borrow().as_ref() {
        let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
    }
}

fn install_events(canvas: &web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
    let target: &web_sys::EventTarget = canvas.as_ref();
    // Mouse.
    {
        let c = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            with_engine(|g| {
                let (x, y) = canvas_pos(&c, e.client_x() as f32, e.client_y() as f32, g.scale);
                g.app.input(Input::MouseMove { x, y });
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        target.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    for (name, down) in [("mousedown", true), ("mouseup", false)] {
        let c = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.prevent_default();
            with_engine(|g| {
                let (x, y) = canvas_pos(&c, e.client_x() as f32, e.client_y() as f32, g.scale);
                let button = match e.button() {
                    0 => 0,
                    2 => 1,
                    _ => 2,
                };
                if down {
                    g.app.input(Input::MouseDown { button, x, y });
                } else {
                    g.app.input(Input::MouseUp { button, x, y });
                }
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        target.add_event_listener_with_callback(name, cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| e.prevent_default())
            as Box<dyn FnMut(web_sys::MouseEvent)>);
        target.add_event_listener_with_callback("contextmenu", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        let c = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::WheelEvent| {
            e.prevent_default();
            with_engine(|g| {
                let (x, y) = canvas_pos(&c, e.client_x() as f32, e.client_y() as f32, g.scale);
                let dy = -(e.delta_y() as f32) / 100.0;
                g.app.input(Input::Wheel { dy, x, y });
            });
        }) as Box<dyn FnMut(web_sys::WheelEvent)>);
        target.add_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    // Keyboard (on the window so the canvas does not need focus).
    let win = window();
    let wtarget: &web_sys::EventTarget = win.as_ref();
    for (name, down) in [("keydown", true), ("keyup", false)] {
        let cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            // Let the browser keep F5/F12 and ctrl shortcuts; swallow the rest.
            let code = e.code();
            let key = e.key();
            if code == "F5"
                || code == "F12"
                || code == "F11"
                || (e.ctrl_key() && (key == "r" || key == "R" || key == "l" || key == "L"))
            {
                return;
            }
            with_engine(|g| {
                if let Some(k) = key_of(&code, &key) {
                    g.app.input(Input::Key {
                        key: k,
                        down,
                        mods: mods(&e),
                    });
                }
                if down && !e.ctrl_key() && !e.meta_key() && key.chars().count() == 1 {
                    let c = key.chars().next().unwrap();
                    if !c.is_control() {
                        g.app.input(Input::Char(c));
                    }
                }
            });
            if code != "Tab" {
                e.prevent_default();
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        wtarget.add_event_listener_with_callback(name, cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    // Touch.
    {
        let c = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default();
            with_engine(|g| {
                g.app.touch = true;
                let t = e.touches();
                if t.length() == 1 {
                    let tt = t.get(0).unwrap();
                    let (x, y) =
                        canvas_pos(&c, tt.client_x() as f32, tt.client_y() as f32, g.scale);
                    g.touch_start = Some((x, y, now_ms()));
                    g.dragging = false;
                    g.long_press_fired = false;
                    g.pinch_dist = None;
                    g.app.input(Input::MouseMove { x, y });
                } else if t.length() >= 2 {
                    let a = t.get(0).unwrap();
                    let b = t.get(1).unwrap();
                    let (ax, ay) =
                        canvas_pos(&c, a.client_x() as f32, a.client_y() as f32, g.scale);
                    let (bx, by) =
                        canvas_pos(&c, b.client_x() as f32, b.client_y() as f32, g.scale);
                    if g.dragging {
                        if let Some((sx, sy, _)) = g.touch_start {
                            g.app.input(Input::MouseUp {
                                button: 0,
                                x: sx,
                                y: sy,
                            });
                        }
                        g.dragging = false;
                    }
                    g.touch_start = None;
                    g.pinch_dist = Some(((ax - bx).powi(2) + (ay - by).powi(2)).sqrt());
                    g.pan_last = Some(((ax + bx) / 2.0, (ay + by) / 2.0));
                }
            });
        }) as Box<dyn FnMut(web_sys::TouchEvent)>);
        target.add_event_listener_with_callback("touchstart", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        let c = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default();
            with_engine(|g| {
                let t = e.touches();
                if t.length() == 1 {
                    let tt = t.get(0).unwrap();
                    let (x, y) =
                        canvas_pos(&c, tt.client_x() as f32, tt.client_y() as f32, g.scale);
                    if let Some((sx, sy, _)) = g.touch_start {
                        if !g.dragging
                            && ((x - sx).abs() > 10.0 || (y - sy).abs() > 10.0)
                            && !g.long_press_fired
                        {
                            g.dragging = true;
                            g.app.input(Input::MouseMove { x: sx, y: sy });
                            g.app.input(Input::MouseDown {
                                button: 0,
                                x: sx,
                                y: sy,
                            });
                        }
                    }
                    if g.dragging {
                        g.app.input(Input::MouseMove { x, y });
                    }
                } else if t.length() >= 2 {
                    let a = t.get(0).unwrap();
                    let b = t.get(1).unwrap();
                    let (ax, ay) =
                        canvas_pos(&c, a.client_x() as f32, a.client_y() as f32, g.scale);
                    let (bx, by) =
                        canvas_pos(&c, b.client_x() as f32, b.client_y() as f32, g.scale);
                    let d = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
                    let (cx, cy) = ((ax + bx) / 2.0, (ay + by) / 2.0);
                    if let Some(pd) = g.pinch_dist {
                        if pd > 0.0 {
                            g.app.input(Input::Pinch {
                                scale: d / pd,
                                x: cx,
                                y: cy,
                            });
                        }
                    }
                    if let Some((px, py)) = g.pan_last {
                        g.app.input(Input::Pan {
                            dx: cx - px,
                            dy: cy - py,
                        });
                    }
                    g.pinch_dist = Some(d);
                    g.pan_last = Some((cx, cy));
                }
            });
        }) as Box<dyn FnMut(web_sys::TouchEvent)>);
        target.add_event_listener_with_callback("touchmove", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    for name in ["touchend", "touchcancel"] {
        let c = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default();
            with_engine(|g| {
                if e.touches().length() == 0 {
                    let changed = e.changed_touches();
                    let pos = changed
                        .get(0)
                        .map(|t| canvas_pos(&c, t.client_x() as f32, t.client_y() as f32, g.scale));
                    if g.dragging {
                        if let Some((x, y)) = pos {
                            g.app.input(Input::MouseMove { x, y });
                            g.app.input(Input::MouseUp { button: 0, x, y });
                        }
                    } else if let Some((sx, sy, _)) = g.touch_start {
                        if !g.long_press_fired {
                            let now = now_ms();
                            if now - g.last_tap < 300.0 {
                                g.app.input(Input::DoubleTap { x: sx, y: sy });
                            } else {
                                g.app.input(Input::MouseMove { x: sx, y: sy });
                                g.app.input(Input::MouseDown {
                                    button: 0,
                                    x: sx,
                                    y: sy,
                                });
                                g.app.input(Input::MouseUp {
                                    button: 0,
                                    x: sx,
                                    y: sy,
                                });
                            }
                            g.last_tap = now;
                        }
                    }
                    g.touch_start = None;
                    g.dragging = false;
                    g.pinch_dist = None;
                    g.pan_last = None;
                }
            });
        }) as Box<dyn FnMut(web_sys::TouchEvent)>);
        target.add_event_listener_with_callback(name, cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    Ok(())
}
