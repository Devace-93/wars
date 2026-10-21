//! Wars desktop: a winit window feeding the shared app with input and presenting its
//! draw list through wgpu.

use clap::Parser;
use std::sync::Arc;
use std::time::Instant;
use wars_app::App;
use wars_render::Renderer;
use wars_ui::{Input, Key, Mods};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Parser, Debug)]
#[command(
    name = "wars",
    about = "Wars: an age-advancing real-time strategy game"
)]
struct Cli {
    /// Skip the menu and start a random-map game against one AI.
    #[arg(long)]
    quick: bool,
    /// Render N frames of a quick game and write a PNG screenshot, then exit.
    #[arg(long)]
    screenshot: Option<String>,
    /// Ticks to simulate before the screenshot.
    #[arg(long, default_value_t = 400)]
    ticks: u32,
    /// Window size.
    #[arg(long, default_value_t = 1280)]
    width: u32,
    #[arg(long, default_value_t = 800)]
    height: u32,
    /// Map seed for --quick / --screenshot.
    #[arg(long, default_value_t = 7)]
    seed: u64,
}

struct Shell {
    cli: Cli,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    app: Option<App>,
    last: Instant,
    mods: Mods,
    frames: u32,
    shot_pending: bool,
    shot_n: u32,
}

fn key_of(code: KeyCode, text: Option<&str>) -> Option<Key> {
    Some(match code {
        KeyCode::Escape => Key::Escape,
        KeyCode::Enter | KeyCode::NumpadEnter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Tab => Key::Tab,
        KeyCode::Space => Key::Space,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::F1 => Key::F(1),
        KeyCode::F2 => Key::F(2),
        KeyCode::F3 => Key::F(3),
        KeyCode::F4 => Key::F(4),
        KeyCode::F5 => Key::F(5),
        KeyCode::F6 => Key::F(6),
        KeyCode::F7 => Key::F(7),
        KeyCode::F8 => Key::F(8),
        KeyCode::F9 => Key::F(9),
        KeyCode::F10 => Key::F(10),
        KeyCode::F11 => Key::F(11),
        KeyCode::F12 => Key::F(12),
        _ => {
            // Letters/digits/punctuation from the physical key so hotkeys work on any layout.
            let s = format!("{code:?}");
            if let Some(rest) = s.strip_prefix("Key") {
                if rest.len() == 1 {
                    return Some(Key::Char(rest.to_lowercase().chars().next().unwrap()));
                }
            }
            if let Some(rest) = s.strip_prefix("Digit") {
                return Some(Key::Char(rest.chars().next().unwrap()));
            }
            match code {
                KeyCode::Minus | KeyCode::NumpadSubtract => Key::Char('-'),
                KeyCode::Equal | KeyCode::NumpadAdd => Key::Char('='),
                KeyCode::Period => Key::Char('.'),
                KeyCode::Comma => Key::Char(','),
                KeyCode::Slash => Key::Char('/'),
                _ => {
                    let c = text.and_then(|t| t.chars().next())?;
                    if c.is_control() {
                        return None;
                    }
                    Key::Char(c)
                }
            }
        }
    })
}

impl ApplicationHandler for Shell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Wars")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.cli.width,
                self.cli.height,
            ));
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let renderer = Renderer::new(window.clone(), size.width, size.height).expect("renderer");
        let mut app = App::new(size.width as f32, size.height as f32);
        if self.cli.quick || self.cli.screenshot.is_some() {
            app.menu.seed = self.cli.seed.to_string();
            app.menu.map_size = 0;
            wars_app::menus::start_single_player(&mut app);
        }
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.app = Some(app);
        self.last = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(window), Some(renderer), Some(app)) = (
            self.window.as_ref(),
            self.renderer.as_mut(),
            self.app.as_mut(),
        ) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                app.input(Input::Resize {
                    w: size.width as f32,
                    h: size.height as f32,
                });
            }
            WindowEvent::CursorMoved { position, .. } => app.input(Input::MouseMove {
                x: position.x as f32,
                y: position.y as f32,
            }),
            WindowEvent::MouseInput { state, button, .. } => {
                let b = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    _ => 2,
                };
                let (x, y) = (app.frame.mouse.0, app.frame.mouse.1);
                let (x, y) = if x == 0.0 && y == 0.0 {
                    (app.screen_w / 2.0, app.screen_h / 2.0)
                } else {
                    (x, y)
                };
                let _ = (x, y);
                let m = mouse_pos(app);
                match state {
                    ElementState::Pressed => app.input(Input::MouseDown {
                        button: b,
                        x: m.0,
                        y: m.1,
                    }),
                    ElementState::Released => app.input(Input::MouseUp {
                        button: b,
                        x: m.0,
                        y: m.1,
                    }),
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => (p.y as f32) / 40.0,
                };
                let m = mouse_pos(app);
                app.input(Input::Wheel { dy, x: m.0, y: m.1 });
            }
            WindowEvent::ModifiersChanged(m) => {
                let s = m.state();
                self.mods = Mods {
                    shift: s.shift_key(),
                    ctrl: s.control_key(),
                    alt: s.alt_key(),
                };
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::F12 && event.state == ElementState::Pressed {
                        self.shot_pending = true;
                    }
                    if let Some(k) = key_of(code, event.text.as_deref()) {
                        app.input(Input::Key {
                            key: k,
                            down: event.state == ElementState::Pressed,
                            mods: self.mods,
                        });
                    }
                }
                if event.state == ElementState::Pressed && !self.mods.ctrl {
                    if let Some(t) = event.text.as_deref() {
                        for c in t.chars() {
                            if !c.is_control() {
                                app.input(Input::Char(c));
                            }
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last).as_secs_f32().min(0.25);
                self.last = now;
                app.frame(dt);
                if let Some(path) = &self.cli.screenshot {
                    self.frames += 1;
                    // Let the sim run the requested ticks (at max speed) before shooting.
                    if let Some(g) = app.game.as_mut() {
                        if g.session.world().tick < self.cli.ticks {
                            if let wars_client::Session::Local(m) = &mut g.session {
                                for _ in 0..40 {
                                    m.step_once(app.data);
                                }
                            }
                        } else if self.frames > 3 {
                            app.frame(0.016);
                            let px = renderer.render_to_pixels(
                                &app.dl,
                                &mut app.text,
                                [0.02, 0.015, 0.01, 1.0],
                            );
                            let (w, h) = renderer.size();
                            std::fs::write(
                                path,
                                wars_render::gpu::encode_png(w as u32, h as u32, &px),
                            )
                            .expect("write png");
                            println!("wrote {path}");
                            event_loop.exit();
                            return;
                        }
                    }
                }
                if self.shot_pending {
                    self.shot_pending = false;
                    self.shot_n += 1;
                    let px =
                        renderer.render_to_pixels(&app.dl, &mut app.text, [0.02, 0.015, 0.01, 1.0]);
                    let (w, h) = renderer.size();
                    let dir = wars_client::saves::data_dir().join("screenshots");
                    let _ = std::fs::create_dir_all(&dir);
                    let path = dir.join(format!("wars-{}-{}.png", std::process::id(), self.shot_n));
                    if std::fs::write(&path, wars_render::gpu::encode_png(w as u32, h as u32, &px))
                        .is_ok()
                    {
                        app.toast(format!("Screenshot saved: {}", path.display()));
                        eprintln!("screenshot {}", path.display());
                    }
                }
                match renderer.render(&app.dl, &mut app.text, [0.02, 0.015, 0.01, 1.0]) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                        let (w, h) = renderer.size();
                        renderer.resize(w as u32, h as u32);
                    }
                    Err(e) => eprintln!("render: {e}"),
                }
                if app.quit {
                    event_loop.exit();
                }
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

fn mouse_pos(app: &App) -> (f32, f32) {
    app.mouse()
}

fn main() {
    let cli = Cli::parse();
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut shell = Shell {
        cli,
        window: None,
        renderer: None,
        app: None,
        last: Instant::now(),
        mods: Mods::default(),
        frames: 0,
        shot_pending: false,
        shot_n: 0,
    };
    event_loop.run_app(&mut shell).expect("run");
}
