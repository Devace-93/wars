//! Integer-only value noise and trigonometry helpers.

fn hash(x: i32, y: i32, seed: u64) -> u32 {
    let mut h = seed ^ ((x as u32 as u64) << 32) ^ (y as u32 as u64);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    (h & 0xff) as u32
}

/// Smooth value noise in 0..=255 with the given cell size (two octaves).
pub fn value_noise(x: i32, y: i32, cell: i32, seed: u64) -> u32 {
    let a = octave(x, y, cell, seed);
    let b = octave(x, y, (cell / 2).max(1), seed ^ 0xabcdef);
    (a * 2 + b) / 3
}

fn octave(x: i32, y: i32, cell: i32, seed: u64) -> u32 {
    let gx = x.div_euclid(cell);
    let gy = y.div_euclid(cell);
    let fx = (x.rem_euclid(cell) * 256) / cell; // 0..256
    let fy = (y.rem_euclid(cell) * 256) / cell;
    // smoothstep in fixed point
    let sx = (fx * fx * (3 * 256 - 2 * fx)) / (256 * 256);
    let sy = (fy * fy * (3 * 256 - 2 * fy)) / (256 * 256);
    let v00 = hash(gx, gy, seed) as i32;
    let v10 = hash(gx + 1, gy, seed) as i32;
    let v01 = hash(gx, gy + 1, seed) as i32;
    let v11 = hash(gx + 1, gy + 1, seed) as i32;
    let top = v00 + ((v10 - v00) * sx) / 256;
    let bottom = v01 + ((v11 - v01) * sx) / 256;
    (top + ((bottom - top) * sy) / 256).clamp(0, 255) as u32
}

/// (cos, sin) of `degrees` scaled by 1000, from a 360-entry table.
pub fn circle_point(degrees: i32) -> (i32, i32) {
    const SIN: [i32; 91] = [
        0, 17, 35, 52, 70, 87, 105, 122, 139, 156, 174, 191, 208, 225, 242, 259, 276, 292, 309,
        326, 342, 358, 375, 391, 407, 423, 438, 454, 469, 485, 500, 515, 530, 545, 559, 574, 588,
        602, 616, 629, 643, 656, 669, 682, 695, 707, 719, 731, 743, 755, 766, 777, 788, 799, 809,
        819, 829, 839, 848, 857, 866, 875, 883, 891, 899, 906, 914, 921, 927, 934, 940, 946, 951,
        956, 961, 966, 970, 974, 978, 982, 985, 988, 990, 993, 995, 996, 998, 999, 999, 1000, 1000,
    ];
    let d = degrees.rem_euclid(360);
    let sin = |a: i32| -> i32 {
        let a = a.rem_euclid(360);
        match a {
            0..=90 => SIN[a as usize],
            91..=180 => SIN[(180 - a) as usize],
            181..=270 => -SIN[(a - 180) as usize],
            _ => -SIN[(360 - a) as usize],
        }
    };
    (sin(d + 90), sin(d))
}
