//! Draws the placeholder pixel art for the map. Runs offline and costs nothing:
//! `cargo run --bin gen_sprites` writes PNGs into `assets/backgrounds/`.
//!
//! The characters are drawn by `gen_ayz` (`cargo run --bin gen_ayz -- --frames assets`), built
//! from the original sprite.

use std::fs;
use std::path::{Path, PathBuf};


type Rgba = [u8; 4];

fn c(r: f32, g: f32, b: f32) -> Rgba {
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]
}

const CLEAR: Rgba = [0, 0, 0, 0];

struct Img {
    w: i32,
    h: i32,
    px: Vec<Rgba>,
}

impl Img {
    fn new(w: u32, h: u32) -> Img {
        Img { w: w as i32, h: h as i32, px: vec![CLEAR; (w * h) as usize] }
    }

    fn set(&mut self, x: i32, y: i32, col: Rgba) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = col;
        }
    }

    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, col: Rgba) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, col);
            }
        }
    }

    fn disc(&mut self, cx: i32, cy: i32, r: i32, col: Rgba) {
        self.ellipse(cx, cy, r, r, col);
    }

    fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, col: Rgba) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        for y in -ry..=ry {
            for x in -rx..=rx {
                if (x * x * ry * ry + y * y * rx * rx) <= rx * rx * ry * ry {
                    self.set(cx + x, cy + y, col);
                }
            }
        }
    }

    /// Draw `col` over whatever is there, with opacity 0..1.
    fn blend(&mut self, x: i32, y: i32, col: Rgba, alpha: f32) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h || alpha <= 0.0 {
            return;
        }
        let idx = (y * self.w + x) as usize;
        let d = self.px[idx];
        let da = d[3] as f32 / 255.0;
        let oa = alpha + da * (1.0 - alpha);
        let f = |s: u8, dd: u8| ((s as f32 * alpha + dd as f32 * da * (1.0 - alpha)) / oa) as u8;
        self.px[idx] = [f(col[0], d[0]), f(col[1], d[1]), f(col[2], d[2]), (oa * 255.0) as u8];
    }

    fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, col: Rgba) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let mut err = dx + dy;
        loop {
            self.set(x0, y0, col);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Enlarge by a whole factor with no smoothing, so every pixel becomes an `n` x `n` block.
    fn upscale(&self, n: i32) -> Img {
        let mut out = Img::new((self.w * n) as u32, (self.h * n) as u32);
        for y in 0..out.h {
            for x in 0..out.w {
                out.px[(y * out.w + x) as usize] = self.px[((y / n) * self.w + x / n) as usize];
            }
        }
        out
    }

    fn save(&self, path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut buf = image::RgbaImage::new(self.w as u32, self.h as u32);
        for y in 0..self.h {
            for x in 0..self.w {
                buf.put_pixel(x as u32, y as u32, image::Rgba(self.px[(y * self.w + x) as usize]));
            }
        }
        buf.save(path).unwrap();
        println!("wrote {}", path.display());
    }
}

fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2]), 255]
}

fn shade(col: Rgba, factor: f32) -> Rgba {
    let f = |x: u8| (x as f32 * factor).clamp(0.0, 255.0) as u8;
    [f(col[0]), f(col[1]), f(col[2]), col[3]]
}

fn gradient(stops: &[(f32, Rgba)], t: f32) -> Rgba {
    for pair in stops.windows(2) {
        let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
        if t <= t1 {
            return mix(c0, c1, ((t - t0) / (t1 - t0)).clamp(0.0, 1.0));
        }
    }
    stops[stops.len() - 1].1
}

struct Rng(u32);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.0 >> 8
    }

    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo).max(1) as u32) as i32
    }

    fn chance(&mut self, percent: u32) -> bool {
        self.next() % 100 < percent
    }
}

/// A repeatable pseudo-random number from two coordinates, for per-stone tones.
fn hash2(a: i32, b: i32) -> u32 {
    let h = (a as u32).wrapping_mul(73_856_093) ^ (b as u32).wrapping_mul(19_349_663);
    (h ^ (h >> 13)).wrapping_mul(1_274_126_177) >> 8
}

// ---------------------------------------------------------------------------------------------
// The map: a lane in Istanbul climbing toward Galata Tower on a sunny day, with an evil portal over the tower
// ---------------------------------------------------------------------------------------------
//
// Everything here is drawn at HALF resolution and enlarged 2x when saved, so every background
// pixel is a chunky 2 x 2 block. The layers are 320 x 108 (640 x 216 in the game); image column
// 160 is world x = 0.

const LAYER_W: i32 = 200;
const LAYER_H: i32 = 108;
const SKY_W: i32 = 192;
/// Row of the floor line (the floor top is at world y = -80).
const GROUND_ROW: i32 = 94;
/// Galata Tower stands at the far end of the lane, with the portal torn open in the sky above its spire.
const TOWER_X: i32 = 93;
const SPIRE_TOP: i32 = 31;
/// Below the HUD (which covers roughly the top 16 rows), directly above the spire.
const PORTAL: (i32, i32) = (TOWER_X, 16);

fn cloud(i: &mut Img, cx: i32, cy: i32, size: i32, rng: &mut Rng) {
    let white = c(1.0, 1.0, 1.0);
    let under = c(0.80, 0.89, 0.98);
    let puffs = 4 + size / 4;
    let mut discs = Vec::new();
    for k in 0..puffs {
        let t = k as f32 / (puffs - 1) as f32;
        let x = cx + ((t - 0.5) * size as f32 * 2.2) as i32;
        // Tall in the middle, low at the ends: a soft rolling top.
        let r = ((0.45 + 0.55 * (t * std::f32::consts::PI).sin()) * size as f32 * 0.5) as i32 + rng.range(0, 2);
        discs.push((x, cy - r / 2, r.max(2)));
    }
    for &(x, y, r) in &discs {
        i.disc(x, y + 1, r, under);
    }
    for &(x, y, r) in &discs {
        i.disc(x, y, r, white);
    }
}

fn wisp(i: &mut Img, x0: i32, y0: i32, len: i32) {
    for t in 0..len {
        let y = y0 + (2.5 * (t as f32 / 8.0).sin()) as i32 - t / 14;
        i.blend(x0 + t, y, c(1.0, 1.0, 1.0), 0.65);
        if t % 3 == 0 {
            i.blend(x0 + t, y + 1, c(1.0, 1.0, 1.0), 0.3);
        }
    }
}

fn sky() -> Img {
    let (w, h) = (SKY_W, LAYER_H);
    let mut i = Img::new(w as u32, h as u32);
    let stops = [(0.0, c(0.28, 0.56, 0.95)), (0.55, c(0.50, 0.76, 0.98)), (1.0, c(0.88, 0.96, 0.99))];
    // Banded gradient with a checker dither between bands.
    let bands = 10;
    for y in 0..h {
        let t = y as f32 / h as f32 * bands as f32;
        let (band, frac) = (t.floor(), t - t.floor());
        for x in 0..w {
            let dither = if frac > 0.65 && (x + y) % 2 == 0 { 1.0 } else { 0.0 };
            i.set(x, y, gradient(&stops, ((band + dither) / bands as f32).min(1.0)));
        }
    }
    // The sun, with stepped halos.
    for (r, a) in [(16, 0.10), (13, 0.12), (10, 0.16)] {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    i.blend(160 + x, 14 + y, c(1.0, 0.97, 0.80), a);
                }
            }
        }
    }
    i.disc(160, 14, 7, c(1.0, 0.98, 0.78));
    i.disc(160, 14, 5, c(1.0, 1.0, 0.92));

    let mut rng = Rng(3);
    cloud(&mut i, 40, 30, 14, &mut rng);
    cloud(&mut i, 100, 18, 10, &mut rng);
    cloud(&mut i, 170, 46, 12, &mut rng);
    cloud(&mut i, 20, 62, 8, &mut rng);
    wisp(&mut i, 70, 40, 56);
    wisp(&mut i, 110, 8, 48);
    wisp(&mut i, 8, 48, 40);
    // Gulls riding the wind.
    let gull = c(0.32, 0.36, 0.48);
    for (x, y) in [(30, 50), (46, 44), (126, 56), (140, 70)] {
        for (dx, dy) in [(-2, 0), (-1, 1), (0, 0), (1, 1), (2, 0)] {
            i.set(x + dx, y + dy, gull);
        }
    }
    i
}
/// The evil portal: a torn, spiralling rift in the sky above the tower, ringed with crackling
/// energy and trailing dark storm clouds, with lightning arcing down to the spire. It is its
/// own layer so the game can make it pulse.
fn portal() -> Img {
    let (w, h) = (LAYER_W, LAYER_H);
    let mut i = Img::new(w as u32, h as u32);
    let mut rng = Rng(66);
    let (cx, cy) = PORTAL;
    let (rx, ry) = (12, 8);

    // Storm clouds spiralling around it, flowing outward and thinning into the sky. Kept close
    // to the portal so they do not wash out the tower below.
    for arm in 0..4 {
        let phase = arm as f32 * std::f32::consts::TAU / 4.0;
        for t in 0..110 {
            let f = t as f32 / 110.0;
            let a = phase + f * 5.5;
            let r = 10.0 + f * 9.0;
            let (x, y) = (cx + (r * a.cos()) as i32, cy + (r * 0.8 * a.sin()) as i32);
            let thick = ((1.0 - f) * 3.0) as i32 + 1;
            if y < SPIRE_TOP - 6 {
                i.ellipse(x, y, thick + 1, thick, c(0.42, 0.16, 0.56));
            }
        }
    }
    // Glow: stepped halos.
    for step in (1..=5).rev() {
        let grow = 1.0 + step as f32 * 0.22;
        let (grx, gry) = ((rx as f32 * grow) as i32, (ry as f32 * grow) as i32);
        for y in -gry..=gry {
            for x in -grx..=grx {
                let d = (x * x) as f32 / (grx * grx) as f32 + (y * y) as f32 / (gry * gry) as f32;
                if d <= 1.0 {
                    i.blend(cx + x, cy + y, c(0.80, 0.20, 0.98), 0.10);
                }
            }
        }
    }
    // The ring and its layers, from the outside in.
    for (grow, col) in [(0, c(0.55, 0.08, 0.76)), (-2, c(0.86, 0.34, 0.98)), (-4, c(0.32, 0.04, 0.50)), (-6, c(0.10, 0.01, 0.18))] {
        i.ellipse(cx, cy, rx + grow, ry + grow * 4 / 5, col);
    }
    // Ragged flames licking off the rim.
    let spikes = 28;
    for k in 0..spikes {
        let a = k as f32 / spikes as f32 * std::f32::consts::TAU;
        let out = 2 + rng.range(0, 4);
        let (x0, y0) = (cx + ((rx - 1) as f32 * a.cos()) as i32, cy + ((ry - 1) as f32 * a.sin()) as i32);
        let (x1, y1) = (cx + ((rx + out) as f32 * a.cos()) as i32, cy + ((ry + out) as f32 * a.sin()) as i32);
        i.line(x0, y0, x1, y1, c(0.92, 0.50, 1.0));
    }
    // Spiral arms dragging into the void.
    for arm in 0..3 {
        for t in 0..90 {
            let f = t as f32 / 90.0;
            let a = f * 8.0 + arm as f32 * std::f32::consts::TAU / 3.0;
            let (x, y) = (cx + (f * (rx - 3) as f32 * a.cos()) as i32, cy + (f * (ry - 3) as f32 * a.sin()) as i32);
            i.set(x, y, mix(c(0.96, 0.62, 1.0), c(0.56, 0.12, 0.82), f));
        }
    }
    // The void at the centre, with two red eyes glaring down at the fight.
    i.ellipse(cx, cy, 5, 6, c(0.02, 0.0, 0.05));
    i.rect(cx - 3, cy - 1, 2, 1, c(1.0, 0.16, 0.20));
    i.rect(cx + 1, cy - 1, 2, 1, c(1.0, 0.16, 0.20));
    i.set(cx - 3, cy - 1, c(1.0, 0.78, 0.72));
    i.set(cx + 1, cy - 1, c(1.0, 0.78, 0.72));

    // Lightning from the portal down to the tip of the spire, and a few stray bolts.
    let (mut x, mut y) = (cx, cy + ry);
    while y < SPIRE_TOP - 3 {
        let toward = (TOWER_X - x).signum();
        let (nx, ny) = (x + toward * rng.range(0, 3) + rng.range(-2, 3), y + rng.range(2, 5));
        let ny = ny.min(SPIRE_TOP - 3);
        i.line(x, y, nx, ny, c(1.0, 0.86, 1.0));
        i.blend(nx + 1, ny, c(0.90, 0.40, 1.0), 0.6);
        i.blend(nx - 1, ny, c(0.90, 0.40, 1.0), 0.6);
        x = nx;
        y = ny;
        if y >= SPIRE_TOP - 3 {
            break;
        }
    }
    i.disc(TOWER_X, SPIRE_TOP - 3, 2, c(1.0, 0.86, 1.0));
    for _ in 0..8 {
        let a = rng.range(0, 360) as f32 * std::f32::consts::PI / 180.0;
        let (mut sx, mut sy) = (cx + ((rx + 3) as f32 * a.cos()) as i32, cy + ((ry + 3) as f32 * a.sin()) as i32);
        for _ in 0..rng.range(2, 5) {
            let (nx, ny) = (sx + (a.cos() * 3.0) as i32 + rng.range(-2, 3), sy + (a.sin() * 3.0) as i32 + rng.range(-2, 3));
            i.line(sx, sy, nx, ny, c(0.98, 0.76, 1.0));
            sx = nx;
            sy = ny;
        }
    }
    // Sparks and shards drifting off it.
    for _ in 0..22 {
        let (x, y) = (cx + rng.range(-34, 35), cy + rng.range(-12, 18));
        i.set(x, y, if rng.chance(50) { c(0.84, 0.36, 0.98) } else { c(1.0, 0.80, 1.0) });
    }
    i
}

/// A cobbled path made of rounded, uneven stones. It wraps left to right, so it tiles.
fn cobblestones() -> Img {
    let (w, h) = (16, 20);
    let mut i = Img::new(w as u32, h as u32);
    let mut rng = Rng(13);
    i.rect(0, 0, w, h, c(0.50, 0.45, 0.46)); // mortar and shadow between the stones
    let mut y = 0;
    let mut course = 0;
    while y < h {
        let ch = if course == 0 { 3 } else { rng.range(4, 6) };
        let mut x = if course % 2 == 0 { 0 } else { -3 };
        while x < w {
            let sw = rng.range(5, 8);
            let t = rng.range(0, 100) as f32 / 100.0;
            let base = if course == 0 { c(0.80, 0.74, 0.68) } else { mix(c(0.58, 0.52, 0.52), c(0.74, 0.68, 0.64), t) };
            // Draw three copies so a stone crossing the edge wraps around.
            for wrap in [-w, 0, w] {
                let (cx, cy) = (x + wrap + sw / 2, y + ch / 2);
                i.ellipse(cx, cy, sw / 2, (ch / 2).max(1), shade(base, 0.82));
                i.ellipse(cx, cy - 1 + (ch == 3) as i32, (sw / 2 - 1).max(1), (ch / 2 - 1).max(1), base);
                i.set(cx - sw / 4, cy - 1, shade(base, 1.2));
            }
            x += sw;
        }
        y += ch;
        course += 1;
    }
    // Tufts of green pushing up between the stones.
    for _ in 0..4 {
        let (x, y) = (rng.range(0, w), rng.range(2, h));
        i.set(x, y, c(0.40, 0.62, 0.30));
    }
    i
}

fn main() {
    let mut args = std::env::args().skip(1);
    let out: PathBuf = args.next().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("assets"));

    // The characters are drawn by `gen_ayz`, from the original sprite, and the buildings (near.png)
    // are traced from the photo by `trace_photo`. This draws the rest of the stage at half
    // resolution; enlarging 2x makes the pixels chunkier.
    let bg = out.join("backgrounds").join("galata");
    sky().upscale(2).save(&bg.join("sky.png"));
    portal().upscale(2).save(&bg.join("portal.png"));
    cobblestones().upscale(2).save(&bg.join("floor.png"));
}
