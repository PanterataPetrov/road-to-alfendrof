//! Turns the lane photo into the stage background (near.png).
//!
//! The photo (`tools/lane_photo.png`) is shrunk to 40% so one picture pixel is one game pixel, reduced to a
//! flat palette and put 54 pixels down from the top so the portal has room above the spire. The buildings
//! continue upwards by reflecting their top rows, and the sky is cut out so the sky and portal layers show
//! through.
//!
//! `cargo run --bin trace_photo -- <assets dir> <preview.png>`

use std::path::Path;

const OUT_W: usize = 400;
const OUT_H: usize = 216;
/// How far down the photo starts, so the spire's tip sits at row 62 under the portal.
const SHIFT: usize = 54;
/// The photo is drawn at this fraction of its size.
const SCALE: f32 = 0.4;
const PALETTE: usize = 56;

type Rgb = [f32; 3];

fn dist2(a: &Rgb, b: &Rgb) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// Deterministic k-means with k-means++ starting points.
fn kmeans(points: &[Rgb], k: usize, iterations: usize) -> Vec<Rgb> {
    let mut state = 12_345u64;
    let mut next = move || {
        state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as f64 / (1u64 << 31) as f64
    };
    let mut centers = vec![points[(next() * points.len() as f64) as usize % points.len()]];
    while centers.len() < k {
        let d2: Vec<f32> = points.iter().map(|p| centers.iter().map(|c| dist2(p, c)).fold(f32::MAX, f32::min)).collect();
        let total: f32 = d2.iter().sum();
        let mut target = next() as f32 * total;
        let mut pick = 0;
        for (i, &d) in d2.iter().enumerate() {
            target -= d;
            if target <= 0.0 {
                pick = i;
                break;
            }
        }
        centers.push(points[pick]);
    }
    for _ in 0..iterations {
        let mut sums = vec![[0.0f32; 3]; k];
        let mut counts = vec![0u32; k];
        for p in points {
            let best = nearest(&centers, p);
            for ch in 0..3 {
                sums[best][ch] += p[ch];
            }
            counts[best] += 1;
        }
        for i in 0..k {
            if counts[i] > 0 {
                for ch in 0..3 {
                    centers[i][ch] = sums[i][ch] / counts[i] as f32;
                }
            }
        }
    }
    centers
}

fn nearest(centers: &[Rgb], p: &Rgb) -> usize {
    let mut best = 0;
    let mut best_d = f32::MAX;
    for (i, c) in centers.iter().enumerate() {
        let d = dist2(p, c);
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

fn main() {
    let mut args = std::env::args().skip(1);
    let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
    let preview = args.next();

    let photo = image::open(concat!(env!("CARGO_MANIFEST_DIR"), "/tools/lane_photo.png")).unwrap().to_rgb8();
    // average the photo down to picture size
    let rows = OUT_H - SHIFT;
    let step = 1.0 / SCALE;
    let mut small = vec![[0.0f32; 3]; OUT_W * rows];
    for y in 0..rows {
        for x in 0..OUT_W {
            let (x0, x1) = ((x as f32 * step) as u32, (((x + 1) as f32 * step) as u32).min(photo.width()));
            let (y0, y1) = ((y as f32 * step) as u32, (((y + 1) as f32 * step) as u32).min(photo.height()));
            let mut sum = [0.0f32; 3];
            let mut n = 0.0;
            for py in y0..y1.max(y0 + 1) {
                for px in x0..x1.max(x0 + 1) {
                    let p = photo.get_pixel(px.min(photo.width() - 1), py.min(photo.height() - 1));
                    for c in 0..3 {
                        sum[c] += p[c] as f32;
                    }
                    n += 1.0;
                }
            }
            small[y * OUT_W + x] = [sum[0] / n, sum[1] / n, sum[2] / n];
        }
    }
    // full picture: the photo's top rows reflected above it
    let mut full = vec![[0.0f32; 3]; OUT_W * OUT_H];
    for r in 0..OUT_H {
        let source = if r >= SHIFT { r - SHIFT } else { (SHIFT - 1 - r).min(rows - 1) };
        full[r * OUT_W..(r + 1) * OUT_W].copy_from_slice(&small[source * OUT_W..(source + 1) * OUT_W]);
    }

    let sample: Vec<Rgb> = full.iter().step_by(5).cloned().collect();
    let palette = kmeans(&sample, PALETTE, 14);
    let colours: Vec<usize> = full.iter().map(|p| nearest(&palette, p)).collect();

    // the sky: bright blue, lavender or white (not warm walls), connected to the top edge
    // the tower (needle, cone, shaft at column 186) is never sky, even where its pale roof is as bright as a cloud
    let tower = |x: usize, y: usize| {
        let off = (x as i32 - 186).abs();
        (60..84).contains(&y) && off <= 2 || (84..=142).contains(&y) && off <= (y as i32 - 84) * 27 / 58 + 1 || (143..=158).contains(&y) && off <= 30
    };
    let is_sky = |i: usize| {
        if tower(i % OUT_W, i / OUT_W) {
            return false;
        }
        let c = palette[colours[i]];
        let luma = 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
        c[2] >= c[0] - 4.0 && luma > 150.0
    };
    let mut sky = vec![false; OUT_W * OUT_H];
    let mut stack: Vec<(usize, usize)> = (0..OUT_W).filter(|&x| is_sky(x)).map(|x| (x, 0)).collect();
    while let Some((x, y)) = stack.pop() {
        let i = y * OUT_W + x;
        if sky[i] || !is_sky(i) {
            continue;
        }
        sky[i] = true;
        if x > 0 {
            stack.push((x - 1, y));
        }
        if x + 1 < OUT_W {
            stack.push((x + 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if y + 1 < OUT_H {
            stack.push((x, y + 1));
        }
    }
    // one more pixel of sky around the edge so no pale halo is left on the buildings
    let edge = sky.clone();
    for y in 1..OUT_H - 1 {
        for x in 1..OUT_W - 1 {
            let i = y * OUT_W + x;
            if !edge[i] && !tower(x, y) && (edge[i - 1] || edge[i + 1] || edge[i - OUT_W] || edge[i + OUT_W]) {
                let c = palette[colours[i]];
                if 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2] > 120.0 && c[2] >= c[0] - 12.0 {
                    sky[i] = true;
                }
            }
        }
    }

    let mut near = image::RgbaImage::new(OUT_W as u32, OUT_H as u32);
    let mut shown = image::RgbaImage::new((OUT_W * 2) as u32, (OUT_H * 2) as u32);
    for y in 0..OUT_H {
        for x in 0..OUT_W {
            let i = y * OUT_W + x;
            let c = palette[colours[i]];
            let rgb = [c[0].round() as u8, c[1].round() as u8, c[2].round() as u8];
            for dy in 0..2u32 {
                for dx in 0..2u32 {
                    shown.put_pixel(x as u32 * 2 + dx, y as u32 * 2 + dy, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
                }
            }
            if !sky[i] {
                near.put_pixel(x as u32, y as u32, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
            }
        }
    }
    let folder = assets.join("backgrounds").join("galata");
    std::fs::create_dir_all(&folder).unwrap();
    near.save(folder.join("near.png")).unwrap();
    if let Some(path) = preview {
        shown.save(Path::new(&path)).unwrap();
    }
    println!("wrote {}", folder.join("near.png").display());
}
