//! Isometric camera: 2:1 diamonds, 64x32 pixels per tile at zoom 1.

pub const TILE_W: f32 = 64.0;
pub const TILE_H: f32 = 32.0;

#[derive(Clone, Copy, Debug)]
pub struct Iso {
    /// World position (tiles, fractional) shown at the screen centre.
    pub cx: f32,
    pub cy: f32,
    pub zoom: f32,
    pub screen_w: f32,
    pub screen_h: f32,
    /// Vertical offset of the map area (e.g. below a top bar).
    pub top: f32,
    pub bottom: f32,
}

impl Iso {
    pub fn new(w: f32, h: f32) -> Iso {
        Iso {
            cx: 0.0,
            cy: 0.0,
            zoom: 1.0,
            screen_w: w,
            screen_h: h,
            top: 0.0,
            bottom: 0.0,
        }
    }
    fn origin(&self) -> (f32, f32) {
        (
            self.screen_w / 2.0,
            self.top + (self.screen_h - self.top - self.bottom) / 2.0,
        )
    }
    /// World (tile units, fractional) to screen pixels.
    pub fn to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        let (ox, oy) = self.origin();
        let dx = wx - self.cx;
        let dy = wy - self.cy;
        (
            ox + (dx - dy) * TILE_W / 2.0 * self.zoom,
            oy + (dx + dy) * TILE_H / 2.0 * self.zoom,
        )
    }
    /// Screen pixels to world coordinates.
    pub fn to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let (ox, oy) = self.origin();
        let a = (sx - ox) / (TILE_W / 2.0 * self.zoom);
        let b = (sy - oy) / (TILE_H / 2.0 * self.zoom);
        ((a + b) / 2.0 + self.cx, (b - a) / 2.0 + self.cy)
    }
    /// Diamond corners of tile (x, y) on screen: top, right, bottom, left.
    pub fn tile_poly(&self, x: i32, y: i32) -> [[f32; 2]; 4] {
        let (tx, ty) = self.to_screen(x as f32, y as f32);
        let (rx, ry) = self.to_screen(x as f32 + 1.0, y as f32);
        let (bx, by) = self.to_screen(x as f32 + 1.0, y as f32 + 1.0);
        let (lx, ly) = self.to_screen(x as f32, y as f32 + 1.0);
        [[tx, ty], [rx, ry], [bx, by], [lx, ly]]
    }
    /// Range of tiles that may be on screen (inclusive bounds, clamped by the caller).
    pub fn visible_tiles(&self) -> (i32, i32, i32, i32) {
        let corners = [
            (0.0, self.top),
            (self.screen_w, self.top),
            (0.0, self.screen_h - self.bottom),
            (self.screen_w, self.screen_h - self.bottom),
        ];
        let mut minx = f32::MAX;
        let mut miny = f32::MAX;
        let mut maxx = f32::MIN;
        let mut maxy = f32::MIN;
        for (sx, sy) in corners {
            let (wx, wy) = self.to_world(sx, sy);
            minx = minx.min(wx);
            miny = miny.min(wy);
            maxx = maxx.max(wx);
            maxy = maxy.max(wy);
        }
        (
            minx.floor() as i32 - 2,
            miny.floor() as i32 - 2,
            maxx.ceil() as i32 + 2,
            maxy.ceil() as i32 + 6,
        )
    }
    pub fn pan(&mut self, dx_px: f32, dy_px: f32) {
        let a = dx_px / (TILE_W / 2.0 * self.zoom);
        let b = dy_px / (TILE_H / 2.0 * self.zoom);
        self.cx += (a + b) / 2.0;
        self.cy += (b - a) / 2.0;
    }
    pub fn clamp_to(&mut self, map_w: i32, map_h: i32) {
        self.cx = self.cx.clamp(0.0, map_w as f32);
        self.cy = self.cy.clamp(0.0, map_h as f32);
    }
    pub fn zoom_at(&mut self, factor: f32, sx: f32, sy: f32, min: f32, max: f32) {
        let (wx, wy) = self.to_world(sx, sy);
        self.zoom = (self.zoom * factor).clamp(min, max);
        let (nx, ny) = self.to_world(sx, sy);
        self.cx += wx - nx;
        self.cy += wy - ny;
    }
}
