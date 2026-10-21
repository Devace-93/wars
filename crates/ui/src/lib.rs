//! wars-ui: a small immediate-mode toolkit drawn with wars-render. Panels, buttons,
//! selectors, text fields, lists and progress bars in the amber/wood theme.

use wars_render::draw::{Clip, Color, DrawList};
use wars_render::text::{Font, TextSystem};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Key {
    Escape,
    Enter,
    Backspace,
    Delete,
    Tab,
    Space,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Char(char),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Platform events fed to the app every frame.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Input {
    MouseMove {
        x: f32,
        y: f32,
    },
    MouseDown {
        button: u8,
        x: f32,
        y: f32,
    },
    MouseUp {
        button: u8,
        x: f32,
        y: f32,
    },
    Wheel {
        dy: f32,
        x: f32,
        y: f32,
    },
    Key {
        key: Key,
        down: bool,
        mods: Mods,
    },
    Char(char),
    Resize {
        w: f32,
        h: f32,
    },
    /// Touch pinch: scale factor around a point (mobile).
    Pinch {
        scale: f32,
        x: f32,
        y: f32,
    },
    /// Two-finger pan in pixels (mobile).
    Pan {
        dx: f32,
        dy: f32,
    },
    LongPress {
        x: f32,
        y: f32,
    },
    DoubleTap {
        x: f32,
        y: f32,
    },
    Back,
}

/// Per-frame input snapshot the widgets read.
#[derive(Clone, Debug, Default)]
pub struct Frame {
    pub mouse: (f32, f32),
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
    pub right_pressed: bool,
    pub right_released: bool,
    pub middle_down: bool,
    pub wheel: f32,
    pub keys: Vec<(Key, Mods)>,
    pub chars: Vec<char>,
    pub mods: Mods,
    pub screen: (f32, f32),
    pub dt: f32,
    pub time: f64,
}

impl Frame {
    pub fn key(&self, k: Key) -> bool {
        self.keys.iter().any(|(kk, _)| *kk == k)
    }
    pub fn key_mods(&self, k: Key) -> Option<Mods> {
        self.keys.iter().find(|(kk, _)| *kk == k).map(|(_, m)| *m)
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn contains(&self, p: (f32, f32)) -> bool {
        p.0 >= self.x && p.0 <= self.x + self.w && p.1 >= self.y && p.1 <= self.y + self.h
    }
    pub fn inset(&self, d: f32) -> Rect {
        Rect::new(self.x + d, self.y + d, self.w - 2.0 * d, self.h - 2.0 * d)
    }
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    pub fn clip(&self) -> Clip {
        Clip {
            x: self.x as i32,
            y: self.y as i32,
            w: self.w as i32,
            h: self.h as i32,
        }
    }
}

pub struct Theme {
    pub bg: Color,
    pub panel: Color,
    pub panel_border: Color,
    pub accent: Color,
    pub accent_dark: Color,
    pub text: Color,
    pub text_dim: Color,
    pub button: Color,
    pub button_hot: Color,
    pub button_down: Color,
    pub danger: Color,
    pub ok: Color,
    pub field: Color,
}

pub const THEME: Theme = Theme {
    bg: Color::rgb(24, 18, 12),
    panel: Color::rgb(58, 42, 26),
    panel_border: Color::rgb(120, 90, 50),
    accent: Color::rgb(245, 158, 11),
    accent_dark: Color::rgb(180, 83, 9),
    text: Color::rgb(245, 235, 215),
    text_dim: Color::rgb(170, 150, 120),
    button: Color::rgb(90, 64, 36),
    button_hot: Color::rgb(120, 86, 46),
    button_down: Color::rgb(70, 48, 26),
    danger: Color::rgb(200, 60, 50),
    ok: Color::rgb(70, 170, 80),
    field: Color::rgb(35, 26, 16),
};

pub struct Ui<'a> {
    pub dl: &'a mut DrawList,
    pub text: &'a mut TextSystem,
    pub frame: &'a Frame,
    /// Which text field has keyboard focus (widget id).
    pub focus: &'a mut Option<u64>,
    /// Set by widgets when the mouse is over something interactive.
    pub hovered_any: bool,
    /// True while a modal/menu consumes clicks on the map.
    pub consumed_click: bool,
}

pub fn id(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

impl<'a> Ui<'a> {
    pub fn new(
        dl: &'a mut DrawList,
        text: &'a mut TextSystem,
        frame: &'a Frame,
        focus: &'a mut Option<u64>,
    ) -> Ui<'a> {
        Ui {
            dl,
            text,
            frame,
            focus,
            hovered_any: false,
            consumed_click: false,
        }
    }
    pub fn hover(&mut self, r: Rect) -> bool {
        let h = r.contains(self.frame.mouse);
        if h {
            self.hovered_any = true;
        }
        h
    }
    pub fn panel(&mut self, r: Rect) {
        self.dl.rect(r.x, r.y, r.w, r.h, THEME.panel);
        self.dl
            .rect_outline(r.x, r.y, r.w, r.h, 2.0, THEME.panel_border);
        self.dl.rect_outline(
            r.x + 2.0,
            r.y + 2.0,
            r.w - 4.0,
            r.h - 4.0,
            1.0,
            THEME.panel.mul(1.25),
        );
    }
    pub fn panel_plain(&mut self, r: Rect, c: Color) {
        self.dl.rect(r.x, r.y, r.w, r.h, c);
    }
    pub fn label(&mut self, x: f32, y: f32, s: &str) {
        self.text
            .draw(self.dl, Font::Regular, 15, x, y, s, THEME.text);
    }
    pub fn label_dim(&mut self, x: f32, y: f32, s: &str) {
        self.text
            .draw(self.dl, Font::Regular, 14, x, y, s, THEME.text_dim);
    }
    pub fn label_color(&mut self, x: f32, y: f32, size: u32, s: &str, c: Color) {
        self.text.draw(self.dl, Font::Regular, size, x, y, s, c);
    }
    pub fn heading(&mut self, x: f32, y: f32, s: &str) {
        self.text
            .draw(self.dl, Font::Bold, 22, x, y, s, THEME.accent);
    }
    pub fn title(&mut self, cx: f32, y: f32, s: &str) {
        self.text.draw_centered(
            self.dl,
            Font::Bold,
            40,
            cx + 2.0,
            y + 2.0,
            s,
            Color::rgba(0, 0, 0, 180),
        );
        self.text
            .draw_centered(self.dl, Font::Bold, 40, cx, y, s, THEME.accent);
    }
    pub fn centered(&mut self, cx: f32, y: f32, size: u32, s: &str, c: Color) {
        self.text
            .draw_centered(self.dl, Font::Regular, size, cx, y, s, c);
    }
    pub fn measure(&mut self, size: u32, s: &str) -> f32 {
        self.text.measure(Font::Regular, size, s)
    }

    /// A push button. Returns true when clicked this frame.
    pub fn button(&mut self, r: Rect, label: &str) -> bool {
        self.button_styled(r, label, true, THEME.button)
    }
    pub fn button_styled(&mut self, r: Rect, label: &str, enabled: bool, base: Color) -> bool {
        let hot = enabled && self.hover(r);
        let down = hot && self.frame.down;
        let c = if !enabled {
            base.mul(0.6)
        } else if down {
            THEME.button_down
        } else if hot {
            THEME.button_hot
        } else {
            base
        };
        self.dl.rect(r.x, r.y, r.w, r.h, c);
        self.dl.rect_outline(
            r.x,
            r.y,
            r.w,
            r.h,
            1.5,
            if hot {
                THEME.accent
            } else {
                THEME.panel_border
            },
        );
        let tc = if enabled { THEME.text } else { THEME.text_dim };
        let size = if r.h < 26.0 { 13 } else { 16 };
        self.text.draw_centered(
            self.dl,
            Font::Regular,
            size,
            r.x + r.w / 2.0,
            r.y + (r.h - size as f32 * 1.1) / 2.0,
            label,
            tc,
        );
        let clicked = hot && self.frame.released;
        if clicked {
            self.consumed_click = true;
        }
        clicked
    }
    /// Small icon-like square button with a short label and tooltip text drawn by the caller.
    pub fn tile_button(&mut self, r: Rect, label: &str, enabled: bool, highlight: bool) -> bool {
        let base = if highlight {
            THEME.accent_dark
        } else {
            THEME.button
        };
        self.button_styled(r, label, enabled, base)
    }
    pub fn checkbox(&mut self, r: Rect, label: &str, v: &mut bool) -> bool {
        let hot = self.hover(r);
        let box_r = Rect::new(r.x, r.y + (r.h - 18.0) / 2.0, 18.0, 18.0);
        self.dl
            .rect(box_r.x, box_r.y, box_r.w, box_r.h, THEME.field);
        self.dl.rect_outline(
            box_r.x,
            box_r.y,
            box_r.w,
            box_r.h,
            1.5,
            if hot {
                THEME.accent
            } else {
                THEME.panel_border
            },
        );
        if *v {
            self.dl
                .rect(box_r.x + 4.0, box_r.y + 4.0, 10.0, 10.0, THEME.accent);
        }
        self.text.draw(
            self.dl,
            Font::Regular,
            15,
            r.x + 26.0,
            r.y + (r.h - 17.0) / 2.0,
            label,
            THEME.text,
        );
        if hot && self.frame.released {
            *v = !*v;
            self.consumed_click = true;
            return true;
        }
        false
    }
    /// `< value >` selector; returns true when changed.
    pub fn select(
        &mut self,
        r: Rect,
        label: &str,
        value: &str,
        idx: &mut usize,
        count: usize,
    ) -> bool {
        if count == 0 {
            return false;
        }
        self.text.draw(
            self.dl,
            Font::Regular,
            15,
            r.x,
            r.y + (r.h - 17.0) / 2.0,
            label,
            THEME.text_dim,
        );
        let lw = 150.0f32.min(r.w * 0.4);
        let bw = 26.0;
        let left = Rect::new(r.x + lw, r.y, bw, r.h);
        let right = Rect::new(r.right() - bw, r.y, bw, r.h);
        let mid = Rect::new(left.right(), r.y, right.x - left.right(), r.h);
        self.dl.rect(mid.x, mid.y, mid.w, mid.h, THEME.field);
        self.text.draw_centered(
            self.dl,
            Font::Regular,
            15,
            mid.x + mid.w / 2.0,
            mid.y + (mid.h - 17.0) / 2.0,
            value,
            THEME.text,
        );
        let mut changed = false;
        if self.button(left, "<") {
            *idx = (*idx + count - 1) % count;
            changed = true;
        }
        if self.button(right, ">") {
            *idx = (*idx + 1) % count;
            changed = true;
        }
        changed
    }
    pub fn slider(&mut self, r: Rect, label: &str, v: &mut f32, min: f32, max: f32) -> bool {
        self.text.draw(
            self.dl,
            Font::Regular,
            15,
            r.x,
            r.y + (r.h - 17.0) / 2.0,
            label,
            THEME.text_dim,
        );
        let lw = 150.0f32.min(r.w * 0.4);
        let track = Rect::new(r.x + lw, r.y + r.h / 2.0 - 3.0, r.w - lw, 6.0);
        self.dl
            .rect(track.x, track.y, track.w, track.h, THEME.field);
        let t = ((*v - min) / (max - min)).clamp(0.0, 1.0);
        self.dl
            .rect(track.x, track.y, track.w * t, track.h, THEME.accent);
        let hit = Rect::new(track.x, r.y, track.w, r.h);
        if self.hover(hit) && self.frame.down {
            let nt = ((self.frame.mouse.0 - track.x) / track.w).clamp(0.0, 1.0);
            *v = min + (max - min) * nt;
            self.consumed_click = true;
            return true;
        }
        false
    }
    /// Single-line text field. Click to focus; typing edits.
    pub fn text_field(
        &mut self,
        wid: u64,
        r: Rect,
        label: &str,
        value: &mut String,
        placeholder: &str,
        password: bool,
    ) -> bool {
        let mut changed = false;
        let lw = if label.is_empty() {
            0.0
        } else {
            150.0f32.min(r.w * 0.4)
        };
        if !label.is_empty() {
            self.text.draw(
                self.dl,
                Font::Regular,
                15,
                r.x,
                r.y + (r.h - 17.0) / 2.0,
                label,
                THEME.text_dim,
            );
        }
        let f = Rect::new(r.x + lw, r.y, r.w - lw, r.h);
        let focused = *self.focus == Some(wid);
        let hot = self.hover(f);
        if hot && self.frame.pressed {
            *self.focus = Some(wid);
            self.consumed_click = true;
        } else if self.frame.pressed && focused && !hot {
            *self.focus = None;
        }
        self.dl.rect(f.x, f.y, f.w, f.h, THEME.field);
        self.dl.rect_outline(
            f.x,
            f.y,
            f.w,
            f.h,
            1.5,
            if focused {
                THEME.accent
            } else {
                THEME.panel_border
            },
        );
        if focused {
            for c in &self.frame.chars {
                if !c.is_control() && value.chars().count() < 120 {
                    value.push(*c);
                    changed = true;
                }
            }
            for (k, m) in &self.frame.keys {
                match k {
                    Key::Backspace => {
                        if m.ctrl {
                            value.clear();
                        } else {
                            value.pop();
                        }
                        changed = true;
                    }
                    Key::Escape | Key::Tab | Key::Enter => *self.focus = None,
                    _ => {}
                }
            }
        }
        let shown: String = if password {
            "•".repeat(value.chars().count())
        } else {
            value.clone()
        };
        if shown.is_empty() && !focused {
            self.text.draw(
                self.dl,
                Font::Regular,
                15,
                f.x + 8.0,
                f.y + (f.h - 17.0) / 2.0,
                placeholder,
                THEME.text_dim.mul(0.8),
            );
        } else {
            self.dl.set_clip(Some(f.inset(2.0).clip()));
            let w = self.text.measure(Font::Regular, 15, &shown);
            let x = if w > f.w - 16.0 {
                f.x + f.w - 8.0 - w
            } else {
                f.x + 8.0
            };
            self.text.draw(
                self.dl,
                Font::Regular,
                15,
                x,
                f.y + (f.h - 17.0) / 2.0,
                &shown,
                THEME.text,
            );
            if focused && (self.frame.time * 2.0) as i64 % 2 == 0 {
                self.dl
                    .rect(x + w + 1.0, f.y + 6.0, 2.0, f.h - 12.0, THEME.accent);
            }
            self.dl.set_clip(None);
        }
        changed
    }
    /// Scrollable list. Returns the index clicked this frame (if any) and keeps `selected`.
    pub fn list(
        &mut self,
        r: Rect,
        items: &[String],
        selected: &mut Option<usize>,
        scroll: &mut f32,
        row_h: f32,
    ) -> Option<usize> {
        self.dl.rect(r.x, r.y, r.w, r.h, THEME.field);
        self.dl
            .rect_outline(r.x, r.y, r.w, r.h, 1.5, THEME.panel_border);
        let hot = self.hover(r);
        if hot && self.frame.wheel != 0.0 {
            *scroll -= self.frame.wheel * row_h * 3.0;
        }
        let max_scroll = (items.len() as f32 * row_h - r.h).max(0.0);
        *scroll = scroll.clamp(0.0, max_scroll);
        self.dl.set_clip(Some(r.inset(2.0).clip()));
        let mut clicked = None;
        for (i, item) in items.iter().enumerate() {
            let y = r.y + 2.0 + i as f32 * row_h - *scroll;
            if y + row_h < r.y || y > r.bottom() {
                continue;
            }
            let row = Rect::new(r.x + 2.0, y, r.w - 4.0, row_h);
            let rh = hot && row.contains(self.frame.mouse);
            if *selected == Some(i) {
                self.dl
                    .rect(row.x, row.y, row.w, row.h, THEME.accent_dark.alpha(0.6));
            } else if rh {
                self.dl
                    .rect(row.x, row.y, row.w, row.h, THEME.button_hot.alpha(0.5));
            }
            self.text.draw(
                self.dl,
                Font::Regular,
                14,
                row.x + 8.0,
                row.y + (row_h - 16.0) / 2.0,
                item,
                THEME.text,
            );
            if rh && self.frame.released {
                *selected = Some(i);
                clicked = Some(i);
                self.consumed_click = true;
            }
        }
        self.dl.set_clip(None);
        if max_scroll > 0.0 {
            let th = (r.h * r.h / (items.len() as f32 * row_h)).max(16.0);
            let ty = r.y + (*scroll / max_scroll) * (r.h - th);
            self.dl
                .rect(r.right() - 6.0, ty, 4.0, th, THEME.panel_border);
        }
        clicked
    }
    pub fn progress(&mut self, r: Rect, frac: f32, c: Color) {
        self.dl.rect(r.x, r.y, r.w, r.h, THEME.field);
        self.dl.rect(r.x, r.y, r.w * frac.clamp(0.0, 1.0), r.h, c);
        self.dl
            .rect_outline(r.x, r.y, r.w, r.h, 1.0, THEME.panel_border);
    }
    /// Tooltip box near the mouse.
    pub fn tooltip(&mut self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        let w = lines
            .iter()
            .map(|l| self.text.measure(Font::Regular, 13, l))
            .fold(0.0f32, f32::max)
            + 16.0;
        let h = lines.len() as f32 * 17.0 + 10.0;
        let (mx, my) = self.frame.mouse;
        let x = (mx + 14.0).min(self.frame.screen.0 - w - 4.0);
        let y = (my - h - 8.0).max(4.0);
        self.dl.rect(x, y, w, h, Color::rgba(20, 14, 8, 235));
        self.dl.rect_outline(x, y, w, h, 1.0, THEME.accent_dark);
        for (i, l) in lines.iter().enumerate() {
            let c = if i == 0 { THEME.accent } else { THEME.text };
            self.text.draw(
                self.dl,
                Font::Regular,
                13,
                x + 8.0,
                y + 5.0 + i as f32 * 17.0,
                l,
                c,
            );
        }
    }
}
