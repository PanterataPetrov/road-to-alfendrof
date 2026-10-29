//! Traces the lane photo into the stage's pixel-art background.
//!
//! `tools/lane_photo.png` is the user's photo of Galata Tower over a street. Only its upper part
//! is used (no people or vehicles are in it). It is reduced to a small flat palette and then to
//! chunky half-resolution cells (each becomes 2 x 2 game pixels). The buildings are continued
//! upward by mirroring, to leave sky above the tower for the portal; the sky is cut out so it can
//! scroll behind; and the sides are extended by mirroring so the stage is wider than the photo.
//!
//! `cargo run --bin trace_photo -- <assets dir> <preview.png>`

use std::path::Path;

/// The traced picture's size in half-resolution cells (640 x 216 game pixels).
const OUT_W: usize = 200;
const OUT_H: usize = 108;
/// The photo's own width in cells; the rest of the stage is mirrored sides.
const PHOTO_W: usize = 200;
/// Rows above the photo's top edge, filled by repeating a short strip of its top rows.
const TOP: usize = 25;
const PHOTO_H: usize = OUT_H - TOP;
const PALETTE: usize = 32;
/// How many of the photo's top rows are repeated to continue the buildings upward. Kept short so
/// the tower's spire (a few rows down) is not repeated into the sky.
const TOP_TILE: usize = 3;

type Rgb = [f32; 3];

fn dist2(a: &Rgb, b: &Rgb) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// k-means with k-means++ starting points, deterministic.
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
    let (pw, ph) = (photo.width() as usize, photo.height() as usize);
    // Square cells: the photo's width spread over PHOTO_W cells, and only as many rows as fit.
    // Cells are 5 photo pixels wide and 6.8 tall: the photo is squashed a little so that its whole
    // height (down to just above the tiny people at the very bottom) fits under the sky.
    let cell = pw as f32 / PHOTO_W as f32;
    let cell_h = 6.8f32;
    let used_rows = ((PHOTO_H as f32 * cell_h) as usize).min(ph);

    // A flat palette from a sample of the part of the photo that is used.
    let sample: Vec<Rgb> = photo
        .enumerate_pixels()
        .filter(|(_, y, _)| (*y as usize) < used_rows)
        .step_by(7)
        .map(|(_, _, p)| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    let palette = kmeans(&sample, PALETTE, 12);

    // Every used photo pixel becomes its nearest palette colour.
    let mut indexed = vec![0u8; pw * used_rows];
    for y in 0..used_rows {
        for x in 0..pw {
            let p = photo.get_pixel(x as u32, y as u32);
            indexed[y * pw + x] = nearest(&palette, &[p[0] as f32, p[1] as f32, p[2] as f32]) as u8;
        }
    }

    // Each cell takes the palette colour most common in its patch of the photo, which keeps edges
    // crisp (averaging would blur them).
    let mut photo_cells = vec![0u8; PHOTO_W * PHOTO_H];
    for cy in 0..PHOTO_H {
        for cx in 0..PHOTO_W {
            let mut counts = [0u32; PALETTE];
            for y in (cy as f32 * cell_h) as usize..(((cy + 1) as f32 * cell_h) as usize).min(used_rows) {
                for x in (cx as f32 * cell) as usize..(((cx + 1) as f32 * cell) as usize).min(pw) {
                    counts[indexed[y * pw + x] as usize] += 1;
                }
            }
            photo_cells[cy * PHOTO_W + cx] = counts.iter().enumerate().max_by_key(|&(_, &n)| n).map(|(i, _)| i as u8).unwrap();
        }
    }
    // Above the photo's top edge, continue the buildings by repeating a short strip of its top
    // rows (sky stays sky).
    let mut cells = vec![0u8; PHOTO_W * OUT_H];
    for r in 0..OUT_H {
        let source_row = if r >= TOP { r - TOP } else { (TOP - 1 - r) % TOP_TILE };
        cells[r * PHOTO_W..(r + 1) * PHOTO_W].copy_from_slice(&photo_cells[source_row * PHOTO_W..(source_row + 1) * PHOTO_W]);
    }

    // Clean up speckle: a cell changes to the colour most of its neighbours share, if that is a
    // clear majority.
    let mut cleaned = cells.clone();
    for y in 1..OUT_H - 1 {
        for x in 1..PHOTO_W - 1 {
            let mut counts = [0u8; PALETTE];
            for dy in 0..3 {
                for dx in 0..3 {
                    counts[cells[(y + dy - 1) * PHOTO_W + x + dx - 1] as usize] += 1;
                }
            }
            let (best, n) = counts.iter().enumerate().max_by_key(|&(_, &n)| n).map(|(i, &n)| (i, n)).unwrap();
            if n >= 6 {
                cleaned[y * PHOTO_W + x] = best as u8;
            }
        }
    }
    let cells = cleaned;

    // The sky: bright palette colours that are blue, lavender or white (not warm like the walls),
    // connected to the top edge. The tower's darker grey-blue cone is not bright enough.
    let is_sky_colour = |i: usize| {
        let c = palette[i];
        let luma = 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
        c[2] >= c[0] - 4.0 && luma > 160.0
    };
    let mut sky = vec![false; PHOTO_W * OUT_H];
    let mut stack: Vec<(usize, usize)> = (0..PHOTO_W).filter(|&x| is_sky_colour(cells[x] as usize)).map(|x| (x, 0)).collect();
    while let Some((x, y)) = stack.pop() {
        if sky[y * PHOTO_W + x] || !is_sky_colour(cells[y * PHOTO_W + x] as usize) {
            continue;
        }
        sky[y * PHOTO_W + x] = true;
        if x > 0 {
            stack.push((x - 1, y));
        }
        if x + 1 < PHOTO_W {
            stack.push((x + 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if y + 1 < OUT_H {
            stack.push((x, y + 1));
        }
    }

    // The photo covers the whole stage width, so every column comes straight from it.
    let margin = (OUT_W - PHOTO_W) / 2;
    let source = |c: usize| -> usize { c - margin };
    let mut near = image::RgbaImage::new((OUT_W * 2) as u32, (OUT_H * 2) as u32);
    let mut shown = image::RgbaImage::new((OUT_W * 4) as u32, (OUT_H * 4) as u32);
    for y in 0..OUT_H {
        for c in 0..OUT_W {
            let s = y * PHOTO_W + source(c);
            let col = palette[cells[s] as usize];
            let rgb = [col[0].round() as u8, col[1].round() as u8, col[2].round() as u8];
            for dy in 0..4u32 {
                for dx in 0..4u32 {
                    shown.put_pixel(c as u32 * 4 + dx, y as u32 * 4 + dy, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
                }
            }
            if sky[s] {
                continue;
            }
            for dy in 0..2u32 {
                for dx in 0..2u32 {
                    near.put_pixel(c as u32 * 2 + dx, y as u32 * 2 + dy, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
                }
            }
        }
    }
    let folder = assets.join("backgrounds").join("galata");
    std::fs::create_dir_all(&folder).unwrap();
    near.save(folder.join("near.png")).unwrap();
    if let Some(path) = preview {
        shown.save(Path::new(&path)).unwrap();
    }

    // Where the spire's tip is, for the portal: the highest non-sky cell near the tower's column.
    let tower_col = ((465.0 / cell) as usize).min(PHOTO_W - 1);
    let tip = (0..OUT_H).find(|&y| (tower_col.saturating_sub(3)..=tower_col + 3).any(|x| !sky[y * PHOTO_W + x]));
    println!(
        "palette colours: {PALETTE}; tower at canvas column {}, spire tip at row {:?}; wrote {}",
        margin + tower_col,
        tip,
        folder.join("near.png").display()
    );
}
