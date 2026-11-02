//! Draws the logo: `cargo run --bin gen_logo -- assets/ui/logo.png assets/ui/emblem.png`.
//! The wordmark: "ROAD TO" in silver over a big golden "ALFENDROF", in a chunky 5 x 7 font with a bevel,
//! outline and drop shadow, over a dithered purple glow. The emblem: a red Templar cross (a cross pattée)
//! on a white heater shield with a gold rim.

use image::{Rgba, RgbaImage};

const GLYPHS: &[(char, [&str; 7])] = &[
    ('A', [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('L', ["#....", "#....", "#....", "#....", "#....", "#....", "#####"]),
    ('F', ["#####", "#....", "#....", "####.", "#....", "#....", "#...."]),
    ('E', ["#####", "#....", "#....", "####.", "#....", "#....", "#####"]),
    ('N', ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"]),
    ('D', ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."]),
    ('R', ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"]),
    ('O', [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('T', ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."]),
];

struct Mask {
    w: i32,
    h: i32,
    on: Vec<bool>,
}

impl Mask {
    fn get(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h && self.on[(y * self.w + x) as usize]
    }
}

/// The text as a mask, each font cell `scale` pixels square, two cells between letters.
fn text_mask(text: &str, scale: i32) -> Mask {
    let letters: Vec<char> = text.chars().collect();
    let w = (letters.len() as i32 * 7 - 2) * scale;
    let h = 7 * scale;
    let mut on = vec![false; (w * h) as usize];
    for (i, ch) in letters.iter().enumerate() {
        let Some((_, rows)) = GLYPHS.iter().find(|(c, _)| c == ch) else { continue };
        for (cy, row) in rows.iter().enumerate() {
            for (cx, b) in row.bytes().enumerate() {
                if b != b'#' {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = (i as i32 * 7 + cx as i32) * scale + dx;
                        let y = cy as i32 * scale + dy;
                        on[(y * w + x) as usize] = true;
                    }
                }
            }
        }
    }
    Mask { w, h, on }
}

fn put(img: &mut RgbaImage, x: i32, y: i32, c: [u8; 4]) {
    if x >= 0 && y >= 0 && x < img.width() as i32 && y < img.height() as i32 {
        img.put_pixel(x as u32, y as u32, Rgba(c));
    }
}

fn lerp(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 4] {
    let m = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t.clamp(0.0, 1.0)) as u8;
    [m(0), m(1), m(2), 255]
}

/// Draw `mask` at (ox, oy): shadow, outline, and a vertical gradient fill with a light top edge and a dark
/// bottom edge.
fn draw_text(img: &mut RgbaImage, mask: &Mask, ox: i32, oy: i32, top: [u8; 3], bottom: [u8; 3], rim: [u8; 4], base: [u8; 4]) {
    let outline = [28, 10, 38, 255];
    let shadow = [12, 4, 22, 210];
    for y in -3..mask.h + 3 {
        for x in -3..mask.w + 3 {
            // shadow: the mask shifted down-right by 3
            let shaded = (-2..=2).any(|d| mask.get(x - 3 + d, y - 3) || mask.get(x - 3, y - 3 + d));
            if shaded && !mask.get(x, y) {
                put(img, ox + x, oy + y, shadow);
            }
        }
    }
    let r = if mask.h >= 28 { 2 } else { 1 };
    for y in -2..mask.h + 2 {
        for x in -2..mask.w + 2 {
            let near = (-r..=r).any(|dy| (-r..=r).any(|dx| dx * dx + dy * dy <= r * r + 1 && mask.get(x + dx, y + dy)));
            if near && !mask.get(x, y) {
                put(img, ox + x, oy + y, outline);
            }
        }
    }
    for y in 0..mask.h {
        for x in 0..mask.w {
            if !mask.get(x, y) {
                continue;
            }
            let mut c = lerp(top, bottom, y as f32 / mask.h as f32);
            if !mask.get(x, y - 1) {
                c = rim;
            } else if !mask.get(x, y + 1) {
                c = base;
            } else if !mask.get(x - 1, y) {
                // a lit left edge
                c = lerp([c[0], c[1], c[2]], [255, 255, 255], 0.25);
            } else if !mask.get(x + 1, y) {
                c = lerp([c[0], c[1], c[2]], [0, 0, 0], 0.25);
            }
            put(img, ox + x, oy + y, c);
        }
    }
}

/// How far inside the shield's edge a point is, 0 outside; used for the rim.
fn shield_half_width(y: i32) -> f32 {
    // straight sides down to y = 44, then curving in to a point at y = 78
    if y < 0 || y > 78 {
        return -1.0;
    }
    if y <= 44 {
        30.0
    } else {
        let t = (y - 44) as f32 / 34.0;
        30.0 * (1.0 - t.powf(1.7))
    }
}

fn emblem() -> RgbaImage {
    let (w, h) = (68, 80);
    let mut img = RgbaImage::new(w as u32, h as u32);
    let cx = 34.0f32;
    let inside = |x: i32, y: i32| -> bool {
        let half = shield_half_width(y);
        // slightly rounded top corners
        let corner = y < 3 && (x as f32 - cx).abs() > half - (3 - y) as f32 * 1.5;
        half > 0.0 && (x as f32 + 0.5 - cx).abs() <= half && !corner
    };
    // how many pixels from the edge (up to 6)
    let depth = |x: i32, y: i32| -> i32 {
        for d in 1..=6 {
            for (dx, dy) in [(-d, 0), (d, 0), (0, -d), (0, d), (-d, -d), (d, d), (-d, d), (d, -d)] {
                if !inside(x + dx, y + dy) {
                    return d;
                }
            }
        }
        7
    };
    for y in 0..h {
        for x in 0..w {
            if !inside(x, y) {
                continue;
            }
            let d = depth(x, y);
            let c = if d == 1 {
                [28, 18, 30, 255]
            } else if d <= 3 {
                // the gold rim, lit from the upper left
                if (x as f32) < cx && y < 40 { [255, 226, 120, 255] } else { [214, 160, 54, 255] }
            } else if d == 4 {
                [120, 78, 30, 255]
            } else {
                // white field with a faint shade towards the bottom
                let t = y as f32 / h as f32;
                let g = (252.0 - 34.0 * t) as u8;
                [g, g, (g as f32 * 0.97) as u8, 255]
            };
            put(&mut img, x, y, c);
        }
    }
    // the cross pattée: arms narrow at the centre and flaring to straight ends
    let (ccx, ccy) = (34.0f32, 34.0f32);
    let (arm_h, arm_v) = (22.0f32, 27.0f32);
    let flare = |d: f32, len: f32| 3.6 + (d / len) * 6.2;
    let in_cross = |x: i32, y: i32| -> bool {
        let (dx, dy) = ((x as f32 + 0.5 - ccx).abs(), (y as f32 + 0.5 - ccy).abs());
        (dx <= arm_h && dy <= flare(dx, arm_h)) || (dy <= arm_v && dx <= flare(dy, arm_v))
    };
    for y in 0..h {
        for x in 0..w {
            if !in_cross(x, y) {
                continue;
            }
            let lit = !in_cross(x - 1, y) || !in_cross(x, y - 1);
            let dark = !in_cross(x + 1, y) || !in_cross(x, y + 1);
            let c = if lit && !dark { [238, 78, 78, 255] } else if dark { [132, 14, 28, 255] } else { [200, 28, 40, 255] };
            put(&mut img, x, y, c);
        }
    }
    // a dark outline round the cross
    let copy = img.clone();
    for y in 0..h {
        for x in 0..w {
            if in_cross(x, y) {
                continue;
            }
            let near = (-1..=1).any(|dy| (-1..=1).any(|dx| in_cross(x + dx, y + dy)));
            if near && copy.get_pixel(x as u32, y as u32)[3] > 0 {
                put(&mut img, x, y, [70, 8, 18, 255]);
            }
        }
    }
    // a gloss on the upper left of the shield
    for y in 6..22 {
        for x in 8..22 {
            if x + y < 24 && inside(x, y) && copy.get_pixel(x as u32, y as u32) == &Rgba([252, 252, 244, 255]) {
                put(&mut img, x, y, [255, 255, 255, 255]);
            }
        }
    }
    img
}

/// The wordmark, small enough to sit beside the emblem (200 x 62).
fn wordmark() -> RgbaImage {
    let (w, h) = (200i32, 62i32);
    let mut img = RgbaImage::new(w as u32, h as u32);
    const BAYER: [[i32; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    let (cx, cy) = (w as f32 / 2.0, 38.0);
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = ((x as f32 - cx) / 100.0, (y as f32 - cy) / 26.0);
            let d = dx * dx + dy * dy;
            if d < 1.0 {
                let strength = ((1.0 - d) * 16.0) as i32;
                if strength > BAYER[(y % 4) as usize][(x % 4) as usize] {
                    put(&mut img, x, y, if d < 0.45 { [150, 60, 200, 255] } else { [96, 38, 150, 255] });
                }
            }
        }
    }
    let road = text_mask("ROAD TO", 2);
    draw_text(&mut img, &road, (w - road.w) / 2, 5, [246, 248, 255], [150, 160, 206], [255, 255, 255, 255], [96, 104, 150, 255]);
    let big = text_mask("ALFENDROF", 3);
    draw_text(&mut img, &big, (w - big.w) / 2, 28, [255, 232, 130], [226, 116, 34], [255, 250, 206, 255], [140, 58, 26, 255]);
    for (x, y) in [(6, 30), (192, 34), (14, 56), (184, 58)] {
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            put(&mut img, x + dx, y + dy, [255, 255, 235, 255]);
        }
    }
    img
}

fn main() {
    let mut args = std::env::args().skip(1);
    let logo = args.next().unwrap_or_else(|| "assets/ui/logo.png".into());
    let emblem_path = args.next().unwrap_or_else(|| "assets/ui/emblem.png".into());
    if let Some(dir) = std::path::Path::new(&logo).parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    wordmark().save(&logo).unwrap();
    emblem().save(&emblem_path).unwrap();
    println!("wrote {logo} and {emblem_path}");
}
