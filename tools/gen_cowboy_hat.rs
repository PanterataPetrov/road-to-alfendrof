//! Draws the Cowboy's hat: `cargo run --bin gen_cowboy_hat -- assets/sprites/cowboy`.
//! For every body drawing in the folder it writes `<name>_mark.png` (the hat on his head, found from the bald top of
//! the head in that drawing), and it writes the thrown hat, `hat_0.png` and `hat_1.png`.

use std::path::Path;

use image::{Rgba, RgbaImage};

/// The hat seen a little from the side (he faces right): a pinched crown, a gold band, and a brim that curls up on
/// the far side and droops over the forehead on the near one. 19 wide, 11 tall.
const HAT: [&str; 11] = [
    ".......kkkkkk......",
    "......kmhhmmmk.....",
    "......kmhmkmmBk....",
    ".....kmmmmmmmBBk...",
    ".....kmmmmmmmBBk...",
    ".....kBBBBBBBBBBk..",
    "....kkgggggggggkk..",
    ".kkmkBBBBBBBBBBkkk.",
    "kmhmmmmmmmmmmmmBBkk",
    ".kkmmmmmmmmmmmBBBBk",
    "...kkkkkkkkkkkkkkk.",
];

fn colour(c: u8) -> Option<Rgba<u8>> {
    match c {
        b'm' => Some(Rgba([156, 102, 54, 255])),
        b'h' => Some(Rgba([190, 134, 78, 255])),
        b'B' => Some(Rgba([112, 70, 36, 255])),
        b'k' => Some(Rgba([46, 28, 18, 255])),
        b'g' => Some(Rgba([226, 176, 64, 255])),
        _ => None,
    }
}

/// Skin: light and warm (not the boots, the vest or the blue tattoo).
fn skin(p: &Rgba<u8>) -> bool {
    p[3] > 200 && p[0] > 170 && p[1] > 120 && p[2] > 85 && p[0] >= p[1] && p[1] >= p[2]
}

/// The skin runs at least `min` wide in row `y`, as (start, end).
fn runs(body: &RgbaImage, y: u32, min: u32) -> Vec<(u32, u32)> {
    let mut found = Vec::new();
    let mut run_start = None;
    for x in 0..=body.width() {
        let on = x < body.width() && skin(body.get_pixel(x, y));
        match (on, run_start) {
            (true, None) => run_start = Some(x),
            (false, Some(s)) => {
                if x - s >= min {
                    found.push((s, x));
                }
                run_start = None;
            }
            _ => {}
        }
    }
    found
}

/// The top of the head: the first row with a run of skin at least 8 wide that stays that wide, around the same
/// middle, for the next few rows (a fist or a raised forearm is narrower or not as tall), and the middle of that run.
fn find_head(body: &RgbaImage, wide: u32, rows: u32) -> Option<(i32, i32)> {
    for y in 0..body.height().saturating_sub(rows) {
        for (s, e) in runs(body, y, wide) {
            let mid = (s + e) as i32 / 2;
            let solid = (1..rows).all(|dy| runs(body, y + dy, wide).iter().any(|&(s2, e2)| ((s2 + e2) as i32 / 2 - mid).abs() <= 3));
            if solid {
                return Some((mid, y as i32));
            }
        }
    }
    None
}

fn draw_hat(img: &mut RgbaImage, left: i32, top: i32, scale: i32, tint: impl Fn(Rgba<u8>) -> Rgba<u8>) {
    for (row, line) in HAT.iter().enumerate() {
        for (col, c) in line.bytes().enumerate() {
            let Some(p) = colour(c) else { continue };
            for dy in 0..scale {
                for dx in 0..scale {
                    let (x, y) = (left + col as i32 * scale + dx, top + row as i32 * scale + dy);
                    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                        img.put_pixel(x as u32, y as u32, tint(p));
                    }
                }
            }
        }
    }
}

fn main() {
    let dir = std::env::args().nth(1).expect("usage: gen_cowboy_hat <sprite folder>");
    let dir = Path::new(&dir);
    let mut done = 0;
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(dir).expect("sprite folder") {
        let path = entry.unwrap().path();
        let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if path.extension().is_none_or(|e| e != "png") || name.ends_with("_mark") || name.starts_with("hat_") || name.starts_with("bullet_") || name.contains("whip") {
            continue;
        }
        let body = image::open(&path).expect("drawing").to_rgba8();
        let mut mark = RgbaImage::new(body.width(), body.height());
        // the tumbling roll drawings are sideways, so they get a looser match
        match find_head(&body, 8, 5).or_else(|| find_head(&body, 5, 3)) {
            Some((cx, top)) => {
                // the brim comes down over the forehead, the crown above it
                draw_hat(&mut mark, cx - 10, top - 4, 1, |p| p);
                done += 1;
            }
            None => missing.push(name.clone()),
        }
        mark.save(dir.join(format!("{name}_mark.png"))).expect("write mark");
    }
    println!("hats on {done} drawings; no head found in: {missing:?}");

    // the thrown hat, twice as big (the same drawing), the second drawing a little brighter
    for (k, lift) in [(0, 0u8), (1, 28u8)] {
        let mut img = RgbaImage::new(42, 26);
        draw_hat(&mut img, 2, 2, 2, |p| Rgba([p[0].saturating_add(lift), p[1].saturating_add(lift), p[2].saturating_add(lift / 2), p[3]]));
        img.save(dir.join(format!("hat_{k}.png"))).expect("write hat");
    }
}
