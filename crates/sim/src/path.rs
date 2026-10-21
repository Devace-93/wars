//! Grid A* pathfinding with 8-connected moves. Deterministic: ties are broken by
//! insertion order through a monotonically increasing sequence number.

use crate::map::Map;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    f: i32,
    g: i32,
    seq: u32,
    idx: u32,
}

impl Ord for Node {
    fn cmp(&self, o: &Self) -> Ordering {
        // min-heap on f, then g (prefer longer travelled = closer), then seq
        o.f.cmp(&self.f)
            .then(self.g.cmp(&o.g))
            .then(o.seq.cmp(&self.seq))
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

const DIRS: [(i32, i32, i32); 8] = [
    (1, 0, 10),
    (-1, 0, 10),
    (0, 1, 10),
    (0, -1, 10),
    (1, 1, 14),
    (1, -1, 14),
    (-1, 1, 14),
    (-1, -1, 14),
];

pub const MAX_EXPANSIONS: usize = 12_000;

fn heuristic(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    let dx = (ax - bx).abs();
    let dy = (ay - by).abs();
    let (mn, mx) = if dx < dy { (dx, dy) } else { (dy, dx) };
    14 * mn + 10 * (mx - mn)
}

/// Find a tile path from `from` to `to`. If the goal is unreachable or blocked, returns the
/// path to the closest reachable tile (by heuristic). `goal_ok` lets the caller accept any
/// tile adjacent to a building footprint as the goal.
pub fn find_path(
    map: &Map,
    from: (i32, i32),
    to: (i32, i32),
    naval: bool,
    goal_ok: &dyn Fn(i32, i32) -> bool,
) -> Vec<(i32, i32)> {
    let w = map.width;
    let n = (map.width * map.height) as usize;
    if !map.in_bounds(from.0, from.1) {
        return Vec::new();
    }
    let start = map.idx(from.0, from.1) as u32;
    let mut g_score = vec![i32::MAX; n];
    let mut came = vec![u32::MAX; n];
    let mut closed = vec![false; n];
    let mut heap = BinaryHeap::new();
    let mut seq = 0u32;
    g_score[start as usize] = 0;
    heap.push(Node {
        f: heuristic(from.0, from.1, to.0, to.1),
        g: 0,
        seq,
        idx: start,
    });
    let mut best_idx = start;
    let mut best_h = heuristic(from.0, from.1, to.0, to.1);
    let mut expansions = 0usize;
    let mut found: Option<u32> = None;
    while let Some(node) = heap.pop() {
        let i = node.idx as usize;
        if closed[i] {
            continue;
        }
        closed[i] = true;
        let x = node.idx as i32 % w;
        let y = node.idx as i32 / w;
        if goal_ok(x, y) {
            found = Some(node.idx);
            break;
        }
        let h = heuristic(x, y, to.0, to.1);
        if h < best_h {
            best_h = h;
            best_idx = node.idx;
        }
        expansions += 1;
        if expansions > MAX_EXPANSIONS {
            break;
        }
        for (dx, dy, cost) in DIRS {
            let (nx, ny) = (x + dx, y + dy);
            if !map.in_bounds(nx, ny) {
                continue;
            }
            let ni = map.idx(nx, ny);
            if closed[ni] {
                continue;
            }
            let goal_tile = goal_ok(nx, ny);
            if !map.passable(nx, ny, naval) && !goal_tile {
                continue;
            }
            // No corner cutting through blocked orthogonal neighbours.
            if dx != 0
                && dy != 0
                && (!map.passable(x + dx, y, naval) || !map.passable(x, y + dy, naval))
            {
                continue;
            }
            let ng = node.g + cost;
            if ng < g_score[ni] {
                g_score[ni] = ng;
                came[ni] = node.idx;
                seq += 1;
                heap.push(Node {
                    f: ng + heuristic(nx, ny, to.0, to.1),
                    g: ng,
                    seq,
                    idx: ni as u32,
                });
            }
        }
    }
    let end = found.unwrap_or(best_idx);
    let mut path = Vec::new();
    let mut cur = end;
    while cur != start && cur != u32::MAX {
        path.push((cur as i32 % w, cur as i32 / w));
        cur = came[cur as usize];
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Terrain;

    #[test]
    fn straight_path() {
        let map = Map::new(10, 10, Terrain::Grass);
        let p = find_path(&map, (0, 0), (5, 0), false, &|x, y| (x, y) == (5, 0));
        assert_eq!(p.len(), 5);
        assert_eq!(*p.last().unwrap(), (5, 0));
    }

    #[test]
    fn goes_around_water() {
        let mut map = Map::new(10, 10, Terrain::Grass);
        for y in 0..9 {
            map.set_terrain(5, y, Terrain::Water);
        }
        let p = find_path(&map, (0, 0), (9, 0), false, &|x, y| (x, y) == (9, 0));
        assert_eq!(*p.last().unwrap(), (9, 0));
        assert!(p.iter().any(|&(_, y)| y == 9));
    }

    #[test]
    fn unreachable_returns_closest() {
        let mut map = Map::new(10, 10, Terrain::Grass);
        for y in 0..10 {
            map.set_terrain(5, y, Terrain::Water);
        }
        let p = find_path(&map, (0, 0), (9, 0), false, &|x, y| (x, y) == (9, 0));
        assert!(!p.is_empty());
        assert!(p.last().unwrap().0 <= 4);
    }
}
