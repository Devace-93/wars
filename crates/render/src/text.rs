//! Text: fontdue rasterises glyphs into a shared 1024x1024 alpha atlas (the same texture
//! the solid shapes sample their white texel from).

use crate::draw::{Color, DrawList};
use std::collections::HashMap;

pub const ATLAS: u32 = 1024;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Font {
    Regular,
    Bold,
}

#[derive(Clone, Copy, Debug)]
struct Glyph {
    uv0: [f32; 2],
    uv1: [f32; 2],
    w: f32,
    h: f32,
    xmin: f32,
    ymin: f32,
    advance: f32,
}

pub struct TextSystem {
    fonts: [fontdue::Font; 2],
    pub pixels: Vec<u8>,
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
    glyphs: HashMap<(Font, char, u32), Glyph>,
    pub dirty: bool,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    pub fn new() -> TextSystem {
        let regular = fontdue::Font::from_bytes(
            include_bytes!("../assets/DejaVuSans.ttf") as &[u8],
            fontdue::FontSettings::default(),
        )
        .expect("font");
        let bold = fontdue::Font::from_bytes(
            include_bytes!("../assets/DejaVuSans-Bold.ttf") as &[u8],
            fontdue::FontSettings::default(),
        )
        .expect("font");
        let mut pixels = vec![0u8; (ATLAS * ATLAS) as usize];
        // White texel block at the origin for solid shapes.
        for y in 0..4 {
            for x in 0..4 {
                pixels[(y * ATLAS + x) as usize] = 255;
            }
        }
        TextSystem {
            fonts: [regular, bold],
            pixels,
            cursor_x: 8,
            cursor_y: 0,
            row_h: 0,
            glyphs: HashMap::new(),
            dirty: true,
        }
    }

    fn glyph(&mut self, font: Font, ch: char, size: u32) -> Glyph {
        let key = (font, ch, size);
        if let Some(g) = self.glyphs.get(&key) {
            return *g;
        }
        let f = &self.fonts[if font == Font::Bold { 1 } else { 0 }];
        let (metrics, bitmap) = f.rasterize(ch, size as f32);
        let (w, h) = (metrics.width as u32, metrics.height as u32);
        if self.cursor_x + w + 1 >= ATLAS {
            self.cursor_x = 0;
            self.cursor_y += self.row_h + 1;
            self.row_h = 0;
        }
        if self.cursor_y + h + 1 >= ATLAS {
            // Atlas full: reset (rare; glyph cache is rebuilt lazily).
            self.glyphs.clear();
            self.cursor_x = 8;
            self.cursor_y = 0;
            self.row_h = 0;
            for p in self.pixels.iter_mut().skip(ATLAS as usize * 4) {
                *p = 0;
            }
        }
        for y in 0..h {
            for x in 0..w {
                self.pixels[((self.cursor_y + y) * ATLAS + self.cursor_x + x) as usize] =
                    bitmap[(y * w + x) as usize];
            }
        }
        let g = Glyph {
            uv0: [
                self.cursor_x as f32 / ATLAS as f32,
                self.cursor_y as f32 / ATLAS as f32,
            ],
            uv1: [
                (self.cursor_x + w) as f32 / ATLAS as f32,
                (self.cursor_y + h) as f32 / ATLAS as f32,
            ],
            w: w as f32,
            h: h as f32,
            xmin: metrics.xmin as f32,
            ymin: metrics.ymin as f32,
            advance: metrics.advance_width,
        };
        self.cursor_x += w + 1;
        self.row_h = self.row_h.max(h);
        self.glyphs.insert(key, g);
        self.dirty = true;
        g
    }

    /// Width in pixels of a string.
    pub fn measure(&mut self, font: Font, size: u32, s: &str) -> f32 {
        s.chars().map(|c| self.glyph(font, c, size).advance).sum()
    }

    /// Draw text with its baseline-top at (x, y). Returns the width.
    pub fn draw(
        &mut self,
        dl: &mut DrawList,
        font: Font,
        size: u32,
        x: f32,
        y: f32,
        s: &str,
        col: Color,
    ) -> f32 {
        let mut pen = x;
        let ascent = size as f32 * 0.93;
        for c in s.chars() {
            let g = self.glyph(font, c, size);
            if g.w > 0.0 && g.h > 0.0 {
                let gx = (pen + g.xmin).round();
                let gy = (y + ascent - g.ymin - g.h).round();
                dl.textured(gx, gy, g.w, g.h, g.uv0, g.uv1, col);
            }
            pen += g.advance;
        }
        pen - x
    }

    pub fn draw_centered(
        &mut self,
        dl: &mut DrawList,
        font: Font,
        size: u32,
        cx: f32,
        y: f32,
        s: &str,
        col: Color,
    ) {
        let w = self.measure(font, size, s);
        self.draw(dl, font, size, (cx - w / 2.0).round(), y, s, col);
    }

    pub fn draw_right(
        &mut self,
        dl: &mut DrawList,
        font: Font,
        size: u32,
        right: f32,
        y: f32,
        s: &str,
        col: Color,
    ) {
        let w = self.measure(font, size, s);
        self.draw(dl, font, size, (right - w).round(), y, s, col);
    }

    /// Word-wrapped paragraph; returns the height used.
    pub fn draw_wrapped(
        &mut self,
        dl: &mut DrawList,
        font: Font,
        size: u32,
        x: f32,
        y: f32,
        max_w: f32,
        s: &str,
        col: Color,
    ) -> f32 {
        let line_h = size as f32 * 1.35;
        let mut cy = y;
        for para in s.split('\n') {
            let mut line = String::new();
            for word in para.split(' ') {
                let candidate = if line.is_empty() {
                    word.to_string()
                } else {
                    format!("{line} {word}")
                };
                if self.measure(font, size, &candidate) > max_w && !line.is_empty() {
                    self.draw(dl, font, size, x, cy, &line, col);
                    cy += line_h;
                    line = word.to_string();
                } else {
                    line = candidate;
                }
            }
            self.draw(dl, font, size, x, cy, &line, col);
            cy += line_h;
        }
        cy - y
    }
}
