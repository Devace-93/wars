//! Deterministic fixed-point arithmetic (16.16) used for every world coordinate,
//! speed and rate in the simulation. No `f32`/`f64` ever enters game logic.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Fixed(pub i32);

pub const FRAC_BITS: u32 = 16;
pub const ONE: i32 = 1 << FRAC_BITS;

impl Fixed {
    pub const ZERO: Fixed = Fixed(0);
    pub const ONE: Fixed = Fixed(ONE);
    pub const HALF: Fixed = Fixed(ONE / 2);
    pub const MAX: Fixed = Fixed(i32::MAX);

    pub const fn from_int(v: i32) -> Fixed {
        Fixed(v << FRAC_BITS)
    }
    /// `num / den` as a fixed value (den must be non-zero).
    pub const fn from_ratio(num: i32, den: i32) -> Fixed {
        Fixed(((num as i64 * ONE as i64) / den as i64) as i32)
    }
    /// Parse a decimal literal such as `0.8` or `1.35` at data-definition time.
    pub fn from_decimal(s: &str) -> Fixed {
        let (neg, s) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s),
        };
        let (int, frac) = match s.split_once('.') {
            Some((a, b)) => (a, b),
            None => (s, ""),
        };
        let int: i64 = int.parse().unwrap_or(0);
        let mut frac_val: i64 = 0;
        let mut scale: i64 = 1;
        for c in frac.chars().take(6) {
            frac_val = frac_val * 10 + (c as u8 - b'0') as i64;
            scale *= 10;
        }
        let raw = int * ONE as i64 + (frac_val * ONE as i64) / scale;
        Fixed(if neg { -(raw as i32) } else { raw as i32 })
    }
    pub const fn to_int(self) -> i32 {
        self.0 >> FRAC_BITS
    }
    pub const fn floor(self) -> Fixed {
        Fixed(self.0 & !(ONE - 1))
    }
    pub fn round(self) -> i32 {
        (self.0 + ONE / 2) >> FRAC_BITS
    }
    pub const fn raw(self) -> i32 {
        self.0
    }
    pub const fn abs(self) -> Fixed {
        Fixed(if self.0 < 0 { -self.0 } else { self.0 })
    }
    pub fn min(self, o: Fixed) -> Fixed {
        if self.0 < o.0 {
            self
        } else {
            o
        }
    }
    pub fn max(self, o: Fixed) -> Fixed {
        if self.0 > o.0 {
            self
        } else {
            o
        }
    }
    pub fn clamp(self, lo: Fixed, hi: Fixed) -> Fixed {
        self.max(lo).min(hi)
    }
    pub fn mul_int(self, v: i32) -> Fixed {
        Fixed(((self.0 as i64 * v as i64).clamp(i32::MIN as i64, i32::MAX as i64)) as i32)
    }
    pub fn div_int(self, v: i32) -> Fixed {
        Fixed(self.0 / v)
    }
    /// Integer square root of a fixed value (result is fixed).
    pub fn sqrt(self) -> Fixed {
        if self.0 <= 0 {
            return Fixed::ZERO;
        }
        // sqrt(x * 2^16) in 16.16 = sqrt(raw << 16)
        let v = (self.0 as u64) << FRAC_BITS;
        Fixed(isqrt_u64(v) as i32)
    }
    /// Convert to f32 for rendering only. Never use the result in game logic.
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / ONE as f32
    }
    /// Percent helper: `self * pct / 100`.
    pub fn pct(self, pct: i32) -> Fixed {
        Fixed(((self.0 as i64 * pct as i64) / 100) as i32)
    }
}

pub fn isqrt_u64(v: u64) -> u64 {
    if v < 2 {
        return v;
    }
    let mut x = (v as f64).sqrt() as u64; // seed only; corrected below so the result is exact
    while x.checked_mul(x).map(|s| s > v).unwrap_or(true) {
        x -= 1;
    }
    while (x + 1).checked_mul(x + 1).map(|s| s <= v).unwrap_or(false) {
        x += 1;
    }
    x
}

impl Add for Fixed {
    type Output = Fixed;
    fn add(self, o: Fixed) -> Fixed {
        Fixed(self.0.wrapping_add(o.0))
    }
}
impl Sub for Fixed {
    type Output = Fixed;
    fn sub(self, o: Fixed) -> Fixed {
        Fixed(self.0.wrapping_sub(o.0))
    }
}
impl Mul for Fixed {
    type Output = Fixed;
    fn mul(self, o: Fixed) -> Fixed {
        Fixed(((self.0 as i64 * o.0 as i64) >> FRAC_BITS) as i32)
    }
}
impl Div for Fixed {
    type Output = Fixed;
    fn div(self, o: Fixed) -> Fixed {
        if o.0 == 0 {
            return Fixed::MAX;
        }
        Fixed((((self.0 as i64) << FRAC_BITS) / o.0 as i64) as i32)
    }
}
impl Neg for Fixed {
    type Output = Fixed;
    fn neg(self) -> Fixed {
        Fixed(-self.0)
    }
}
impl AddAssign for Fixed {
    fn add_assign(&mut self, o: Fixed) {
        self.0 = self.0.wrapping_add(o.0);
    }
}
impl SubAssign for Fixed {
    fn sub_assign(&mut self, o: Fixed) {
        self.0 = self.0.wrapping_sub(o.0);
    }
}
impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.to_f32())
    }
}
impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.to_f32())
    }
}

/// A 2D fixed-point vector (world coordinates are in tiles).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Fixed,
    pub y: Fixed,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 {
        x: Fixed::ZERO,
        y: Fixed::ZERO,
    };
    pub const fn new(x: Fixed, y: Fixed) -> Vec2 {
        Vec2 { x, y }
    }
    pub const fn from_tile(x: i32, y: i32) -> Vec2 {
        Vec2 {
            x: Fixed::from_int(x),
            y: Fixed::from_int(y),
        }
    }
    /// Centre of a tile.
    pub fn tile_center(x: i32, y: i32) -> Vec2 {
        Vec2 {
            x: Fixed::from_int(x) + Fixed::HALF,
            y: Fixed::from_int(y) + Fixed::HALF,
        }
    }
    pub fn tile(self) -> (i32, i32) {
        (self.x.to_int(), self.y.to_int())
    }
    pub fn len_sq(self) -> Fixed {
        self.x * self.x + self.y * self.y
    }
    pub fn len(self) -> Fixed {
        self.len_sq().sqrt()
    }
    pub fn dist(self, o: Vec2) -> Fixed {
        (self - o).len()
    }
    pub fn dist_sq(self, o: Vec2) -> Fixed {
        (self - o).len_sq()
    }
    /// Chebyshev distance in tiles (what range checks use for buildings).
    pub fn scale(self, k: Fixed) -> Vec2 {
        Vec2 {
            x: self.x * k,
            y: self.y * k,
        }
    }
    /// Move `step` toward `target`, never overshooting. Returns the new position and
    /// whether the target was reached.
    pub fn step_toward(self, target: Vec2, step: Fixed) -> (Vec2, bool) {
        let d = target - self;
        let len = d.len();
        if len <= step {
            (target, true)
        } else {
            let k = step / len;
            (self + d.scale(k), false)
        }
    }
    /// Facing direction index (0..8) from this toward `o`: 0 = +x (east), clockwise in screen space.
    pub fn facing_toward(self, o: Vec2) -> u8 {
        let d = o - self;
        let (dx, dy) = (d.x.0 as i64, d.y.0 as i64);
        if dx == 0 && dy == 0 {
            return 0;
        }
        // Use octant tests without trig.
        let adx = dx.abs();
        let ady = dy.abs();
        // tan(22.5°) ≈ 0.4142 -> compare ady*1000 vs adx*414 etc.
        let near_x = ady * 1000 < adx * 414;
        let near_y = adx * 1000 < ady * 414;
        match (dx >= 0, dy >= 0, near_x, near_y) {
            (true, _, true, _) => 0,
            (false, _, true, _) => 4,
            (_, true, _, true) => 2,
            (_, false, _, true) => 6,
            (true, true, _, _) => 1,
            (false, true, _, _) => 3,
            (false, false, _, _) => 5,
            (true, false, _, _) => 7,
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }
}
impl fmt::Debug for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = Fixed::from_decimal("1.5");
        let b = Fixed::from_int(2);
        assert_eq!((a * b).to_int(), 3);
        assert_eq!((b / a).round(), 1);
        assert_eq!(Fixed::from_int(16).sqrt(), Fixed::from_int(4));
        assert_eq!(Fixed::from_decimal("0.25").sqrt(), Fixed::HALF);
        assert_eq!(Fixed::from_decimal("-0.5"), -Fixed::HALF);
    }

    #[test]
    fn step_toward_never_overshoots() {
        let a = Vec2::from_tile(0, 0);
        let b = Vec2::from_tile(3, 4);
        let (p, done) = a.step_toward(b, Fixed::from_int(1));
        assert!(!done);
        assert!((p.len() - Fixed::ONE).abs() < Fixed::from_decimal("0.01"));
        let (p, done) = a.step_toward(b, Fixed::from_int(10));
        assert!(done);
        assert_eq!(p, b);
    }

    #[test]
    fn facing() {
        let o = Vec2::from_tile(5, 5);
        assert_eq!(o.facing_toward(Vec2::from_tile(9, 5)), 0);
        assert_eq!(o.facing_toward(Vec2::from_tile(5, 9)), 2);
        assert_eq!(o.facing_toward(Vec2::from_tile(1, 5)), 4);
        assert_eq!(o.facing_toward(Vec2::from_tile(5, 1)), 6);
        assert_eq!(o.facing_toward(Vec2::from_tile(9, 9)), 1);
    }
}
