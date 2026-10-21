//! CPU-side geometry: every frame is a list of coloured, optionally textured triangles.

use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Color(pub [f32; 4]);

impl Color {
    pub const WHITE: Color = Color([1.0, 1.0, 1.0, 1.0]);
    pub const BLACK: Color = Color([0.0, 0.0, 0.0, 1.0]);
    pub const TRANSPARENT: Color = Color([0.0, 0.0, 0.0, 0.0]);
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color([
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ])
    }
    pub fn hex(h: u32) -> Color {
        Color::rgb(
            ((h >> 16) & 0xff) as u8,
            ((h >> 8) & 0xff) as u8,
            (h & 0xff) as u8,
        )
    }
    pub fn alpha(self, a: f32) -> Color {
        Color([self.0[0], self.0[1], self.0[2], a])
    }
    pub fn mul(self, k: f32) -> Color {
        Color([
            (self.0[0] * k).min(1.0),
            (self.0[1] * k).min(1.0),
            (self.0[2] * k).min(1.0),
            self.0[3],
        ])
    }
    pub fn mix(self, o: Color, t: f32) -> Color {
        let f = |a: f32, b: f32| a + (b - a) * t;
        Color([
            f(self.0[0], o.0[0]),
            f(self.0[1], o.0[1]),
            f(self.0[2], o.0[2]),
            f(self.0[3], o.0[3]),
        ])
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// A scissor rectangle in pixels applied to the following draws.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Clip {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Debug)]
pub struct Batch {
    pub clip: Option<Clip>,
    pub index_start: u32,
    pub index_count: u32,
}

#[derive(Clone, Debug, Default)]
pub struct DrawList {
    pub verts: Vec<Vertex>,
    pub idx: Vec<u32>,
    pub batches: Vec<Batch>,
    cur_clip: Option<Clip>,
}

/// UV of the solid white texel in the atlas.
pub const WHITE_UV: [f32; 2] = [0.5 / 1024.0, 0.5 / 1024.0];

impl DrawList {
    pub fn new() -> DrawList {
        DrawList::default()
    }
    pub fn clear(&mut self) {
        self.verts.clear();
        self.idx.clear();
        self.batches.clear();
        self.cur_clip = None;
    }
    pub fn set_clip(&mut self, clip: Option<Clip>) {
        self.cur_clip = clip;
    }
    fn batch(&mut self) -> &mut Batch {
        let need_new = match self.batches.last() {
            Some(b) => b.clip != self.cur_clip,
            None => true,
        };
        if need_new {
            let start = self.idx.len() as u32;
            self.batches.push(Batch {
                clip: self.cur_clip,
                index_start: start,
                index_count: 0,
            });
        }
        self.batches.last_mut().unwrap()
    }
    pub fn push_tri(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        let base = self.verts.len() as u32;
        self.verts.extend_from_slice(&[a, b, c]);
        self.idx.extend_from_slice(&[base, base + 1, base + 2]);
        self.batch().index_count += 3;
    }
    pub fn push_quad(&mut self, v: [Vertex; 4]) {
        let base = self.verts.len() as u32;
        self.verts.extend_from_slice(&v);
        self.idx
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        self.batch().index_count += 6;
    }
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        let v = |px: f32, py: f32| Vertex {
            pos: [px, py],
            uv: WHITE_UV,
            color: c.0,
        };
        self.push_quad([v(x, y), v(x + w, y), v(x + w, y + h), v(x, y + h)]);
    }
    pub fn rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, t: f32, c: Color) {
        self.rect(x, y, w, t, c);
        self.rect(x, y + h - t, w, t, c);
        self.rect(x, y, t, h, c);
        self.rect(x + w - t, y, t, h, c);
    }
    pub fn tri(&mut self, a: [f32; 2], b: [f32; 2], c: [f32; 2], col: Color) {
        let v = |p: [f32; 2]| Vertex {
            pos: p,
            uv: WHITE_UV,
            color: col.0,
        };
        self.push_tri(v(a), v(b), v(c));
    }
    /// Convex polygon as a fan.
    pub fn poly(&mut self, pts: &[[f32; 2]], col: Color) {
        if pts.len() < 3 {
            return;
        }
        for i in 1..pts.len() - 1 {
            self.tri(pts[0], pts[i], pts[i + 1], col);
        }
    }
    pub fn poly_outline(&mut self, pts: &[[f32; 2]], t: f32, col: Color) {
        for i in 0..pts.len() {
            let a = pts[i];
            let b = pts[(i + 1) % pts.len()];
            self.line(a, b, t, col);
        }
    }
    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, col: Color) {
        let n = if r < 4.0 {
            8
        } else if r < 12.0 {
            14
        } else {
            24
        };
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            pts.push([cx + r * a.cos(), cy + r * a.sin()]);
        }
        self.poly(&pts, col);
    }
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, col: Color) {
        let n = 16;
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            pts.push([cx + rx * a.cos(), cy + ry * a.sin()]);
        }
        self.poly(&pts, col);
    }
    pub fn line(&mut self, a: [f32; 2], b: [f32; 2], t: f32, col: Color) {
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let len = (dx * dx + dy * dy).sqrt().max(0.0001);
        let nx = -dy / len * t * 0.5;
        let ny = dx / len * t * 0.5;
        let v = |p: [f32; 2]| Vertex {
            pos: p,
            uv: WHITE_UV,
            color: col.0,
        };
        self.push_quad([
            v([a[0] + nx, a[1] + ny]),
            v([b[0] + nx, b[1] + ny]),
            v([b[0] - nx, b[1] - ny]),
            v([a[0] - nx, a[1] - ny]),
        ]);
    }
    /// Textured quad (used by text).
    pub fn textured(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv0: [f32; 2],
        uv1: [f32; 2],
        c: Color,
    ) {
        let v = |px: f32, py: f32, u: f32, vv: f32| Vertex {
            pos: [px, py],
            uv: [u, vv],
            color: c.0,
        };
        self.push_quad([
            v(x, y, uv0[0], uv0[1]),
            v(x + w, y, uv1[0], uv0[1]),
            v(x + w, y + h, uv1[0], uv1[1]),
            v(x, y + h, uv0[0], uv1[1]),
        ]);
    }
    pub fn is_empty(&self) -> bool {
        self.idx.is_empty()
    }
}
