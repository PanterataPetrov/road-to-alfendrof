//! Draws the logo: `cargo run --bin gen_logo -- assets/ui/logo.png`.
//! "ROAD TO" in silver over a big golden "ALFENDROF", in a chunky 5 x 7 font with a bevel, outline and drop
//! shadow, over a dithered purple glow.

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

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "assets/ui/logo.png".into());
    let (w, h) = (300i32, 90i32);
    let mut img = RgbaImage::new(w as u32, h as u32);

    // portal glow behind the big word: a dithered purple ellipse
    const BAYER: [[i32; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    let (cx, cy) = (w as f32 / 2.0, 58.0);
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = ((x as f32 - cx) / 150.0, (y as f32 - cy) / 36.0);
            let d = dx * dx + dy * dy;
            if d < 1.0 {
                let strength = ((1.0 - d) * 16.0) as i32;
                if strength > BAYER[(y % 4) as usize][(x % 4) as usize] {
                    let c = if d < 0.45 { [150, 60, 200, 255] } else { [96, 38, 150, 255] };
                    put(&mut img, x, y, c);
                }
            }
        }
    }

    // ROAD TO in silver with gold ornaments either side
    let road = text_mask("ROAD TO", 3);
    let rx = (w - road.w) / 2;
    draw_text(&mut img, &road, rx, 6, [246, 248, 255], [150, 160, 206], [255, 255, 255, 255], [96, 104, 150, 255]);
    for (x0, x1) in [(8, rx - 8), (rx + road.w + 8, w - 8)] {
        for x in x0..x1 {
            put(&mut img, x, 16, [255, 214, 90, 255]);
            put(&mut img, x, 17, [160, 96, 30, 255]);
        }
        // a diamond at the inner end
        let dx = if x0 == 8 { x1 } else { x0 };
        for (ox, oy) in [(0, -2), (-1, -1), (0, -1), (1, -1), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)] {
            put(&mut img, dx + ox, 16 + oy, [255, 232, 140, 255]);
        }
    }

    // ALFENDROF, big and golden
    let big = text_mask("ALFENDROF", 4);
    let bx = (w - big.w) / 2;
    draw_text(&mut img, &big, bx, 42, [255, 232, 130], [226, 116, 34], [255, 250, 206, 255], [140, 58, 26, 255]);

    // sparkles
    for (x, y) in [(14, 40), (284, 44), (30, 78), (268, 80), (150, 4)] {
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            put(&mut img, x + dx, y + dy, [255, 255, 235, 255]);
        }
    }

    if let Some(dir) = std::path::Path::new(&out).parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    img.save(&out).unwrap();
    println!("wrote {out}");
}
