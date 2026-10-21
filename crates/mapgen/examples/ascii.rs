//! Print a generated map as ASCII: `cargo run -p wars-mapgen --example ascii -- arabia 7`
use wars_mapgen::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mt = args
        .get(1)
        .and_then(|s| MapType::from_name(s))
        .unwrap_or(MapType::Arabia);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let data = wars_data::data();
    let params = MapParams {
        map_type: mt,
        size: MapSize::Tiny,
        seed,
        ..Default::default()
    };
    let w = generate(data, &params);
    let mut grid: Vec<Vec<char>> = w
        .map
        .tiles
        .chunks(w.map.width as usize)
        .map(|row| {
            row.iter()
                .map(|t| match t.terrain {
                    wars_sim::Terrain::Water => '~',
                    wars_sim::Terrain::DeepWater => '≈',
                    wars_sim::Terrain::Shallows => '-',
                    wars_sim::Terrain::Forest => '♣',
                    wars_sim::Terrain::Beach => ':',
                    wars_sim::Terrain::Dirt => ',',
                    _ => '.',
                })
                .collect()
        })
        .collect();
    for e in w.iter() {
        let def = data.object(e.ty);
        let (x, y, fw, fh) = w.footprint(data, e);
        let c = if e.owner.is_gaia() {
            def.glyph
        } else {
            char::from_digit(e.owner.0 as u32, 10).unwrap()
        };
        for yy in y..y + fh {
            for xx in x..x + fw {
                if w.map.in_bounds(xx, yy) {
                    grid[yy as usize][xx as usize] = c;
                }
            }
        }
    }
    for row in grid {
        println!("{}", row.into_iter().collect::<String>());
    }
    println!("{} entities, {} relics", w.count(), w.total_relics);
}
