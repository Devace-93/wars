use wars_render::draw::DrawList;
use wars_render::text::{Font, TextSystem, ATLAS};
use wars_render::Color;

fn glyph_has_pixels(t: &TextSystem, dl: &DrawList, quad: usize) -> (usize, usize, usize, u32) {
    // quad index -> 4 vertices; read uv0/uv1 and count non-zero atlas pixels inside.
    let v0 = dl.verts[quad * 4];
    let v2 = dl.verts[quad * 4 + 2];
    let x0 = (v0.uv[0] * ATLAS as f32) as usize;
    let y0 = (v0.uv[1] * ATLAS as f32) as usize;
    let x1 = (v2.uv[0] * ATLAS as f32) as usize;
    let y1 = (v2.uv[1] * ATLAS as f32) as usize;
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            if t.pixels[y * ATLAS as usize + x] > 0 {
                n += 1;
            }
        }
    }
    (x0, y0, x1 - x0, n)
}

#[test]
fn every_glyph_of_a_label_lands_in_the_atlas() {
    let mut t = TextSystem::new();
    let mut dl = DrawList::new();
    // Warm the cache the way the UI does: measure first, then draw.
    let s = "Civilisation Screenshot saved: /home/enrique/x.png";
    t.measure(Font::Regular, 15, s);
    t.draw(&mut dl, Font::Regular, 15, 10.0, 10.0, s, Color::WHITE);
    let quads = dl.verts.len() / 4;
    let non_space = s.chars().filter(|c| *c != ' ').count();
    assert_eq!(quads, non_space, "one quad per visible glyph");
    for q in 0..quads {
        let (x, y, w, n) = glyph_has_pixels(&t, &dl, q);
        assert!(
            n > 0,
            "glyph quad {q} at atlas ({x},{y}) w{w} has no pixels"
        );
    }
}

#[test]
fn many_sizes_and_bold_do_not_corrupt_each_other() {
    let mut t = TextSystem::new();
    let mut dl = DrawList::new();
    for size in [11u32, 12, 13, 14, 15, 16, 17, 22, 40] {
        for font in [Font::Regular, Font::Bold] {
            t.draw(
                &mut dl,
                font,
                size,
                0.0,
                0.0,
                "The quick brown fox jumps over the lazy dog 0123456789 ÁÉÍÓÚ ñ ×·",
                Color::WHITE,
            );
        }
    }
    let quads = dl.verts.len() / 4;
    let mut empty = 0;
    for q in 0..quads {
        let (_, _, _, n) = glyph_has_pixels(&t, &dl, q);
        if n == 0 {
            empty += 1;
        }
    }
    assert_eq!(empty, 0, "{empty} of {quads} glyph quads are empty");
}

#[test]
fn batches_cover_every_index_across_clip_changes() {
    use wars_render::draw::Clip;
    let mut dl = DrawList::new();
    dl.rect(0.0, 0.0, 1.0, 1.0, Color::WHITE);
    dl.set_clip(Some(Clip {
        x: 0,
        y: 0,
        w: 10,
        h: 10,
    }));
    dl.rect(0.0, 0.0, 1.0, 1.0, Color::WHITE);
    dl.set_clip(None);
    dl.rect(0.0, 0.0, 1.0, 1.0, Color::WHITE);
    dl.tri([0.0, 0.0], [1.0, 0.0], [0.0, 1.0], Color::WHITE);
    let covered: u32 = dl.batches.iter().map(|b| b.index_count).sum();
    assert_eq!(covered as usize, dl.idx.len());
    let mut expected = 0;
    for b in &dl.batches {
        assert_eq!(b.index_start, expected, "batches are contiguous");
        expected += b.index_count;
    }
}
