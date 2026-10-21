//! Fog of war: explored and currently visible tiles per player.

use crate::ids::*;
use crate::stats;
use crate::types::*;
use crate::world::World;

pub fn update_vision(world: &mut World, data: &GameData) {
    let w = world.map.width;
    let h = world.map.height;
    let n = (w * h) as usize;
    let nplayers = world.players.len();
    // Collect (owner, tile, radius) once.
    let mut sources: Vec<(PlayerId, i32, i32, i32)> = Vec::new();
    for e in world.iter() {
        if e.owner.is_gaia() || e.inside.is_some() {
            continue;
        }
        let st = stats::stats_for(world, data, e.owner, e.ty);
        let mut los = st.los.round();
        if e.build_progress < 1000 {
            los = (los / 2).max(2);
        }
        let (x, y) = e.pos.tile();
        sources.push((e.owner, x, y, los));
    }
    for pid in 1..nplayers {
        let me = PlayerId(pid as u8);
        let mut visible = vec![0u8; n];
        let shared: Vec<&(PlayerId, i32, i32, i32)> =
            sources.iter().filter(|s| world.ally(me, s.0)).collect();
        for &(_, cx, cy, r) in shared {
            let r2 = r * r + r; // slightly rounded circle
            for dy in -r..=r {
                let y = cy + dy;
                if y < 0 || y >= h {
                    continue;
                }
                for dx in -r..=r {
                    if dx * dx + dy * dy > r2 {
                        continue;
                    }
                    let x = cx + dx;
                    if x < 0 || x >= w {
                        continue;
                    }
                    let i = (y * w + x) as usize;
                    visible[i] = visible[i].saturating_add(1);
                }
            }
        }
        let p = &mut world.players[pid];
        for i in 0..n {
            if visible[i] > 0 {
                p.explored[i] = true;
            }
        }
        p.visible = visible;
    }
}
