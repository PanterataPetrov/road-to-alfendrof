//! Builds Ayz's default pose from the original sprite, pixel for pixel.
//!
//! `ref_sprite.txt` is the original pixel art, read exactly from its image: one letter per
//! pixel, `.` for empty. The original is reproduced first, then Ayz is made by editing those same
//! pixels (recolouring, widening, redrawing the hair and adding his features), so the art style is
//! the original's own, not an imitation of it.
//!
//! `cargo run --bin gen_ayz -- <out.png>` writes the original and the new Ayz side by side.

use std::path::Path;

const REF: &str = include_str!("ref_sprite.txt");

/// The original palette (letters A-R) and the colours used for Ayz (lowercase).
fn color(ch: char) -> Option<[u8; 3]> {
    Some(match ch {
        '.' => return None,
        // Original sprite.
        'A' => [76, 85, 144],
        'B' => [145, 49, 50],
        'C' => [179, 61, 61],
        'D' => [236, 214, 200],
        'E' => [26, 19, 13],
        'F' => [255, 229, 212],
        'G' => [40, 45, 77],
        'H' => [51, 56, 96],
        'I' => [59, 41, 27],
        'J' => [234, 234, 234],
        'K' => [232, 195, 169],
        'L' => [162, 162, 162],
        'M' => [197, 197, 197],
        'N' => [109, 117, 179],
        'O' => [191, 172, 158],
        'P' => [253, 253, 253],
        'Q' => [119, 119, 119],
        'R' => [67, 60, 54],
        // Ayz: the white shirt, in the original's two tones (plus a deeper fold tone).
        'w' => [240, 241, 250],
        'v' => [186, 192, 216],
        'u' => [150, 156, 188],
        // Orange trousers, with the original jeans' base, shadow, deep shadow and highlight.
        'o' => [246, 142, 32],
        'q' => [196, 98, 26],
        'p' => [140, 64, 22],
        'r' => [255, 190, 104],
        // The scar is drawn with the original's own dark shirt red (B) so it matches its palette.
        // His skin: the original's four skin tones, warmed up.
        'a' => [238, 196, 158],
        'b' => [222, 174, 134],
        'd' => [204, 152, 112],
        'e' => [166, 118, 90],
        // Blue-black hair with lighter clumps, so it stays distinct from the pure-black glasses
        // frames (g) and moustache; the scar is drawn in the original's dark shirt red.
        'k' => [28, 30, 50],
        'l' => [74, 80, 118],
        'g' => [8, 8, 10],
        // The summoned crescent and the shooting star: gold in the same three flat tones.
        'Y' => [250, 205, 80],
        'y' => [205, 150, 50],
        'z' => [255, 240, 170],
        'W' => [255, 255, 255],
        // The scar: a slightly redder, darker version of his own skin, so it does not stick out.
        'x' => [196, 132, 108],
        // the Gunslinger: brown hair, an olive corduroy blazer, mustard trousers, black boots.
        'c' => [92, 60, 40],
        'f' => [130, 90, 60],
        'i' => [58, 36, 24],
        'S' => [84, 90, 54],
        's' => [58, 64, 38],
        'T' => [112, 118, 72],
        'm' => [190, 138, 42],
        'n' => [146, 98, 32],
        'j' => [104, 70, 26],
        'h' => [226, 178, 86],
        'X' => [34, 34, 38],
        'Z' => [70, 70, 78],
        'V' => [14, 14, 16],
        // The Gunslinger's cowboy hat (leather brown, shadow, light, band) and his red tie.
        '1' => [150, 100, 58],
        '2' => [104, 66, 38],
        '3' => [196, 148, 96],
        '4' => [46, 34, 30],
        // The Gunslinger's textured hair: flat, light, shadow, deep.
        '5' => [119, 95, 59],
        '6' => [158, 128, 84],
        '7' => [88, 68, 42],
        '8' => [58, 44, 28],
        // The Professor's fish: blue body, pale belly, dark back, orange tail and fins.
        '#' => [58, 138, 214],
        '%' => [150, 206, 248],
        '&' => [28, 78, 150],
        '@' => [250, 158, 56],
        // Cowboy From Hell's grey jeans.
        '/' => [112, 112, 118],
        '(' => [62, 62, 68],
        ')' => [88, 88, 94],
        '|' => [158, 158, 164],
        // Cowboy From Hell's skin: a little darker and a little paler than the others'.
        '<' => [214, 180, 149],
        '>' => [199, 160, 128],
        '?' => [183, 137, 108],
        ';' => [148, 109, 87],
        ':' => [236, 212, 196],
        // Cowboy From Hell: a dark brown ribbed tank top and brown leather boots.
        '$' => [84, 66, 60],
        '!' => [56, 42, 40],
        '^' => [114, 94, 86],
        '*' => [150, 98, 56],
        '+' => [98, 62, 34],
        '=' => [204, 150, 94],
        // Anaran: dark crimson hair, copper armour, a deeper and a brighter robe red.
        ']' => [48, 8, 22],
        '{' => [112, 22, 46],
        '}' => [160, 44, 70],
        '[' => [70, 12, 32],
        ',' => [186, 110, 62],
        '\'' => [138, 78, 44],
        '"' => [234, 166, 104],
        '-' => [214, 84, 76],
        '_' => [112, 28, 36],
        // Anaran's copper boots (hand-edited): gold, bronze and a bright highlight.
        '~' => [235, 168, 83],
        '`' => [187, 133, 64],
        '\\' => [255, 195, 119],
        // The scroll's parchment, and the goblins: skin, shade, bright, tunic.
        't' => [242, 228, 196],
        'U' => [206, 186, 146],
        'à' => [96, 170, 70],
        'è' => [58, 120, 50],
        'ì' => [150, 210, 110],
        'ò' => [120, 80, 50],
        // Junkhead's whip leather (the coil on his belt): dark, mid, light.
        'ä' => [86, 52, 30],
        'ö' => [128, 80, 46],
        'ü' => [182, 128, 78],
        // Graypersona's gothic dress: black, its fold shadow and highlight, grey and silver lace.
        'ê' => [22, 20, 28],
        'ï' => [10, 10, 14],
        'î' => [52, 50, 66],
        'ô' => [96, 94, 112],
        'û' => [186, 186, 200],
        // The fedora's purple magic.
        '9' => [176, 70, 226],
        '0' => [112, 34, 168],
        _ => panic!("no colour for {ch:?}"),
    })
}

#[derive(Clone)]
struct Grid {
    w: usize,
    h: usize,
    cells: Vec<char>,
}

impl Grid {
    fn parse(text: &str) -> Grid {
        let rows: Vec<Vec<char>> = text.lines().map(|l| l.chars().collect()).collect();
        let w = rows[0].len();
        assert!(rows.iter().all(|r| r.len() == w), "ragged rows");
        Grid { w, h: rows.len(), cells: rows.into_iter().flatten().collect() }
    }

    fn from_rows(rows: Vec<Vec<char>>) -> Grid {
        let w = rows[0].len();
        assert!(rows.iter().all(|r| r.len() == w), "ragged rows");
        Grid { w, h: rows.len(), cells: rows.into_iter().flatten().collect() }
    }

    fn row(&self, y: usize) -> Vec<char> {
        self.cells[y * self.w..(y + 1) * self.w].to_vec()
    }

    fn get(&self, x: i32, y: i32) -> char {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            '.'
        } else {
            self.cells[y as usize * self.w + x as usize]
        }
    }

    fn set(&mut self, x: i32, y: i32, ch: char) {
        if x >= 0 && y >= 0 && x < self.w as i32 && y < self.h as i32 {
            self.cells[y as usize * self.w + x as usize] = ch;
        }
    }

    /// Replace letters according to `pairs`, only inside the rows `y0..=y1`.
    fn recolor(&mut self, y0: usize, y1: usize, pairs: &[(char, char)]) {
        for y in y0..=y1.min(self.h - 1) {
            for x in 0..self.w {
                let ch = self.cells[y * self.w + x];
                if let Some(&(_, to)) = pairs.iter().find(|(from, _)| *from == ch) {
                    self.cells[y * self.w + x] = to;
                }
            }
        }
    }

    /// Draw a line of `ch`, optionally only over cells currently holding `only`.
    fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, ch: char, only: Option<char>) {
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
        for k in 0..=steps {
            let x = x0 + ((x1 - x0) as f32 * k as f32 / steps as f32).round() as i32;
            let y = y0 + ((y1 - y0) as f32 * k as f32 / steps as f32).round() as i32;
            if only.map_or(true, |o| self.get(x, y) == o) {
                self.set(x, y, ch);
            }
        }
    }

    fn render(&self, scale: u32) -> image::RgbaImage {
        let mut img = image::RgbaImage::from_pixel(self.w as u32 * scale, self.h as u32 * scale, image::Rgba([255, 255, 255, 255]));
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(c) = color(self.cells[y * self.w + x]) {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            img.put_pixel(x as u32 * scale + dx, y as u32 * scale + dy, image::Rgba([c[0], c[1], c[2], 255]));
                        }
                    }
                }
            }
        }
        img
    }
}

/// The widened body: the original's rows are copied with extra columns inserted, so the torso and
/// legs get wider using the original's own pixels.
fn widen(r: &Grid) -> Grid {
    let pad = |n: usize| std::iter::repeat('.').take(n);
    let mut rows = Vec::new();
    for y in 0..r.h {
        let s = r.row(y);
        let mut row: Vec<char> = Vec::new();
        if y <= 18 {
            // Head: shifted over so it sits on the middle of the wider body.
            row.extend(pad(7));
            row.extend(&s);
            row.extend(pad(4));
        } else if y <= 22 {
            // Shoulders and neck: widen out at the shoulders so the neck keeps its size.
            row.extend(pad(3));
            row.extend(&s[..=5]);
            row.extend(std::iter::repeat(s[5]).take(4));
            row.extend(&s[6..=23]);
            row.extend(std::iter::repeat(s[23]).take(4));
            row.extend(&s[24..]);
        } else if y <= 37 {
            // Torso: widen down the middle.
            row.extend(pad(3));
            row.extend(&s[..=14]);
            row.extend(std::iter::repeat(s[14]).take(8));
            row.extend(&s[15..]);
        } else {
            // Legs: each one gets thicker.
            row.extend(pad(3));
            row.extend(&s[..=6]);
            row.extend(std::iter::repeat(s[6]).take(4));
            row.extend(&s[7..=22]);
            row.extend(std::iter::repeat(s[22]).take(4));
            row.extend(&s[23..]);
        }
        rows.push(row);
    }
    Grid::from_rows(rows)
}

fn ayz_default() -> Grid {
    let mut g = widen(&Grid::parse(REF));

    // Give the face one more row of cheek under the glasses, so the moustache has skin above it
    // instead of merging into the glasses frames. Everything below moves down a row.
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.insert(18, rows[17].clone());
    g = Grid::from_rows(rows);

    // The original's left arm runs off the edge of its image. Finish it: a rounded outer edge
    // with the same shading.
    for (y, from) in [(26, 2), (27, 1), (28, 0), (29, 0), (30, 0), (31, 0), (32, 0), (33, 0), (34, 1), (35, 2)] {
        for x in from..3 {
            let ch = if x == from && from == 0 { 'K' } else { 'D' };
            if g.get(x, y) == '.' {
                g.set(x, y, ch);
            }
        }
    }

    // Recolour: skin warmed up, shirt white, jeans orange, hair black. The hands in the original
    // are pale grey; Ayz has bare hands.
    g.recolor(0, 99, &[('F', 'a'), ('D', 'b'), ('K', 'd'), ('O', 'e')]);
    g.recolor(20, 99, &[('C', 'w'), ('B', 'v'), ('A', 'o'), ('G', 'p'), ('H', 'q'), ('N', 'r')]);
    g.recolor(0, 99, &[('E', 'k'), ('I', 'l')]);
    g.recolor(25, 46, &[('J', 'a'), ('M', 'b'), ('P', 'a')]);

    // The original's glints in the glasses were white (P); the recolour above is for rows 24 and
    // down only, so they are untouched.

    // Head (the head is shifted 7 columns right of the original)
    let hx = |c: i32| c + 7;
    // A bald forehead: clear the fringe that hangs over it, leaving skin, the way the original
    // draws skin beside the hair.
    for y in 8..=12 {
        for c in 12..=19 {
            let ch = match c {
                12 => 'd',
                19 => 'b',
                _ => 'a',
            };
            g.set(hx(c), y, ch);
        }
    }
    // The crescent scar, opening to the right.
    for (c, y) in [(17, 9), (16, 10), (16, 11), (17, 12)] {
        g.set(hx(c), y, 'x');
    }
    // A completely bald top of the head: the crown's hair becomes a rounded skin dome, shaded
    // like the rest of the skin (a shadow edge on the left, the original's darker skin on the
    // right). The hair stays only down the sides.
    for (y, from, to) in [(5, 20, 24), (6, 18, 25), (7, 17, 26)] {
        for x in from..=to {
            let ch = if y == 5 {
                'b'
            } else if x == from {
                'd'
            } else if x == to {
                'b'
            } else {
                'a'
            };
            g.set(x, y, ch);
        }
    }
    // Remove the stray crown hairs left floating beside the dome.
    for (x, y) in [(17, 0), (19, 0), (28, 1), (26, 0), (27, 1)] {
        if matches!(g.get(x, y), 'k' | 'l') {
            g.set(x, y, '.');
        }
    }
    // The glasses frames in pure black, so they read against the blue-black hair around them
    // (in the original they are the same dark as the hair, but the hair there is brown). The
    // frame cells, in the original's coordinates: the top and bottom bars of each lens and its
    // sides.
    for (c, y) in [
        (13, 13), (14, 13), (12, 14), (12, 15), (15, 14), (15, 15), (13, 16), (14, 16),
        (17, 13), (18, 13), (16, 14), (19, 14), (19, 15), (17, 16), (18, 16),
    ] {
        g.set(hx(c), y, 'g');
    }
    // A lighter clump of hair right beside the lenses, as the original has.
    for (c, y) in [(11, 14), (11, 15), (20, 14), (20, 15)] {
        g.set(hx(c), y, 'l');
    }
    // A scar that shows up: a dark red crescent with a lighter edge, opening to the right.
    for (c, y) in [(17, 9), (16, 10), (16, 11), (17, 12)] {
        g.set(hx(c), y, 'x');
    }
    // A moustache with drooping ends, with the new row of cheek between it and the glasses.
    for c in 13..=19 {
        g.set(hx(c), 18, 'g');
    }
    g.set(hx(13), 19, 'g');
    g.set(hx(19), 19, 'g');
    for c in 15..=17 {
        g.set(hx(c), 19, 'd');
    }

    // Long hair: one continuous mass down each side. It starts at the width the head's own hair
    // already has at the jaw, swells as it falls over the shoulder, then tapers smoothly to a
    // point, so the edges are straight slopes and not a staircase. A few strand lines run
    // down its length.
    let (first, swell, last) = (17, 23, 37);
    // `centre` drifts a little with each row; `max_w` is the width at the swell.
    let flow = |g: &mut Grid, centre: f32, drift: f32, start_w: f32, max_w: f32| {
        let mut bottom = (0, first);
        for y in first..=last {
            let w = if y <= swell {
                start_w + (max_w - start_w) * (y - first) as f32 / (swell - first) as f32
            } else {
                max_w * (last - y) as f32 / (last - swell) as f32
            };
            if w < 1.0 {
                break;
            }
            let cl = centre + drift * (y - first) as f32;
            let width = w.round() as i32;
            let xl = (cl - w / 2.0).round() as i32;
            let xr = xl + width - 1;
            for x in xl..=xr {
                // Strand lines: dashed lighter streaks that follow the slope of the mass.
                let streak = [1, 3].iter().any(|&s| x == xl + s && x < xr && (y * 2 + s * 3) % 7 < 3);
                g.set(x, y, if streak { 'l' } else { 'k' });
            }
            bottom = ((xl + xr) / 2, y);
        }
        // A single pixel point at the very end.
        g.set(bottom.0, bottom.1 + 1, 'k');
    };
    flow(&mut g, 15.0, -0.08, 3.0, 7.0); // left, the heavier side, as in the original
    flow(&mut g, 27.2, 0.04, 2.0, 5.0); // right

    // The belly
    belly(&mut g);

    // Finish the shoes, which the original's image cuts off
    let last = g.h - 1;
    let shoe_row = g.row(last);
    let mut extra: Vec<Vec<char>> = Vec::new();
    extra.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'L' } else { c }).collect());
    extra.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'Q' } else { '.' }).collect());
    extra.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'J' } else { '.' }).collect());
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.extend(extra);
    g = Grid::from_rows(rows);

    // Crop the empty rows above the hair.
    let top = (0..g.h).find(|&y| g.row(y).iter().any(|&c| c != '.')).unwrap_or(0);
    let rows: Vec<Vec<char>> = (top..g.h).map(|y| g.row(y)).collect();
    Grid::from_rows(rows)
}

/// The huge belly, drawn over the widened torso and the top of the trousers: white shirt with the
/// original's two-tone shading (shadow along the lower right), creases fanning out from the
/// navel, a stretched hem, and bare belly hanging out below it.
fn belly(g: &mut Grid) {
    // A round belly, but only a modest bulge past the chest (it was too big).
    let (cy, cx, rx, ry) = (38.0f32, 21.0f32, 14.5f32, 9.0f32);
    for y in 0..g.h as i32 {
        for x in 0..g.w as i32 {
            let (dx, dy) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
            let d = dx * dx + dy * dy;
            if d > 1.0 {
                continue;
            }
            let cur = g.get(x, y);
            // Leave the arms and the hanging hair in front of the belly.
            if "abdeklg".contains(cur) && y <= 37 {
                continue;
            }
            let lower = dy > 0.45;
            let rim = (dx + dy) > 0.2;
            let ch = if lower {
                if d > 0.93 {
                    'd'
                } else if d > 0.80 && rim {
                    'b'
                } else {
                    'a'
                }
            } else if d > 0.93 && rim {
                'u'
            } else if d > 0.78 && rim {
                'v'
            } else {
                'w'
            };
            g.set(x, y, ch);
        }
    }
    // The stretched hem where the shirt ends.
    let hem_y = (cy + 0.45 * ry).round() as i32 - 1;
    for x in 0..g.w as i32 {
        if g.get(x, hem_y) == 'w' {
            g.set(x, hem_y, 'v');
        }
    }
    // Creases fanning out from the navel.
    let (nx, ny) = (cx as i32 + 1, cy as i32 + 1);
    for deg in [150.0f32, 185.0, 215.0, 325.0, 355.0, 30.0] {
        let a = deg.to_radians();
        let (x0, y0) = (nx as f32 + 4.0 * a.cos(), ny as f32 + 4.0 * a.sin() * 0.8);
        let (x1, y1) = (nx as f32 + 12.0 * a.cos(), ny as f32 + 12.0 * a.sin() * 0.8);
        g.line(x0.round() as i32, y0.round() as i32, x1.round() as i32, y1.round() as i32, 'v', Some('w'));
    }
    // The navel.
    g.set(nx, ny, 'd');
    g.set(nx + 1, ny, 'd');
    g.set(nx, ny + 1, 'e');
}

// The poses, all drawn from the default pose
//
// The default pose is split into a body without arms and the two original arms. Every upper-body
// drawing puts the body on a canvas and adds arms: either the original arm (where a pose keeps
// the default arm) or a new one drawn with the same shading rules as the original (a flat base
// tone, a darker patch along the shadow edge, lighter forearms). The lower-body drawings keep the
// trousers and redraw the legs and shoes.

/// Where the default pose sits on a drawing's canvas. The canvases leave room on the left for a
/// swinging back arm, on the right for outstretched arms, and above for raised hands.
const OX: i32 = 12;
const OY: i32 = 8;
/// Rows 0..=41 of the default pose are the upper body; the lower body starts at row 36 and so
/// overlaps the upper body, whose belly hangs over the trousers.
const UPPER_LAST_ROW: i32 = 41;
const LOWER_FIRST_ROW: i32 = 36;

const HAND_MASK: [&str; 6] = [".####.", "######", "######", "######", ".####.", "..##.."];

use std::collections::HashSet;
use t2f::anim::{Frame, CENTRE_X, FRAME_W, FULL_H, UPPER_H};

impl Grid {
    fn blank(w: i32, h: i32) -> Grid {
        Grid { w: w as usize, h: h as usize, cells: vec!['.'; (w * h) as usize] }
    }

    /// The image with a transparent background, one pixel per cell.
    fn to_png(&self) -> image::RgbaImage {
        let mut img = image::RgbaImage::new(self.w as u32, self.h as u32);
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(c) = color(self.cells[y * self.w + x]) {
                    img.put_pixel(x as u32, y as u32, image::Rgba([c[0], c[1], c[2], 255]));
                }
            }
        }
        img
    }
}

/// The cells covered by a straight-sided limb along a polyline.
fn tube(points: &[(f32, f32)], width: f32) -> HashSet<(i32, i32)> {
    let mut cells = HashSet::new();
    for pair in points.windows(2) {
        let (p0, p1) = (pair[0], pair[1]);
        let (dx, dy) = (p1.0 - p0.0, p1.1 - p0.1);
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        let (ux, uy) = (dx / len, dy / len);
        let (min_x, max_x) = ((p0.0.min(p1.0) - width) as i32, (p0.0.max(p1.0) + width) as i32);
        let (min_y, max_y) = ((p0.1.min(p1.1) - width) as i32, (p0.1.max(p1.1) + width) as i32);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let (rx, ry) = (x as f32 + 0.5 - p0.0, y as f32 + 0.5 - p0.1);
                let along = rx * ux + ry * uy;
                let across = (-rx * uy + ry * ux).abs();
                if along >= -1.0 && along <= len + 1.0 && across <= width / 2.0 {
                    cells.insert((x, y));
                }
            }
        }
    }
    cells
}

/// Draw an arm in the original's shading: the upper arm in the mid skin tone with a darker edge
/// on its left, the forearm and hand lighter with a darker edge below and on the right.
fn draw_arm(canvas: &mut Grid, shoulder: (f32, f32), elbow: (f32, f32), hand: (f32, f32)) {
    let upper = tube(&[shoulder, elbow], 7.0);
    let fore = tube(&[elbow, hand], 6.5);
    let mut fist = HashSet::new();
    for (y, row) in HAND_MASK.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                fist.insert((hand.0.round() as i32 - 3 + x as i32, hand.1.round() as i32 - 3 + y as i32));
            }
        }
    }
    let all: HashSet<_> = upper.union(&fore).chain(fist.iter()).cloned().collect();
    for &(x, y) in &all {
        let open_left = !all.contains(&(x - 1, y));
        let open_right = !all.contains(&(x + 1, y));
        let open_below = !all.contains(&(x, y + 1));
        let ch = if fist.contains(&(x, y)) || fore.contains(&(x, y)) {
            if open_right || open_below { 'b' } else { 'a' }
        } else if open_left || open_below {
            'd' // the shadow edge: the left of the upper arm and its underside
        } else {
            'b'
        };
        canvas.set(x + OX, y + OY, ch);
    }
}

/// Draw a leg of the trousers in the original's jeans shading, recoloured orange: a flat base,
/// a darker edge down the right and along the bottom, a darker cuff, and a few light pixels where
/// the cloth catches the light at the knee.
fn draw_leg(canvas: &mut Grid, oy: i32, hip: (f32, f32), knee: (f32, f32), ankle: (f32, f32)) {
    let leg = tube(&[hip, knee, ankle], 12.0);
    for &(x, y) in &leg {
        let open_right = !leg.contains(&(x + 1, y));
        let open_below = !leg.contains(&(x, y + 1));
        let ch = if y >= ankle.1.round() as i32 - 1 {
            'q' // the cuff
        } else if open_below {
            'p'
        } else if open_right {
            'q'
        } else {
            'o'
        };
        canvas.set(x + OX, y + oy, ch);
    }
    let (kx, ky) = (knee.0.round() as i32, knee.1.round() as i32);
    for (dx, dy) in [(-3, -1), (-2, -1), (-3, 0), (-2, 1)] {
        if canvas.get(kx + dx + OX, ky + dy + oy) == 'o' {
            canvas.set(kx + dx + OX, ky + dy + oy, 'r');
        }
    }
}

/// The cells of the shoe in a rectangle of the default pose, as (dx, dy, letter) from its corner.
fn grab_shoe(g: &Grid, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32, char)> {
    let mut cells = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let ch = g.get(x, y);
            if "JLMQR".contains(ch) {
                cells.push((x - x0, y - y0, ch));
            }
        }
    }
    cells
}

fn stamp_shoe(canvas: &mut Grid, oy: i32, shoe: &[(i32, i32, char)], x: i32, y: i32) {
    for &(dx, dy, ch) in shoe {
        canvas.set(x + dx + OX, y + dy + oy, ch);
    }
}

/// The body without arms, and the original's arms as lists of cells.
struct Parts {
    body: Grid,
    left_arm: Vec<(i32, i32, char)>,
    right_arm: Vec<(i32, i32, char)>,
    left_shoe: Vec<(i32, i32, char)>,
    right_shoe: Vec<(i32, i32, char)>,
    /// The Gunslinger's hat as cells (it is drawn as a separate overlay); empty for the others.
    hat: Vec<(i32, i32, char)>,
}

fn split(default: &Grid) -> Parts {
    let mut body = default.clone();
    let (mut left_arm, mut right_arm) = (Vec::new(), Vec::new());
    for y in 20..=37i32 {
        for x in 0..default.w as i32 {
            let ch = default.get(x, y);
            if !"abde".contains(ch) {
                continue;
            }
            if x <= 11 {
                left_arm.push((x, y, ch));
                body.set(x, y, if x >= 9 { 'w' } else { '.' });
            } else if x >= 32 {
                right_arm.push((x, y, ch));
                body.set(x, y, if x <= 33 { 'w' } else { '.' });
            }
        }
    }
    Parts {
        body,
        left_arm,
        right_arm,
        left_shoe: grab_shoe(default, 4, 49, 16, 54),
        right_shoe: grab_shoe(default, 27, 49, 40, 54),
        hat: Vec::new(),
    }
}

enum ArmSpec {
    Keep,
    Pose { elbow: (f32, f32), hand: (f32, f32) },
}

fn pose(elbow: (f32, f32), hand: (f32, f32)) -> ArmSpec {
    ArmSpec::Pose { elbow, hand }
}

/// The shoulders, in the default pose's coordinates: where an arm leaves the sleeve.
const LEFT_SHOULDER: (f32, f32) = (7.5, 20.5);
const RIGHT_SHOULDER: (f32, f32) = (35.5, 20.5);

type Joint = (f32, f32);

/// The legs: standing as drawn, a pose given by each leg's knee and ankle, or the squat.
enum Legs {
    Stand,
    Pose { left: (Joint, Joint), right: (Joint, Joint) },
    Squat,
}

fn legs(lk: Joint, la: Joint, rk: Joint, ra: Joint) -> Legs {
    Legs::Pose { left: (lk, la), right: (rk, ra) }
}

/// One whole-body drawing: where the arms are, how far the torso leans forward (pixels at the
/// head, pivoting at the hips), how far it is dropped, the legs, and the mouth (0 shut, 1 talking, 2 open).
struct Spec {
    arms: (ArmSpec, ArmSpec),
    lean: f32,
    dy: i32,
    legs: Legs,
    mouth: u8,
    /// Where the index finger is raised (the hand), if it is.
    finger: Option<Joint>,
}

fn spec(frame: Frame) -> Spec {
    use ArmSpec::Keep;
    let s = |arms: (ArmSpec, ArmSpec), lean: f32, dy: i32, legs: Legs, mouth: u8| Spec { arms, lean, dy, legs, mouth, finger: None };
    let stand = || Legs::Stand;
    let squat = || Legs::Squat;
    match frame {
        Frame::Idle(0) => s((Keep, Keep), 0.0, 0, stand(), 0),
        Frame::Idle(_) => s((Keep, Keep), 0.0, 1, stand(), 0),
        // Walking: a six-beat stride. The body bobs and leans into it, the arms swing against the
        // legs, and the feet step far apart on the contact beats.
        Frame::Walk(0) => s((pose((9.0, 28.0), (14.0, 33.0)), pose((36.0, 29.0), (34.0, 35.0))), 2.0, 0, legs((7.0, 45.0), (3.0, 47.0), (33.0, 45.0), (37.0, 48.0)), 0),
        Frame::Walk(1) => s((pose((7.0, 28.0), (9.0, 34.0)), pose((38.0, 28.0), (38.0, 34.0))), 2.0, 1, legs((10.0, 45.0), (7.0, 45.0), (32.0, 46.0), (33.0, 48.0)), 0),
        Frame::Walk(2) => s((pose((5.0, 28.0), (3.0, 34.0)), pose((40.0, 28.0), (42.0, 33.0))), 3.0, -1, legs((14.0, 43.0), (16.0, 45.0), (31.0, 45.0), (31.0, 48.0)), 0),
        Frame::Walk(3) => s((pose((2.0, 28.0), (0.0, 34.0)), pose((41.0, 28.0), (45.0, 32.0))), 2.0, 0, legs((12.0, 45.0), (14.0, 48.0), (29.0, 45.0), (27.0, 47.0)), 0),
        Frame::Walk(4) => s((pose((5.0, 28.0), (4.0, 34.0)), pose((39.0, 28.0), (39.0, 34.0))), 2.0, 1, legs((10.0, 45.0), (11.0, 48.0), (33.0, 45.0), (32.0, 45.0)), 0),
        Frame::Walk(_) => s((pose((9.0, 28.0), (14.0, 33.0)), pose((36.0, 29.0), (34.0, 35.0))), 3.0, -1, legs((10.0, 45.0), (10.0, 48.0), (35.0, 43.0), (36.0, 45.0)), 0),
        // Jumping: launch with arms thrown up and knees tucked, hang at the top with arms out,
        // then fall with the legs reaching down.
        // Air attacks: a flying kick, and a two-fisted slam from overhead.
        Frame::AirSoft(0) => s((Keep, pose((38.0, 26.0), (33.0, 22.0))), 1.0, 0, legs((5.0, 45.0), (8.0, 47.0), (36.0, 45.0), (33.0, 47.0)), 0),
        Frame::AirSoft(_) => s((pose((3.0, 26.0), (0.0, 32.0)), Keep), -3.0, 0, legs((9.0, 45.0), (10.0, 47.0), (37.0, 43.0), (46.0, 46.0)), 0),
        Frame::AirHard(0) => s((pose((1.0, 15.0), (5.0, 6.0)), pose((42.0, 14.0), (38.0, 5.0))), -2.0, 0, legs((5.0, 45.0), (8.0, 47.0), (36.0, 45.0), (33.0, 47.0)), 0),
        Frame::AirHard(_) => s((pose((6.0, 30.0), (14.0, 38.0)), pose((40.0, 30.0), (46.0, 38.0))), 6.0, 0, legs((6.0, 45.0), (3.0, 48.0), (38.0, 45.0), (41.0, 48.0)), 0),
        Frame::JumpUp => s((pose((2.0, 24.0), (-2.0, 17.0)), pose((42.0, 24.0), (46.0, 17.0))), 2.0, 0, legs((5.0, 45.0), (8.0, 47.0), (36.0, 45.0), (33.0, 47.0)), 0),
        Frame::JumpTop => s((pose((1.0, 22.0), (-5.0, 23.0)), pose((43.0, 22.0), (49.0, 23.0))), 0.0, 0, legs((4.0, 44.0), (9.0, 45.0), (37.0, 44.0), (32.0, 45.0)), 0),
        Frame::JumpFall => s((pose((2.0, 22.0), (0.0, 13.0)), pose((42.0, 20.0), (44.0, 11.0))), -1.0, 0, legs((9.0, 45.0), (8.0, 48.0), (32.0, 45.0), (33.0, 48.0)), 0),
        // Hurt: the head snaps back with the arms flung up, then he reels.
        Frame::Hurt(0) => s((pose((3.0, 24.0), (-1.0, 18.0)), pose((40.0, 23.0), (44.0, 16.0))), -6.0, 0, stand(), 2),
        Frame::Hurt(_) => s((Keep, Keep), -3.0, 1, stand(), 2),
        Frame::Block => s((Keep, pose((40.0, 24.0), (37.0, 15.0))), -1.0, 0, stand(), 0),
        Frame::Ko => s((pose((3.0, 24.0), (-1.0, 18.0)), pose((40.0, 23.0), (44.0, 16.0))), -6.0, 0, stand(), 2),
        // Crouching, and the two crouching attacks (a low jab, and an uppercut).
        Frame::Crouch => s((Keep, Keep), 2.0, 13, squat(), 0),
        Frame::CrouchSoft(0) => s((Keep, pose((40.0, 28.0), (36.0, 25.0))), 2.0, 13, squat(), 0),
        Frame::CrouchSoft(_) => s((Keep, pose((44.0, 26.0), (55.0, 26.0))), 4.0, 13, squat(), 0),
        Frame::CrouchHard(0) => s((Keep, pose((38.0, 30.0), (32.0, 32.0))), 0.0, 13, squat(), 0),
        Frame::CrouchHard(_) => s((Keep, pose((42.0, 22.0), (46.0, 8.0))), 5.0, 11, squat(), 0),
        // Soft attack: cock the front fist, then jab out stepping in.
        Frame::Soft(0) => s((Keep, pose((40.0, 25.0), (34.0, 21.0))), 1.0, 0, stand(), 0),
        Frame::Soft(_) => s((Keep, pose((44.0, 22.0), (55.0, 21.0))), 4.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (36.0, 48.0)), 0),
        // Hard attack: pull in, wind up low, lunge with the back arm swung behind, follow through.
        Frame::Hard(0) => s((Keep, pose((38.0, 27.0), (31.0, 25.0))), -2.0, 0, stand(), 0),
        Frame::Hard(1) => s((pose((5.0, 27.0), (11.0, 23.0)), pose((37.0, 31.0), (31.0, 33.0))), -4.0, 1, legs((8.0, 45.0), (4.0, 48.0), (32.0, 45.0), (35.0, 48.0)), 0),
        Frame::Hard(2) => s((pose((3.0, 28.0), (0.0, 34.0)), pose((47.0, 21.0), (60.0, 19.0))), 7.0, 1, legs((6.0, 46.0), (0.0, 48.0), (36.0, 45.0), (42.0, 48.0)), 0),
        Frame::Hard(_) => s((Keep, pose((46.0, 26.0), (55.0, 29.0))), 3.0, 0, legs((8.0, 45.0), (3.0, 48.0), (34.0, 45.0), (40.0, 48.0)), 0),
        // Summon: gather, spread, raise, hands together above the head, point, follow through.
        Frame::Summon(0) => s((pose((9.0, 28.0), (17.0, 31.0)), pose((33.0, 28.0), (25.0, 31.0))), -1.0, 1, stand(), 0),
        Frame::Summon(1) => s((pose((1.0, 25.0), (-3.0, 17.0)), pose((42.0, 25.0), (47.0, 17.0))), 0.0, 0, stand(), 0),
        Frame::Summon(2) => s((pose((1.0, 15.0), (5.0, 6.0)), pose((42.0, 14.0), (38.0, 5.0))), -2.0, -1, stand(), 0),
        Frame::Summon(3) => s((pose((2.0, 10.0), (15.0, -2.0)), pose((41.0, 10.0), (28.0, -2.0))), -3.0, -1, stand(), 0),
        Frame::Summon(4) => s((pose((1.0, 14.0), (3.0, 5.0)), pose((45.0, 17.0), (57.0, 15.0))), 3.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (37.0, 48.0)), 0),
        Frame::Summon(_) => s((Keep, pose((45.0, 23.0), (55.0, 26.0))), 2.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (37.0, 48.0)), 0),
        Frame::Speak(0) => s((Keep, pose((42.0, 25.0), (45.0, 17.0))), 0.0, 0, stand(), 1),
        Frame::Speak(_) => s((Keep, pose((41.0, 27.0), (44.0, 22.0))), 0.0, 0, stand(), 2),
        // The lecture: a breath, then the finger goes up while he talks, wags, and comes down.
        Frame::Lecture(0) => s((Keep, Keep), -2.0, 0, stand(), 1),
        Frame::Lecture(1) => Spec { arms: (Keep, pose((43.0, 15.0), (41.0, 9.0))), lean: 0.0, dy: 0, legs: stand(), mouth: 2, finger: Some((41.0, 9.0)) },
        Frame::Lecture(2) => Spec { arms: (Keep, pose((46.0, 12.0), (46.0, 5.0))), lean: 3.0, dy: 0, legs: stand(), mouth: 1, finger: Some((46.0, 5.0)) },
        Frame::Lecture(_) => Spec { arms: (Keep, pose((42.0, 20.0), (41.0, 13.0))), lean: 1.0, dy: 0, legs: stand(), mouth: 1, finger: Some((41.0, 13.0)) },
        // Alternating punches: the right fist cocks and strikes, then the left. (The Cowboy and the
        // Gunslinger have their own, see cb_spec and gs_spec.)
        Frame::Punch(0) => s((Keep, pose((40.0, 25.0), (34.0, 21.0))), 1.0, 0, stand(), 0),
        Frame::Punch(1) => s((Keep, pose((44.0, 22.0), (55.0, 21.0))), 4.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (36.0, 48.0)), 0),
        Frame::Punch(2) => s((pose((5.0, 27.0), (11.0, 23.0)), Keep), 1.0, 0, stand(), 0),
        Frame::Punch(_) => s((pose((24.0, 23.0), (46.0, 21.0)), Keep), 4.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (36.0, 48.0)), 0),
        // The kicks are the Cowboy's own (see cb_spec).
        Frame::Kick(_) => s((Keep, Keep), 0.0, 0, stand(), 0),
        // Commanding the raven: a finger up, the arm thrown out at the enemy, then back.
        Frame::Point(0) => Spec { arms: (Keep, pose((42.0, 17.0), (44.0, 10.0))), lean: 0.0, dy: 0, legs: stand(), mouth: 1, finger: Some((44.0, 10.0)) },
        Frame::Point(1) => s((Keep, pose((44.0, 22.0), (57.0, 20.0))), 4.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (36.0, 48.0)), 1),
        Frame::Point(_) => s((Keep, pose((41.0, 26.0), (45.0, 24.0))), 1.0, 0, stand(), 0),
        // The collar grab: beckon, pull him in, hold him by the collar talking (then with the
        // mouth shut), shove him away with both hands, recover. The hand that holds is about 25 px
        // in front of him: where the enemy's collar is once he has been pulled in.
        Frame::Grab(0) => s((Keep, pose((40.0, 25.0), (49.0, 22.0))), 2.0, 0, stand(), 0),
        Frame::Grab(1) => s((pose((9.0, 26.0), (14.0, 22.0)), pose((38.0, 24.0), (46.0, 21.0))), 3.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (36.0, 48.0)), 0),
        Frame::Grab(2) => s((pose((9.0, 26.0), (15.0, 22.0)), pose((39.0, 22.0), (46.0, 19.0))), 4.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (36.0, 48.0)), 2),
        Frame::Grab(3) => s((pose((9.0, 26.0), (15.0, 22.0)), pose((39.0, 22.0), (46.0, 19.0))), 4.0, 0, legs((8.0, 45.0), (5.0, 48.0), (33.0, 45.0), (36.0, 48.0)), 1),
        Frame::Grab(4) => s((pose((22.0, 25.0), (50.0, 22.0)), pose((43.0, 21.0), (58.0, 20.0))), 7.0, 1, legs((6.0, 46.0), (0.0, 48.0), (36.0, 45.0), (42.0, 48.0)), 2),
        Frame::Grab(_) => s((Keep, pose((38.0, 27.0), (41.0, 29.0))), 1.0, 0, stand(), 1),
        // Only the Gunslinger has the volley; the others never draw these.
        Frame::Gun(_) => s((Keep, Keep), 0.0, 0, stand(), 0),
        // Anaran's reading and staff slam are his own (see an_spec).
        Frame::Read(_) | Frame::Staff(_) => s((Keep, Keep), 0.0, 0, stand(), 0),
        // The roll is made from the finished crouch drawing (`--roll-frames`), not drawn here.
        Frame::Roll(_) | Frame::Boost(_) | Frame::Lean(_) | Frame::Write(_) => s((Keep, Keep), 0.0, 0, stand(), 0),
    }
}

/// The torso (head to hips, with arms and mouth) in the default pose's coordinates on the canvas.
fn torso(parts: &Parts, arms: (ArmSpec, ArmSpec), mouth: u8, finger: Option<Joint>) -> Grid {
    let mut c = Grid::blank(FRAME_W, UPPER_H);
    for y in 0..=UPPER_LAST_ROW {
        for x in 0..parts.body.w as i32 {
            let ch = parts.body.get(x, y);
            if ch != '.' {
                c.set(x + OX, y + OY, ch);
            }
        }
    }
    if mouth > 0 {
        for x in 22..=24 {
            c.set(x + OX, 14 + OY, 'g');
        }
        if mouth > 1 {
            c.set(23 + OX, 15 + OY, 'g');
        }
    }
    let (left, right) = arms;
    for (spec, shoulder, kept) in [(left, LEFT_SHOULDER, &parts.left_arm), (right, RIGHT_SHOULDER, &parts.right_arm)] {
        match spec {
            ArmSpec::Keep => {
                for &(x, y, ch) in kept {
                    c.set(x + OX, y + OY, ch);
                }
            }
            ArmSpec::Pose { elbow, hand } => draw_arm(&mut c, shoulder, elbow, hand),
        }
    }
    // The raised index finger above a fist.
    if let Some((hx, hy)) = finger {
        let (hx, hy) = (hx.round() as i32, hy.round() as i32);
        for k in 0..8 {
            c.set(hx + OX, hy - 3 - k + OY, 'a');
            c.set(hx + 1 + OX, hy - 3 - k + OY, 'b');
        }
    }
    // Put the sleeves back over the tops of the arms.
    for y in 14..=21 {
        for x in (3..=12).chain(30..=39) {
            let ch = parts.body.get(x, y);
            if ch == 'w' || ch == 'v' {
                c.set(x + OX, y + OY, ch);
            }
        }
    }
    c
}

/// The legs and the waist of the trousers, drawn under the torso.
fn leg_layer(parts: &Parts, legs: &Legs) -> Grid {
    let mut c = Grid::blank(FRAME_W, FULL_H);
    let copy_rows = |c: &mut Grid, y0: i32, y1: i32| {
        for y in y0..=y1 {
            for x in 0..parts.body.w as i32 {
                let ch = parts.body.get(x, y);
                if ch != '.' {
                    c.set(x + OX, y + OY, ch);
                }
            }
        }
    };
    copy_rows(&mut c, LOWER_FIRST_ROW, 41);
    let (left_hip, right_hip) = ((10.0, 41.0), (31.0, 41.0));
    match legs {
        Legs::Stand => copy_rows(&mut c, 42, 54),
        Legs::Pose { left, right } => {
            draw_leg(&mut c, OY, left_hip, left.0, left.1);
            stamp_shoe(&mut c, OY, &parts.left_shoe, left.1 .0 as i32 - 6, left.1 .1 as i32 + 1);
            draw_leg(&mut c, OY, right_hip, right.0, right.1);
            stamp_shoe(&mut c, OY, &parts.right_shoe, right.1 .0 as i32 - 4, right.1 .1 as i32 + 1);
        }
        Legs::Squat => {
            draw_leg(&mut c, OY, (10.0, 49.0), (1.0, 46.0), (6.0, 48.0));
            stamp_shoe(&mut c, OY, &parts.left_shoe, 0, 49);
            draw_leg(&mut c, OY, (31.0, 49.0), (41.0, 46.0), (35.0, 48.0));
            stamp_shoe(&mut c, OY, &parts.right_shoe, 31, 49);
        }
    }
    c
}

/// A whole-body drawing (upright, even for the KO), legs first, then the torso leaned and dropped over them.
fn drawing_upright(parts: &Parts, frame: Frame, raven: bool) -> Grid {
    let spec = spec(frame);
    let mut canvas = leg_layer(parts, &spec.legs);
    let mut upper = torso(parts, spec.arms, spec.mouth, spec.finger);
    if raven {
        // The raven perches on his front shoulder and leans with the torso.
        stamp_raven(&mut upper, &raven_perched(), RAVEN_PERCH.0, RAVEN_PERCH.1);
    }
    for cy in 0..UPPER_H {
        // The lean pivots at the hips, so the head moves furthest.
        let r = (cy - OY).clamp(0, UPPER_LAST_ROW);
        let shift = (spec.lean * (UPPER_LAST_ROW - r) as f32 / UPPER_LAST_ROW as f32).round() as i32;
        for x in 0..FRAME_W {
            let ch = upper.get(x, cy);
            let (nx, ny) = (x + shift, cy + spec.dy);
            if ch != '.' && nx >= 0 && nx < FRAME_W && ny >= 0 && ny < FULL_H {
                canvas.set(nx, ny, ch);
            }
        }
    }
    canvas
}

/// The drawing with the scar left off, and the scar alone as a white mask (the game tints it, to
/// fade it in while the special charges and to make it glow when it is ready).
fn drawing_and_mark(parts: &Parts, frame: Frame) -> (Grid, Grid) {
    drawing_and_mark_with(parts, frame, false)
}

fn drawing_and_mark_with(parts: &Parts, frame: Frame, raven: bool) -> (Grid, Grid) {
    let upright = drawing_upright(parts, frame, raven);
    let mut body = upright.clone();
    let mut mark = Grid::blank(FRAME_W, FULL_H);
    for y in 0..FULL_H {
        for x in 0..FRAME_W {
            if upright.get(x, y) == 'x' {
                mark.set(x, y, 'W');
                body.set(x, y, 'a');
            }
        }
    }
    if frame == Frame::Ko {
        return (lying_with(&body, &upright), lying_with(&mark, &upright));
    }
    (body, mark)
}

/// The fallen fighter: the drawing turned onto its back (head to the left), resting on the floor.
fn lying(g: &Grid) -> Grid {
    lying_with(g, g)
}

/// `g` turned onto its back, placed using the bounds of `bounds_from` (so an overlay lines up).
fn lying_with(g: &Grid, bounds_from: &Grid) -> Grid {
    let place = |g: &Grid| -> Vec<(i32, i32, char)> {
        let mut cells = Vec::new();
        for y in 0..g.h as i32 {
            for x in 0..g.w as i32 {
                let ch = g.get(x, y);
                if ch != '.' {
                    cells.push((y, g.w as i32 - 1 - x, ch));
                }
            }
        }
        cells
    };
    let bounds = place(bounds_from);
    let (min_x, max_x) = (bounds.iter().map(|c| c.0).min().unwrap(), bounds.iter().map(|c| c.0).max().unwrap());
    let max_y = bounds.iter().map(|c| c.1).max().unwrap();
    let centre = (min_x + max_x) / 2;
    let mut out = Grid::blank(FRAME_W, FULL_H);
    for (x, y, ch) in place(g) {
        out.set(x - centre + CENTRE_X, y - max_y + FULL_H - 1, ch);
    }
    out
}

#[allow(dead_code)]
fn lying_old(g: &Grid) -> Grid {
    let mut cells = Vec::new();
    for y in 0..g.h as i32 {
        for x in 0..g.w as i32 {
            let ch = g.get(x, y);
            if ch != '.' {
                // Turn a quarter counter-clockwise.
                cells.push((y, g.w as i32 - 1 - x, ch));
            }
        }
    }
    let (min_x, max_x) = (cells.iter().map(|c| c.0).min().unwrap(), cells.iter().map(|c| c.0).max().unwrap());
    let max_y = cells.iter().map(|c| c.1).max().unwrap();
    let centre = (min_x + max_x) / 2;
    let mut out = Grid::blank(FRAME_W, FULL_H);
    for (x, y, ch) in cells {
        out.set(x - centre + CENTRE_X, y - max_y + FULL_H - 1, ch);
    }
    out
}

/// The summoned crescent (32 x 32) in flat gold: a light rim up and left, a shadow edge down and
/// right. The two drawings differ in where the glints are.
fn crescent(variant: usize) -> Grid {
    let mut cells = HashSet::new();
    for y in 0..32 {
        for x in 0..32 {
            let outer = (x - 15) * (x - 15) + (y - 16) * (y - 16) <= 14 * 14;
            let bite = (x - 22) * (x - 22) + (y - 13) * (y - 13) <= 11 * 11;
            if outer && !bite {
                cells.insert((x, y));
            }
        }
    }
    let mut g = Grid::blank(32, 32);
    for &(x, y) in &cells {
        let ch = if !cells.contains(&(x - 1, y)) || !cells.contains(&(x, y - 1)) {
            'z'
        } else if !cells.contains(&(x + 1, y)) || !cells.contains(&(x, y + 1)) {
            'y'
        } else {
            'Y'
        };
        g.set(x, y, ch);
    }
    let glints: [(i32, i32); 3] = if variant == 0 { [(28, 4), (3, 28), (27, 27)] } else { [(5, 4), (30, 18), (12, 30)] };
    for (x, y) in glints {
        g.set(x, y, 'W');
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if g.get(x + dx, y + dy) == '.' {
                g.set(x + dx, y + dy, 'z');
            }
        }
    }
    g
}

/// The shooting star (44 x 20) in flat gold, heading right with a tail behind it.
fn star(variant: usize) -> Grid {
    let mut g = Grid::blank(44, 20);
    // The tail fans out and brightens toward the star.
    for x in 0..28 {
        let half = (x * 5 / 28) as i32;
        for dy in -half..=half {
            if variant == 1 && (x + dy.abs() as usize) % 5 == 0 {
                continue; // flicker
            }
            let ch = if dy == 0 && x > 8 { 'z' } else if dy.abs() == half && half > 1 { 'y' } else { 'Y' };
            g.set(x as i32, 10 + dy, ch);
        }
    }
    let template = [
        "....#....", "....#....", "...###...", "#########", ".#######.", "..#####..", "..##.##..", ".##...##.", ".#.....#.",
    ];
    let mut cells = HashSet::new();
    for (y, row) in template.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    cells.insert((26 + x as i32 * 2 + dx, 1 + y as i32 * 2 + dy));
                }
            }
        }
    }
    for &(x, y) in &cells {
        let ch = if !cells.contains(&(x - 1, y)) || !cells.contains(&(x, y - 1)) {
            'z'
        } else if !cells.contains(&(x + 1, y)) || !cells.contains(&(x, y + 1)) {
            'y'
        } else {
            'Y'
        };
        g.set(x, y, ch);
    }
    for (x, y) in [(34, 8), (35, 8), (34, 9), (35, 9)] {
        g.set(x, y, 'W');
    }
    g
}

/// A small gold star for the sky beside the moon (18 x 18), twinkling between two drawings.
fn moon_star(variant: usize) -> Grid {
    let template = [
        "....#....", "....#....", "...###...", "#########", ".#######.", "..#####..", "..##.##..", ".##...##.", ".#.....#.",
    ];
    let mut cells = HashSet::new();
    for (y, row) in template.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    cells.insert((x as i32 * 2 + dx, y as i32 * 2 + dy));
                }
            }
        }
    }
    let mut g = Grid::blank(18, 18);
    for &(x, y) in &cells {
        let ch = if !cells.contains(&(x - 1, y)) || !cells.contains(&(x, y - 1)) {
            'z'
        } else if !cells.contains(&(x + 1, y)) || !cells.contains(&(x, y + 1)) {
            'y'
        } else {
            'Y'
        };
        g.set(x, y, ch);
    }
    // The centre glints; on the other drawing the glints sit round it.
    for (x, y) in [(8, 8), (9, 8), (8, 9), (9, 9)] {
        g.set(x, y, 'W');
    }
    if variant == 1 {
        for (x, y) in [(0, 0), (17, 2), (1, 16), (16, 15)] {
            g.set(x, y, 'z');
        }
    }
    g
}

/// The fish the moon spits out (28 x 14), heading right: blue with a pale belly and a dark back,
/// an orange tail and fin, a round eye. The tail flaps between the two drawings.
fn fish(variant: usize) -> Grid {
    let mut g = Grid::blank(28, 14);
    let mut body = HashSet::new();
    for y in 0..14 {
        for x in 6..28 {
            let (dx, dy) = ((x as f32 - 17.0) / 11.0, (y as f32 - 6.5) / 5.6);
            if dx * dx + dy * dy <= 1.0 {
                body.insert((x, y));
            }
        }
    }
    for &(x, y) in &body {
        let ch = if !body.contains(&(x, y - 1)) || !body.contains(&(x + 1, y)) && y < 6 {
            '&'
        } else if y >= 8 {
            '%'
        } else {
            '#'
        };
        g.set(x, y, ch);
    }
    // The tail, a fan of orange behind the body.
    let spread = if variant == 0 { 5 } else { 4 };
    for x in 0..7 {
        let half = (6 - x) * spread / 6 + 1;
        let lift = if variant == 0 { 0 } else { x / 3 - 1 };
        for dy in -half..=half {
            g.set(x, 6 + dy + lift, '@');
        }
    }
    // The back fin and a belly fin.
    for (x, y) in [(15, 0), (16, 0), (17, 0), (16, 1), (14, 1)] {
        g.set(x, y, '@');
    }
    for (x, y) in [(16, 13), (17, 13), (15, 12)] {
        g.set(x, y, '@');
    }
    // The eye and the gill.
    g.set(23, 5, 'W');
    g.set(24, 5, 'g');
    g.set(24, 6, 'g');
    g.set(20, 7, '&');
    g.set(20, 8, '&');
    g
}

fn invert(img: &image::RgbaImage) -> image::RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        if p[3] > 0 {
            *p = image::Rgba([255 - p[0], 255 - p[1], 255 - p[2], p[3]]);
        }
    }
    out
}

/// Every drawing of Ayz as (file name, image).
fn all_frames(default: &Grid, raven: bool) -> Vec<(String, image::RgbaImage)> {
    let parts = split(default);
    let mut frames = Vec::new();
    let mut every = Frame::all();
    every.extend((0..t2f::anim::LECTURE_FRAMES).map(Frame::Lecture));
    every.extend((0..t2f::anim::PUNCH_FRAMES).map(Frame::Punch));
    if raven {
        // The Professor's new moves (the negative twin keeps his old drawings).
        every.extend((0..t2f::anim::POINT_FRAMES).map(Frame::Point));
        every.extend((0..t2f::anim::GRAB_FRAMES).map(Frame::Grab));
    }
    for f in every {
        let (body, mark) = drawing_and_mark_with(&parts, f, raven);
        frames.push((f.file_stem(), body.to_png()));
        if raven {
            // The same pose with the raven gone from the shoulder (while it is out flying).
            frames.push((format!("{}_nr", f.file_stem()), drawing_and_mark_with(&parts, f, false).0.to_png()));
        }
        frames.push((format!("{}_mark", f.file_stem()), mark.to_png()));
    }
    for v in 0..t2f::anim::CRESCENT_FRAMES {
        frames.push((format!("crescent_{v}"), crescent(v).to_png()));
    }
    for v in 0..t2f::anim::STAR_FRAMES {
        frames.push((format!("star_{v}"), star(v).to_png()));
        frames.push((format!("moon_star_{v}"), moon_star(v).to_png()));
        frames.push((format!("fish_{v}"), fish(v).to_png()));
    }
    frames
}


// The Professor's raven (black with a blue sheen, like his hair), the rose it drops, and the
// words ("uh", "meh") he speaks at the enemy. The shapes are drawn from ellipses and polygons so
// that the perched, flying, pecking and clawing drawings all match.

/// Where the perched raven's feet are in the torso drawing (x centre, y bottom row).
const RAVEN_PERCH: (i32, i32) = (48, 22);

fn rv_ellipse(g: &mut Grid, cx: f32, cy: f32, rx: f32, ry: f32, ch: char) {
    for y in 0..g.h as i32 {
        for x in 0..g.w as i32 {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            if dx * dx + dy * dy <= 1.0 {
                g.set(x, y, ch);
            }
        }
    }
}

fn rv_poly(g: &mut Grid, pts: &[(f32, f32)], ch: char) {
    for y in 0..g.h as i32 {
        for x in 0..g.w as i32 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut inside = false;
            for i in 0..pts.len() {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                if (a.1 > py) != (b.1 > py) && px < (b.0 - a.0) * (py - a.1) / (b.1 - a.1) + a.0 {
                    inside = !inside;
                }
            }
            if inside {
                g.set(x, y, ch);
            }
        }
    }
}

/// The raven's body and head, facing right, with its beak and eye. (`head` is the centre of the
/// head, `beak` the tip of the beak.)
fn rv_body(g: &mut Grid, body: (f32, f32, f32, f32), head: (f32, f32), beak: (f32, f32)) {
    rv_ellipse(g, body.0, body.1, body.2, body.3, 'k');
    rv_ellipse(g, head.0, head.1, 2.6, 2.4, 'k');
    // The beak: a wedge from the head to its tip (the upper edge lighter).
    rv_poly(g, &[(head.0 + 1.5, head.1 - 1.2), (beak.0, beak.1), (head.0 + 1.5, head.1 + 1.0)], 'g');
    g.set((head.0 + 1.0) as i32, (head.1 - 0.9) as i32, 'Z');
    // The eye.
    g.set((head.0 + 0.6) as i32, (head.1 - 0.6) as i32, 'u');
    // A blue sheen along the back.
    for k in 0..3 {
        g.set((body.0 - 1.5 + k as f32 * 1.5) as i32, (body.1 - body.3 + 0.8 + k as f32 * 0.2) as i32, 'l');
    }
}

fn rv_tail(g: &mut Grid, from: (f32, f32), tip: (f32, f32)) {
    rv_poly(g, &[(from.0, from.1 - 1.4), (tip.0, tip.1 - 0.8), (tip.0 + 0.5, tip.1 + 1.4), (from.0, from.1 + 1.4)], 'k');
}

/// Sitting on the shoulder, facing right (14 x 11).
fn raven_perched() -> Grid {
    let mut g = Grid::blank(14, 11);
    rv_body(&mut g, (6.5, 6.4, 5.0, 3.3), (10.6, 3.0), (14.0, 3.9));
    rv_tail(&mut g, (3.0, 7.4), (0.0, 10.0));
    // The wing folded over the body, a darker line for it.
    for x in 3..=8 {
        g.set(x, 7 + (x - 3) / 3, 'g');
    }
    for x in [6, 8] {
        g.set(x, 10, 'g');
    }
    g
}

/// In the air, a wing up or down (0 up, 1 down), facing right (24 x 16).
fn raven_flying(flap: usize) -> Grid {
    let mut g = Grid::blank(24, 16);
    rv_body(&mut g, (11.5, 9.0, 5.2, 2.9), (17.5, 6.9), (22.5, 7.9));
    rv_tail(&mut g, (7.0, 9.6), (1.0, 12.0));
    if flap == 0 {
        rv_poly(&mut g, &[(8.0, 8.0), (14.0, 8.0), (6.0, 0.5), (2.5, 2.0)], 'k');
        for (x, y) in [(5, 3), (6, 4), (7, 5)] {
            g.set(x, y, 'l');
        }
    } else {
        rv_poly(&mut g, &[(8.0, 10.0), (14.0, 10.0), (7.0, 15.5), (3.0, 14.0)], 'k');
        for (x, y) in [(6, 13), (7, 12), (8, 11)] {
            g.set(x, y, 'l');
        }
    }
    g
}

/// Pecking: the body leans, the head cocked back (0) or thrust forward and down (1). (24 x 16)
fn raven_pecking(strike: usize) -> Grid {
    let mut g = Grid::blank(24, 16);
    if strike == 0 {
        rv_body(&mut g, (10.0, 9.4, 5.2, 3.2), (15.5, 5.0), (19.0, 6.0));
    } else {
        rv_body(&mut g, (10.0, 9.4, 5.2, 3.2), (16.0, 8.6), (21.5, 11.8));
    }
    rv_tail(&mut g, (6.0, 10.2), (1.0, 12.5));
    for x in [8, 11] {
        g.set(x, 13, 'g');
        g.set(x, 14, 'g');
    }
    g
}

/// Clawing: wings up and the talons thrust forward and down. (24 x 16)
fn raven_clawing(flap: usize) -> Grid {
    let mut g = raven_flying(flap);
    for k in 0..5 {
        g.set(12 + k, 12 + k / 2, 'Z');
        g.set(10 + k, 12 + k / 2, 'Z');
    }
    for (x, y) in [(17, 14), (18, 14), (17, 15), (15, 14)] {
        g.set(x, y, 'u');
    }
    g
}

/// The rose the raven drops (8 x 12), with a glint in the second drawing.
fn rose(variant: usize) -> Grid {
    let mut g = Grid::blank(8, 12);
    rv_poly(&mut g, &[(3.5, 7.0), (3.5, 11.8), (4.5, 11.8), (4.5, 7.0)], 'à');
    rv_poly(&mut g, &[(4.5, 8.5), (7.5, 7.0), (6.0, 10.0)], 'è');
    rv_ellipse(&mut g, 4.0, 3.5, 3.2, 3.0, 'C');
    rv_ellipse(&mut g, 4.0, 4.6, 2.2, 1.8, 'B');
    for (x, y) in [(3, 2), (4, 2), (3, 3)] {
        g.set(x, y, '-');
    }
    g.set(4, 3, 'B');
    g.set(5, 4, 'C');
    if variant == 1 {
        g.set(1, 0, 'W');
        g.set(6, 1, 'W');
    }
    g
}

/// Stamp a sprite with its feet (bottom middle) at (x, y).
fn stamp_raven(canvas: &mut Grid, sprite: &Grid, x: i32, y: i32) {
    for sy in 0..sprite.h as i32 {
        for sx in 0..sprite.w as i32 {
            let ch = sprite.get(sx, sy);
            if ch != '.' {
                canvas.set(x - sprite.w as i32 / 2 + sx, y - sprite.h as i32 + 1 + sy, ch);
            }
        }
    }
}

/// A word in a tiny 3 x 5 pixel font (m is 5 wide), each font pixel `WORD_SCALE` image pixels big
/// (2.4: strokes are 2 or 3 pixels wide), white with a dark outline; the second drawing wobbles a
/// little up.
const WORD_SCALE: f32 = 2.4;

fn word_art(word: &str, variant: usize) -> Grid {
    let glyph = |c: char| -> Vec<&'static str> {
        match c {
            'u' => vec!["x.x", "x.x", "x.x", "x.x", "xxx"],
            'h' => vec!["x..", "x..", "xxx", "x.x", "x.x"],
            'm' => vec!["x...x", "xx.xx", "x.x.x", "x...x", "x...x"],
            'e' => vec!["xxx", "x..", "xxx", "x..", "xxx"],
            _ => vec!["...", "...", "...", "...", "..."],
        }
    };
    // The word at font resolution.
    let glyphs: Vec<Vec<&str>> = word.chars().map(glyph).collect();
    let letters_w: usize = glyphs.iter().map(|g| g[0].len()).sum::<usize>() + glyphs.len() - 1;
    let mut font = vec![vec![false; letters_w]; 5];
    let mut x0 = 0;
    for rows in &glyphs {
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                font[y][x0 + x] = c == 'x';
            }
        }
        x0 += rows[0].len() + 1;
    }
    // Scaled up by sampling (nearest), with room for the outline.
    let (w, h) = ((letters_w as f32 * WORD_SCALE).ceil() as i32, (5.0 * WORD_SCALE).ceil() as i32);
    let pad = 2;
    let lift = variant as i32 * 2;
    let mut g = Grid::blank(w + pad * 2, h + pad * 2 + 2);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = ((x as f32 / WORD_SCALE) as usize, (y as f32 / WORD_SCALE) as usize);
            if font[fy.min(4)][fx.min(letters_w - 1)] {
                g.set(pad + x, pad + 2 + y - lift, 'W');
            }
        }
    }
    // The dark outline: every empty cell touching the white.
    let white: Vec<(i32, i32)> = (0..g.h as i32).flat_map(|y| (0..g.w as i32).map(move |x| (x, y))).filter(|&(x, y)| g.get(x, y) == 'W').collect();
    for (x, y) in white {
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if g.get(x + dx, y + dy) == '.' {
                    g.set(x + dx, y + dy, 'g');
                }
            }
        }
    }
    g
}

/// Every effect drawing of the Professor's new moves, by file name.
fn raven_effects() -> Vec<(String, image::RgbaImage)> {
    let mut out = Vec::new();
    for v in 0..2 {
        out.push((format!("raven_fly_{v}"), raven_flying(v).to_png()));
        out.push((format!("raven_peck_{v}"), raven_pecking(v).to_png()));
        out.push((format!("raven_claw_{v}"), raven_clawing(v).to_png()));
        out.push((format!("rose_{v}"), rose(v).to_png()));
        out.push((format!("word_uh_{v}"), word_art("uh", v).to_png()));
        out.push((format!("word_meh_{v}"), word_art("meh", v).to_png()));
    }
    out
}

/// The raven and the words, big, on a pale background, for a quick look (`--raven <out.png>`).
fn raven_sheet() -> image::RgbaImage {
    let items: Vec<(String, image::RgbaImage)> = {
        let mut v = vec![("perched".to_string(), raven_perched().to_png())];
        v.extend(raven_effects());
        v
    };
    let scale = 10u32;
    let cols = 5u32;
    let (cw, ch) = (50 * scale, 26 * scale);
    let rows = (items.len() as u32 + cols - 1) / cols;
    let mut sheet = image::RgbaImage::from_pixel(cols * cw, rows * ch, image::Rgba([186, 214, 238, 255]));
    for (i, (_, img)) in items.iter().enumerate() {
        let (ox, oy) = ((i as u32 % cols) * cw + scale, (i as u32 / cols) * ch + scale);
        for y in 0..img.height() * scale {
            for x in 0..img.width() * scale {
                let p = img.get_pixel(x / scale, y / scale);
                if p[3] > 0 {
                    sheet.put_pixel(ox + x, oy + y, *p);
                }
            }
        }
    }
    sheet
}

/// A contact sheet of every whole-body drawing, then the two effects.
fn contact_sheet(default: &Grid, scale: u32) -> image::RgbaImage {
    let parts = split(default);
    let mut every = Frame::all();
    every.extend((0..t2f::anim::LECTURE_FRAMES).map(Frame::Lecture));
    let cells: Vec<image::RgbaImage> = every.into_iter().map(|f| drawing_and_mark(&parts, f).0.to_png()).collect();
    let columns = 8u32;
    let (cw, ch) = (FRAME_W as u32 * scale, FULL_H as u32 * scale);
    let rows = (cells.len() as u32 + columns - 1) / columns;
    let mut sheet = image::RgbaImage::from_pixel(columns * (cw + 8) + 8, rows * (ch + 8) + 8 + 40 * scale, image::Rgba([196, 214, 232, 255]));
    for (k, cell) in cells.iter().enumerate() {
        let (col, row) = (k as u32 % columns, k as u32 / columns);
        let (ox, oy) = (8 + col * (cw + 8), 8 + row * (ch + 8));
        for y in 0..ch {
            for x in 0..cw {
                let p = cell.get_pixel(x / scale, y / scale);
                sheet.put_pixel(ox + x, oy + y, if p[3] > 0 { *p } else { image::Rgba([232, 240, 248, 255]) });
            }
        }
    }
    let mut x = 8;
    let y = rows * (ch + 8) + 8;
    for img in [crescent(0).to_png(), crescent(1).to_png(), star(0).to_png(), star(1).to_png()] {
        for yy in 0..img.height() * scale {
            for xx in 0..img.width() * scale {
                let p = img.get_pixel(xx / scale, yy / scale);
                if p[3] > 0 {
                    sheet.put_pixel(x + xx, y + yy, *p);
                }
            }
        }
        x += img.width() * scale + 8 * scale;
    }
    sheet
}

// the Gunslinger: the original's own pose and proportions (the eyes, the stance and the arms
// are the original's pixels), slimmer than the Professor and dressed differently

fn soul_default() -> Grid {
    soul_with_hair(true)
}

/// The Gunslinger's default pose; without `hair` the long locks at the sides are left out.
fn soul_with_hair(hair: bool) -> Grid {
    let mut g = Grid::parse(REF);
    // The same extra row of cheek as the Professor, so every row matches his.
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.insert(18, rows[17].clone());
    g = Grid::from_rows(rows);
    // The original's left arm runs off the edge of its image: finish it.
    for (y, from) in [(26, 2), (27, 1), (28, 0), (29, 0), (30, 0), (31, 0), (32, 0), (33, 0), (34, 1), (35, 2)] {
        for x in from..3 {
            let ch = if x == from && from == 0 { 'K' } else { 'D' };
            if g.get(x, y) == '.' {
                g.set(x, y, ch);
            }
        }
    }
    // Finish the shoes, which the original's image cuts off (made into boots below).
    let shoe_row = g.row(g.h - 1);
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'L' } else { c }).collect());
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'Q' } else { '.' }).collect());
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'J' } else { '.' }).collect());
    g = Grid::from_rows(rows);

    // The hands (the original's pale grey cells), remembered so the sleeves leave them alone.
    let mut hands = std::collections::HashSet::new();
    for y in 25..=46 {
        for x in 0..g.w as i32 {
            if "JMP".contains(g.get(x, y)) {
                hands.insert((x, y));
            }
        }
    }
    g.recolor(0, 99, &[('F', 'a'), ('D', 'b'), ('K', 'd'), ('O', 'e')]);
    g.recolor(25, 46, &[('J', 'a'), ('M', 'b'), ('P', 'a')]);
    g.recolor(0, 99, &[('E', 'c'), ('I', 'f')]);
    g.recolor(20, 45, &[('C', 'S'), ('B', 's')]);
    g.recolor(20, 99, &[('A', 'm'), ('H', 'n'), ('G', 'j'), ('N', 'h')]);
    g.recolor(47, 99, &[('J', 'Z'), ('M', 'Z'), ('L', 'X'), ('Q', 'V'), ('R', 'V')]);

    // No glasses: the frames become skin, leaving the original's eyes at their original size.
    for (c, y) in [
        (13, 13), (14, 13), (12, 14), (12, 15), (15, 14), (15, 15), (13, 16), (14, 16),
        (17, 13), (18, 13), (16, 14), (19, 14), (19, 15), (17, 16), (18, 16),
    ] {
        g.set(c, y, 'a');
    }
    // The original's forehead is one pixel wider on the left than the face under it, which makes
    // the upper face look shifted: trim that column back to hair.
    for y in 11..=13 {
        if "abdeKFD".contains(g.get(11, y)) {
            g.set(11, y, 'c');
        }
    }
    // A smile with teeth.
    g.set(14, 18, 'e');
    g.set(18, 18, 'e');
    for c in 15..=17 {
        g.set(c, 18, 'w');
        g.set(c, 19, 'e');
    }

    // The blazer's sleeves run down the arms to the wrists, with a white cuff, and the forearm and
    // fist below it are bare. The left arm is bent, so its sleeve ends higher.
    for (range, last) in [(0..9, 30), (23..g.w as i32, 33)] {
        for y in 21..=last {
            for x in range.clone() {
                let ch = g.get(x, y);
                if "abde".contains(ch) && !hands.contains(&(x, y)) {
                    g.set(x, y, if y == last { 'w' } else if "de".contains(ch) { 's' } else { 'S' });
                }
            }
        }
    }

    // The white shirt down the open front with a collar, the blazer's lapels either side.
    let cx = 13;
    for y in 21..=40 {
        let half = if y <= 25 { 4 } else { 3 };
        for x in cx - half..cx + half {
            g.set(x, y, if x == cx + half - 1 { 'v' } else { 'w' });
        }
        g.set(cx - half - 1, y, 'T');
        g.set(cx + half, y, 's');
    }
    for (x, y) in [(cx - 4, 21), (cx + 3, 21), (cx - 3, 22), (cx + 2, 22)] {
        g.set(x, y, 'v');
    }
    // A few fold highlights on the corduroy.
    for (x, y) in [(4, 27), (5, 31), (6, 36), (21, 27), (20, 32), (19, 38), (6, 39), (17, 39)] {
        if g.get(x, y) == 'S' {
            g.set(x, y, 'T');
        }
    }

    // Long straight hair, parted in the middle: one smooth lock down each side, tucked behind
    // the cheeks, swelling a little over the shoulder and tapering to a point on the chest. Drawn
    // for the left side from a table of (first column, last column) per row and mirrored about
    // the middle of the face. Dark at the inner edge, a lighter strand or two along it, and a
    // ragged end, in the original's flat tones. The cheeks stay bare (no sideburns).
    // First clear the original's own hair beside and below the face.
    for y in 9..=32 {
        for x in (0..=11).chain(20..g.w as i32) {
            if "cfi".contains(g.get(x, y)) {
                g.set(x, y, '.');
            }
        }
    }
    let locks: [(i32, i32, i32); 17] = [
        (12, 8, 10), (13, 8, 10), (14, 8, 10), (15, 8, 10), (16, 8, 10), (17, 7, 9), (18, 7, 9), (19, 7, 9),
        (20, 7, 9), (21, 7, 9), (22, 7, 9), (23, 7, 9), (24, 7, 9), (25, 7, 9), (26, 7, 8), (27, 7, 8), (28, 7, 7),
    ];
    for &(y, x0, x1) in locks.iter().filter(|_| hair) {
        for x in x0..=x1 {
            // Plain dark brown with one lighter strand running down the outer edge.
            let ch = if x == x0 && y % 3 != 0 { 'f' } else { 'c' };
            g.set(x, y, ch);
            g.set(31 - x, y, ch);
        }
    }
    for y in 12..=18 {
        for x in [11, 20] {
            if y <= 16 || g.get(x, y) == '.' {
                g.set(x, y, 'b');
            }
        }
    }

    // The red tie: a knot at the collar, then a blade that widens and ends in a point. Drawn with
    // the original's own two shirt reds.
    for (x, y, ch) in [(12, 21, 'B'), (13, 21, 'B'), (12, 22, 'B'), (13, 22, 'C')] {
        g.set(x, y, ch);
    }
    for y in 23..=36 {
        let wide = (27..=33).contains(&y);
        for x in 12..=(if wide { 14 } else { 13 }) {
            let last = x == if wide { 14 } else { 13 };
            g.set(x, y, if last { 'B' } else { 'C' });
        }
    }
    for (x, y) in [(13, 37), (13, 38)] {
        g.set(x, y, 'B');
    }
    g.set(12, 37, 'C');

    // The black puritan hat: a tall crown, flat on top and a little narrower there, with a
    // grey band and a square gold buckle, and a narrow brim sitting low on the forehead.
    let h = 2;
    for y in 2 + h..=7 + h {
        let (x0, x1) = if y <= 3 + h { (9, 18) } else { (8, 19) };
        for x in x0..=x1 {
            let ch = if x == x0 { 'Z' } else if x == x1 { 'V' } else { 'X' };
            g.set(x, y, ch);
        }
    }
    for x in 9..=18 {
        g.set(x, 2 + h, 'Z');
    }
    for y in 6 + h..=7 + h {
        for x in 8..=19 {
            g.set(x, y, 'Q');
        }
    }
    // The buckle: a square gold ring.
    for (x, y) in [(12, 5), (13, 5), (14, 5), (15, 5), (12, 6), (15, 6), (12, 7), (15, 7), (12, 8), (13, 8), (14, 8), (15, 8)] {
        g.set(x, y + h, 'Y');
    }
    for x in 4..=23 {
        g.set(x, 8 + h, if x < 8 { 'Z' } else if x > 19 { 'V' } else { 'X' });
    }
    for x in 12..=15 {
        g.set(x, 8 + h, 'Y');
    }
    for x in 5..=22 {
        g.set(x, 9 + h, 'V');
    }
    // No stray tufts of hair beside the crown.
    for y in 0..=8 + h {
        for x in (0..8).chain(20..g.w as i32) {
            if "cfi".contains(g.get(x, y)) {
                g.set(x, y, '.');
            }
        }
    }
    // The brim's shadow across the forehead.
    for x in 8..=19 {
        if "adbF".contains(g.get(x, 10 + h)) {
            g.set(x, 10 + h, 'b');
        }
    }

    let top = (0..g.h).find(|&y| g.row(y).iter().any(|&c| c != '.')).unwrap_or(0);
    let rows: Vec<Vec<char>> = (top..g.h).map(|y| g.row(y)).collect();
    Grid::from_rows(rows)
}


// Graypersona: the Gunslinger's slim body and face (the original's own pixels), with dark brown
// curly hair, a feminine face, and a gothic dress: a black laced corset, silver choker and lace,
// long sleeves and a flared skirt with a lace hem, over black boots.

fn gp_default() -> Grid {
    let mut g = soul_default();
    let w = g.w as i32;
    // No hat, and none of the Gunslinger's straight hair.
    for y in 0..=9 {
        for x in 0..w {
            if "ZXVQY".contains(g.get(x, y)) {
                g.set(x, y, '.');
            }
        }
    }
    for y in 0..=33 {
        for x in (0..=10).chain(21..w) {
            if "cfi".contains(g.get(x, y)) {
                g.set(x, y, '.');
            }
        }
    }
    // The forehead's brim shadow goes.
    for x in 11..=20 {
        if g.get(x, 8) == 'b' {
            g.set(x, 8, 'a');
        }
    }

    // The face: lashes and liner, dark red lips
    for (x, y) in [(12, 9), (13, 9), (14, 9), (17, 9), (18, 9), (19, 9)] {
        g.set(x, y, 'V');
    }
    for x in 15..=17 {
        g.set(x, 14, 'C');
        g.set(x, 15, 'B');
    }

    // The dress
    // The olive blazer becomes black: sleeves, body, folds.
    for y in 16..g.h as i32 {
        for x in 0..w {
            match g.get(x, y) {
                'S' => g.set(x, y, 'ê'),
                's' => g.set(x, y, 'ï'),
                'T' => g.set(x, y, 'î'),
                _ => {}
            }
        }
    }
    // Long sleeves end in a silver lace cuff (the blazer's white cuff).
    for y in 21..=36 {
        for x in (0..9).chain(23..w) {
            if g.get(x, y) == 'w' {
                g.set(x, y, if (x + y) % 2 == 0 { 'û' } else { 'ô' });
            }
        }
    }
    // Skin shows at the neck and chest; a silver choker; a lace neckline; then the laced corset.
    for y in 17..=19 {
        for x in 10..=16 {
            g.set(x, y, if x < 14 { 'a' } else { 'b' });
        }
    }
    for x in 11..=16 {
        g.set(x, 17, 'û');
    }
    g.set(13, 18, 'û');
    g.set(13, 19, 'ô');
    for x in 10..=16 {
        g.set(x, 20, if x % 2 == 0 { 'û' } else { 'ô' });
    }
    for y in 21..=36 {
        for x in 10..=16 {
            g.set(x, y, if x == 10 || x == 16 { 'î' } else { 'ê' });
        }
        // Lacing: a silver cord down the middle with a cross-over every fourth row.
        if y % 2 == 1 {
            g.set(13, y, 'û');
        }
        if y % 4 == 2 {
            g.set(12, y, 'ô');
            g.set(14, y, 'ô');
        }
    }
    // What is left of the trousers at the waist.
    for x in 0..w {
        if "mnhj".contains(g.get(x, 35)) {
            g.set(x, 35, 'ê');
        }
    }
    // The flared skirt: the trousers' outline with the gap between the legs filled in, black with
    // fan-shaped folds, down to a silver lace hem; the legs below it are black tights.
    let trousers = "mnhj";
    for y in 36..=47 {
        let xs: Vec<i32> = (0..w).filter(|&x| trousers.contains(g.get(x, y))).collect();
        let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) else { continue };
        for x in lo..=hi {
            let spread = (y - 35) as f32 / 12.0;
            let fold = [-9.0f32, -5.0, -1.0, 3.0, 7.0].iter().any(|k| (13.0 + k * spread * 1.4).round() as i32 == x);
            let shade = [-9.0f32, -5.0, -1.0, 3.0, 7.0].iter().any(|k| (14.0 + k * spread * 1.4).round() as i32 == x);
            let ch = if y == 47 {
                if x % 2 == 0 { 'û' } else { 'ô' }
            } else if x <= lo + 1 && y > 37 {
                'î'
            } else if x >= hi - 1 {
                'ï'
            } else if fold {
                'î'
            } else if shade {
                'ï'
            } else {
                'ê'
            };
            g.set(x, y, ch);
        }
    }
    for y in 48..=49 {
        for x in 0..w {
            if trousers.contains(g.get(x, y)) {
                let left = !trousers.contains(g.get(x - 1, y));
                g.set(x, y, if left { 'î' } else { 'ê' });
            }
        }
    }
    // The mustard cuffs above the boots are black too.
    for y in 48..=51 {
        for x in 0..w {
            if trousers.contains(g.get(x, y)) {
                g.set(x, y, 'ê');
            }
        }
    }
    // Scalloped lace drops from the hem between the legs.
    for x in 0..w {
        if g.get(x, 47) == 'û' && g.get(x, 48) == '.' {
            g.set(x, 48, 'û');
        }
    }

    // Dark brown curly hair
    // A big rounded crown, curtain bangs either side of a middle part, and ringlets over the
    // shoulders, each row a span per side (mirrored about the middle of the face). Wavy edges and
    // a lighter curl and a deep shadow scattered through it.
    let spans: [(i32, i32, i32); 31] = [
        (1, 11, 20), (2, 9, 22), (3, 7, 24), (4, 6, 25), (5, 5, 26), (6, 5, 26), (7, 5, 26), (8, 5, 26),
        (9, 5, 12), (10, 5, 11), (11, 5, 10), (12, 4, 10), (13, 5, 10), (14, 4, 10), (15, 5, 10),
        (16, 4, 10), (17, 5, 10), (18, 4, 9), (19, 5, 9), (20, 4, 9), (21, 5, 9), (22, 4, 9),
        (23, 5, 9), (24, 4, 9), (25, 5, 9), (26, 4, 8), (27, 5, 8), (28, 4, 8), (29, 5, 7), (30, 4, 7), (31, 5, 6),
    ];
    for &(y, x0, x1) in &spans {
        for x in x0..=x1 {
            for xx in [x, 31 - x] {
                // The face and the arm are in front of the hair below the bangs.
                if y >= 9 && "abde".contains(g.get(xx, y)) && (11..=20).contains(&xx) {
                    continue;
                }
                let ch = if (xx + 2 * y) % 7 == 0 {
                    'c'
                } else if (2 * xx + y) % 11 == 0 {
                    '4'
                } else {
                    'i'
                };
                g.set(xx, y, ch);
            }
        }
    }
    // The middle part: a thin line of scalp down the crown.
    for y in 2..=8 {
        g.set(15, y, 'b');
    }
    g
}

/// A sprite read back from a PNG as letters, by exact colour.
fn grid_from_png(path: &str) -> Grid {
    grid_from_png_with(path, &[])
}

/// Like `grid_from_png`, with extra colours (checked first) that stand for a palette letter.
fn grid_from_png_with(path: &str, extra: &[([u8; 3], char)]) -> Grid {
    let img = image::open(path).unwrap().to_rgba8();
    let letters: Vec<char> = "ABCDEFGHIJKLMNOPQRabcdefghijklmnopqrstuvwxyzSTUVWXYZ0123456789$!^*+=<>?;:/()|{}[],-_'\"~`\\tUàèìò".chars().collect();
    let table: Vec<(char, [u8; 3])> = letters.iter().filter_map(|&c| std::panic::catch_unwind(|| color(c)).ok().flatten().map(|rgb| (c, rgb))).collect();
    let mut g = Grid::blank(img.width() as i32, img.height() as i32);
    for y in 0..img.height() {
        for x in 0..img.width() {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                continue;
            }
            let ch = extra.iter().find(|(rgb, _)| rgb[..] == p.0[..3]).map(|(_, c)| *c).or_else(|| table.iter().find(|(_, rgb)| rgb[..] == p.0[..3]).map(|(c, _)| *c)).unwrap_or_else(|| panic!("unknown colour {:?} at {x},{y}", &p.0[..3]));
            g.set(x as i32, y as i32, ch);
        }
    }
    g
}

/// Texture and shade the flat-coloured hair of a hand-edited Gunslinger sprite. The hair is found by
/// its colour; light comes from the upper left like on the rest of the sprite, so each lock has a
/// highlight down its left edge, a shadow down its right edge and under its tip, a dark contact
/// shadow where it lies against the face or jacket, and dashed strands of light and dark between.
fn texture_hair(input: &str, output: &str) {
    const FLAT: [u8; 3] = [119, 95, 59];
    const LIGHT: [u8; 3] = [158, 128, 84];
    const SHADOW: [u8; 3] = [88, 68, 42];
    const DEEP: [u8; 3] = [58, 44, 28];
    let mut img = image::open(input).unwrap().to_rgba8();
    let (w, h) = (img.width() as i32, img.height() as i32);
    let is_hair = |img: &image::RgbaImage, x: i32, y: i32| {
        x >= 0 && y >= 0 && x < w && y < h && {
            let p = img.get_pixel(x as u32, y as u32);
            p[3] > 0 && p.0[..3] == FLAT
        }
    };
    let solid = |img: &image::RgbaImage, x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && img.get_pixel(x as u32, y as u32)[3] > 0;
    let original = img.clone();
    let hash = |a: i32, b: i32| -> u32 {
        let mut v = (a as u32).wrapping_mul(2654435761) ^ (b as u32).wrapping_mul(40503).wrapping_add(7);
        v ^= v >> 13;
        v = v.wrapping_mul(2246822519);
        v ^ (v >> 15)
    };
    for y in 0..h {
        for x in 0..w {
            if !is_hair(&original, x, y) {
                continue;
            }
            let left_open = !is_hair(&original, x - 1, y);
            let right_open = !is_hair(&original, x + 1, y);
            let below_open = !is_hair(&original, x, y + 1);
            let right_touches_body = right_open && solid(&original, x + 1, y);
            // Dashed vertical strands: each column is cut into short segments of its own length.
            let seg = (y + (x * 5) % 4) / 3;
            let strand = hash(x, seg) % 6;
            let tone = if right_touches_body {
                DEEP
            } else if below_open {
                SHADOW
            } else if right_open {
                SHADOW
            } else if left_open {
                if y % 4 == 3 { FLAT } else { LIGHT }
            } else {
                match strand {
                    0 => SHADOW,
                    1 => LIGHT,
                    _ => FLAT,
                }
            };
            img.put_pixel(x as u32, y as u32, image::Rgba([tone[0], tone[1], tone[2], 255]));
        }
    }
    img.save(output).unwrap();
    println!("wrote {output}");
}

// The Scribe's poses (the old Gunslinger's body), drawn from his default pose (`gunslinger_textured.png`)
//
// The same machinery as the Professor's (torso leaned and dropped over legs, arms drawn as tubes),
// with his own joints, sleeves, trousers and hands, and the Professor's arm and leg positions
// moved over to his proportions. His hat is not part of the body: it is a separate overlay that
// the game shows only once the special is ready, so the body is drawn bareheaded.

/// Where his default pose sits on a drawing's canvas.
const GOX: i32 = 20;
const GOY: i32 = 7;
/// His legs start at row 39; the waist of the trousers begins at row 35.
const G_UPPER_LAST: i32 = 39;
const G_LOWER_FIRST: i32 = 35;
const G_TORSO_H: i32 = GOY + G_UPPER_LAST + 1;
const G_LEFT_SHOULDER: (f32, f32) = (4.0, 19.5);
const G_RIGHT_SHOULDER: (f32, f32) = (23.0, 19.5);

#[derive(Clone, Copy, PartialEq)]
enum Hand {
    Fist,
    Palm,
    Point,
    /// A fist holding a quill like a dagger, nib forward.
    Quill,
    /// A fist holding a quill like a pen, nib down and to the left.
    Pen,
}

/// His body without arms or hat, the original arms as cells, his boots and his hat.
fn gs_split(default: &Grid) -> Parts {
    let mut body = default.clone();
    let (mut left_arm, mut right_arm) = (Vec::new(), Vec::new());
    for y in 17..=39i32 {
        for x in 0..default.w as i32 {
            let ch = default.get(x, y);
            let left = (x <= 4 && "SsTwabde".contains(ch)) || ((5..=8).contains(&x) && (23..=31).contains(&y) && "abdew".contains(ch));
            let right = x >= 22 && "SsTwabde".contains(ch);
            if left {
                left_arm.push((x, y, ch));
                body.set(x, y, if x >= 5 { 's' } else { '.' });
            } else if right {
                right_arm.push((x, y, ch));
                body.set(x, y, '.');
            }
        }
    }
    // The hat (rows 0..=7) comes off, and the hair underneath is drawn in: a rounded crown of the
    // same brown, lighter on the left, darker on the right, with a parting down the middle.
    let mut hat = Vec::new();
    for y in 0..=7i32 {
        for x in 0..default.w as i32 {
            let ch = default.get(x, y);
            if "ZXVQY".contains(ch) {
                hat.push((x, y, ch));
                body.set(x, y, '.');
            }
        }
    }
    for (y, x0, x1) in [(2, 10, 16), (3, 9, 17), (4, 8, 18), (5, 7, 19), (6, 7, 19), (7, 7, 19)] {
        for x in x0..=x1 {
            let ch = if x == x0 { '6' } else if x == x1 || x == x1 - 1 && y > 4 { '7' } else if x == 13 { '8' } else if (x + y) % 5 == 0 { '6' } else { '5' };
            body.set(x, y, ch);
        }
    }
    Parts {
        body,
        left_arm,
        right_arm,
        left_shoe: gs_boot(default, 0, 49, 11, 55),
        right_shoe: gs_boot(default, 20, 49, 31, 55),
        hat,
    }
}

fn gs_boot(g: &Grid, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32, char)> {
    let mut cells = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let ch = g.get(x, y);
            if "ZXV".contains(ch) {
                cells.push((x - x0, y - y0, ch));
            }
        }
    }
    cells
}

fn gs_stamp_boot(canvas: &mut Grid, shoe: &[(i32, i32, char)], x: i32, y: i32) {
    for &(dx, dy, ch) in shoe {
        canvas.set(x + dx + GOX, y + dy + GOY, ch);
    }
}

/// His hat, small enough to hold in a hand (12 x 5).
const SMALL_HAT: [&str; 5] = ["...ZXXXXV...", "...ZXXXXV...", "...QQYYQQ...", "ZXXXXXXXXXXV", "..VVVVVVVV.."];

fn gs_hat_in_hand(canvas: &mut Grid, hand: Joint) {
    let (x0, y0) = (hand.0.round() as i32 - 6, hand.1.round() as i32 - 5);
    for (dy, row) in SMALL_HAT.iter().enumerate() {
        for (dx, ch) in row.chars().enumerate() {
            if ch != '.' {
                canvas.set(x0 + dx as i32 + GOX, y0 + dy as i32 + GOY, ch);
            }
        }
    }
}

/// A blazer sleeve (olive, darker on the left and underneath), a white cuff and a bare hand.
fn gs_arm(canvas: &mut Grid, shoulder: Joint, elbow: Joint, hand: Joint, kind: Hand) {
    let wrist = (elbow.0 + (hand.0 - elbow.0) * 0.72, elbow.1 + (hand.1 - elbow.1) * 0.72);
    let upper = tube(&[shoulder, elbow], 7.0);
    let fore = tube(&[elbow, wrist], 6.5);
    let sleeve: HashSet<_> = upper.union(&fore).cloned().collect();
    for &(x, y) in &sleeve {
        let open_left = !sleeve.contains(&(x - 1, y));
        let open_right = !sleeve.contains(&(x + 1, y));
        let open_below = !sleeve.contains(&(x, y + 1));
        let ch = if open_left || open_below || open_right { 's' } else if (x * 3 + y * 5) % 11 == 0 { 'T' } else { 'S' };
        canvas.set(x + GOX, y + GOY, ch);
    }
    // The white cuff at the wrist.
    let (dx, dy) = (hand.0 - elbow.0, hand.1 - elbow.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.01);
    let (ux, uy) = (dx / len, dy / len);
    for &(x, y) in &fore {
        let (rx, ry) = (x as f32 + 0.5 - wrist.0, y as f32 + 0.5 - wrist.1);
        if rx * ux + ry * uy > -1.6 {
            canvas.set(x + GOX, y + GOY, 'w');
        }
    }
    // The hand.
    let mut cells: HashSet<(i32, i32)> = HashSet::new();
    let (hx, hy) = (hand.0.round() as i32, hand.1.round() as i32);
    for (y, row) in HAND_MASK.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                cells.insert((hx - 3 + x as i32, hy - 3 + y as i32));
            }
        }
    }
    match kind {
        Hand::Fist => {}
        // An open hand: the palm stretched out along the arm, for the slap and the throw.
        Hand::Palm => cells.extend(tube(&[hand, (hand.0 + ux * 5.0, hand.1 + uy * 5.0)], 5.0)),
        Hand::Quill | Hand::Pen => {}
        // A finger gun: the index finger out along the arm, the thumb cocked up.
        Hand::Point => {
            cells.extend(tube(&[hand, (hand.0 + ux * 8.0, hand.1 + uy * 8.0)], 2.4));
            cells.insert((hx, hy - 4));
            cells.insert((hx, hy - 5));
            cells.insert((hx + 1, hy - 4));
        }
    }
    for &(x, y) in &cells {
        let ch = if !cells.contains(&(x + 1, y)) || !cells.contains(&(x, y + 1)) { 'b' } else { 'a' };
        canvas.set(x + GOX, y + GOY, ch);
    }
    match kind {
        Hand::Quill => gs_quill(canvas, hand, (ux, uy)),
        Hand::Pen => gs_quill(canvas, hand, (-0.6, 0.8)),
        _ => {}
    }
}

/// A feather quill held in the fist at `hand`: a white shaft with a vane on the back end and a dark
/// nib `dir` ahead of the hand.
fn gs_quill(canvas: &mut Grid, hand: Joint, dir: (f32, f32)) {
    let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt().max(0.01);
    let (dx, dy) = (dir.0 / len, dir.1 / len);
    let (px, py) = (-dy, dx);
    let at = |t: f32, side: f32| ((hand.0 + dx * t + px * side).round() as i32 + GOX, (hand.1 + dy * t + py * side).round() as i32 + GOY);
    let mut t = -7.0;
    while t <= 7.5 {
        let (x, y) = at(t, 0.0);
        canvas.set(x, y, if t >= 5.5 { 'g' } else if t >= 4.0 { 'u' } else { 'w' });
        // the vane: a feather along the back end of the shaft
        if t <= -2.0 {
            let wide = if t <= -3.0 && t >= -6.0 { 2.0 } else { 1.0 };
            for side in [-wide, wide] {
                let (vx, vy) = at(t, side);
                canvas.set(vx, vy, if side < 0.0 { 'v' } else { 'u' });
            }
        }
        t += 0.5;
    }
}

/// Mustard trousers in the default pose's shading: a base tone, a darker edge down the right and
/// along the bottom, a darker cuff and a few light pixels at the knee.
fn gs_leg(canvas: &mut Grid, hip: Joint, knee: Joint, ankle: Joint) {
    let leg = tube(&[hip, knee, ankle], 10.5);
    for &(x, y) in &leg {
        let open_right = !leg.contains(&(x + 1, y));
        let open_below = !leg.contains(&(x, y + 1));
        let ch = if y >= ankle.1.round() as i32 - 1 {
            'j'
        } else if open_below || open_right {
            'n'
        } else {
            'm'
        };
        canvas.set(x + GOX, y + GOY, ch);
    }
    let (kx, ky) = (knee.0.round() as i32, knee.1.round() as i32);
    for (dx, dy) in [(-3, -1), (-2, -1), (-3, 0), (-2, 1)] {
        if canvas.get(kx + dx + GOX, ky + dy + GOY) == 'm' {
            canvas.set(kx + dx + GOX, ky + dy + GOY, 'h');
        }
    }
}

fn gs_pose_arm(spec: ArmSpec, right: bool) -> ArmSpec {
    let dx = if right { -12.5 } else { -3.5 };
    match spec {
        ArmSpec::Keep => ArmSpec::Keep,
        ArmSpec::Pose { elbow, hand } => ArmSpec::Pose { elbow: (elbow.0 + dx, elbow.1 - 1.5), hand: (hand.0 + dx, hand.1 - 1.5) },
    }
}

fn gs_legs(legs: Legs) -> Legs {
    // His legs are longer from the hip (row 39) to the floor (row 48).
    let map = |j: Joint, dx: f32| (j.0 + dx, 39.0 + (j.1 - 41.0) * 9.0 / 7.0);
    match legs {
        Legs::Stand => Legs::Stand,
        Legs::Squat => Legs::Squat,
        Legs::Pose { left, right } => Legs::Pose { left: (map(left.0, -4.0), map(left.1, -4.0)), right: (map(right.0, -9.5), map(right.1, -9.5)) },
    }
}

/// His hands: a quill in the fist for the stabs, an open hand under the book and a pen in the other
/// while he writes, open hands for the slaps.
fn gs_hands(frame: Frame) -> (Hand, Hand) {
    match frame {
        Frame::Soft(1) | Frame::CrouchSoft(1) | Frame::AirSoft(0) | Frame::AirSoft(1) => (Hand::Fist, Hand::Quill),
        Frame::Punch(_) | Frame::Hard(1) | Frame::Hard(2) | Frame::Hard(3) | Frame::AirHard(1) => (Hand::Quill, Hand::Quill),
        Frame::Write(_) => (Hand::Palm, Hand::Pen),
        Frame::Summon(_) | Frame::Speak(_) => (Hand::Fist, Hand::Palm),
        _ => (Hand::Fist, Hand::Fist),
    }
}

/// A drawing that is specific to him, with the arms already in his own coordinates, and where
/// the hat is held (if it is in his hand) or whether it is still on his head.
struct GsSpec {
    spec: Spec,
    hat_in_hand: Option<Joint>,
    hat_on_head: bool,
}

fn gs_spec(frame: Frame) -> GsSpec {
    use ArmSpec::Keep;
    let stance = || legs((8.0, 45.0), (4.0, 48.0), (33.0, 45.0), (36.0, 48.0));
    let own = |arms: (ArmSpec, ArmSpec), lean: f32, dy: i32, legs: Legs| GsSpec {
        spec: Spec { arms, lean, dy, legs, mouth: 0, finger: None },
        hat_in_hand: None,
        hat_on_head: false,
    };
    match frame {
        // The quill stabs: the right hand cocks and strikes, then the left.
        Frame::Punch(0) => own((Keep, pose((25.0, 25.0), (29.0, 19.0))), 0.0, 0, Legs::Stand),
        Frame::Punch(1) => own((Keep, pose((34.0, 21.0), (44.0, 19.0))), 3.0, 0, stance()),
        Frame::Punch(2) => own((pose((4.0, 25.0), (9.0, 18.0)), Keep), 0.0, 0, Legs::Stand),
        Frame::Punch(_) => own((pose((14.0, 22.0), (33.0, 19.0)), Keep), 3.0, 0, stance()),
        // Writing: the book opens in the left hand, the right hand writes (two strokes), then a flourish
        // with the quill held high.
        Frame::Write(0) => own((pose((6.0, 28.0), (11.0, 29.0)), pose((24.0, 26.0), (21.0, 21.0))), 0.0, 0, Legs::Stand),
        Frame::Write(1) => own((pose((6.0, 28.0), (11.0, 29.0)), pose((24.0, 27.0), (18.0, 25.0))), 1.0, 0, Legs::Stand),
        Frame::Write(2) => own((pose((6.0, 28.0), (11.0, 29.0)), pose((25.0, 26.0), (21.0, 24.0))), 1.0, 0, Legs::Stand),
        Frame::Write(_) => own((pose((6.0, 28.0), (11.0, 29.0)), pose((29.0, 16.0), (31.0, 7.0))), 2.0, 0, Legs::Stand),
        // Everything else is the Professor's pose moved over to his proportions.
        other => {
            let mut sp = spec(other);
            if matches!(sp.legs, Legs::Squat) {
                // The Professor's squat is 13 pixels deep; his sprite is a little taller, so he drops 15.
                sp.dy += 2;
            }
            sp.arms = (gs_pose_arm(sp.arms.0, false), gs_pose_arm(sp.arms.1, true));
            GsSpec { spec: sp, hat_in_hand: None, hat_on_head: false }
        }
    }
}

/// The closed book on his belt, on the left hip: a dark spine, a brown cover with a gold clasp and
/// the pages showing along the right edge.
fn gs_belt_book(c: &mut Grid) {
    let rows = ["äääää", "äöööt", "äöüöt", "äööYU", "äöööt", "äääää"];
    for (dy, row) in rows.iter().enumerate() {
        for (dx, ch) in row.chars().enumerate() {
            c.set(5 + dx as i32 + GOX, 34 + dy as i32 + GOY, ch);
        }
    }
}

/// The book open in his hands while he writes (`k` = 0 half open, 1..=3 more and more written on the
/// right page), in front of his chest.
fn gs_open_book(c: &mut Grid, k: usize) {
    let (cx, cy) = (14, 27);
    let hw: i32 = if k == 0 { 4 } else { 7 };
    // the cover, which shows as a brown border below and at each side
    for x in -hw - 1..=hw + 1 {
        c.set(cx + x + GOX, cy + 5 + GOY, 'ä');
    }
    for y in -3..=5 {
        c.set(cx - hw - 1 + GOX, cy + y + GOY, 'ö');
        c.set(cx + hw + 1 + GOX, cy + y + GOY, 'ö');
    }
    // the pages, with a line of writing every other row
    for y in -4..=4 {
        for x in -hw..=hw {
            let page = if x == 0 { 'U' } else { 't' };
            c.set(cx + x + GOX, cy + y + GOY, page);
        }
    }
    for line in 0..3 {
        let y = cy - 2 + line * 2;
        for x in 2..=hw.min(2 + 2 * k as i32) {
            c.set(cx + x + GOX, y + GOY, 'ä');
        }
        // the left page is already full
        if k > 0 {
            for x in -hw + 1..=-1 {
                if (x + line) % 4 != 0 {
                    c.set(cx + x + GOX, y + GOY, 'U');
                }
            }
        }
    }
}

/// The torso, without the hat, and the hat on its own layer (empty when it is not on his head).
fn gs_torso(parts: &Parts, frame: Frame, gs: &GsSpec) -> (Grid, Grid) {
    let mut c = Grid::blank(FRAME_W, G_TORSO_H);
    let mut hat = Grid::blank(FRAME_W, G_TORSO_H);
    for y in 0..=G_UPPER_LAST {
        for x in 0..parts.body.w as i32 {
            let ch = parts.body.get(x, y);
            if ch != '.' {
                c.set(x + GOX, y + GOY, ch);
            }
        }
    }
    if gs.hat_on_head {
        for &(x, y, ch) in &parts.hat {
            hat.set(x + GOX, y + GOY, ch);
        }
    }
    // Speaking: an open mouth in place of the smile.
    let mouth = match frame {
        Frame::Speak(0) => 1,
        Frame::Speak(1) => 2,
        Frame::Hurt(_) | Frame::Ko => 2,
        _ => gs.spec.mouth,
    };
    if mouth > 0 {
        for x in 13..=15 {
            c.set(x + GOX, 14 + GOY, 'g');
        }
        if mouth > 1 {
            c.set(14 + GOX, 15 + GOY, 'g');
        }
    }
    let (kl, kr) = gs_hands(frame);
    for (spec, shoulder, kept, kind) in [(&gs.spec.arms.0, G_LEFT_SHOULDER, &parts.left_arm, kl), (&gs.spec.arms.1, G_RIGHT_SHOULDER, &parts.right_arm, kr)] {
        match spec {
            ArmSpec::Keep => {
                for &(x, y, ch) in kept {
                    c.set(x + GOX, y + GOY, ch);
                }
            }
            ArmSpec::Pose { elbow, hand } => gs_arm(&mut c, shoulder, *elbow, *hand, kind),
        }
    }
    if let Frame::Write(k) = frame {
        gs_open_book(&mut c, k);
    } else {
        gs_belt_book(&mut c);
    }
    if let Some(at) = gs.hat_in_hand {
        gs_hat_in_hand(&mut c, at);
    }
    // His hair falls in front of the shoulders and the sleeves: put it back over them.
    for y in 8..=31 {
        for x in 0..14 {
            let ch = parts.body.get(x, y);
            if "5678".contains(ch) {
                c.set(x + GOX, y + GOY, ch);
            }
        }
    }
    (c, hat)
}

fn gs_leg_layer(parts: &Parts, legs: &Legs) -> Grid {
    let mut c = Grid::blank(FRAME_W, FULL_H);
    let copy_rows = |c: &mut Grid, y0: i32, y1: i32| {
        for y in y0..=y1 {
            for x in 0..parts.body.w as i32 {
                let ch = parts.body.get(x, y);
                if ch != '.' {
                    c.set(x + GOX, y + GOY, ch);
                }
            }
        }
    };
    copy_rows(&mut c, G_LOWER_FIRST, G_UPPER_LAST);
    let (left_hip, right_hip) = ((6.0, 39.0), (21.5, 39.0));
    match legs {
        Legs::Stand => copy_rows(&mut c, G_UPPER_LAST + 1, 55),
        Legs::Pose { left, right } => {
            gs_leg(&mut c, left_hip, left.0, left.1);
            gs_stamp_boot(&mut c, &parts.left_shoe, left.1 .0 as i32 - 5, left.1 .1 as i32 + 1);
            gs_leg(&mut c, right_hip, right.0, right.1);
            gs_stamp_boot(&mut c, &parts.right_shoe, right.1 .0 as i32 - 4, right.1 .1 as i32 + 1);
        }
        Legs::Squat => {
            gs_leg(&mut c, (6.0, 47.0), (-3.0, 44.0), (2.0, 48.0));
            gs_stamp_boot(&mut c, &parts.left_shoe, -3, 49);
            gs_leg(&mut c, (21.5, 47.0), (30.0, 44.0), (25.0, 48.0));
            gs_stamp_boot(&mut c, &parts.right_shoe, 21, 49);
        }
    }
    c
}

/// The body drawing (bareheaded) and the hat overlay for a frame.
fn gs_drawing(parts: &Parts, frame: Frame) -> (Grid, Grid) {
    let gs = gs_spec(frame);
    let legs = gs_legs(match &gs.spec.legs {
        Legs::Stand => Legs::Stand,
        Legs::Squat => Legs::Squat,
        Legs::Pose { left, right } => Legs::Pose { left: *left, right: *right },
    });
    let mut canvas = gs_leg_layer(parts, &legs);
    let (upper, hat_layer) = gs_torso(parts, frame, &gs);
    let mut hat_canvas = Grid::blank(FRAME_W, FULL_H);
    for cy in 0..G_TORSO_H {
        let r = (cy - GOY).clamp(0, G_UPPER_LAST);
        let shift = (gs.spec.lean * (G_UPPER_LAST - r) as f32 / G_UPPER_LAST as f32).round() as i32;
        for x in 0..FRAME_W {
            let (nx, ny) = (x + shift, cy + gs.spec.dy);
            if nx < 0 || nx >= FRAME_W || ny < 0 || ny >= FULL_H {
                continue;
            }
            let ch = upper.get(x, cy);
            if ch != '.' {
                canvas.set(nx, ny, ch);
            }
            let hc = hat_layer.get(x, cy);
            if hc != '.' {
                hat_canvas.set(nx, ny, hc);
            }
        }
    }
    if frame == Frame::Ko {
        let upright = {
            let mut u = canvas.clone();
            for y in 0..FULL_H {
                for x in 0..FRAME_W {
                    let h = hat_canvas.get(x, y);
                    if h != '.' {
                        u.set(x, y, h);
                    }
                }
            }
            u
        };
        return (lying_with(&canvas, &upright), lying_with(&hat_canvas, &upright));
    }
    (canvas, hat_canvas)
}



/// Every frame the Gunslinger has: the common set and his volley.
fn gs_frame_list() -> Vec<Frame> {
    let mut v = Frame::all();
    v.extend((0..t2f::anim::PUNCH_FRAMES).map(Frame::Punch));
    v.extend((0..t2f::anim::WRITE_FRAMES).map(Frame::Write));
    v
}

/// Every drawing of the Scribe as (file name, image): the poses and his effects.
fn gs_frames(default: &Grid) -> Vec<(String, image::RgbaImage)> {
    let parts = gs_split(default);
    let mut frames = Vec::new();
    for f in gs_frame_list() {
        let (body, _hat) = gs_drawing(&parts, f);
        frames.push((f.file_stem(), body.to_png()));
    }
    for (name, make) in scribe_effects() {
        for v in 0..2 {
            frames.push((format!("{name}_{v}"), make(v).to_png()));
        }
    }
    frames
}

fn gs_contact_sheet(default: &Grid, scale: u32) -> image::RgbaImage {
    let parts = gs_split(default);
    let cells: Vec<image::RgbaImage> = gs_frame_list()
        .into_iter()
        .map(|f| {
            gs_drawing(&parts, f).0.to_png()
        })
        .collect();
    let columns = 8u32;
    let (cw, ch) = (FRAME_W as u32 * scale, FULL_H as u32 * scale);
    let rows = (cells.len() as u32 + columns - 1) / columns;
    let mut sheet = image::RgbaImage::from_pixel(columns * (cw + 8) + 8, rows * (ch + 8) + 8 + 60 * scale, image::Rgba([196, 214, 232, 255]));
    for (k, cell) in cells.iter().enumerate() {
        let (col, row) = (k as u32 % columns, k as u32 / columns);
        let (ox, oy) = (8 + col * (cw + 8), 8 + row * (ch + 8));
        for y in 0..ch {
            for x in 0..cw {
                let p = cell.get_pixel(x / scale, y / scale);
                sheet.put_pixel(ox + x, oy + y, if p[3] > 0 { *p } else { image::Rgba([232, 240, 248, 255]) });
            }
        }
    }
    let mut x = 8;
    let y = rows * (ch + 8) + 8;
    for img in scribe_effects().into_iter().flat_map(|(_, make)| [make(0).to_png(), make(1).to_png()]) {
        for yy in 0..img.height() * scale {
            for xx in 0..img.width() * scale {
                let p = img.get_pixel(xx / scale, yy / scale);
                if p[3] > 0 {
                    sheet.put_pixel(x + xx, y + yy, *p);
                }
            }
        }
        x += img.width() * scale + 8 * scale;
    }
    sheet
}


// ---------------------------------------------------------------------------------------------
// The Scribe's effects: the blue particle, a falling knife, the portal and Jesus.

fn scribe_effects() -> Vec<(&'static str, fn(usize) -> Grid)> {
    vec![("orb", scribe_orb), ("knife", scribe_knife), ("portal", scribe_portal), ("jesus", scribe_jesus)]
}

/// The blue particle (12 x 12): a bright core in glowing rings; the second drawing is a size bigger.
fn scribe_orb(v: usize) -> Grid {
    let mut g = Grid::blank(12, 12);
    let r = if v == 0 { 4.6 } else { 5.4 };
    for y in 0..12 {
        for x in 0..12 {
            let d = (((x as f32 - 5.5) * (x as f32 - 5.5)) + ((y as f32 - 5.5) * (y as f32 - 5.5))).sqrt();
            let ch = if d <= 1.6 { 'W' } else if d <= r - 2.4 { '%' } else if d <= r - 1.0 { '#' } else if d <= r { '&' } else { '.' };
            g.set(x, y, ch);
        }
    }
    g
}

/// A knife (18 x 7), heading right: a steel blade, a small guard, a dark handle.
fn scribe_knife(v: usize) -> Grid {
    let mut g = Grid::blank(18, 7);
    for x in 6..18 {
        let half = if x < 14 { 1 } else if x < 16 { 1 } else { 0 };
        for dy in -half..=half {
            g.set(x, 3 + dy, if dy < 0 { 'J' } else if dy == 0 { 'M' } else { 'L' });
        }
    }
    g.set(17, 3, 'W');
    for dy in -2..=2 {
        g.set(5, 3 + dy, 'Y');
    }
    for x in 0..5 {
        g.set(x, 3, 'ö');
        g.set(x, 2, 'ä');
        g.set(x, 4, 'ä');
    }
    // a glint that moves along the blade
    g.set(8 + 4 * v as i32, 2, 'W');
    g
}

/// The portal (44 x 26): a swirl of violet with a dark centre and sparks; it turns between drawings.
fn scribe_portal(v: usize) -> Grid {
    let mut g = Grid::blank(44, 26);
    for y in 0..26 {
        for x in 0..44 {
            let (dx, dy) = ((x as f32 - 21.5) / 21.0, (y as f32 - 12.5) / 12.0);
            let d = dx * dx + dy * dy;
            if d <= 1.0 {
                let angle = dy.atan2(dx) + d * 3.2 + v as f32 * 0.9;
                let arm = (angle * 2.0).sin();
                let ch = if d > 0.82 { '0' } else if d < 0.16 { 'g' } else if arm > 0.45 { '9' } else if arm > -0.25 { '0' } else { 'g' };
                g.set(x, y, ch);
            }
        }
    }
    // the bright rim and a few sparks
    for k in 0..40 {
        let a = k as f32 / 40.0 * std::f32::consts::TAU;
        g.set((21.5 + 20.5 * a.cos()).round() as i32, (12.5 + 11.5 * a.sin()).round() as i32, if (k + v) % 3 == 0 { 'W' } else { '9' });
    }
    for (x, y) in [(3, 4), (40, 6), (38, 21), (6, 20), (21, 1), (22, 24)] {
        g.set(x + v as i32 - 1, y, 'W');
    }
    g
}

/// Jesus (34 x 54), arms spread, a halo over his head, in a white robe with a red sash. The second
/// drawing lets the hem and the halo's glints move.
fn scribe_jesus(v: usize) -> Grid {
    let mut g = Grid::blank(34, 54);
    // the halo: a gold ring over the head
    for k in 0..48 {
        let a = k as f32 / 48.0 * std::f32::consts::TAU;
        g.set((17.0 + 8.5 * a.cos()).round() as i32, (4.0 + 2.8 * a.sin()).round() as i32, if (k + v * 3) % 8 == 0 { 'z' } else { 'Y' });
    }
    // long hair behind the head and shoulders
    for y in 8..=24 {
        let half = if y < 11 { 5 } else if y < 18 { 6 } else { 5 };
        for x in 17 - half..=17 + half {
            g.set(x, y, if x == 17 - half { 'f' } else { 'c' });
        }
    }
    // the face
    for y in 8..=17 {
        let half = if y < 10 { 3 } else if y < 15 { 4 } else { 3 };
        for x in 17 - half..=17 + half {
            g.set(x, y, if x == 17 + half { 'b' } else { 'a' });
        }
    }
    // hair on top of the head: a parting down the middle and a fringe over the forehead
    for (y, half) in [(6, 3), (7, 5), (8, 6), (9, 5)] {
        for x in 17 - half..=17 + half {
            g.set(x, y, if x == 17 { 'i' } else if x < 17 - half + 2 { 'f' } else { 'c' });
        }
    }
    for x in [14, 15, 19, 20] {
        g.set(x, 10, 'c');
    }
    g.set(15, 12, 'g');
    g.set(19, 12, 'g');
    // the beard and a calm mouth
    for y in 15..=19 {
        let half = if y == 19 { 1 } else { 3 };
        for x in 17 - half..=17 + half {
            if y > 15 || x != 17 {
                g.set(x, y, 'c');
            }
        }
    }
    g.set(17, 16, 'v');
    g.set(16, 16, 'v');
    g.set(18, 16, 'v');
    // the robe: shoulders to a wide hem, shaded on the right
    for y in 20..=52 {
        let t = (y - 20) as f32 / 32.0;
        let half = (5.0 + t * 8.5).round() as i32 + if y > 46 && (y + v as i32) % 2 == 0 { 1 } else { 0 };
        for x in 17 - half..=17 + half {
            g.set(x, y, if x >= 17 + half - 1 { 'u' } else if x > 17 + half / 2 { 'v' } else { 'w' });
        }
    }
    // the arms spread wide, sleeves flaring, with open hands
    for (side, tip) in [(-1, 2), (1, 31)] {
        for k in 0..13 {
            let x = 17 + side * (5 + k);
            let y = 22 - k / 3;
            let thick = 2 + k / 5;
            for dy in -thick..=thick {
                g.set(x, y + dy, if dy == thick { 'v' } else { 'w' });
            }
        }
        for dy in -1..=1 {
            g.set(tip, 18 + dy, 'a');
            g.set(tip + side * -1, 18 + dy, 'b');
        }
    }
    // the red sash across the chest
    g.line(12, 21, 24, 40, 'B', None);
    g.line(13, 21, 25, 40, 'C', None);
    g
}
// Cowboy From Hell: the original's own pose and eyes, bald, in a dark brown ribbed tank top, with
// silver earrings and a chain, and big brown cowboy boots

fn cowboy_default() -> Grid {
    cowboy_build(true)
}

/// The cowboy's default pose; without `boots` the legs keep the original sprite's plain shoes.
fn cowboy_build(boots: bool) -> Grid {
    let mut g = Grid::parse(REF);
    // The same extra row of cheek as the others, so every row matches.
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.insert(18, rows[17].clone());
    g = Grid::from_rows(rows);
    // The original's left arm runs off the edge of its image: finish it.
    for (y, from) in [(26, 2), (27, 1), (28, 0), (29, 0), (30, 0), (31, 0), (32, 0), (33, 0), (34, 1), (35, 2)] {
        for x in from..3 {
            let ch = if x == from && from == 0 { 'K' } else { 'D' };
            if g.get(x, y) == '.' {
                g.set(x, y, ch);
            }
        }
    }
    // Finish the shoes, which the original's image cuts off (they become boots below).
    let shoe_row = g.row(g.h - 1);
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'L' } else { c }).collect());
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'Q' } else { '.' }).collect());
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'J' } else { '.' }).collect());
    g = Grid::from_rows(rows);

    // The hands (the original's pale grey cells), remembered so the tattoo leaves them alone.
    let mut hands = std::collections::HashSet::new();
    for y in 25..=46 {
        for x in 0..g.w as i32 {
            if "JMP".contains(g.get(x, y)) {
                hands.insert((x, y));
            }
        }
    }

    // Skin, the hands, the tank top and the trousers.
    g.recolor(0, 99, &[('F', 'a'), ('D', 'b'), ('K', 'd'), ('O', 'e')]);
    g.recolor(25, 46, &[('J', 'a'), ('M', 'b'), ('P', 'a')]);
    // The short sleeves become bare arms; the body becomes a ribbed tank top.
    for y in 20..=24 {
        for x in (0..9).chain(21..g.w as i32) {
            match g.get(x, y) {
                'C' => g.set(x, y, 'a'),
                'B' => g.set(x, y, 'b'),
                _ => {}
            }
        }
    }
    // The tank top's armholes are cut high and clean: everything of the old sleeve (and the old
    // shirt's edge beside the arm) down to the bottom of the armpit becomes bare arm.
    for y in 20..=29 {
        for x in (0..=8).chain(21..g.w as i32) {
            match g.get(x, y) {
                'C' => g.set(x, y, 'a'),
                'B' => g.set(x, y, 'b'),
                _ => {}
            }
        }
    }
    g.recolor(20, 45, &[('C', '$'), ('B', '!')]);
    // A touch of shadow in the armpits, where the arm meets the tank top.
    for y in 24..=28 {
        for x in [8, 9, 20, 21] {
            if "ab".contains(g.get(x, y)) {
                g.set(x, y, 'd');
            }
        }
    }

    // A completely bald head
    // The original's hair goes (E and I, in the head), the glasses frames become skin.
    for y in 0..=19 {
        for x in 0..g.w as i32 {
            if "EI".contains(g.get(x, y)) {
                g.set(x, y, '.');
            }
        }
    }
    // The skull: a rounded dome, lighter on the left and shaded down the right, with a highlight.
    // A little less forehead than the original's: the dome starts a row lower.
    let skull: [(i32, i32, i32); 8] = [(6, 12, 16), (7, 10, 18), (8, 9, 19), (9, 8, 20), (10, 8, 20), (11, 8, 20), (12, 8, 20), (13, 8, 20)];
    for &(y, x0, x1) in &skull {
        for x in x0..=x1 {
            let ch = if x == x0 { 'd' } else if x >= x1 - 1 { 'b' } else if (y == 6 || y == 7) && (12..=14).contains(&x) { 'F' } else { 'a' };
            g.set(x, y, ch);
        }
    }
    // First clear what is left of the original's lower face, so only the new, sharper jaw remains.
    for y in 17..=19 {
        for x in 8..=21 {
            g.set(x, y, '.');
        }
    }
    // The face under it: plain skin around the original's eyes, narrowing to a jaw and chin, with a
    // shaded edge down each side.
    for (y, x0, x1) in [(14, 8, 20), (15, 8, 20), (16, 9, 19), (17, 10, 19), (18, 11, 18), (19, 13, 16)] {
        for x in x0..=x1 {
            g.set(x, y, if x == x0 { 'd' } else if x == x1 { 'b' } else { 'a' });
        }
    }
    // The eyes, at their original size (glint, then iris), under strong dark brows.
    for (x, y, ch) in [(13, 14, 'P'), (14, 14, 'I'), (13, 15, 'P'), (14, 15, 'I'), (17, 14, 'P'), (18, 14, 'I'), (17, 15, 'P'), (18, 15, 'I')] {
        g.set(x, y, ch);
    }
    // No eyebrows.

    // A nose, a stern mouth and a short goatee.
    for (x, y) in [(15, 16), (16, 16)] {
        g.set(x, y, 'd');
    }
    // A relaxed, slightly upturned mouth.
    for (x, y) in [(14, 18), (15, 18), (16, 18)] {
        g.set(x, y, 'e');
    }
    // A small, clean goatee just under the lip.
    for (x, y) in [(15, 19), (16, 19), (15, 20), (16, 20)] {
        g.set(x, y, 'R');
    }
    // Small ears just outside the skull, with a silver earring in each lobe.
    for (x, y) in [(7, 14), (7, 15), (21, 14), (21, 15), (8, 16), (20, 16)] {
        g.set(x, y, if x < 10 { 'd' } else { 'b' });
    }
    for (x, y) in [(7, 16), (21, 16)] {
        g.set(x, y, 'M');
    }
    // The neck.
    for y in 19..=22 {
        for x in 11..=18 {
            if g.get(x, y) == '.' || "$!".contains(g.get(x, y)) {
                g.set(x, y, if x == 11 { 'd' } else { 'b' });
            }
        }
    }

    // The tank top: a round neckline, straps, ribbing, and a silver chain
    // The neckline scoops down in skin, edged with the darker rib.
    for (y, x0, x1) in [(20, 11, 18), (21, 12, 17), (22, 13, 16), (23, 14, 15)] {
        for x in x0..=x1 {
            g.set(x, y, 'a');
        }
        g.set(x0 - 1, y, '!');
        g.set(x1 + 1, y, '!');
    }
    for (x, y) in [(14, 24), (15, 24)] {
        g.set(x, y, '!');
    }
    // The neck sits in the shade of his jaw: a dark band right under the chin, getting lighter
    // lower down, and the whole neck a shade darker than the face.
    for x in 11..=18 {
        for (y, tone) in [(19, 'b'), (20, 'e'), (21, 'd'), (22, 'b'), (23, 'b'), (24, 'b')] {
            if "abd".contains(g.get(x, y)) && !((12..=17).contains(&x) && y == 19) {
                g.set(x, y, tone);
            }
        }
    }
    // Bare shoulders either side of the straps.
    for y in 20..=25 {
        for x in (5..=9).chain(20..=25) {
            if "$!".contains(g.get(x, y)) && (x <= 8 || x >= 21) {
                g.set(x, y, if x < 12 { 'a' } else { 'b' });
            }
        }
    }
    // Ribs: pale vertical lines down the tank top.
    for y in 26..=40 {
        for x in (11..=20).step_by(3) {
            if g.get(x, y) == '$' && y % 3 != 0 {
                g.set(x, y, '^');
            }
        }
    }
    // The chain: a silver V from the neck down to the chest, with a small clasp.
    for (x, y) in [(11, 20), (12, 21), (13, 22), (14, 23), (15, 24), (16, 24), (17, 23), (18, 22), (19, 21), (19, 20)] {
        g.set(x, y, 'M');
    }
    g.set(16, 25, 'J');

    // Big brown cowboy boots
    // Junkhead keeps the original's plain shoes (the Professor's) instead.
    if boots {
        let jeans = "AGHN";
        // The boot shafts run up over the lower leg: the leg from row 47 down becomes leather.
        for y in 47..g.h as i32 {
            for x in 0..g.w as i32 {
                let ch = g.get(x, y);
                if jeans.contains(ch) || "JLMQR".contains(ch) {
                    let right_edge = !jeans.contains(g.get(x + 1, y)) && !"JLMQR*+=".contains(g.get(x + 1, y));
                    let left_edge = !jeans.contains(g.get(x - 1, y)) && !"JLMQR*+=".contains(g.get(x - 1, y));
                    let new = if y == 47 || right_edge { '+' } else if left_edge { '=' } else { '*' };
                    g.set(x, y, new);
                }
            }
        }
        // Toes, a little wider and pointed outward, and a dark sole and heel.
        let last = g.h as i32 - 1;
        for (y, grow) in [(last - 2, 1), (last - 1, 2), (last, 2)] {
            let row = g.row(y as usize);
            let xs: Vec<i32> = row.iter().enumerate().filter(|(_, &c)| "*+=".contains(c)).map(|(x, _)| x as i32).collect();
            // The left boot's toe goes left, the right boot's to the right.
            let left_boot_hi = xs.iter().copied().filter(|&x| x < 14).max();
            let right_boot_lo = xs.iter().copied().filter(|&x| x >= 14).min();
            if let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) {
                for k in 1..=grow {
                    g.set(lo - k, y, '*');
                    g.set(hi + k, y, '*');
                }
            }
            let _ = (left_boot_hi, right_boot_lo);
        }
        for x in 0..g.w as i32 {
            if "*=".contains(g.get(x, last)) {
                g.set(x, last, '+');
            }
        }
        // Spurs: a silver spike sticking out of the outer heel of each boot, with a little rowel.
        let boot_edges: Vec<i32> = (0..g.w as i32).filter(|&x| "*+=".contains(g.get(x, 52))).collect();
        if let (Some(&lo), Some(&hi)) = (boot_edges.first(), boot_edges.last()) {
            for (x, y, ch) in [(lo - 1, 53, 'L'), (lo - 2, 53, 'L'), (lo - 3, 52, 'M'), (lo - 3, 53, 'Q'), (lo - 3, 54, 'M'),
                               (hi + 1, 53, 'L'), (hi + 2, 53, 'L'), (hi + 3, 52, 'M'), (hi + 3, 53, 'Q'), (hi + 3, 54, 'M')] {
                g.set(x, y, ch);
            }
        }
        // Stitching: a light line down the middle of each shaft with a V at the top.
        for boot in [(0, 13), (14, 40)] {
            let xs: Vec<i32> = (boot.0..=boot.1).filter(|&x| "*+=".contains(g.get(x, 50))).collect();
            if let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) {
                let cx = (lo + hi) / 2;
                for y in 50..=53 {
                    g.set(cx, y, '=');
                }
                g.set(cx - 1, 49, '=');
                g.set(cx + 1, 49, '=');
                g.set(cx, 50, '=');
            }
        }
    }

    // A whale tattoo on his left arm (the bent one, on the left of the picture): the bicep and
    // tricep only, from just below the shoulder down to the elbow, not the forearm. It only paints over the arm's skin, inside a one-pixel
    // margin, so the arm keeps its shape: a dark ink outline, a blue body with lighter blue
    // highlights, dark belly lines, a white eye and a dark tail flick.
    // The side of the arm next to the body stays bare: the tattoo is on the outer side only.
    let arm_full: std::collections::HashSet<(i32, i32)> = (25..=30)
        .flat_map(|y| (0..=8).map(move |x| (x, y)))
        .filter(|&(x, y)| "abd".contains(g.get(x, y)) && !hands.contains(&(x, y)))
        .collect();
    let arm: std::collections::HashSet<(i32, i32)> = arm_full.iter().cloned().filter(|&(x, _)| x <= 4).collect();
    let mut tattoo: Vec<(i32, i32)> = arm.iter().cloned().collect();
    tattoo.sort();
    for &(x, y) in &tattoo {
        let edge = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| !arm_full.contains(&(x + dx, y + dy)));
        let ch = if edge {
            'G'
        } else if (x + 2 * y) % 5 == 0 {
            'N'
        } else if y % 3 == 0 && (x + y) % 2 == 0 {
            'G'
        } else {
            'A'
        };
        g.set(x, y, ch);
    }
    // The eye, near the front of the whale (towards the shoulder), and a tail flick at the wrist end.
    if let Some(&(x, y)) = tattoo.iter().filter(|&&(x, y)| y >= 27 && x >= 1 && g.get(x, y) == 'A').min_by_key(|&&(x, y)| (y, x)) {
        g.set(x, y, 'P');
    }
    if let Some(&(x, y)) = tattoo.iter().max_by_key(|&&(x, y)| (y, -x)) {
        g.set(x, y, 'G');
        g.set(x + 1, y.saturating_sub(1), 'G');
    }

    // Grey jeans (below the arms, so the tattoo's blue is untouched).
    g.recolor(38, 99, &[('A', '/'), ('G', '('), ('H', ')'), ('N', '|')]);
    // His own skin tones.
    g.recolor(0, 99, &[('a', '<'), ('b', '>'), ('d', '?'), ('e', ';'), ('F', ':')]);

    let top = (0..g.h).find(|&y| g.row(y).iter().any(|&c| c != '.')).unwrap_or(0);
    let rows: Vec<Vec<char>> = (top..g.h).map(|y| g.row(y)).collect();
    Grid::from_rows(rows)
}

// Cowboy From Hell's poses: the Gunslinger's machinery (same proportions and joints), with his
// own bare arms, grey jeans and boots. The whale tattoo is also an overlay, so it can glow.

/// His default pose gets two blank rows on top so that every coordinate matches the Gunslinger's.
fn cb_default() -> Grid {
    let d = cowboy_default();
    let mut rows: Vec<Vec<char>> = vec![vec!['.'; d.w]; 2];
    rows.extend((0..d.h).map(|y| d.row(y)));
    Grid::from_rows(rows)
}

/// Junkhead's colours from the cowboy's: black tank top (body, dark rib, pale rib) and a red tattoo.
fn junkhead_recolour(img: &mut image::RgbaImage) {
        for p in img.pixels_mut() {
            let new = match (p[0], p[1], p[2]) {
                (84, 66, 60) => [26, 26, 30],
                (56, 42, 40) => [12, 12, 15],
                (114, 94, 86) => [58, 58, 64],
                // The whale tattoo: its ink, body and highlight blues become reds.
                (40, 45, 77) => [84, 16, 22],
                (76, 85, 144) => [176, 40, 44],
                (109, 117, 179) => [226, 88, 84],
                _ => continue,
            };
            p.0 = [new[0], new[1], new[2], p[3]];
        }
}

/// Junkhead's default pose: the finished 1x drawing (`tools/junkhead_final.png`, made
/// from `--junkhead-bare`). Its black tank top and red tattoo are mapped back to the cowboy's
/// tank-top and tattoo letters, so the pose code treats them like the cowboy's (`junkhead_recolour`
/// turns them black and red again when the frames are written).
fn jh_default() -> Grid {
    grid_from_png_with(
        "tools/junkhead_final.png",
        &[([26, 26, 30], '$'), ([12, 12, 15], '!'), ([58, 58, 64], '^'), ([84, 16, 22], 'G'), ([176, 40, 44], 'A'), ([226, 88, 84], 'N')],
    )
}

const CB_SKIN: &str = "<>?;:";
const CB_TATTOO: &str = "GANP";

fn cb_split(default: &Grid) -> Parts {
    let mut body = default.clone();
    let (mut left_arm, mut right_arm) = (Vec::new(), Vec::new());
    for y in 20..=39i32 {
        for x in 0..default.w as i32 {
            let ch = default.get(x, y);
            let skin = CB_SKIN.contains(ch) || CB_TATTOO.contains(ch);
            if x <= 8 && skin {
                left_arm.push((x, y, ch));
                body.set(x, y, '.');
            } else if x >= 21 && skin {
                right_arm.push((x, y, ch));
                body.set(x, y, '.');
            }
        }
    }
    Parts {
        body,
        left_arm,
        right_arm,
        left_shoe: cb_boot(default, 0, 43, 12, 55),
        right_shoe: cb_boot(default, 20, 43, 34, 55),
        hat: Vec::new(),
    }
}

fn cb_boot(g: &Grid, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32, char)> {
    let mut cells = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let ch = g.get(x, y);
            if "*+=LMJQ".contains(ch) {
                cells.push((x - x0, y - y0, ch));
            }
        }
    }
    cells
}

/// A bare arm (skin tones, darker down the left and underneath) and a fist, open hand or finger
/// gun. The left arm's upper part carries the whale tattoo; its cells are returned so the game
/// can make it glow.
fn cb_arm(canvas: &mut Grid, shoulder: Joint, elbow: Joint, hand: Joint, left: bool) -> Vec<(i32, i32)> {
    let upper = tube(&[shoulder, elbow], 7.0);
    let fore = tube(&[elbow, hand], 6.5);
    let mut fist = HashSet::new();
    for (y, row) in HAND_MASK.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                fist.insert((hand.0.round() as i32 - 3 + x as i32, hand.1.round() as i32 - 3 + y as i32));
            }
        }
    }
    let all: HashSet<_> = upper.union(&fore).chain(fist.iter()).cloned().collect();
    for &(x, y) in &all {
        let open_left = !all.contains(&(x - 1, y));
        let open_right = !all.contains(&(x + 1, y));
        let open_below = !all.contains(&(x, y + 1));
        let ch = if fist.contains(&(x, y)) || fore.contains(&(x, y)) {
            if open_right || open_below { '>' } else { '<' }
        } else if open_left || open_below {
            '?'
        } else {
            '>'
        };
        canvas.set(x + GOX, y + GOY, ch);
    }
    let mut tattoo = Vec::new();
    if left {
        // The outer half of the upper arm, between the shoulder and the elbow.
        let (dx, dy) = (elbow.0 - shoulder.0, elbow.1 - shoulder.1);
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        let (ux, uy) = (dx / len, dy / len);
        let mut normal = (-uy, ux);
        if normal.0 > 0.0 {
            normal = (-normal.0, -normal.1);
        }
        let cells: HashSet<(i32, i32)> = upper
            .iter()
            .cloned()
            .filter(|&(x, y)| {
                let (rx, ry) = (x as f32 + 0.5 - shoulder.0, y as f32 + 0.5 - shoulder.1);
                let along = rx * ux + ry * uy;
                let out = rx * normal.0 + ry * normal.1;
                along > 3.0 && along < len - 1.0 && out > 0.6 && !fist.contains(&(x, y))
            })
            .collect();
        let mut sorted: Vec<_> = cells.iter().cloned().collect();
        sorted.sort();
        for &(x, y) in &sorted {
            let edge = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(ax, ay)| !upper.contains(&(x + ax, y + ay)) && false)
                || [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(ax, ay)| !cells.contains(&(x + ax, y + ay)) && upper.contains(&(x + ax, y + ay)) == false);
            let ch = if edge { 'G' } else if (x + 2 * y) % 5 == 0 { 'N' } else { 'A' };
            canvas.set(x + GOX, y + GOY, ch);
            tattoo.push((x, y));
        }
    }
    tattoo
}

/// An index finger stretched out along the forearm (past the fist) and the thumb up like a hammer: a finger gun.
fn cb_finger_gun(canvas: &mut Grid, elbow: Joint, hand: Joint) {
    let (dx, dy) = (hand.0 - elbow.0, hand.1 - elbow.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.01);
    let (ux, uy) = (dx / len, dy / len);
    // the side of the hand that is up (or, for an arm pointing straight up, the outer side)
    let mut n = (-uy, ux);
    if n.1 > 0.0 || (n.1 == 0.0 && n.0 < 0.0) {
        n = (-n.0, -n.1);
    }
    let put = |canvas: &mut Grid, x: f32, y: f32, ch: char| canvas.set(x.round() as i32 + GOX, y.round() as i32 + GOY, ch);
    for t in 3..=8 {
        let (cx, cy) = (hand.0 + ux * t as f32, hand.1 + uy * t as f32);
        let tip = t == 8;
        put(canvas, cx, cy, if tip { '?' } else { '<' });
        put(canvas, cx - n.0 * 0.9, cy - n.1 * 0.9, '>');
    }
    // the thumb, standing up from the knuckles
    for (along, up) in [(1.0f32, 3.0f32), (0.0, 4.0), (0.0, 3.0)] {
        put(canvas, hand.0 + ux * along + n.0 * up, hand.1 + uy * along + n.1 * up, '<');
    }
}

fn cb_leg(canvas: &mut Grid, hip: Joint, knee: Joint, ankle: Joint) {
    let leg = tube(&[hip, knee, ankle], 10.5);
    for &(x, y) in &leg {
        let open_right = !leg.contains(&(x + 1, y));
        let open_below = !leg.contains(&(x, y + 1));
        let ch = if open_below || open_right { ')' } else { '/' };
        canvas.set(x + GOX, y + GOY, ch);
    }
    let (kx, ky) = (knee.0.round() as i32, knee.1.round() as i32);
    for (dx, dy) in [(-3, -1), (-2, -1), (-3, 0), (-2, 1)] {
        if canvas.get(kx + dx + GOX, ky + dy + GOY) == '/' {
            canvas.set(kx + dx + GOX, ky + dy + GOY, '|');
        }
    }
}

/// Where each of his own drawings has the arms and legs (the rest is the Professor's pose moved
/// over to his proportions).
fn cb_spec(frame: Frame) -> GsSpec {
    use ArmSpec::Keep;
    let stance = || legs((8.0, 45.0), (4.0, 48.0), (33.0, 45.0), (36.0, 48.0));
    let own = |arms: (ArmSpec, ArmSpec), lean: f32, dy: i32, legs: Legs| GsSpec {
        spec: Spec { arms, lean, dy, legs, mouth: 0, finger: None },
        hat_in_hand: None,
        hat_on_head: false,
    };
    match frame {
        // Alternating punches: right wind-up and strike, left wind-up and strike.
        Frame::Punch(0) => own((Keep, pose((24.0, 26.0), (21.0, 22.0))), 0.0, 0, Legs::Stand),
        Frame::Punch(1) => own((Keep, pose((36.0, 21.0), (47.0, 20.0))), 4.0, 0, stance()),
        Frame::Punch(2) => own((pose((3.0, 26.0), (6.0, 22.0)), Keep), 0.0, 0, Legs::Stand),
        Frame::Punch(_) => own((pose((13.0, 22.0), (35.0, 19.0)), Keep), 4.0, 0, stance()),
        // The leap: the crouch before it, tucked in the air, then a kick with each leg.
        Frame::Kick(0) => own((Keep, Keep), 2.0, 15, Legs::Squat),
        Frame::Kick(1) => own((pose((1.0, 24.0), (-3.0, 20.0)), pose((30.0, 24.0), (34.0, 20.0))), -1.0, 0, legs((6.0, 43.0), (9.0, 45.0), (36.0, 43.0), (33.0, 45.0))),
        Frame::Kick(2) => own((pose((1.0, 23.0), (-3.0, 19.0)), pose((30.0, 22.0), (34.0, 18.0))), -4.0, 0, legs((10.0, 44.0), (10.0, 46.0), (36.0, 45.0), (47.0, 48.0))),
        Frame::Kick(_) => own((pose((1.0, 23.0), (-3.0, 19.0)), pose((30.0, 22.0), (34.0, 18.0))), -4.0, 0, legs((14.0, 45.0), (30.0, 48.0), (33.0, 44.0), (33.0, 46.0))),
        // The Gun Boost: crouched with both guns pointing at the ground, firing (the flash is added in
        // `cb_drawing_with`), rising from the recoil with the arms trailing down, then shooting down leaning back.
        Frame::Boost(0) => own((pose((1.0, 24.0), (3.0, 29.0)), pose((25.0, 25.0), (27.0, 30.0))), 2.0, 15, Legs::Squat),
        Frame::Boost(1) => own((pose((1.0, 23.0), (3.0, 28.0)), pose((25.0, 24.0), (27.0, 29.0))), 0.0, 13, Legs::Squat),
        Frame::Boost(2) => own((pose((0.0, 28.0), (-1.0, 38.0)), pose((26.0, 29.0), (28.0, 39.0))), 0.0, 0, legs((6.0, 43.0), (9.0, 45.0), (36.0, 43.0), (33.0, 45.0))),
        Frame::Boost(_) => own((pose((7.0, 27.0), (13.0, 38.0)), pose((29.0, 26.0), (37.0, 37.0))), -4.0, 0, legs((10.0, 44.0), (7.0, 47.0), (32.0, 44.0), (30.0, 47.0))),
        // Leaning back as he shoots in the air: a little, more, all the way (the gun hand rising as he goes back).
        Frame::Lean(0) => own((pose((3.0, 26.0), (6.0, 22.0)), pose((33.0, 22.0), (44.0, 19.0))), -1.0, 0, stance()),
        Frame::Lean(1) => own((pose((3.0, 26.0), (6.0, 22.0)), pose((32.0, 20.0), (42.0, 14.0))), -3.0, 0, legs((9.0, 44.0), (5.0, 47.0), (32.0, 44.0), (30.0, 47.0))),
        Frame::Lean(_) => own((pose((3.0, 26.0), (6.0, 22.0)), pose((31.0, 17.0), (40.0, 8.0))), -6.0, 0, legs((10.0, 44.0), (5.0, 46.0), (30.0, 45.0), (25.0, 47.0))),
        other => {
            let mut sp = spec(other);
            if matches!(sp.legs, Legs::Squat) {
                sp.dy += 2;
            }
            sp.arms = (gs_pose_arm(sp.arms.0, false), gs_pose_arm(sp.arms.1, true));
            GsSpec { spec: sp, hat_in_hand: None, hat_on_head: false }
        }
    }
}

fn cb_leg_layer(parts: &Parts, legs: &Legs) -> Grid {
    let mut c = Grid::blank(FRAME_W, FULL_H);
    let copy_rows = |c: &mut Grid, y0: i32, y1: i32| {
        for y in y0..=y1 {
            for x in 0..parts.body.w as i32 {
                let ch = parts.body.get(x, y);
                if ch != '.' {
                    c.set(x + GOX, y + GOY, ch);
                }
            }
        }
    };
    copy_rows(&mut c, G_LOWER_FIRST, G_UPPER_LAST);
    let (left_hip, right_hip) = ((6.0, 39.0), (21.5, 39.0));
    // The boots are tall: their tops are 6 rows above where the Gunslinger's start.
    match legs {
        Legs::Stand => copy_rows(&mut c, G_UPPER_LAST + 1, 55),
        Legs::Pose { left, right } => {
            cb_leg(&mut c, left_hip, left.0, left.1);
            gs_stamp_boot(&mut c, &parts.left_shoe, left.1 .0 as i32 - 5, left.1 .1 as i32 - 5);
            cb_leg(&mut c, right_hip, right.0, right.1);
            gs_stamp_boot(&mut c, &parts.right_shoe, right.1 .0 as i32 - 4, right.1 .1 as i32 - 5);
        }
        Legs::Squat => {
            cb_leg(&mut c, (6.0, 47.0), (-3.0, 44.0), (2.0, 48.0));
            gs_stamp_boot(&mut c, &parts.left_shoe, -3, 43);
            cb_leg(&mut c, (21.5, 47.0), (30.0, 44.0), (25.0, 48.0));
            gs_stamp_boot(&mut c, &parts.right_shoe, 21, 43);
        }
    }
    c
}

/// The body drawing and the tattoo glow overlay for a frame.
fn cb_drawing(parts: &Parts, frame: Frame) -> (Grid, Grid) {
    cb_drawing_with(parts, frame, cb_spec, false)
}

/// A coil of whip on the left hip (picture left, behind him): 5 rows from the belt down, drawn in
/// the torso so it leans and drops with it. Its top-left corner is at (3, 35) in the default pose.
const BELT_COIL: [&str; 5] = [".äää.", "äüöüä", "äö.öä", "äüöüä", ".ääää"];

fn cb_drawing_with(parts: &Parts, frame: Frame, spec_of: fn(Frame) -> GsSpec, belt_whip: bool) -> (Grid, Grid) {
    let gs = spec_of(frame);
    let legs = gs_legs(match &gs.spec.legs {
        Legs::Stand => Legs::Stand,
        Legs::Squat => Legs::Squat,
        Legs::Pose { left, right } => Legs::Pose { left: *left, right: *right },
    });
    let mut canvas = cb_leg_layer(parts, &legs);
    // The torso, with the arms.
    let mut upper = Grid::blank(FRAME_W, FULL_H);
    let mut glow = Grid::blank(FRAME_W, FULL_H);
    for y in 0..=G_UPPER_LAST {
        for x in 0..parts.body.w as i32 {
            let ch = parts.body.get(x, y);
            if ch != '.' {
                upper.set(x + GOX, y + GOY, ch);
            }
        }
    }
    let mouth = match frame {
        Frame::Speak(0) => 1,
        Frame::Speak(1) | Frame::Hurt(_) | Frame::Ko => 2,
        _ => gs.spec.mouth,
    };
    if mouth > 0 {
        for x in 14..=16 {
            upper.set(x + GOX, 14 + GOY, 'g');
        }
        if mouth > 1 {
            upper.set(15 + GOX, 15 + GOY, 'g');
        }
    }
    for (spec, shoulder, kept, left) in [(&gs.spec.arms.0, G_LEFT_SHOULDER, &parts.left_arm, true), (&gs.spec.arms.1, G_RIGHT_SHOULDER, &parts.right_arm, false)] {
        match spec {
            ArmSpec::Keep => {
                for &(x, y, ch) in kept {
                    upper.set(x + GOX, y + GOY, ch);
                    if left && "GAN".contains(ch) {
                        glow.set(x + GOX, y + GOY, 'W');
                    }
                }
            }
            ArmSpec::Pose { elbow, hand } => {
                for (x, y) in cb_arm(&mut upper, shoulder, *elbow, *hand, left) {
                    glow.set(x + GOX, y + GOY, 'W');
                }
            }
        }
    }
    // The finger guns: the shooting hand points a finger along the arm with the thumb up (not Junkhead's).
    if !belt_whip && matches!(frame, Frame::Punch(_) | Frame::Lean(_) | Frame::Boost(_)) {
        let fires = match frame {
            Frame::Punch(0) | Frame::Punch(1) | Frame::Lean(_) => (false, true),
            Frame::Punch(_) => (true, false),
            _ => (true, true),
        };
        for (arm, fires) in [(&gs.spec.arms.0, fires.0), (&gs.spec.arms.1, fires.1)] {
            if let (ArmSpec::Pose { elbow, hand }, true) = (arm, fires) {
                cb_finger_gun(&mut upper, *elbow, *hand);
            }
        }
    }
    if belt_whip {
        for (dy, row) in BELT_COIL.iter().enumerate() {
            for (dx, ch) in row.chars().enumerate() {
                if ch != '.' {
                    upper.set(3 + dx as i32 + GOX, 35 + dy as i32 + GOY, ch);
                }
            }
        }
    }
    // The guns going off at the ground: a hot white and yellow burst under each hand.
    if frame == Frame::Boost(1) {
        for (hx, hy) in [(3, 36), (27, 37)] {
            for (dx, dy, ch) in [(0, 0, 'W'), (-1, 0, 'Y'), (1, 0, 'Y'), (0, 1, 'W'), (-2, 1, 'Y'), (2, 1, 'Y'), (-1, 1, 'W'), (1, 1, 'W'), (0, 2, 'Y'), (-3, 2, 'Y'), (3, 2, 'Y'), (-2, 0, 'Y'), (2, 0, 'Y')] {
                upper.set(hx + dx + GOX, hy + dy + GOY, ch);
            }
        }
    }
    let mut mark = Grid::blank(FRAME_W, FULL_H);
    for cy in 0..FULL_H {
        let r = (cy - GOY).clamp(0, G_UPPER_LAST);
        let shift = (gs.spec.lean * (G_UPPER_LAST - r) as f32 / G_UPPER_LAST as f32).round() as i32;
        for x in 0..FRAME_W {
            let (nx, ny) = (x + shift, cy + gs.spec.dy);
            if nx < 0 || nx >= FRAME_W || ny < 0 || ny >= FULL_H {
                continue;
            }
            let ch = upper.get(x, cy);
            if ch != '.' {
                canvas.set(nx, ny, ch);
            }
            let m = glow.get(x, cy);
            if m != '.' {
                mark.set(nx, ny, m);
            }
        }
    }
    if frame == Frame::Ko {
        return (lying_with(&canvas, &canvas), lying_with(&mark, &canvas));
    }
    (canvas, mark)
}

fn cb_frame_list() -> Vec<Frame> {
    let mut v = Frame::all();
    v.extend((0..t2f::anim::PUNCH_FRAMES).map(Frame::Punch));
    v.extend((0..t2f::anim::KICK_FRAMES).map(Frame::Kick));
    v.extend((0..t2f::anim::BOOST_FRAMES).map(Frame::Boost));
    v.extend((0..t2f::anim::LEAN_FRAMES).map(Frame::Lean));
    v
}

fn cb_frames(default: &Grid) -> Vec<(String, image::RgbaImage)> {
    let parts = cb_split(default);
    let mut frames = Vec::new();
    for f in cb_frame_list() {
        let (body, mark) = cb_drawing(&parts, f);
        frames.push((f.file_stem(), body.to_png()));
        frames.push((format!("{}_mark", f.file_stem()), mark.to_png()));
    }
    frames
}

fn cb_contact_sheet(default: &Grid, scale: u32) -> image::RgbaImage {
    let parts = cb_split(default);
    let cells: Vec<image::RgbaImage> = cb_frame_list().into_iter().map(|f| cb_drawing(&parts, f).0.to_png()).collect();
    let columns = 8u32;
    let (cw, ch) = (FRAME_W as u32 * scale, FULL_H as u32 * scale);
    let rows = (cells.len() as u32 + columns - 1) / columns;
    let mut sheet = image::RgbaImage::from_pixel(columns * (cw + 8) + 8, rows * (ch + 8) + 8, image::Rgba([196, 214, 232, 255]));
    for (k, cell) in cells.iter().enumerate() {
        let (col, row) = (k as u32 % columns, k as u32 / columns);
        let (ox, oy) = (8 + col * (cw + 8), 8 + row * (ch + 8));
        for y in 0..ch {
            for x in 0..cw {
                let p = cell.get_pixel(x / scale, y / scale);
                sheet.put_pixel(ox + x, oy + y, if p[3] > 0 { *p } else { image::Rgba([232, 240, 248, 255]) });
            }
        }
    }
    sheet
}

/// The curse's drawings into `assets/effects/`: a glow in the shape of a rose in bloom (layers of
/// petals fanning out from a bud) round the cursed fighter, and a wide solid blue circle round the
/// whole rose. Both are cut in two vertically (a left and a right half) so each half can be
/// switched off on its own.
fn write_curse_art(assets: &Path) {
    let size = 132u32;
    let (cx, cy) = (size as f32 / 2.0, size as f32 / 2.0);
    let ring_r = 58.0f32;
    let ring_thick = 7.0f32;
    let mut ring = [image::RgbaImage::new(size, size), image::RgbaImage::new(size, size)];
    let mut petals = [image::RgbaImage::new(size, size), image::RgbaImage::new(size, size)];
    // Petal layers, outermost first: (petals, phase, inner radius, outer radius, widest half-angle,
    // body colour, outline colour).
    let layers: [(usize, f32, f32, f32, f32, [u8; 4], [u8; 4]); 3] = [
        (8, 0.0, 22.0, 53.0, 0.47, [74, 146, 255, 128], [28, 88, 222, 205]),
        (8, std::f32::consts::PI / 8.0, 12.0, 40.0, 0.52, [108, 178, 255, 150], [48, 118, 236, 215]),
        (5, 0.35, 4.0, 23.0, 0.78, [150, 208, 255, 175], [78, 148, 246, 225]),
    ];
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let r = (dx * dx + dy * dy).sqrt();
            let side = if dx < 0.0 { 0 } else { 1 };
            // A clear gap runs down the middle, so the left and right halves are visibly apart.
            if dx.abs() < 3.5 {
                continue;
            }
            // The ring: a wide solid band, lighter on the inside edge and darker on the outside.
            let band = r - ring_r;
            if band.abs() <= ring_thick / 2.0 {
                let tone = if band < -ring_thick / 2.0 + 1.3 {
                    [172, 216, 255, 255]
                } else if band > ring_thick / 2.0 - 1.3 {
                    [24, 62, 188, 255]
                } else {
                    [58, 128, 255, 255]
                };
                ring[side].put_pixel(x, y, image::Rgba(tone));
            }
            // The rose: each layer's petals are pointed at the base, widest about two thirds out and
            // rounded at the tip; inner layers sit on top of the outer ones.
            let ang = dy.atan2(dx);
            let mut pixel: Option<[u8; 4]> = None;
            for &(n, phase, r_in, r_out, w_max, body, outline) in layers.iter() {
                if r < r_in || r > r_out {
                    continue;
                }
                let step = std::f32::consts::TAU / n as f32;
                let delta = ((ang - phase) / step).rem_euclid(1.0) - 0.5; // -0.5..0.5 across one petal's slot
                let delta = delta * step; // radians from the petal's middle
                let t = (r - r_in) / (r_out - r_in);
                let half = w_max * (1.0 - ((t - 0.62) / 0.62).powi(2)).max(0.0).sqrt() * (1.0 - 0.35 * t.powi(6));
                if delta.abs() < half {
                    let edge = (half - delta.abs()) * r < 1.6 || r > r_out - 1.6;
                    let vein = delta.abs() < 0.035 && t > 0.15 && t < 0.8;
                    pixel = Some(if edge {
                        outline
                    } else if vein {
                        [body[0].saturating_add(50), body[1].saturating_add(40), 255, body[3].saturating_add(40)]
                    } else {
                        body
                    });
                }
            }
            // The bud in the middle.
            if r < 4.5 {
                pixel = Some(if r < 2.5 { [236, 246, 255, 235] } else { [160, 212, 255, 210] });
            }
            if let Some(p) = pixel {
                petals[side].put_pixel(x, y, image::Rgba(p));
            }
        }
    }
    let folder = assets.join("effects");
    std::fs::create_dir_all(&folder).unwrap();
    for (name, imgs) in [("curse_ring", &ring), ("curse_petals", &petals)] {
        imgs[0].save(folder.join(format!("{name}_left.png"))).unwrap();
        imgs[1].save(folder.join(format!("{name}_right.png"))).unwrap();
    }
}

// Anaran, Arch-Historian: the original's own pose, pale skin and eyes (brown), a pompous high elf
// with long dark crimson hair, red robes with copper armour, and a black cape behind him

fn anaran_default() -> Grid {
    anaran_with_cape(1)
}

fn anaran_with_cape(cape: u8) -> Grid {
    let mut g = Grid::parse(REF);
    // The same extra row of cheek as the others, so every row matches.
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.insert(18, rows[17].clone());
    g = Grid::from_rows(rows);
    for (y, from) in [(26, 2), (27, 1), (28, 0), (29, 0), (30, 0), (31, 0), (32, 0), (33, 0), (34, 1), (35, 2)] {
        for x in from..3 {
            let ch = if x == from && from == 0 { 'K' } else { 'D' };
            if g.get(x, y) == '.' {
                g.set(x, y, ch);
            }
        }
    }
    // Finish the shoes (they become copper boots below).
    let shoe_row = g.row(g.h - 1);
    let mut rows: Vec<Vec<char>> = (0..g.h).map(|y| g.row(y)).collect();
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'L' } else { c }).collect());
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'Q' } else { '.' }).collect());
    rows.push(shoe_row.iter().map(|&c| if "JLMQR".contains(c) { 'J' } else { '.' }).collect());
    g = Grid::from_rows(rows);

    // His pale elven skin is the original's own skin (F, D, K, O); the hands (the original's pale
    // grey cells) become bare hands.
    // The left hand (the original's pale grey cells) is remembered, so the bracer on that arm can sit
    // on the wrist: the part of the forearm just beside the hand.
    let left_hand: Vec<(i32, i32)> = (25..=46)
        .flat_map(|y| (0..=9).map(move |x| (x, y)))
        .filter(|&(x, y)| "JMP".contains(g.get(x, y)))
        .collect();
    g.recolor(25, 46, &[('J', 'F'), ('M', 'D'), ('P', 'F')]);

    // Head: no glasses, brown eyes (the original's), dark crimson hair
    for (c, y) in [
        (13, 13), (14, 13), (12, 14), (12, 15), (15, 14), (15, 15), (13, 16), (14, 16),
        (17, 13), (18, 13), (16, 14), (19, 14), (19, 15), (17, 16), (18, 16),
    ] {
        g.set(c + 7 - 7, y, 'F');
    }
    let eyes = [(14, 14), (14, 15), (18, 14), (18, 15)];
    for y in 0..=19 {
        for x in 0..g.w as i32 {
            // The original's hair goes: his hair is the Gunslinger's, pasted in below.
            match g.get(x, y) {
                'E' => g.set(x, y, '.'),
                'I' if !eyes.contains(&(x, y)) => g.set(x, y, '.'),
                _ => {}
            }
        }
    }
    // Haughty brows, arched high, in the hair's deep colour, and a faint smirk.
    for (x, y) in [(13, 12), (14, 11), (15, 11), (17, 11), (18, 11), (19, 12)] {
        g.set(x, y, '[');
    }
    for (x, y) in [(14, 18), (15, 18), (16, 18)] {
        g.set(x, y, 'O');
    }
    // Red robes with copper armour
    // The original's red shirt is the robe; the short sleeves become the robe's red sleeves for now.
    // A copper band down the front of the robe, a belt with a buckle, and a high red trim.
    for y in 22..=40 {
        for x in [14, 15] {
            if "CB".contains(g.get(x, y)) {
                g.set(x, y, if x == 14 { ',' } else { '\'' });
            }
        }
    }
    for y in 35..=36 {
        for x in 9..=20 {
            if "CB".contains(g.get(x, y)) {
                g.set(x, y, if y == 35 { '"' } else { ',' });
            }
        }
    }
    for (x, y) in [(14, 35), (15, 35), (14, 36), (15, 36)] {
        g.set(x, y, '"');
    }
    // The trousers' blue becomes the long robe's red, with a copper hem.
    g.recolor(37, 58, &[('A', 'C'), ('H', 'B'), ('G', '_'), ('N', '-')]);
    for x in 0..g.w as i32 {
        if "CB_-".contains(g.get(x, 52)) {
            g.set(x, 52, ',');
        }
    }
    // Copper vambraces on the forearms.
    // The left bracer follows the wrist: forearm cells two to four steps from the hand form the band.
    let dist = |x: i32, y: i32| left_hand.iter().map(|&(hx, hy)| (hx - x).abs().max((hy - y).abs())).min().unwrap_or(99);
    for y in 26..=40 {
        for x in 0..=9 {
            let c = g.get(x, y);
            if "FDK".contains(c) && !left_hand.contains(&(x, y)) {
                let d = dist(x, y);
                if (2..=4).contains(&d) {
                    g.set(x, y, if d == 2 || d == 4 { '"' } else { ',' });
                }
            }
        }
    }
    for y in 34..=37 {
        for x in 25..=30 {
            if "FDK".contains(g.get(x, y)) {
                g.set(x, y, if y == 34 || y == 37 { '"' } else { ',' });
            }
        }
    }
    // Copper boots.
    g.recolor(47, 99, &[('J', '"'), ('M', ','), ('L', ','), ('Q', '\''), ('R', '\'')]);

    // The robe is a long robe: the skirt closes the gap between the legs
    for y in 41..=52 {
        let row = g.row(y as usize);
        let xs: Vec<usize> = row.iter().enumerate().filter(|(_, &c)| c != '.').map(|(x, _)| x).collect();
        if let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) {
            for x in lo..=hi {
                if g.get(x as i32, y) == '.' {
                    let ch = if y == 52 { ',' } else if (x as i32 * 2 + y) % 7 == 0 { '_' } else { 'B' };
                    g.set(x as i32, y, ch);
                }
            }
        }
    }

    // The mantle, and the cape: `cape` picks which design (see the options)
    // 1 = a short mantle over the shoulders and a long cape flaring out behind his legs like a bell;
    // 2 = a long open cloak that covers both arms, lined in red, with the robe showing in the front;
    // 3 = just the short mantle (no long cape).
    let centre = 14.5f32;
    if cape == 2 {
        // The cloak: black over both shoulders and arms down to the elbows (a little shorter over
        // the bent left arm), open down the front, with a copper hem, copper edging and a chain.
        for y in 19..=34 {
            let opening = 3.4 + if y < 22 { 1.2 } else { 0.0 };
            for x in 0..g.w as i32 {
                let c = g.get(x, y);
                if c == '.' {
                    continue;
                }
                let dx = (x as f32 + 0.5 - centre).abs();
                let (left, right) = (x <= 9, x >= 20);
                let hem = if left { 31 } else { 33 };
                let covered = (y <= hem && (left || right) && !(left && y >= 22 && "FDKO".contains(c) && x >= 5 && y >= 27)) || (y <= 24 && dx > opening);
                if !covered {
                    continue;
                }
                let tone = if y == hem && (left || right) { ',' } else if (x + y) % 6 == 0 { 'Z' } else if dx > 11.0 || y >= hem - 1 { 'V' } else { 'X' };
                g.set(x, y, tone);
                // The copper edging down the front opening.
                if (left && x == 9 || right && x == 20) && y > 21 && y < hem {
                    g.set(x, y, '"');
                }
            }
        }
        // A zigzag hem: little copper points hanging off the bottom edge.
        for x in 0..g.w as i32 {
            for hem in [31, 33] {
                if g.get(x, hem) == ',' && x % 2 == 0 && g.get(x, hem + 1) != '.' && !"FDKO".contains(g.get(x, hem + 1)) {
                    g.set(x, hem + 1, '\u{27}');
                }
            }
        }
    } else {
        // The short mantle: covers the shoulders and the upper arms, opens in a V at the front, has
        // a copper hem and edging.
        for y in 19..=27 {
            let half = (4.6 - (y as f32 - 20.0) * 0.55).max(0.0);
            for x in 0..g.w as i32 {
                let c = g.get(x, y);
                if c == '.' {
                    continue;
                }
                let hand = "FDKO".contains(c) && y >= 26;
                let dx = (x as f32 + 0.5 - centre).abs();
                // (No hem row over the left arm: it would sit just under the cape like a stray band.)
                if dx > half && !hand && !(y == 27 && x <= 8) {
                    let tone = if y == 27 { ',' } else if (x + y) % 6 == 0 { 'Z' } else if y >= 26 || dx > 12.0 { 'V' } else { 'X' };
                    g.set(x, y, tone);
                } else if dx > half - 1.0 && dx <= half && !hand && y >= 21 {
                    g.set(x, y, '"');
                }
            }
        }
        for x in 0..g.w as i32 {
            if g.get(x, 27) == ',' && x > 8 && x % 3 == 0 && g.get(x, 28) != '.' && !"FDKO".contains(g.get(x, 28)) {
                g.set(x, 28, '\u{27}');
            }
        }
    }
    // The chain at the throat and the brooch, whichever cape it is.
    // The chain's links and the ruby brooch are mirror images either side of the middle of the neck.
    for x in [10, 12, 14, 15, 17, 19] {
        if g.get(x, 21) != '.' {
            g.set(x, 21, '"');
        }
    }
    for (x, y, ch) in [(14, 22, ','), (15, 22, ','), (14, 23, '-'), (15, 23, '-'), (14, 24, ','), (15, 24, ',')] {
        g.set(x, y, ch);
    }

    // The picture gets room on both sides (for the cape's flare, and for arms in the poses).
    let (pad_left, pad_right) = (3usize, 6usize);
    let rows: Vec<Vec<char>> = (0..g.h)
        .map(|y| {
            let mut r = vec!['.'; pad_left];
            r.extend(g.row(y));
            r.extend(std::iter::repeat('.').take(pad_right));
            r
        })
        .collect();
    g = Grid::from_rows(rows);
    if cape == 1 {
        // The long cape, seen from the front: mostly hidden behind him, showing as a black bell that
        // flares out from behind the arms and the robe down to a zigzag hem at the shins, with
        // folds in it, and filling any gap beside the arms.
        for y in 30..=51 {
            let row = g.row(y as usize);
            let xs: Vec<usize> = row.iter().enumerate().filter(|(_, &c)| c != '.').map(|(x, _)| x as usize).collect();
            let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) else { continue };
            let t = (y - 30) as f32 / 21.0;
            let flare = (t * 4.0).round() as i32 + 1;
            // Beside the arms nothing sticks out to the left either; below the left arm the cape fills the
            // space between the arm's outer edge and the robe, and from the hips down it flares.
            if y <= 42 {
                for x in 3..lo as i32 {
                    if g.get(x, y) == '.' {
                        g.set(x, y, if (x + y) % 5 == 0 { 'Z' } else if x == 3 { 'V' } else { 'X' });
                    }
                }
            }
            for k in 1..=flare {
                let tone = |x: i32| if k == flare { 'V' } else if (x + y) % 5 == 0 { 'Z' } else { 'X' };
                let (lx, rx) = (lo as i32 - k, hi as i32 + k);
                if y >= 43 && g.get(lx, y) == '.' {
                    g.set(lx, y, tone(lx));
                }
                // Nothing to the right of the right arm: the flare only starts below it.
                if y >= 43 && g.get(rx, y) == '.' {
                    g.set(rx, y, tone(rx));
                }
            }
        }
        for y in 24..=50 {
            let row = g.row(y as usize);
            let xs: Vec<usize> = row.iter().enumerate().filter(|(_, &c)| c != '.').map(|(x, _)| x).collect();
            if let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) {
                for x in lo..=hi {
                    if g.get(x as i32, y) == '.' {
                        g.set(x as i32, y, if (x as i32 + y) % 6 == 0 { 'Z' } else { 'V' });
                    }
                }
            }
        }
        // The zigzag hem: every other black cell on the last two rows goes.
        for x in 0..g.w as i32 {
            for (y, phase) in [(51, 0), (50, 1)] {
                if "XVZ".contains(g.get(x, y)) && (x + phase) % 2 == 0 && y == 51 {
                    g.set(x, y, '.');
                }
            }
        }
    }

    // His hair is the Gunslinger's, as drawn and textured by hand, in dark crimson
    let gunslinger = grid_from_png("tools/gunslinger_textured.png");
    let hair = gs_split(&gunslinger).body;
    for y in 0..hair.h as i32 {
        for x in 0..hair.w as i32 {
            let tone = match hair.get(x, y) {
                '5' => '{',
                '6' => '}',
                '7' => '[',
                '8' => ']',
                _ => continue,
            };
            // The Gunslinger's drawing is cropped 4 rows lower than this one, and this one has been
            // given 3 columns of room on the left for the cape.
            g.set(x + 3, y + 4, tone);
        }
    }
    // Pointed elven ears, swept up and out, in front of the hair.
    for (x, y, ch) in [
        (10, 14, 'K'), (10, 15, 'D'), (9, 13, 'K'), (9, 14, 'K'), (8, 12, 'K'), (8, 13, 'D'), (7, 11, 'D'),
        (20, 14, 'K'), (20, 15, 'D'), (21, 13, 'K'), (21, 14, 'K'), (22, 12, 'K'), (22, 13, 'D'), (23, 11, 'D'),
    ] {
        g.set(x + 3, y, ch);
    }

    // No holes in the face: whatever the hair and the glasses frames left empty between the hair and
    // the cheek is filled, in the hair's deep tone next to the hair and in skin on the face.
    for y in 8..=19 {
        for x in 11..=22 {
            if g.get(x, y) == '.' {
                g.set(x, y, if x <= 12 { '{' } else if x <= 14 { 'D' } else { 'F' });
            }
        }
    }

    let top = (0..g.h).find(|&y| g.row(y).iter().any(|&c| c != '.')).unwrap_or(0);
    let rows: Vec<Vec<char>> = (top..g.h).map(|y| g.row(y)).collect();
    Grid::from_rows(rows)
}

/// Small icons into `assets/effects/`: the shield with a red cross that shows over a fighter whose
/// block is disabled.
fn write_icons(assets: &Path) {
    let (w, h) = (20u32, 24u32);
    let mut img = image::RgbaImage::new(w, h);
    let shield = |x: i32, y: i32| -> bool {
        // Wide at the top, rounded shoulders, narrowing to a point at the bottom.
        let (fx, fy) = (x as f32 - 9.5, y as f32);
        let half = if fy < 12.0 { 8.5 - (0.0f32).max(1.5 - fy) } else { 8.5 * (1.0 - ((fy - 12.0) / 11.5).powf(1.6)) };
        fy >= 0.0 && fy <= 22.5 && fx.abs() <= half
    };
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            if !shield(x, y) {
                continue;
            }
            let edge = !shield(x - 1, y) || !shield(x + 1, y) || !shield(x, y - 1) || !shield(x, y + 1);
            let tone = if edge { [38, 42, 58, 255] } else if x < 7 { [176, 186, 202, 255] } else if x > 12 { [112, 122, 142, 255] } else { [148, 158, 176, 255] };
            img.put_pixel(x as u32, y as u32, image::Rgba(tone));
        }
    }
    // The red cross-out, thick, with a dark edge so it reads on anything.
    for k in 0..16 {
        for (a, b) in [(2 + k, 3 + k), (17 - k, 3 + k)] {
            for (dx, dy) in [(-1, 0), (0, 0), (1, 0), (0, 1)] {
                let (x, y) = (a + dx, b + dy);
                if x >= 0 && y >= 0 && (x as u32) < w && (y as u32) < h {
                    img.put_pixel(x as u32, y as u32, image::Rgba([232, 40, 44, 255]));
                }
            }
        }
    }
    let folder = assets.join("effects");
    std::fs::create_dir_all(&folder).unwrap();
    img.save(folder.join("block_broken.png")).unwrap();
}

// Anaran's poses, drawn from the hand-edited default pose (`tools/anaran_final.png`)
//
// His picture is the standard frame moved 3 columns right (room for the cape) and 1 row up, so
// canvas = grid + (AOX, AOY). Joints are given in the Gunslinger's frame (as everywhere else) and
// converted with `an_j`. The cape, the mantle and the hair are part of the body; arms are drawn
// bare (skin with a copper bracer and, for the light attack, a scroll) and the mantle and the hair
// are stamped back over them. His robe is a skirt: legs are drawn as robe tubes with the gap
// between them filled, so it reads as a robe whatever the feet do.

const AOX: i32 = GOX - 3;
const AOY: i32 = GOY + 1;

/// A point in the Gunslinger's frame as a point in his picture's grid.
fn an_j(j: Joint) -> Joint {
    (j.0 + 3.0, j.1 - 1.0)
}

const AN_SKIN: &str = "FDKO";
const AN_COPPER: &str = ",\"'~`\\";

#[derive(Clone, Copy, PartialEq)]
enum AnHand {
    Fist,
    Palm,
    Scroll,
}

fn an_default() -> Grid {
    grid_from_png("tools/anaran_final.png")
}

fn an_split(default: &Grid) -> Parts {
    let mut body = default.clone();
    let (mut left_arm, mut right_arm) = (Vec::new(), Vec::new());
    for y in 21..=37i32 {
        for x in 0..default.w as i32 {
            let ch = default.get(x, y);
            let skin = AN_SKIN.contains(ch);
            let copper = AN_COPPER.contains(ch);
            let left = x <= 10 && y <= 30 && (skin || (copper && y >= 23));
            let right = x >= 25 && ((skin && y >= 21) || (copper && x >= 28 && y >= 29));
            if left {
                left_arm.push((x, y, ch));
                body.set(x, y, '.');
            } else if right {
                right_arm.push((x, y, ch));
                body.set(x, y, '.');
            }
        }
    }
    // The boots, relative to the left and right halves of his picture.
    let boot = |x0: i32, x1: i32| {
        let mut cells = Vec::new();
        for y in 48..=54 {
            for x in x0..=x1 {
                let ch = default.get(x, y);
                if "~`\\".contains(ch) || (ch == ',' && y >= 49) {
                    cells.push((x - x0, y - 48, ch));
                }
            }
        }
        cells
    };
    Parts { body, left_arm, right_arm, left_shoe: boot(3, 17), right_shoe: boot(23, 37), hat: Vec::new() }
}

/// A bare arm (pale skin, darker down the left and underneath), a copper bracer near the wrist,
/// and the hand: a fist, an open palm, or a fist round a scroll whose ribbon snaps out. Returns
/// nothing; the mantle and the hair are put back over the top afterwards.
fn an_arm(canvas: &mut Grid, shoulder: Joint, elbow: Joint, hand: Joint, kind: AnHand) {
    let (shoulder, elbow, hand) = (an_j(shoulder), an_j(elbow), an_j(hand));
    let upper = tube(&[shoulder, elbow], 7.0);
    let fore = tube(&[elbow, hand], 6.5);
    let mut fist = HashSet::new();
    for (y, row) in HAND_MASK.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                fist.insert((hand.0.round() as i32 - 3 + x as i32, hand.1.round() as i32 - 3 + y as i32));
            }
        }
    }
    let all: HashSet<_> = upper.union(&fore).chain(fist.iter()).cloned().collect();
    // The bracer: the forearm cells about a third of the way from the wrist.
    let (dx, dy) = (hand.0 - elbow.0, hand.1 - elbow.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.01);
    let (ux, uy) = (dx / len, dy / len);
    for &(x, y) in &all {
        let open_left = !all.contains(&(x - 1, y));
        let open_right = !all.contains(&(x + 1, y));
        let open_below = !all.contains(&(x, y + 1));
        let mut ch = if fist.contains(&(x, y)) || fore.contains(&(x, y)) {
            if open_right || open_below { 'D' } else { 'F' }
        } else if open_left || open_below {
            'K'
        } else {
            'D'
        };
        if fore.contains(&(x, y)) && !fist.contains(&(x, y)) {
            let along = (x as f32 + 0.5 - elbow.0) * ux + (y as f32 + 0.5 - elbow.1) * uy;
            if along > len * 0.45 && along < len * 0.8 {
                ch = if along < len * 0.5 || along > len * 0.75 { '"' } else { ',' };
            }
        }
        canvas.set(x + AOX, y + AOY, ch);
    }
    match kind {
        AnHand::Fist => {}
        AnHand::Palm => {
            for (x, y) in tube(&[hand, (hand.0 + ux * 5.0, hand.1 + uy * 5.0)], 5.0) {
                canvas.set(x + AOX, y + AOY, 'F');
            }
        }
        AnHand::Scroll => {
            // A rolled sheet of papyrus held crosswise in his fist: light, shaded underneath, the
            // spiral of the sheet showing at both ends, and a round red wax seal in the middle with
            // two short red ribbon tails.
            let (px, py) = (-uy, ux);
            let c = (hand.0 + ux * 1.5, hand.1 + uy * 1.5);
            let (a, b) = ((c.0 - px * 6.0, c.1 - py * 6.0), (c.0 + px * 6.0, c.1 + py * 6.0));
            let roll = tube(&[a, b], 5.4);
            for &(x, y) in &roll {
                let shade = !roll.contains(&(x, y + 1)) || !roll.contains(&(x + 1, y));
                canvas.set(x + AOX, y + AOY, if shade { 'U' } else { 't' });
            }
            for end in [a, b] {
                let (cx, cy) = (end.0.round() as i32, end.1.round() as i32);
                for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
                    if roll.contains(&(cx + dx, cy + dy)) {
                        canvas.set(cx + dx + AOX, cy + dy + AOY, if dx == 0 && dy == 0 { '\u{27}' } else { 'U' });
                    }
                }
            }
            // the wax seal
            let (sx, sy) = (c.0.round() as i32, c.1.round() as i32);
            for dy in -2..=2i32 {
                for dx in -2..=2i32 {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d <= 2.3 {
                        canvas.set(sx + dx + AOX, sy + dy + AOY, if d > 1.6 { 'B' } else { 'C' });
                    }
                }
            }
            canvas.set(sx - 1 + AOX, sy - 1 + AOY, '-');
            // two ribbon tails
            for k in 3..=5 {
                canvas.set(sx + AOX + (ux * k as f32 * 0.8).round() as i32, sy + AOY + (uy * k as f32 * 0.8).round() as i32 + 1, 'B');
                canvas.set(sx + AOX + (ux * k as f32 * 0.8).round() as i32 + 1, sy + AOY + (uy * k as f32 * 0.8).round() as i32 + 2, 'C');
            }
        }
    }
}

/// A leg of the long robe: a tube of red (darker down the right and along the bottom) with the copper
/// hem at the ankle. The boot is stamped on afterwards.
fn an_leg(canvas: &mut Grid, hip: Joint, knee: Joint, ankle: Joint) {
    let (hip, knee, ankle) = (an_j(hip), an_j(knee), an_j(ankle));
    let leg = tube(&[hip, knee, ankle], 13.0);
    for &(x, y) in &leg {
        let open_right = !leg.contains(&(x + 1, y));
        let open_below = !leg.contains(&(x, y + 1));
        let ch = if y >= ankle.1.round() as i32 { ',' } else if open_below || open_right { 'B' } else { 'C' };
        canvas.set(x + AOX, y + AOY, ch);
    }
}

fn an_stamp_boot(canvas: &mut Grid, shoe: &[(i32, i32, char)], x: i32, y: i32) {
    for &(dx, dy, ch) in shoe {
        canvas.set(x + dx + GOX, y + dy + GOY, ch);
    }
}

fn an_hands(frame: Frame) -> (AnHand, AnHand) {
    match frame {
        Frame::Punch(1) => (AnHand::Fist, AnHand::Scroll),
        Frame::Punch(3) => (AnHand::Scroll, AnHand::Fist),
        Frame::Punch(0) => (AnHand::Fist, AnHand::Scroll),
        Frame::Punch(2) => (AnHand::Scroll, AnHand::Fist),
        Frame::Hard(_) => (AnHand::Palm, AnHand::Fist),
        Frame::Staff(_) => (AnHand::Palm, AnHand::Fist),
        Frame::Read(3) => (AnHand::Fist, AnHand::Scroll),
        Frame::Summon(_) => (AnHand::Palm, AnHand::Palm),
        _ => (AnHand::Fist, AnHand::Fist),
    }
}

/// His own drawings' arms and legs (in the Gunslinger's frame); everything else is the Professor's
/// pose moved over to his proportions.
fn an_spec(frame: Frame) -> GsSpec {
    use ArmSpec::Keep;
    let stance = || legs((8.0, 45.0), (4.0, 48.0), (33.0, 45.0), (36.0, 48.0));
    let own = |arms: (ArmSpec, ArmSpec), lean: f32, dy: i32, legs: Legs, mouth: u8| GsSpec {
        spec: Spec { arms, lean, dy, legs, mouth, finger: None },
        hat_in_hand: None,
        hat_on_head: false,
    };
    match frame {
        // The scroll whip: right arm winds up and snaps, then the left.
        Frame::Punch(0) => own((Keep, pose((26.0, 24.0), (23.0, 20.0))), 0.0, 0, Legs::Stand, 0),
        Frame::Punch(1) => own((Keep, pose((36.0, 22.0), (44.0, 21.0))), 3.0, 0, stance(), 0),
        Frame::Punch(2) => own((pose((4.0, 25.0), (8.0, 20.0)), Keep), 0.0, 0, Legs::Stand, 0),
        Frame::Punch(_) => own((pose((14.0, 22.0), (32.0, 20.0)), Keep), 3.0, 0, stance(), 0),
        // The dramatic pronouncement: he draws breath, flings his arms wide, then thrusts the right
        // hand out as the runes leave, mouth open all the while.
        Frame::Hard(0) => own((Keep, pose((30.0, 15.0), (32.0, 7.0))), -2.0, 0, Legs::Stand, 2),
        Frame::Hard(1) => own((pose((0.0, 22.0), (-5.0, 16.0)), pose((38.0, 22.0), (44.0, 16.0))), -1.0, 0, Legs::Stand, 2),
        Frame::Hard(2) => own((Keep, pose((38.0, 20.0), (50.0, 19.0))), 5.0, 0, stance(), 2),
        Frame::Hard(_) => own((Keep, pose((34.0, 24.0), (41.0, 22.0))), 3.0, 0, stance(), 1),
        // Reading a scroll: he unrolls it between his hands, reads it (the scroll glows, his
        // mouth moves, he leans a little), then rolls it up again. He stands still while he reads.
        Frame::Read(0) => own((pose((7.0, 27.0), (18.0, 23.0)), pose((33.0, 27.0), (28.0, 23.0))), 1.0, 0, Legs::Stand, 0),
        Frame::Read(1) => own((pose((6.0, 27.0), (14.0, 24.0)), pose((36.0, 28.0), (33.0, 24.0))), 2.0, 0, Legs::Stand, 2),
        Frame::Read(2) => own((pose((6.0, 27.0), (14.0, 24.0)), pose((36.0, 28.0), (33.0, 24.0))), 3.0, 0, Legs::Stand, 1),
        Frame::Read(_) => own((Keep, pose((30.0, 24.0), (34.0, 21.0))), 2.0, 0, Legs::Stand, 1),
        // The staff slam: raised, swung back over his shoulder, slammed on the ground in front of
        // him, then leaned on.
        Frame::Staff(0) => own((Keep, pose((30.0, 15.0), (32.0, 7.0))), -2.0, 0, Legs::Stand, 2),
        Frame::Staff(1) => own((Keep, pose((28.0, 11.0), (30.0, 2.0))), -4.0, 0, Legs::Stand, 2),
        Frame::Staff(2) => own((Keep, pose((38.0, 23.0), (46.0, 30.0))), 5.0, 1, stance(), 2),
        Frame::Staff(_) => own((Keep, pose((36.0, 24.0), (42.0, 29.0))), 2.0, 0, stance(), 0),
        other => {
            let mut sp = spec(other);
            if matches!(sp.legs, Legs::Squat) {
                sp.dy += 2;
            }
            sp.arms = (gs_pose_arm(sp.arms.0, false), gs_pose_arm(sp.arms.1, true));
            GsSpec { spec: sp, hat_in_hand: None, hat_on_head: false }
        }
    }
}

/// Where the ends of his staff are, relative to his right hand, in the frames that use it (his
/// heavy attacks): raised, swung forward, thrust out at the opponent, then planted.
fn an_staff(frame: Frame) -> Option<((f32, f32), (f32, f32))> {
    match frame {
        Frame::Hard(0) => Some(((-2.0, 14.0), (3.0, -12.0))),
        Frame::Hard(1) => Some(((-10.0, 10.0), (8.0, -14.0))),
        Frame::Hard(2) => Some(((-18.0, 3.0), (10.0, -1.0))),
        Frame::Hard(_) => Some(((-1.0, 17.0), (2.0, -14.0))),
        Frame::AirHard(0) => Some(((-3.0, 12.0), (3.0, -8.0))),
        Frame::AirHard(_) => Some(((3.0, 14.0), (-2.0, -14.0))),
        Frame::Staff(0) => Some(((-2.0, 14.0), (3.0, -12.0))),
        Frame::Staff(1) => Some(((7.0, 12.0), (-9.0, -13.0))),
        // planted on the floor in front of him
        Frame::Staff(2) => Some(((1.0, 24.0), (-1.0, -10.0))),
        Frame::Staff(_) => Some(((0.0, 23.0), (3.0, -11.0))),
        _ => None,
    }
}

/// A copper-shod wooden shaft with a golden orb in a copper claw at its head.
fn an_draw_staff(canvas: &mut Grid, hand: Joint, bottom: (f32, f32), top: (f32, f32)) {
    let (b, t) = ((hand.0 + bottom.0, hand.1 + bottom.1), (hand.0 + top.0, hand.1 + top.1));
    let shaft = tube(&[b, t], 2.2);
    for &(x, y) in &shaft {
        let shadow = !shaft.contains(&(x + 1, y)) || !shaft.contains(&(x, y + 1));
        canvas.set(x + AOX, y + AOY, if shadow { '\'' } else { ',' });
    }
    // the shoe at the foot
    for (x, y) in tube(&[b, (b.0 + (t.0 - b.0) * 0.12, b.1 + (t.1 - b.1) * 0.12)], 2.6) {
        canvas.set(x + AOX, y + AOY, '`');
    }
    // the claw and the orb
    let (dx, dy) = (t.0 - b.0, t.1 - b.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.01);
    let (ux, uy) = (dx / len, dy / len);
    let orb = (t.0 + ux * 3.0, t.1 + uy * 3.0);
    for (x, y) in tube(&[(t.0 - ux, t.1 - uy), (t.0 + ux * 1.5, t.1 + uy * 1.5)], 4.2) {
        canvas.set(x + AOX, y + AOY, '~');
    }
    for y in -4..=4i32 {
        for x in -4..=4i32 {
            let (cx, cy) = (orb.0.round() as i32 + x, orb.1.round() as i32 + y);
            let d = ((x * x + y * y) as f32).sqrt();
            if d <= 2.8 {
                canvas.set(cx + AOX, cy + AOY, if d <= 1.0 { 'W' } else if d <= 2.0 { 'z' } else { 'y' });
            }
        }
    }
}

fn an_leg_layer(parts: &Parts, legs: &Legs) -> Grid {
    let mut c = Grid::blank(FRAME_W, FULL_H);
    let copy_rows = |c: &mut Grid, y0: i32, y1: i32| {
        for y in y0..=y1 {
            for x in 0..parts.body.w as i32 {
                let ch = parts.body.get(x, y);
                if ch != '.' {
                    c.set(x + AOX, y + AOY, ch);
                }
            }
        }
    };
    // Rows 34..=38 of his picture are the waist (they overlap the torso's last rows).
    copy_rows(&mut c, 34, 38);
    match legs {
        Legs::Stand => copy_rows(&mut c, 39, 55),
        Legs::Pose { left, right } => {
            an_leg(&mut c, (6.0, 39.0), left.0, left.1);
            an_leg(&mut c, (21.5, 39.0), right.0, right.1);
            // The robe is one garment: fill between the legs, below the hips.
            for y in 36..=(AOY + 49) {
                let row = c.row(y as usize);
                let xs: Vec<usize> = row.iter().enumerate().filter(|(_, &ch)| ch != '.').map(|(x, _)| x).collect();
                if let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) {
                    for x in lo..=hi {
                        if c.get(x as i32, y) == '.' {
                            c.set(x as i32, y, 'B');
                        }
                    }
                }
            }
            an_stamp_boot(&mut c, &parts.left_shoe, left.1 .0 as i32 - 5, left.1 .1 as i32 + 1);
            an_stamp_boot(&mut c, &parts.right_shoe, right.1 .0 as i32 - 4, right.1 .1 as i32 + 1);
        }
        Legs::Squat => {
            // Crouching: the robe spreads out round his feet like a bell.
            an_leg(&mut c, (6.0, 47.0), (-3.0, 44.0), (2.0, 48.0));
            an_leg(&mut c, (21.5, 47.0), (30.0, 44.0), (25.0, 48.0));
            for y in 40..=(AOY + 49) {
                let row = c.row(y as usize);
                let xs: Vec<usize> = row.iter().enumerate().filter(|(_, &ch)| ch != '.').map(|(x, _)| x).collect();
                if let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) {
                    for x in lo..=hi {
                        if c.get(x as i32, y) == '.' {
                            c.set(x as i32, y, 'B');
                        }
                    }
                }
            }
            an_stamp_boot(&mut c, &parts.left_shoe, -3, 49);
            an_stamp_boot(&mut c, &parts.right_shoe, 21, 49);
        }
    }
    c
}

/// The body drawing and the glow overlay (the ruby at his throat) for a frame.
fn an_drawing(parts: &Parts, frame: Frame) -> (Grid, Grid) {
    let gs = an_spec(frame);
    let legs = gs_legs(match &gs.spec.legs {
        Legs::Stand => Legs::Stand,
        Legs::Squat => Legs::Squat,
        Legs::Pose { left, right } => Legs::Pose { left: *left, right: *right },
    });
    let mut canvas = an_leg_layer(parts, &legs);
    let mut upper = Grid::blank(FRAME_W, G_TORSO_H + 1);
    let mut glow = Grid::blank(FRAME_W, G_TORSO_H + 1);
    for y in 0..=38 {
        for x in 0..parts.body.w as i32 {
            let ch = parts.body.get(x, y);
            if ch != '.' {
                upper.set(x + AOX, y + AOY, ch);
                // The ruby at his throat: the red cells of the brooch.
                if (17..=19).contains(&x) && (17..=19).contains(&y) && "BC-".contains(ch) {
                    glow.set(x + AOX, y + AOY, 'W');
                }
            }
        }
    }
    let mouth = match frame {
        Frame::Speak(0) => 1,
        Frame::Speak(1) | Frame::Hurt(_) | Frame::Ko => 2,
        _ => gs.spec.mouth,
    };
    if mouth > 0 {
        for x in 16..=18 {
            upper.set(x + AOX, 13 + AOY, 'g');
        }
        if mouth > 1 {
            upper.set(17 + AOX, 14 + AOY, 'g');
        }
    }
    // The open scroll between his hands, drawn first so that the fists close over its edges.
    if let (Frame::Read(k @ 0..=2), ArmSpec::Pose { hand: lh, .. }, ArmSpec::Pose { hand: rh, .. }) = (frame, &gs.spec.arms.0, &gs.spec.arms.1) {
        an_open_scroll(&mut upper, an_j(*lh), an_j(*rh), k);
    }
    let (hl, hr) = an_hands(frame);
    for (side, (spec, shoulder, kept, hand)) in [(&gs.spec.arms.0, G_LEFT_SHOULDER, &parts.left_arm, hl), (&gs.spec.arms.1, G_RIGHT_SHOULDER, &parts.right_arm, hr)].into_iter().enumerate() {
        // His staff is drawn first so that the fist closes over its shaft.
        if let (1, ArmSpec::Pose { hand: h, .. }, Some((bottom, top))) = (side, spec, an_staff(frame)) {
            an_draw_staff(&mut upper, an_j(*h), bottom, top);
        }
        match spec {
            ArmSpec::Keep => {
                for &(x, y, ch) in kept {
                    upper.set(x + AOX, y + AOY, ch);
                }
            }
            ArmSpec::Pose { elbow, hand: h } => an_arm(&mut upper, shoulder, *elbow, *h, hand),
        }
    }
    // The mantle (and the cape's high collar) and the hair lie over the shoulders and the arms.
    for y in 13..=21 {
        for x in 0..parts.body.w as i32 {
            let ch = parts.body.get(x, y);
            if "XVZ".contains(ch) {
                upper.set(x + AOX, y + AOY, ch);
            }
        }
    }
    for y in 6..=34 {
        for x in 0..16 {
            let ch = parts.body.get(x, y);
            if "{}[]".contains(ch) {
                upper.set(x + AOX, y + AOY, ch);
            }
        }
    }
    let mut mark = Grid::blank(FRAME_W, FULL_H);
    for cy in 0..(G_TORSO_H + 1) {
        let r = (cy - AOY).clamp(0, 38);
        let shift = (gs.spec.lean * (38 - r) as f32 / 38.0).round() as i32;
        for x in 0..FRAME_W {
            let (nx, ny) = (x + shift, cy + gs.spec.dy);
            if nx < 0 || nx >= FRAME_W || ny < 0 || ny >= FULL_H {
                continue;
            }
            let ch = upper.get(x, cy);
            if ch != '.' {
                canvas.set(nx, ny, ch);
            }
            let m = glow.get(x, cy);
            if m != '.' {
                mark.set(nx, ny, m);
            }
        }
    }
    if frame == Frame::Ko {
        return (lying_with(&canvas, &canvas), lying_with(&mark, &canvas));
    }
    (canvas, mark)
}

fn an_frame_list() -> Vec<Frame> {
    let mut v = Frame::all();
    v.extend((0..t2f::anim::PUNCH_FRAMES).map(Frame::Punch));
    v.extend((0..t2f::anim::READ_FRAMES).map(Frame::Read));
    v.extend((0..t2f::anim::STAFF_FRAMES).map(Frame::Staff));
    v
}

/// An unrolled scroll held between his hands (`l` and `r`, in his grid): parchment with a darker
/// rim, lines of writing, a gold glow running along them as he reads, and sparks above it in the
/// last reading drawing. The first drawing (0) is only just unrolled, so it is narrow.
fn an_open_scroll(canvas: &mut Grid, l: Joint, r: Joint, k: usize) {
    let (x0, x1) = (l.0.round() as i32 + 2, r.0.round() as i32 - 2);
    let cy = ((l.1 + r.1) / 2.0).round() as i32;
    let (y0, y1) = (cy - 5, cy + 4);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let rim = y == y0 || y == y1 || x == x0 || x == x1;
            canvas.set(x + AOX, y + AOY, if rim { 'U' } else { 't' });
        }
    }
    // the writing: short strokes with gaps between the words
    for (i, y) in [y0 + 2, y0 + 4, y0 + 6].into_iter().enumerate() {
        for x in (x0 + 2)..(x1 - 1) {
            if (x + i as i32 * 2) % 5 != 0 {
                canvas.set(x + AOX, y + AOY, 'R');
            }
        }
    }
    if k >= 1 {
        // the glow runs along the line he is reading
        let width = (x1 - x0 - 3).max(1);
        let gx = x0 + 2 + (k as i32 * 6) % width;
        for dx in 0..3 {
            canvas.set(gx + dx + AOX, y0 + 4 + AOY, 'y');
            canvas.set(gx + dx + AOX, y0 + 2 + AOY, 'z');
        }
    }
    if k == 2 {
        for (dx, dy, ch) in [(3, -3, 'z'), (7, -6, 'y'), (11, -3, 'z'), (5, -8, 'y'), (9, -9, 'z')] {
            canvas.set(x0 + dx + AOX, y0 + dy + AOY, ch);
        }
    }
}

/// His magic missile (14 x 9), pointing right: a golden orb with a streaming tail; the second
/// drawing flickers.
fn missile(variant: usize) -> Grid {
    let mut g = Grid::blank(14, 9);
    rv_poly(&mut g, &[(0.0, 4.5), (8.0, 2.4), (8.0, 6.6)], 'y');
    rv_poly(&mut g, &[(2.0, 4.5), (8.0, 3.3), (8.0, 5.7)], 'Y');
    rv_ellipse(&mut g, 9.6, 4.5, 3.4, 3.1, 'Y');
    rv_ellipse(&mut g, 10.0, 4.3, 2.3, 2.0, 'z');
    rv_ellipse(&mut g, 10.3, 4.2, 1.1, 1.0, 'W');
    if variant == 1 {
        for (x, y) in [(1, 3), (0, 5), (3, 6), (2, 2)] {
            g.set(x, y, 'z');
        }
    } else {
        for (x, y) in [(1, 5), (0, 4), (3, 2)] {
            g.set(x, y, 'z');
        }
    }
    g
}

fn an_frames(default: &Grid) -> Vec<(String, image::RgbaImage)> {
    let parts = an_split(default);
    let mut frames = Vec::new();
    for f in an_frame_list() {
        let (body, mark) = an_drawing(&parts, f);
        frames.push((f.file_stem(), body.to_png()));
        frames.push((format!("{}_mark", f.file_stem()), mark.to_png()));
    }
    for v in 0..2 {
        frames.push((format!("runes_{v}"), runes(v).to_png()));
        frames.push((format!("missile_{v}"), missile(v).to_png()));
    }
    for (i, g) in goblin_frames().into_iter().enumerate() {
        frames.push((format!("goblin_{i}"), g.to_png()));
    }
    frames
}

fn an_contact_sheet(default: &Grid, scale: u32) -> image::RgbaImage {
    let parts = an_split(default);
    let cells: Vec<image::RgbaImage> = an_frame_list().into_iter().map(|f| an_drawing(&parts, f).0.to_png()).collect();
    let columns = 8u32;
    let (cw, ch) = (FRAME_W as u32 * scale, FULL_H as u32 * scale);
    let rows = (cells.len() as u32 + columns - 1) / columns;
    let mut sheet = image::RgbaImage::from_pixel(columns * (cw + 8) + 8, rows * (ch + 8) + 8 + 30 * scale, image::Rgba([196, 214, 232, 255]));
    for (k, cell) in cells.iter().enumerate() {
        let (col, row) = (k as u32 % columns, k as u32 / columns);
        let (ox, oy) = (8 + col * (cw + 8), 8 + row * (ch + 8));
        for y in 0..ch {
            for x in 0..cw {
                let p = cell.get_pixel(x / scale, y / scale);
                sheet.put_pixel(ox + x, oy + y, if p[3] > 0 { *p } else { image::Rgba([232, 240, 248, 255]) });
            }
        }
    }
    // The effects underneath: the runes, then the goblins' drawings.
    let mut x = 8;
    let y = rows * (ch + 8) + 8;
    let mut effects = vec![runes(0).to_png(), runes(1).to_png()];
    effects.extend(goblin_frames().into_iter().map(|g| g.to_png()));
    for img in effects {
        for yy in 0..img.height() * scale {
            for xx in 0..img.width() * scale {
                let p = img.get_pixel(xx / scale, yy / scale);
                if p[3] > 0 {
                    sheet.put_pixel(x + xx, y + yy, *p);
                }
            }
        }
        x += img.width() * scale + 6 * scale;
    }
    sheet
}

/// The wave of golden runes (36 x 24), heading right: nested crescents of light with glowing
/// glyphs in them. The two drawings differ in where the glyphs sit.
fn runes(variant: usize) -> Grid {
    let mut g = Grid::blank(36, 24);
    for y in 0..24 {
        for x in 0..36 {
            let (dx, dy) = ((x as f32 - 8.0) / 27.0, (y as f32 - 11.5) / 11.5);
            let outer = dx * dx + dy * dy;
            let (ix, iy) = ((x as f32 - 3.0) / 25.0, (y as f32 - 11.5) / 9.5);
            let inner = ix * ix + iy * iy;
            // Three bands: a bright rim, a body, a faded tail.
            if outer <= 1.0 && inner > 1.0 {
                let t = (outer - 0.78) / 0.22;
                let ch = if outer > 0.93 { 'z' } else if inner > 1.5 { 'y' } else { 'Y' };
                let _ = t;
                g.set(x, y, ch);
            }
        }
    }
    // Glyphs: little crosses and ticks glowing inside the wave.
    let spots: [(i32, i32); 5] = if variant == 0 { [(20, 5), (26, 11), (21, 17), (14, 9), (30, 11)] } else { [(18, 7), (27, 8), (23, 14), (16, 15), (31, 12)] };
    for (x, y) in spots {
        g.set(x, y, 'W');
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if g.get(x + dx, y + dy) != '.' {
                g.set(x + dx, y + dy, 'W');
            }
        }
    }
    g
}

/// One goblin drawing (15 x 16) from a template, facing right.
fn goblin_base() -> Vec<Vec<char>> {
    [
        "...............",
        "..ì.........ì..",
        "..àà..àààà..àà.",
        "...àààààààààà..",
        "...àYVàààYVàà..",
        "....àààààààà...",
        "....àà.WW.àà...",
        ".....èèèèèè....",
        "....òòòòòòòò...",
        "...àòòòòòòòòà..",
        "...àòòòòòòòòà..",
        "....òòòòòòòò...",
        "....èè....èè...",
        "....àà....àà...",
        "...VVV....VVV..",
        "...............",
    ]
    .iter()
    .map(|r| r.chars().collect())
    .collect()
}

/// The goblin's drawings: 0-1 waiting (a little bob), 2-3 running, 4 biting.
fn goblin_frames() -> Vec<Grid> {
    let base = goblin_base();
    let to_grid = |rows: Vec<Vec<char>>| Grid::from_rows(rows);
    let mut frames = Vec::new();
    // Waiting: the second drawing is a step down (the body bobs) with the eyes shut.
    frames.push(to_grid(base.clone()));
    let mut bob = base.clone();
    bob.remove(0);
    bob.insert(14, vec!['.'; 15]);
    for row in bob.iter_mut().skip(3).take(2) {
        for c in row.iter_mut() {
            if *c == 'Y' {
                *c = 'è';
            }
        }
    }
    frames.push(to_grid(bob));
    // Running: one leg forward and one back, then the other way, body leaning forward.
    for (a, b) in [(2usize, 8usize), (8, 2)] {
        let mut r = base.clone();
        for y in 12..=14 {
            r[y] = vec!['.'; 15];
        }
        for (x0, rows) in [(a, [("èè", 'è'), ("àà", 'à'), ("VVV", 'V')]), (b, [("èè", 'è'), ("àà", 'à'), ("VVV", 'V')])] {
            for (k, (s, _)) in rows.iter().enumerate() {
                for (i, c) in s.chars().enumerate() {
                    if x0 + i < 15 {
                        r[12 + k][x0 + i] = c;
                    }
                }
            }
        }
        frames.push(to_grid(r));
    }
    // Biting: the mouth wide open with teeth, arms up.
    let mut bite = base.clone();
    for (x, y, c) in [(6, 6, 'V'), (7, 6, 'W'), (8, 6, 'W'), (9, 6, 'V'), (7, 7, 'V'), (8, 7, 'V'), (2, 8, 'à'), (2, 7, 'à'), (12, 8, 'à'), (12, 7, 'à')] {
        bite[y][x] = c;
    }
    frames.push(to_grid(bite));
    frames
}


// Junkhead's whip. Each pose that uses it has a layer drawn over the body: the same left edge as
// the body drawing, but WHIP_W wide so the whip can reach far in front of him. The game puts the
// layer over the body (see `anim::whip_layer` for which one and `render::animate_whips`).

const WHIP_W: i32 = 176;
const WHIP_DARK: [u8; 4] = [86, 52, 30, 255];
const WHIP_MID: [u8; 4] = [128, 80, 46, 255];
const WHIP_LIGHT: [u8; 4] = [182, 128, 78, 255];
const WHIP_TIP: [u8; 4] = [236, 218, 180, 255];

/// Junkhead's own poses where the cowboy's do not fit a whip: the raised arm of the spin, the
/// lash, the air attacks (arms out for the spin, the whip arm forward for the air heavy).
fn jh_spec(frame: Frame) -> GsSpec {
    use ArmSpec::Keep;
    let stance = || legs((8.0, 45.0), (4.0, 48.0), (33.0, 45.0), (36.0, 48.0));
    let own = |arms: (ArmSpec, ArmSpec), lean: f32, legs: Legs| GsSpec {
        spec: Spec { arms, lean, dy: 0, legs, mouth: 0, finger: None },
        hat_in_hand: None,
        hat_on_head: false,
    };
    match frame {
        Frame::Summon(0) => own((Keep, pose((40.0, 21.0), (38.0, 10.0))), 0.0, Legs::Stand),
        Frame::Summon(1) | Frame::Summon(2) | Frame::Summon(3) => own((Keep, pose((40.0, 12.0), (37.0, 2.0))), -1.0, Legs::Stand),
        Frame::Summon(4) => own((Keep, pose((36.0, 19.0), (47.0, 17.0))), 4.0, stance()),
        Frame::Summon(_) => own((Keep, pose((33.0, 24.0), (40.0, 29.0))), 2.0, stance()),
        Frame::AirSoft(_) => own(
            (pose((1.0, 22.0), (-5.0, 20.0)), pose((43.0, 22.0), (49.0, 20.0))),
            0.0,
            legs((4.0, 44.0), (9.0, 45.0), (37.0, 44.0), (32.0, 45.0)),
        ),
        Frame::AirHard(0) => own((Keep, pose((38.0, 14.0), (34.0, 3.0))), -2.0, legs((5.0, 45.0), (8.0, 47.0), (36.0, 45.0), (33.0, 47.0))),
        Frame::AirHard(_) => own((Keep, pose((36.0, 20.0), (48.0, 22.0))), 5.0, legs((6.0, 45.0), (3.0, 48.0), (38.0, 45.0), (41.0, 48.0))),
        other => cb_spec(other),
    }
}

/// Where a hand is on the finished drawing (the arm's joint, moved by the lean and the drop).
fn jh_hand_xy(frame: Frame, left: bool) -> (i32, i32) {
    let gs = jh_spec(frame);
    let arm = if left { &gs.spec.arms.0 } else { &gs.spec.arms.1 };
    let ArmSpec::Pose { hand, .. } = arm else { panic!("{frame:?} has no posed arm on that side") };
    let r = (hand.1.round() as i32).clamp(0, G_UPPER_LAST);
    let shift = (gs.spec.lean * (G_UPPER_LAST - r) as f32 / G_UPPER_LAST as f32).round() as i32;
    (hand.0.round() as i32 + GOX + shift, hand.1.round() as i32 + GOY + gs.spec.dy)
}

fn wl_canvas() -> image::RgbaImage {
    image::RgbaImage::new(WHIP_W as u32, FULL_H as u32)
}

fn wl_plot(img: &mut image::RgbaImage, x: i32, y: i32, c: [u8; 4]) {
    if x >= 0 && y >= 0 && x < WHIP_W && y < FULL_H {
        img.put_pixel(x as u32, y as u32, image::Rgba(c));
    }
}

/// Paint a path of pixels as a whip: dark leather, thicker at the handle, a lighter glint every
/// few pixels and a pale cracker at the tip.
fn wl_paint(img: &mut image::RgbaImage, path: &[(i32, i32)], handle_thick: bool) {
    let len = path.len();
    for (k, &(x, y)) in path.iter().enumerate() {
        let c = if k + 3 >= len {
            WHIP_TIP
        } else if k % 5 == 2 {
            WHIP_LIGHT
        } else if k % 5 == 4 {
            WHIP_MID
        } else {
            WHIP_DARK
        };
        wl_plot(img, x, y, c);
        if handle_thick && k < 8 {
            wl_plot(img, x, y + 1, WHIP_DARK);
        }
    }
}

fn wl_dedupe(points: impl Iterator<Item = (f32, f32)>) -> Vec<(i32, i32)> {
    let mut path: Vec<(i32, i32)> = Vec::new();
    for (x, y) in points {
        let p = (x.round() as i32, y.round() as i32);
        if path.last() != Some(&p) {
            path.push(p);
        }
    }
    path
}

/// A whip along a quadratic curve from `p0` (the hand) to `p1` (the tip), bowed towards `c`.
fn wl_curve(img: &mut image::RgbaImage, p0: (f32, f32), c: (f32, f32), p1: (f32, f32)) {
    let n = 800;
    let path = wl_dedupe((0..=n).map(|i| {
        let t = i as f32 / n as f32;
        let (a, b, d) = ((1.0 - t) * (1.0 - t), 2.0 * (1.0 - t) * t, t * t);
        (a * p0.0 + b * c.0 + d * p1.0, a * p0.1 + b * c.1 + d * p1.1)
    }));
    wl_paint(img, &path, true);
}

/// An arc of an ellipse (the whip spinning), `sweep` degrees long starting at `phase`.
fn wl_ring(img: &mut image::RgbaImage, centre: (f32, f32), rx: f32, ry: f32, phase: f32, sweep: f32) {
    let n = 900;
    let path = wl_dedupe((0..=n).map(|i| {
        let a = (phase + sweep * i as f32 / n as f32).to_radians();
        (centre.0 + rx * a.cos(), centre.1 + ry * a.sin())
    }));
    wl_paint(img, &path, false);
}

/// Every whip layer, by file name.
fn jh_whip_layers() -> Vec<(String, image::RgbaImage)> {
    let mut out: Vec<(String, image::RgbaImage)> = Vec::new();
    let tip_x = |reach: i32| (CENTRE_X + reach) as f32;
    let handle = |frame: Frame, left: bool| {
        let h = jh_hand_xy(frame, left);
        (h.0 as f32 + 3.0, h.1 as f32 + 0.5)
    };
    // The whip cocked back over the shoulder.
    let windup = |s: (f32, f32)| {
        let mut img = wl_canvas();
        wl_curve(&mut img, s, (s.0 - 16.0, s.1 - 20.0), (s.0 - 32.0, s.1 - 4.0));
        img
    };
    // Straight out in front, with a little bow, to `tip`.
    let strike = |s: (f32, f32), tip: (f32, f32)| {
        let mut img = wl_canvas();
        wl_curve(&mut img, s, ((s.0 + tip.0) / 2.0, (s.1 + tip.1) / 2.0 - 3.0), tip);
        img
    };
    // Slack, hanging after the lash.
    let slack = |s: (f32, f32)| {
        let mut img = wl_canvas();
        wl_curve(&mut img, s, (s.0 + 24.0, s.1 + 3.0), (s.0 + 12.0, s.1 + 24.0));
        img
    };
    let floor = (FULL_H - 3) as f32;

    for (k, left) in [(0, false), (1, false), (2, true), (3, true)] {
        let s = handle(Frame::Punch(k), left);
        if k % 2 == 0 {
            out.push((format!("punch_{k}_whip"), windup(s)));
        } else {
            out.push((format!("punch_{k}_whip"), strike(s, (tip_x(60), s.1 + 1.0))));
            out.push((format!("punch_{k}_whipx"), strike(s, (tip_x(105), s.1 + 2.0))));
        }
    }
    out.push(("crouch_soft_0_whip".into(), windup(handle(Frame::CrouchSoft(0), false))));
    out.push(("crouch_soft_1_whip".into(), strike(handle(Frame::CrouchSoft(1), false), (tip_x(72), floor))));
    for k in 0..2 {
        out.push((format!("hard_{k}_whip"), windup(handle(Frame::Hard(k), false))));
    }
    let s = handle(Frame::Hard(2), false);
    out.push(("hard_2_whip".into(), strike(s, (tip_x(115), s.1 + 1.0))));
    out.push(("hard_3_whip".into(), slack(handle(Frame::Hard(3), false))));
    out.push(("air_hard_0_whip".into(), windup(handle(Frame::AirHard(0), false))));
    let s = handle(Frame::AirHard(1), false);
    out.push(("air_hard_1_whip".into(), strike(s, (tip_x(115), (s.1 + 26.0).min(floor)))));

    // The spin overhead: the whip circles the raised hand, a quarter turn per drawing.
    let hand = handle(Frame::Summon(2), false);
    for k in 0..4 {
        let mut img = wl_canvas();
        wl_ring(&mut img, (hand.0 - 3.0, hand.1 - 1.0), 22.0, 5.5, k as f32 * 90.0, 290.0);
        out.push((format!("whip_ring_{k}"), img));
    }
    // The lash: high (head and neck height) or low (the floor), then slack.
    let s = handle(Frame::Summon(4), false);
    out.push(("summon_4_whip_hi".into(), strike(s, (tip_x(128), 16.0))));
    out.push(("summon_4_whip_lo".into(), strike(s, (tip_x(128), floor))));
    out.push(("summon_5_whip".into(), slack(handle(Frame::Summon(5), false))));
    // The air spin: the whip circles him from his head to his legs.
    let centre = (CENTRE_X as f32, FULL_H as f32 - 31.0);
    for (k, base) in [(0, 0.0), (1, 180.0)] {
        for (suffix, extra) in [("", 0.0), ("b", 90.0)] {
            let mut img = wl_canvas();
            wl_ring(&mut img, centre, 27.0, 33.0, base + extra, 290.0);
            out.push((format!("air_soft_{k}_whip{suffix}"), img));
        }
    }
    out
}

/// Whether the whip hangs coiled on his belt in this pose: everywhere except the attacks that
/// have it in his hand (the light whips, the heavy, the air spin and heavy, the special).
fn jh_belt_whip(frame: Frame) -> bool {
    !matches!(
        frame,
        Frame::Punch(_) | Frame::Soft(_) | Frame::CrouchSoft(_) | Frame::CrouchHard(_) | Frame::Hard(_) | Frame::AirSoft(_) | Frame::AirHard(_) | Frame::Summon(_)
    )
}

/// Junkhead's poses and the whip layers over them.
fn jh_frames(default: &Grid) -> Vec<(String, image::RgbaImage)> {
    let parts = cb_split(default);
    let mut frames = Vec::new();
    for f in cb_frame_list() {
        let (body, mark) = cb_drawing_with(&parts, f, jh_spec, jh_belt_whip(f));
        frames.push((f.file_stem(), body.to_png()));
        frames.push((format!("{}_mark", f.file_stem()), mark.to_png()));
    }
    frames.extend(jh_whip_layers());
    frames
}

/// The poses that use the whip, each with its layer on top, for a quick look.
fn jh_whip_sheet(default: &Grid) -> image::RgbaImage {
    let parts = cb_split(default);
    let shots: [(Frame, &str); 14] = [
        (Frame::Punch(0), "punch_0_whip"),
        (Frame::Punch(1), "punch_1_whip"),
        (Frame::Punch(1), "punch_1_whipx"),
        (Frame::Punch(3), "punch_3_whip"),
        (Frame::CrouchSoft(1), "crouch_soft_1_whip"),
        (Frame::Hard(1), "hard_1_whip"),
        (Frame::Hard(2), "hard_2_whip"),
        (Frame::Hard(3), "hard_3_whip"),
        (Frame::AirSoft(1), "air_soft_1_whip"),
        (Frame::AirHard(1), "air_hard_1_whip"),
        (Frame::Summon(2), "whip_ring_1"),
        (Frame::Summon(4), "summon_4_whip_hi"),
        (Frame::Summon(4), "summon_4_whip_lo"),
        (Frame::Summon(5), "summon_5_whip"),
    ];
    let layers: std::collections::HashMap<String, image::RgbaImage> = jh_whip_layers().into_iter().collect();
    let (cols, scale) = (3u32, 3u32);
    let (cw, ch) = (WHIP_W as u32 * scale + 8, FULL_H as u32 * scale + 8);
    let rows = (shots.len() as u32 + cols - 1) / cols;
    let mut sheet = image::RgbaImage::from_pixel(cols * cw, rows * ch, image::Rgba([196, 214, 232, 255]));
    for (i, (frame, layer)) in shots.iter().enumerate() {
        let (mut body, _) = cb_drawing_with(&parts, *frame, jh_spec, jh_belt_whip(*frame));
        let mut body = body.to_png();
        junkhead_recolour(&mut body);
        let mut cell = image::RgbaImage::new(WHIP_W as u32, FULL_H as u32);
        image::imageops::overlay(&mut cell, &body, 0, 0);
        image::imageops::overlay(&mut cell, &layers[*layer], 0, 0);
        let (ox, oy) = ((i as u32 % cols) * cw + 4, (i as u32 / cols) * ch + 4);
        for y in 0..FULL_H as u32 * scale {
            for x in 0..WHIP_W as u32 * scale {
                let p = cell.get_pixel(x / scale, y / scale);
                if p[3] > 0 {
                    sheet.put_pixel(ox + x, oy + y, *p);
                }
            }
        }
    }
    sheet
}


// ---------------------------------------------------------------------------------------------
// The roll drawings, made from a character's crouch drawing.

/// The rows and columns the opaque pixels of `img` cover: (x0, y0, x1, y1), inclusive.
fn opaque_bounds(img: &image::RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 0 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    (x0 != u32::MAX).then_some((x0, y0, x1, y1))
}

/// Resize to `w` x `h` taking the nearest pixel, so no new colours appear.
fn nearest_resize(img: &image::RgbaImage, w: u32, h: u32) -> image::RgbaImage {
    let mut out = image::RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (((x as f32 + 0.5) * img.width() as f32 / w as f32) as u32, ((y as f32 + 0.5) * img.height() as f32 / h as f32) as u32);
            out.put_pixel(x, y, *img.get_pixel(sx.min(img.width() - 1), sy.min(img.height() - 1)));
        }
    }
    out
}

/// Turn clockwise by `deg` degrees, taking the nearest pixel, into a square big enough for any angle.
/// Single-pixel holes the sampling leaves are filled from their neighbours.
fn rotate_clockwise(img: &image::RgbaImage, deg: f32) -> image::RgbaImage {
    let (w, h) = (img.width() as f32, img.height() as f32);
    let side = (w * w + h * h).sqrt().ceil() as u32 + 2;
    let (sin, cos) = deg.to_radians().sin_cos();
    let mut out = image::RgbaImage::new(side, side);
    for oy in 0..side {
        for ox in 0..side {
            let (dx, dy) = (ox as f32 + 0.5 - side as f32 / 2.0, oy as f32 + 0.5 - side as f32 / 2.0);
            let (sx, sy) = (dx * cos + dy * sin + w / 2.0, -dx * sin + dy * cos + h / 2.0);
            if sx >= 0.0 && sy >= 0.0 && sx < w && sy < h {
                out.put_pixel(ox, oy, *img.get_pixel(sx as u32, sy as u32));
            }
        }
    }
    let copy = out.clone();
    for y in 1..side - 1 {
        for x in 1..side - 1 {
            if copy.get_pixel(x, y)[3] > 0 {
                continue;
            }
            let mut near: Vec<image::Rgba<u8>> = Vec::new();
            for (dx, dy) in [(-1i32, -1i32), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                let p = *copy.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32);
                if p[3] > 0 {
                    near.push(p);
                }
            }
            if near.len() >= 6 {
                // the most common neighbour colour
                let best = near.iter().max_by_key(|c| near.iter().filter(|d| d == c).count()).unwrap();
                out.put_pixel(x, y, *best);
            }
        }
    }
    out
}

/// The ten roll drawings of `img` (the same size as `body`, the crouch drawing they are measured
/// from): a tuck, a ball turned 0, 45, ... 315 degrees, and a rise. The ball touches the floor.
fn roll_drawings(img: &image::RgbaImage, body: &image::RgbaImage) -> Vec<image::RgbaImage> {
    let (bx0, by0, bx1, by1) = opaque_bounds(body).expect("the crouch drawing is not empty");
    let (w, h) = (bx1 - bx0 + 1, by1 - by0 + 1);
    let crop = |i: &image::RgbaImage| image::imageops::crop_imm(i, bx0, by0, w, h).to_image();
    let (body_c, img_c) = (crop(body), crop(img));
    let centre_x = (bx0 + bx1) as f32 / 2.0;
    // (width squash, height squash, turn in degrees)
    let mut specs: Vec<(f32, f32, f32)> = vec![(0.92, 0.84, 0.0)];
    specs.extend((0..8).map(|k| (0.80, 0.70, k as f32 * 45.0)));
    specs.push((0.96, 0.92, 0.0));
    specs
        .into_iter()
        .map(|(sw, sh, deg)| {
            let (nw, nh) = (((w as f32 * sw).round() as u32).max(1), ((h as f32 * sh).round() as u32).max(1));
            let (bs, is) = (nearest_resize(&body_c, nw, nh), nearest_resize(&img_c, nw, nh));
            let (br, ir) = if deg == 0.0 { (bs, is) } else { (rotate_clockwise(&bs, deg), rotate_clockwise(&is, deg)) };
            // The opaque part of the body decides where it sits: centred on the crouch, bottom on the floor.
            let (rx0, _ry0, rx1, ry1) = opaque_bounds(&br).expect("the squashed drawing is not empty");
            let left = (centre_x - (rx0 + rx1) as f32 / 2.0).round() as i32;
            let top = by1 as i32 - ry1 as i32;
            let mut out = image::RgbaImage::new(body.width(), body.height());
            for (x, y, p) in ir.enumerate_pixels() {
                let (ox, oy) = (x as i32 + left, y as i32 + top);
                if p[3] > 0 && ox >= 0 && oy >= 0 && ox < out.width() as i32 && oy < out.height() as i32 {
                    out.put_pixel(ox as u32, oy as u32, *p);
                }
            }
            out
        })
        .collect()
}


// ---------------------------------------------------------------------------------------------
// Skill icons for the bottom-right skill bar: 24 x 24, MMORPG style. A glossy gradient tile with a
// light rim, and a bold, outlined, shaded emblem for the attack. One set per character (the
// Negaprofessor's is the Professor's, inverted). `assets/icons/<sprite folder>/<light|heavy|special>.png`.
// ---------------------------------------------------------------------------------------------

type Rgb = [u8; 3];
const IC: i32 = 24;

fn ic_put(img: &mut image::RgbaImage, x: i32, y: i32, c: Rgb) {
    if x >= 0 && y >= 0 && x < IC && y < IC {
        img.put_pixel(x as u32, y as u32, image::Rgba([c[0], c[1], c[2], 255]));
    }
}

fn ic_ellipse(img: &mut image::RgbaImage, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgb) {
    for y in 0..IC {
        for x in 0..IC {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            if dx * dx + dy * dy <= 1.0 {
                ic_put(img, x, y, c);
            }
        }
    }
}

fn ic_ring(img: &mut image::RgbaImage, cx: f32, cy: f32, rx: f32, ry: f32, thick: f32, c: Rgb) {
    for y in 0..IC {
        for x in 0..IC {
            let (px, py) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let outer = (px / rx).powi(2) + (py / ry).powi(2);
            let inner = (px / (rx - thick).max(0.1)).powi(2) + (py / (ry - thick).max(0.1)).powi(2);
            if outer <= 1.0 && inner >= 1.0 {
                ic_put(img, x, y, c);
            }
        }
    }
}

fn ic_poly(img: &mut image::RgbaImage, pts: &[(f32, f32)], c: Rgb) {
    for y in 0..IC {
        for x in 0..IC {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut inside = false;
            for i in 0..pts.len() {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                if (a.1 > py) != (b.1 > py) && px < (b.0 - a.0) * (py - a.1) / (b.1 - a.1) + a.0 {
                    inside = !inside;
                }
            }
            if inside {
                ic_put(img, x, y, c);
            }
        }
    }
}

fn ic_line(img: &mut image::RgbaImage, a: (f32, f32), b: (f32, f32), w: f32, c: Rgb) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = (dx * dx + dy * dy).max(0.0001);
    for y in 0..IC {
        for x in 0..IC {
            let (px, py) = (x as f32 + 0.5 - a.0, y as f32 + 0.5 - a.1);
            let t = ((px * dx + py * dy) / len2).clamp(0.0, 1.0);
            let (qx, qy) = (px - t * dx, py - t * dy);
            if qx * qx + qy * qy <= (w / 2.0) * (w / 2.0) {
                ic_put(img, x, y, c);
            }
        }
    }
}

/// A smooth curve through `pts` (a polyline of line pieces), `w` thick.
fn ic_path(img: &mut image::RgbaImage, pts: &[(f32, f32)], w: f32, c: Rgb) {
    for pair in pts.windows(2) {
        ic_line(img, pair[0], pair[1], w, c);
    }
}

fn ic_bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> Vec<(f32, f32)> {
    (0..=24)
        .map(|i| {
            let t = i as f32 / 24.0;
            let u = 1.0 - t;
            (
                u * u * u * p0.0 + 3.0 * u * u * t * p1.0 + 3.0 * u * t * t * p2.0 + t * t * t * p3.0,
                u * u * u * p0.1 + 3.0 * u * u * t * p1.1 + 3.0 * u * t * t * p2.1 + t * t * t * p3.1,
            )
        })
        .collect()
}

fn ic_opaque(img: &image::RgbaImage, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < IC && y < IC && img.get_pixel(x as u32, y as u32)[3] > 0
}

/// Light on the upper-left edge of every shape and shade on the lower-right (the "3D" look), then
/// a dark outline round the whole emblem.
fn ic_finish(img: &mut image::RgbaImage) {
    let copy = img.clone();
    for y in 0..IC {
        for x in 0..IC {
            if !ic_opaque(&copy, x, y) {
                continue;
            }
            let p = copy.get_pixel(x as u32, y as u32).0;
            let lit = |k: i32| image::Rgba([p[0].saturating_add(k as u8), p[1].saturating_add(k as u8), p[2].saturating_add(k as u8), 255]);
            let dark = |k: u8| image::Rgba([p[0].saturating_sub(k), p[1].saturating_sub(k), p[2].saturating_sub(k), 255]);
            if !ic_opaque(&copy, x - 1, y) || !ic_opaque(&copy, x, y - 1) {
                img.put_pixel(x as u32, y as u32, lit(46));
            } else if !ic_opaque(&copy, x + 1, y) || !ic_opaque(&copy, x, y + 1) {
                img.put_pixel(x as u32, y as u32, dark(46));
            }
        }
    }
    let copy = img.clone();
    for y in 0..IC {
        for x in 0..IC {
            if ic_opaque(&copy, x, y) {
                continue;
            }
            let near = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1), (-1, 1), (1, -1)].iter().any(|&(dx, dy)| ic_opaque(&copy, x + dx, y + dy));
            if near {
                img.put_pixel(x as u32, y as u32, image::Rgba([14, 10, 20, 255]));
            }
        }
    }
}

/// The tile: a vertical gradient, a dark outer rim, a light inner rim in `accent`, a glossy
/// highlight up in the corner and the emblem on top.
fn ic_tile(emblem: &image::RgbaImage, top: Rgb, bottom: Rgb, accent: Rgb) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(IC as u32, IC as u32);
    let lerp = |a: u8, b: u8, t: f32| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    for y in 0..IC {
        for x in 0..IC {
            // round the four corners off
            let corner = (x == 0 || x == IC - 1) && (y == 0 || y == IC - 1);
            if corner {
                continue;
            }
            let t = y as f32 / (IC - 1) as f32;
            let mut c = [lerp(top[0], bottom[0], t), lerp(top[1], bottom[1], t), lerp(top[2], bottom[2], t)];
            // gloss in the upper left
            if x + y < 13 {
                for k in 0..3 {
                    c[k] = c[k].saturating_add(26);
                }
            }
            let outer = x == 0 || y == 0 || x == IC - 1 || y == IC - 1;
            let inner = x == 1 || y == 1 || x == IC - 2 || y == IC - 2;
            let px = if outer {
                [12, 8, 16]
            } else if inner {
                // the light rim: bright at the top left, dimmer at the bottom right
                let k = if x + y < IC { 1.0 } else { 0.55 };
                [(accent[0] as f32 * k) as u8, (accent[1] as f32 * k) as u8, (accent[2] as f32 * k) as u8]
            } else {
                c
            };
            img.put_pixel(x as u32, y as u32, image::Rgba([px[0], px[1], px[2], 255]));
        }
    }
    image::imageops::overlay(&mut img, emblem, 0, 0);
    img
}

fn ic_new() -> image::RgbaImage {
    image::RgbaImage::new(IC as u32, IC as u32)
}

// ---- the emblems ------------------------------------------------------------------------------

/// A scroll open on the left, a golden missile leaving it for the upper right.
fn em_scroll_missile() -> image::RgbaImage {
    let mut g = ic_new();
    ic_poly(&mut g, &[(8.0, 12.0), (12.0, 9.0), (15.0, 7.0), (15.0, 10.0), (11.0, 13.0)], [205, 150, 50]);
    ic_poly(&mut g, &[(4.0, 10.0), (17.0, 10.0), (17.0, 19.0), (4.0, 19.0)], [242, 228, 196]);
    ic_ellipse(&mut g, 4.5, 14.5, 2.0, 4.8, [206, 186, 146]);
    ic_ellipse(&mut g, 16.8, 14.5, 2.0, 4.8, [206, 186, 146]);
    for (y, x1) in [(13.5, 15.0), (15.5, 14.0), (17.5, 12.0)] {
        ic_line(&mut g, (6.5, y), (x1, y), 1.0, [130, 40, 40]);
    }
    ic_ellipse(&mut g, 17.5, 6.0, 3.4, 3.2, [250, 205, 80]);
    ic_ellipse(&mut g, 17.0, 5.6, 1.9, 1.8, [255, 245, 190]);
    ic_finish(&mut g);
    g
}

/// The staff planted on the ground with its orb up, two shock rings round its foot.
fn em_staff_slam() -> image::RgbaImage {
    let mut g = ic_new();
    ic_ring(&mut g, 12.0, 19.0, 10.5, 3.4, 1.6, [190, 140, 255]);
    ic_ring(&mut g, 12.0, 19.0, 6.0, 2.0, 1.3, [235, 210, 255]);
    ic_line(&mut g, (12.0, 18.0), (12.0, 6.0), 2.4, [150, 100, 60]);
    ic_line(&mut g, (11.2, 17.0), (11.2, 7.0), 0.9, [200, 150, 100]);
    ic_ellipse(&mut g, 12.0, 4.8, 3.6, 3.4, [250, 205, 80]);
    ic_ellipse(&mut g, 11.4, 4.2, 1.6, 1.5, [255, 248, 200]);
    ic_finish(&mut g);
    g
}

/// A snarling goblin face.
fn em_goblin() -> image::RgbaImage {
    let mut g = ic_new();
    ic_poly(&mut g, &[(2.0, 9.0), (8.0, 9.0), (7.0, 15.0)], [96, 170, 70]);
    ic_poly(&mut g, &[(22.0, 9.0), (16.0, 9.0), (17.0, 15.0)], [96, 170, 70]);
    ic_ellipse(&mut g, 12.0, 13.0, 7.2, 7.8, [112, 190, 80]);
    ic_ellipse(&mut g, 12.0, 17.0, 5.5, 3.5, [88, 156, 62]);
    ic_ellipse(&mut g, 8.8, 11.5, 2.0, 1.6, [255, 232, 80]);
    ic_ellipse(&mut g, 15.2, 11.5, 2.0, 1.6, [255, 232, 80]);
    ic_put(&mut g, 9, 11, [160, 20, 20]);
    ic_put(&mut g, 15, 11, [160, 20, 20]);
    ic_line(&mut g, (6.0, 9.0), (10.5, 10.5), 1.0, [40, 70, 30]);
    ic_line(&mut g, (18.0, 9.0), (13.5, 10.5), 1.0, [40, 70, 30]);
    ic_line(&mut g, (7.5, 17.0), (16.5, 17.0), 1.8, [40, 24, 24]);
    for x in [8, 10, 13, 15] {
        ic_put(&mut g, x, 17, [250, 250, 240]);
    }
    ic_finish(&mut g);
    g
}

/// A speech bubble with a word in it (a 3 x 5 pixel font, two pixels per dot).
fn em_speech(word: &str) -> image::RgbaImage {
    let mut g = ic_new();
    ic_poly(&mut g, &[(2.0, 4.0), (22.0, 4.0), (22.0, 17.0), (2.0, 17.0)], [250, 250, 255]);
    ic_poly(&mut g, &[(5.0, 17.0), (11.0, 17.0), (5.0, 22.0)], [250, 250, 255]);
    let glyph = |c: char| -> [&'static str; 5] {
        match c {
            'u' => ["x.x", "x.x", "x.x", "x.x", "xxx"],
            'h' => ["x..", "x..", "xxx", "x.x", "x.x"],
            'm' => ["x...x", "xx.xx", "x.x.x", "x...x", "x...x"],
            'e' => ["xxx", "x..", "xxx", "x..", "xxx"],
            _ => ["...", "...", "...", "...", "..."],
        }
    };
    let letters: Vec<[&str; 5]> = word.chars().map(glyph).collect();
    let width: i32 = letters.iter().map(|l| l[0].len() as i32 * 2 + 2).sum::<i32>() - 2;
    let mut x0 = 12 - width / 2;
    for l in &letters {
        for (y, row) in l.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if ch == 'x' {
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        ic_put(&mut g, x0 + x as i32 * 2 + dx, 5 + y as i32 * 2 + dy, [40, 44, 90]);
                    }
                }
            }
        }
        x0 += l[0].len() as i32 * 2 + 2;
    }
    ic_finish(&mut g);
    g
}

/// A fist gripping a shirt collar.
fn em_collar_grab() -> image::RgbaImage {
    let mut g = ic_new();
    ic_poly(&mut g, &[(2.0, 14.0), (12.0, 21.0), (22.0, 14.0), (22.0, 23.0), (2.0, 23.0)], [240, 241, 250]);
    ic_poly(&mut g, &[(2.0, 14.0), (12.0, 21.0), (12.0, 23.0), (2.0, 20.0)], [186, 192, 216]);
    ic_line(&mut g, (4.0, 15.0), (12.0, 20.0), 1.0, [150, 156, 188]);
    ic_line(&mut g, (20.0, 15.0), (12.0, 20.0), 1.0, [150, 156, 188]);
    ic_ellipse(&mut g, 12.0, 9.0, 6.0, 5.2, [238, 196, 158]);
    for k in 0..4 {
        ic_ellipse(&mut g, 8.0 + k as f32 * 2.7, 5.2 + (k % 2) as f32 * 0.7, 1.7, 1.7, [238, 196, 158]);
    }
    ic_ellipse(&mut g, 12.0, 12.0, 5.2, 2.6, [222, 174, 134]);
    ic_line(&mut g, (9.0, 14.5), (15.0, 14.5), 1.2, [238, 196, 158]);
    // the knuckles: dark creases between the fingers
    for x in [9.5f32, 12.5, 15.5] {
        ic_line(&mut g, (x, 5.5), (x, 10.5), 0.9, [186, 134, 104]);
    }
    ic_finish(&mut g);
    g
}

/// A raven with a rose.
fn em_raven_rose() -> image::RgbaImage {
    let mut g = ic_new();
    ic_poly(&mut g, &[(7.0, 12.0), (2.0, 17.0), (9.0, 15.5)], [28, 30, 50]);
    ic_ellipse(&mut g, 10.5, 11.0, 6.5, 4.4, [34, 38, 66]);
    ic_poly(&mut g, &[(6.0, 10.0), (13.0, 6.0), (10.0, 3.0), (4.0, 5.0)], [74, 80, 118]);
    ic_ellipse(&mut g, 16.5, 8.0, 3.0, 2.8, [34, 38, 66]);
    ic_poly(&mut g, &[(18.5, 7.0), (23.0, 8.6), (18.5, 9.6)], [20, 20, 24]);
    ic_put(&mut g, 17, 7, [220, 224, 255]);
    ic_line(&mut g, (18.5, 19.0), (18.5, 23.0), 1.4, [70, 140, 60]);
    ic_ellipse(&mut g, 18.5, 17.0, 3.2, 3.0, [200, 60, 70]);
    ic_ellipse(&mut g, 18.5, 17.6, 1.9, 1.7, [150, 40, 50]);
    ic_put(&mut g, 17, 16, [255, 150, 150]);
    ic_finish(&mut g);
    g
}

/// A finger gun with a spark at the muzzle.
fn em_finger_gun() -> image::RgbaImage {
    let mut g = ic_new();
    ic_ellipse(&mut g, 8.0, 16.0, 5.5, 4.6, [238, 196, 158]);
    ic_poly(&mut g, &[(9.0, 10.0), (19.0, 10.0), (19.0, 13.5), (9.0, 13.5)], [238, 196, 158]);
    ic_poly(&mut g, &[(6.0, 11.0), (10.0, 11.0), (9.0, 4.5), (6.5, 5.5)], [238, 196, 158]);
    ic_line(&mut g, (10.0, 13.0), (19.0, 13.0), 1.0, [222, 174, 134]);
    ic_ellipse(&mut g, 21.0, 9.0, 2.6, 2.6, [255, 232, 120]);
    ic_ellipse(&mut g, 21.0, 9.0, 1.2, 1.2, [255, 255, 235]);
    ic_line(&mut g, (21.0, 3.5), (21.0, 14.5), 1.0, [255, 232, 120]);
    ic_finish(&mut g);
    g
}

fn whip_colors() -> (Rgb, Rgb) {
    ([128, 80, 46], [196, 140, 90])
}

/// A curled whip with its handle.
fn em_whip_curl() -> image::RgbaImage {
    let mut g = ic_new();
    let (mid, light) = whip_colors();
    ic_line(&mut g, (2.5, 21.0), (7.5, 16.5), 3.2, [86, 52, 30]);
    let path = ic_bezier((7.5, 16.5), (23.0, 19.0), (-1.0, 7.0), (18.0, 3.5));
    ic_path(&mut g, &path, 2.2, mid);
    let hi: Vec<(f32, f32)> = path.iter().map(|p| (p.0 - 0.5, p.1 - 0.7)).collect();
    ic_path(&mut g, &hi, 0.8, light);
    ic_ellipse(&mut g, 19.0, 3.5, 1.6, 1.4, [236, 218, 180]);
    ic_finish(&mut g);
    g
}

/// A whip with a grapple hook at the end.
fn em_whip_hook() -> image::RgbaImage {
    let mut g = ic_new();
    let (mid, light) = whip_colors();
    ic_line(&mut g, (2.5, 21.0), (6.5, 17.0), 3.0, [86, 52, 30]);
    let path = ic_bezier((6.5, 17.0), (10.0, 20.0), (9.0, 10.0), (15.0, 9.0));
    ic_path(&mut g, &path, 2.0, mid);
    let hi: Vec<(f32, f32)> = path.iter().map(|p| (p.0 - 0.4, p.1 - 0.6)).collect();
    ic_path(&mut g, &hi, 0.7, light);
    // the hook: a silver barbed head
    ic_poly(&mut g, &[(14.0, 6.0), (22.0, 8.5), (16.5, 12.0)], [200, 204, 220]);
    ic_poly(&mut g, &[(14.0, 12.5), (19.0, 11.0), (15.0, 16.0)], [160, 164, 184]);
    ic_ellipse(&mut g, 14.5, 9.5, 2.2, 2.2, [120, 124, 144]);
    ic_finish(&mut g);
    g
}

/// A whip spinning in a spiral, with sparks.
fn em_whip_spiral() -> image::RgbaImage {
    let mut g = ic_new();
    let (mid, light) = whip_colors();
    let pts: Vec<(f32, f32)> = (0..=60)
        .map(|i| {
            let t = i as f32 / 60.0;
            let a = t * 4.2 * std::f32::consts::PI + 0.6;
            let r = 1.2 + t * 9.5;
            (12.0 + r * a.cos(), 12.0 + r * 0.9 * a.sin())
        })
        .collect();
    ic_path(&mut g, &pts, 2.2, mid);
    let hi: Vec<(f32, f32)> = pts.iter().map(|p| (p.0 - 0.4, p.1 - 0.6)).collect();
    ic_path(&mut g, &hi, 0.7, light);
    let tip = *pts.last().unwrap();
    ic_ellipse(&mut g, tip.0, tip.1, 1.8, 1.6, [255, 240, 200]);
    for (x, y) in [(3, 3), (20, 19), (21, 4), (3, 20)] {
        ic_put(&mut g, x, y, [255, 240, 170]);
        ic_put(&mut g, x + 1, y, [255, 240, 170]);
        ic_put(&mut g, x, y + 1, [255, 240, 170]);
    }
    ic_finish(&mut g);
    g
}

/// The Cowboy's light: a finger gun with a lasso loop round it.
fn em_lasso_gun() -> image::RgbaImage {
    let mut g = em_finger_gun();
    ic_ring(&mut g, 14.0, 17.0, 7.0, 4.6, 1.7, [196, 140, 84]);
    ic_poly(&mut g, &[(19.5, 20.0), (22.0, 21.0), (21.0, 23.0), (18.5, 22.0)], [150, 98, 56]);
    ic_finish(&mut g);
    g
}

/// The Cowboy's heavy: both pistols fire at the ground and the recoil throws him up.
fn em_gun_boost() -> image::RgbaImage {
    let mut g = ic_new();
    for x in [7.5f32, 16.5] {
        // the pistol, pointing down, with its grip
        ic_poly(&mut g, &[(x - 2.2, 3.0), (x + 2.2, 3.0), (x + 2.2, 12.0), (x - 2.2, 12.0)], [168, 170, 184]);
        ic_poly(&mut g, &[(x - 2.2, 3.0), (x + 4.5, 3.0), (x + 4.5, 6.0), (x - 2.2, 6.0)], [120, 122, 138]);
        // the blast
        ic_poly(&mut g, &[(x - 3.0, 12.0), (x + 3.0, 12.0), (x, 21.0)], [255, 150, 36]);
        ic_poly(&mut g, &[(x - 1.6, 12.0), (x + 1.6, 12.0), (x, 18.0)], [255, 234, 120]);
    }
    // smoke and debris at the floor
    ic_ellipse(&mut g, 12.0, 21.5, 9.0, 1.5, [226, 214, 196]);
    ic_finish(&mut g);
    g
}

/// The Cowboy's special: his hat with the glowing blue curse ring round it.
fn em_hat_ring() -> image::RgbaImage {
    let mut g = ic_new();
    ic_ring(&mut g, 12.0, 12.0, 10.5, 9.5, 2.0, [112, 184, 255]);
    ic_poly(&mut g, &[(8.0, 13.5), (8.0, 9.5), (10.0, 7.0), (14.0, 7.0), (16.0, 9.5), (16.0, 13.5)], [150, 100, 58]);
    ic_ellipse(&mut g, 12.0, 14.0, 7.5, 2.2, [118, 76, 42]);
    ic_poly(&mut g, &[(8.0, 12.0), (16.0, 12.0), (16.0, 13.4), (8.0, 13.4)], [40, 30, 28]);
    ic_finish(&mut g);
    g
}

/// The Scribe's light: a feather quill with a drop of ink.
fn em_quill() -> image::RgbaImage {
    let mut g = ic_new();
    ic_poly(&mut g, &[(7.0, 18.0), (9.0, 9.0), (15.0, 4.0), (21.0, 3.0), (19.5, 10.0), (13.0, 16.0)], [244, 244, 252]);
    ic_poly(&mut g, &[(12.0, 12.0), (15.0, 7.0), (20.0, 4.5), (19.0, 9.0), (14.0, 13.0)], [170, 190, 236]);
    ic_line(&mut g, (4.5, 20.5), (17.0, 6.0), 1.4, [214, 200, 170]);
    ic_poly(&mut g, &[(2.5, 22.5), (6.0, 18.0), (7.5, 19.5)], [40, 40, 52]);
    ic_ellipse(&mut g, 4.0, 15.5, 1.6, 2.0, [64, 72, 160]);
    ic_finish(&mut g);
    g
}

/// The Scribe's heavy: a purple portal with knives pouring out of it.
fn em_knife_rain() -> image::RgbaImage {
    let mut g = ic_new();
    ic_ellipse(&mut g, 12.0, 5.0, 10.0, 3.4, [128, 56, 200]);
    ic_ellipse(&mut g, 12.0, 5.0, 6.5, 1.9, [16, 4, 34]);
    for (x, y) in [(6.0f32, 8.0), (12.0, 11.0), (18.0, 9.0)] {
        ic_line(&mut g, (x, y - 3.0), (x, y), 2.0, [124, 80, 46]);
        ic_poly(&mut g, &[(x - 2.4, y), (x + 2.4, y), (x - 1.8, y + 1.2), (x + 1.8, y + 1.2), (x, y + 10.0)], [216, 222, 236]);
        ic_poly(&mut g, &[(x - 2.4, y), (x + 2.4, y), (x + 2.4, y + 1.0), (x - 2.4, y + 1.0)], [214, 170, 70]);
    }
    ic_finish(&mut g);
    g
}

/// The Scribe's special: Jesus with a halo, and his beam of light coming down.
fn em_jesus_beam() -> image::RgbaImage {
    let mut g = ic_new();
    // the beam
    ic_poly(&mut g, &[(9.0, 12.0), (15.0, 12.0), (20.5, 22.0), (3.5, 22.0)], [255, 238, 150]);
    ic_poly(&mut g, &[(10.5, 12.0), (13.5, 12.0), (16.5, 22.0), (7.5, 22.0)], [255, 255, 238]);
    // Jesus: arms out, a white robe, a head with long hair, a gold halo
    ic_path(&mut g, &[(4.0, 8.5), (9.0, 9.0), (15.0, 9.0), (20.0, 8.5)], 2.0, [244, 244, 252]);
    ic_poly(&mut g, &[(9.0, 8.0), (15.0, 8.0), (16.5, 13.0), (7.5, 13.0)], [244, 244, 252]);
    ic_ellipse(&mut g, 12.0, 5.5, 3.0, 3.4, [124, 80, 46]);
    ic_ellipse(&mut g, 12.0, 6.0, 2.0, 2.3, [238, 196, 158]);
    ic_ring(&mut g, 12.0, 2.4, 4.5, 1.6, 1.0, [255, 214, 80]);
    ic_finish(&mut g);
    g
}

/// Every skill icon: (sprite folder, button, image).
fn skill_icons() -> Vec<(&'static str, &'static str, image::RgbaImage)> {
    let tile = |e: image::RgbaImage, top: Rgb, bottom: Rgb, accent: Rgb| ic_tile(&e, top, bottom, accent);
    let mut v = vec![
        ("anaran", "light", tile(em_scroll_missile(), [96, 78, 214], [28, 18, 96], [240, 200, 90])),
        ("anaran", "heavy", tile(em_staff_slam(), [152, 92, 204], [48, 22, 84], [226, 176, 255])),
        ("anaran", "special", tile(em_goblin(), [112, 190, 92], [24, 76, 38], [204, 244, 154])),
        ("ayz", "light", tile(em_speech("uh"), [140, 172, 214], [44, 68, 112], [232, 238, 252])),
        ("ayz", "heavy", tile(em_collar_grab(), [242, 154, 62], [128, 58, 18], [255, 224, 154])),
        ("ayz", "special", tile(em_raven_rose(), [78, 66, 138], [18, 14, 52], [196, 176, 255])),
        ("cowboy", "light", tile(em_lasso_gun(), [224, 92, 82], [98, 28, 30], [255, 204, 184])),
        ("cowboy", "heavy", tile(em_gun_boost(), [214, 142, 70], [96, 50, 24], [255, 222, 154])),
        ("cowboy", "special", tile(em_hat_ring(), [82, 134, 232], [22, 38, 102], [172, 224, 255])),
        ("scribe", "light", tile(em_quill(), [154, 164, 92], [58, 68, 32], [232, 232, 172])),
        ("scribe", "heavy", tile(em_knife_rain(), [114, 62, 164], [38, 18, 72], [232, 192, 255])),
        ("scribe", "special", tile(em_jesus_beam(), [236, 196, 88], [122, 82, 24], [255, 244, 176])),
        ("junkhead", "light", tile(em_whip_curl(), [212, 72, 72], [78, 18, 26], [255, 172, 162])),
        ("junkhead", "heavy", tile(em_whip_hook(), [142, 142, 154], [48, 48, 60], [222, 222, 238])),
        ("junkhead", "special", tile(em_whip_spiral(), [232, 62, 84], [88, 10, 40], [255, 192, 124])),
    ];
    // The Negaprofessor's icons are the Professor's in negative colours.
    let nega: Vec<_> = v.iter().filter(|(f, _, _)| *f == "ayz").map(|(_, b, img)| ("evilayz", *b, invert(img))).collect();
    v.extend(nega);
    v
}

/// The skill icons, large, in rows by character (`--skill-icons <assets dir> <sheet.png>` also writes them).
fn skill_icon_sheet() -> image::RgbaImage {
    let icons = skill_icons();
    let scale = 8u32;
    let cell = IC as u32 * scale + 12;
    let rows = icons.len() as u32 / 3;
    let mut sheet = image::RgbaImage::from_pixel(3 * cell, rows * cell, image::Rgba([60, 64, 76, 255]));
    for (k, (_, _, img)) in icons.iter().enumerate() {
        let (ox, oy) = ((k as u32 % 3) * cell + 6, (k as u32 / 3) * cell + 6);
        for y in 0..IC as u32 * scale {
            for x in 0..IC as u32 * scale {
                let p = img.get_pixel(x / scale, y / scale);
                if p[3] > 0 {
                    sheet.put_pixel(ox + x, oy + y, *p);
                }
            }
        }
    }
    sheet
}

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next().unwrap_or_else(|| "ayz_default.png".into());

    // `--frames <assets dir> <contact sheet.png>`: draw every pose and write them into the game's
    // sprite folders (Evilayz is the same drawings, inverted), plus a contact sheet.
    if first == "--frames" {
        let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
        let default = ayz_default();
        for (name, negative) in [("ayz", false), ("evilayz", true)] {
            let folder = assets.join("sprites").join(name);
            std::fs::create_dir_all(&folder).unwrap();
            // The Professor has a raven (perched on every pose, and the flying drawings, the rose
            // and the words); the negative twin has exactly the same, inverted.
            let mut every = all_frames(&default, true);
            every.extend(raven_effects());
            for (frame, img) in every {
                // Neither of them uses the old fish or the moon's star any more.
                if frame.starts_with("fish_") || frame.starts_with("moon_star_") {
                    continue;
                }
                // The scar overlay stays white: the game tints it.
                let img = if negative && !frame.ends_with("_mark") { invert(&img) } else { img };
                img.save(folder.join(format!("{frame}.png"))).unwrap();
            }
        }
        if let Some(sheet) = args.next() {
            contact_sheet(&default, 4).save(&sheet).unwrap();
        }
        println!("wrote all frames");
        return;
    }

    // `--scribe <default pose.png> <assets dir> <contact sheet.png>`: draw every pose of the Scribe from
    // the default pose into the game's sprite folder, with his effects and a contact sheet.
    if first == "--scribe" {
        let default = grid_from_png(&args.next().expect("default pose png"));
        let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
        let folder = assets.join("sprites").join("scribe");
        std::fs::create_dir_all(&folder).unwrap();
        for (frame, img) in gs_frames(&default) {
            img.save(folder.join(format!("{frame}.png"))).unwrap();
        }
        if let Some(sheet) = args.next() {
            gs_contact_sheet(&default, 4).save(&sheet).unwrap();
        }
        println!("wrote the Scribe's frames");
        return;
    }

    // `--roll-frames <assets dir> [sheet.png]`: the roll drawings for every character, made from its
    // crouch drawing (and the `_mark` and `_nr` versions): the crouch tucked tighter, a tight ball turned
    // in eighths of a turn, and a rise. `sheet.png` is a look at the first character's.
    if first == "--roll-frames" {
        let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
        let sheet = args.next();
        let mut sheet_done = false;
        let mut folders: Vec<_> = std::fs::read_dir(assets.join("sprites")).unwrap().flatten().map(|e| e.path()).filter(|p| p.join("crouch.png").exists()).collect();
        folders.sort();
        for folder in folders {
            let body = image::open(folder.join("crouch.png")).unwrap().to_rgba8();
            for suffix in ["", "_nr", "_mark"] {
                let path = folder.join(format!("crouch{suffix}.png"));
                if !path.exists() {
                    continue;
                }
                let img = image::open(&path).unwrap().to_rgba8();
                let frames = roll_drawings(&img, &body);
                for (k, f) in frames.iter().enumerate() {
                    f.save(folder.join(format!("roll_{k}{suffix}.png"))).unwrap();
                }
                if suffix.is_empty() && !sheet_done {
                    if let Some(out) = &sheet {
                        let scale = 4;
                        let (w, h) = (body.width() * scale, body.height() * scale);
                        let mut s = image::RgbaImage::from_pixel(w * frames.len() as u32, h, image::Rgba([196, 214, 232, 255]));
                        for (k, f) in frames.iter().enumerate() {
                            for y in 0..h {
                                for x in 0..w {
                                    let p = f.get_pixel(x / scale, y / scale);
                                    if p[3] > 0 {
                                        s.put_pixel(k as u32 * w + x, y, *p);
                                    }
                                }
                            }
                        }
                        s.save(out).unwrap();
                        sheet_done = true;
                    }
                }
            }
            println!("rolled {}", folder.display());
        }
        return;
    }

    // `--dump <in.png> <out.txt>`: a sprite as letters, for reading its coordinates.
    if first == "--dump" {
        let g = grid_from_png(&args.next().expect("input png"));
        let text: Vec<String> = (0..g.h).map(|y| format!("{y:2} {}", g.row(y).iter().collect::<String>())).collect();
        std::fs::write(args.next().expect("output txt"), text.join("
")).unwrap();
        return;
    }

    // `--texture-hair <in.png> <out.png>`: the hair in the hand-drawn sprite is one flat colour; give it
    // texture and shading without touching its shape or anything else.
    if first == "--texture-hair" {
        let input = args.next().expect("input png");
        let output = args.next().expect("output png");
        texture_hair(&input, &output);
        return;
    }

    // `--icons <assets dir>`: the small icons (the broken-shield icon) into assets/effects.
    if first == "--icons" {
        write_icons(&std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into())));
        println!("wrote the icons");
        return;
    }

    // `--cowboy-frames <assets dir> <contact sheet.png>`: draw every pose of Cowboy From Hell from his
    // default pose into the game's sprite folder, and the curse's drawings into
    // assets/effects.
    if first == "--cowboy-frames" {
        let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
        let default = cb_default();
        let folder = assets.join("sprites").join("cowboy");
        std::fs::create_dir_all(&folder).unwrap();
        for (frame, img) in cb_frames(&default) {
            img.save(folder.join(format!("{frame}.png"))).unwrap();
        }
        write_curse_art(&assets);
        write_icons(&assets);
        if let Some(sheet) = args.next() {
            cb_contact_sheet(&default, 4).save(&sheet).unwrap();
        }
        println!("wrote the cowboy's frames and the curse art");
        return;
    }

    // `--junkhead-bare <out.png>`: Junkhead's default pose at the real pixel size (one image pixel
    // per sprite pixel, transparent background), for editing by hand.
    if first == "--junkhead-bare" {
        let out = args.next().unwrap_or_else(|| "junkhead_default_1x.png".into());
        let mut one = jh_default().to_png();
        junkhead_recolour(&mut one);
        one.save(&out).unwrap(); // (the same drawing as tools/junkhead_final.png)
        println!("wrote {out} ({}x{})", one.width(), one.height());
        return;
    }

    // `--junkhead-hands`: where his whip hand is in each lashing pose (image pixels, for src/anim.rs).
    if first == "--junkhead-hands" {
        for (name, frame, left) in [
            ("punch_1", Frame::Punch(1), false),
            ("punch_3", Frame::Punch(3), true),
            ("crouch_soft_1", Frame::CrouchSoft(1), false),
            ("hard_2", Frame::Hard(2), false),
            ("air_hard_1", Frame::AirHard(1), false),
            ("summon_4", Frame::Summon(4), false),
        ] {
            let h = jh_hand_xy(frame, left);
            println!("{name} {} {}", h.0 as f32 + 3.0, h.1 as f32 + 0.5);
        }
        return;
    }

    // `--junkhead-whip <out.png>`: his whip poses (the body and the whip layer together), no files
    // are written to the game.
    if first == "--junkhead-whip" {
        let out = args.next().unwrap_or_else(|| "junkhead_whip.png".into());
        jh_whip_sheet(&jh_default()).save(&out).unwrap();
        println!("wrote {out}");
        return;
    }

    // `--skill-icons <assets dir> <sheet.png>`: draw the skill-bar icons of every character into
    // `assets/icons/<sprite folder>/<light|heavy|special>.png`, and a big contact sheet to look at.
    if first == "--skill-icons" {
        let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
        for (folder, button, img) in skill_icons() {
            let dir = assets.join("icons").join(folder);
            std::fs::create_dir_all(&dir).unwrap();
            img.save(dir.join(format!("{button}.png"))).unwrap();
        }
        if let Some(sheet) = args.next() {
            skill_icon_sheet().save(&sheet).unwrap();
        }
        println!("wrote the skill icons");
        return;
    }

    // `--raven <out.png>`: the raven (perched, flying, pecking, clawing), the rose and the words, big.
    if first == "--raven" {
        let out = args.next().unwrap_or_else(|| "raven.png".into());
        raven_sheet().save(&out).unwrap();
        println!("wrote {out}");
        return;
    }

    // `--cowboy-bare <out.png>`: the Cowboy From Hell's default pose at the real pixel size (same
    // canvas as `--junkhead-bare`), for editing by hand.
    if first == "--cowboy-bare" {
        let out = args.next().unwrap_or_else(|| "cowboy_default_1x.png".into());
        let one = cb_default().to_png();
        one.save(&out).unwrap();
        println!("wrote {out} ({}x{})", one.width(), one.height());
        return;
    }

    // `--graypersona <out.png>`: her default pose (12x, white background) and the 1x file
    // (`<out>_1x.png`, transparent) for editing by hand. No other poses are drawn.
    if first == "--graypersona" {
        let out = args.next().unwrap_or_else(|| "graypersona_default.png".into());
        let g = gp_default();
        g.render(12).save(&out).unwrap();
        let mut one = g.render(1);
        for p in one.pixels_mut() {
            if p.0 == [255, 255, 255, 255] {
                *p = image::Rgba([0, 0, 0, 0]);
            }
        }
        one.save(Path::new(&out).with_file_name("graypersona_default_1x.png")).unwrap();
        println!("wrote {out}");
        return;
    }

    // `--junkhead <out.png>`: only Junkhead's default pose next to the cowboy's, for review (no
    // other poses are drawn).
    if first == "--junkhead" {
        let out = args.next().unwrap_or_else(|| "junkhead_default.png".into());
        let (a, mut b) = (cb_default().to_png(), jh_default().to_png());
        junkhead_recolour(&mut b);
        let scale = 10;
        let (w, h) = (a.width() * scale, a.height() * scale);
        let mut sheet = image::RgbaImage::from_pixel(w * 2 + 3 * scale, h, image::Rgba([196, 214, 232, 255]));
        for (k, img) in [a, b].iter().enumerate() {
            for y in 0..h {
                for x in 0..w {
                    let p = img.get_pixel(x / scale, y / scale);
                    if p[3] > 0 {
                        sheet.put_pixel(x + k as u32 * (w + 3 * scale), y, *p);
                    }
                }
            }
        }
        sheet.save(&out).unwrap();
        println!("wrote {out}");
        return;
    }

    // `--junkhead-frames <assets dir> <contact sheet.png>`: Junkhead is the Cowboy From Hell with a
    // black tank top (for now). His folder gets every cowboy drawing with the tank top's browns
    // (body, dark rib, pale rib) swapped for blacks, (the portrait is made by hand; this does not write it).
    if first == "--junkhead-frames" {
        let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
        let blacken = junkhead_recolour;
        let default = jh_default();
        let folder = assets.join("sprites").join("junkhead");
        std::fs::create_dir_all(&folder).unwrap();
        for (frame, mut img) in jh_frames(&default) {
            blacken(&mut img);
            img.save(folder.join(format!("{frame}.png"))).unwrap();
        }
        if let Some(sheet) = args.next() {
            let (a, b) = (cb_default().to_png(), {
                let mut j = default.to_png();
                blacken(&mut j);
                j
            });
            let scale = 10;
            let (w, h) = (a.width() * scale, a.height() * scale);
            let mut out = image::RgbaImage::from_pixel(w * 2 + 3 * scale, h, image::Rgba([196, 214, 232, 255]));
            for (k, img) in [a, b].iter().enumerate() {
                for y in 0..h {
                    for x in 0..w {
                        let p = img.get_pixel(x / scale, y / scale);
                        if p[3] > 0 {
                            out.put_pixel(x + k as u32 * (w + 3 * scale), y, *p);
                        }
                    }
                }
            }
            out.save(&sheet).unwrap();
        }
        println!("wrote Junkhead's frames and portrait");
        return;
    }

    // `--anaran-capes <out.png>`: the cape designs side by side (1 bell, 2 open cloak, 3 mantle only).
    if first == "--anaran-capes" {
        let out = args.next().unwrap_or_else(|| "anaran_capes.png".into());
        let scale = 10;
        let imgs: Vec<image::RgbaImage> = [1u8, 2, 3].iter().map(|&c| anaran_with_cape(c).render(scale)).collect();
        let gap = 4 * scale;
        let height = imgs.iter().map(|i| i.height()).max().unwrap();
        let width: u32 = imgs.iter().map(|i| i.width()).sum::<u32>() + gap * (imgs.len() as u32 - 1);
        let mut sheet = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 255, 255, 255]));
        let mut x = 0;
        for img in &imgs {
            image::imageops::overlay(&mut sheet, img, x as i64, (height - img.height()) as i64);
            x += img.width() + gap;
        }
        sheet.save(&out).unwrap();
        println!("wrote {out}");
        return;
    }

    // `--anaran-frames <assets dir> <contact sheet.png>`: draw every pose of Anaran from the hand-edited
    // default pose into the game's sprite folder, plus the runes and the goblins.
    if first == "--anaran-frames" {
        let assets = std::path::PathBuf::from(args.next().unwrap_or_else(|| "assets".into()));
        let default = an_default();
        let folder = assets.join("sprites").join("anaran");
        std::fs::create_dir_all(&folder).unwrap();
        for (frame, img) in an_frames(&default) {
            img.save(folder.join(format!("{frame}.png"))).unwrap();
        }
        if let Some(sheet) = args.next() {
            an_contact_sheet(&default, 4).save(&sheet).unwrap();
        }
        println!("wrote Anaran's frames");
        return;
    }

    // `--anaran-bare <out.png>`: Anaran at the real pixel size (one image pixel per sprite pixel, transparent
    // background) for editing by hand, and a 12x copy to look at.
    if first == "--anaran-bare" {
        let out = args.next().unwrap_or_else(|| "anaran_default_1x.png".into());
        let grid = anaran_default();
        let mut one = grid.render(1);
        for p in one.pixels_mut() {
            if p.0 == [255, 255, 255, 255] {
                *p = image::Rgba([0, 0, 0, 0]);
            }
        }
        one.save(&out).unwrap();
        let big = Path::new(&out).with_file_name("anaran_default_x12.png");
        grid.render(12).save(&big).unwrap();
        println!("wrote {out} ({}x{}) and {}", one.width(), one.height(), big.display());
        return;
    }

    // `--anaran <out.png>`: Anaran's default pose next to the Professor's, for review.
    if first == "--anaran" {
        let out = args.next().unwrap_or_else(|| "anaran_default.png".into());
        let scale = 12;
        let (a, b) = (ayz_default().render(scale), anaran_default().render(scale));
        let gap = 3 * scale;
        let height = a.height().max(b.height());
        let mut sheet = image::RgbaImage::from_pixel(a.width() + gap + b.width(), height, image::Rgba([255, 255, 255, 255]));
        image::imageops::overlay(&mut sheet, &a, 0, (height - a.height()) as i64);
        image::imageops::overlay(&mut sheet, &b, (a.width() + gap) as i64, (height - b.height()) as i64);
        sheet.save(&out).unwrap();
        b.save(Path::new(&out).with_file_name("anaran_default_only.png")).unwrap();
        let text: Vec<String> = (0..anaran_default().h).map(|y| format!("{y:2} {}", anaran_default().row(y).iter().collect::<String>())).collect();
        std::fs::write(Path::new(&out).with_extension("txt"), text.join("\n")).unwrap();
        println!("wrote {out}");
        return;
    }

    // `--cowboy <out.png>`: Cowboy From Hell's default pose next to the Professor's, for review.
    if first == "--cowboy" {
        let out = args.next().unwrap_or_else(|| "cowboy_default.png".into());
        let scale = 12;
        let (a, b) = (ayz_default().render(scale), cowboy_default().render(scale));
        let gap = 3 * scale;
        let height = a.height().max(b.height());
        let mut sheet = image::RgbaImage::from_pixel(a.width() + gap + b.width(), height, image::Rgba([255, 255, 255, 255]));
        image::imageops::overlay(&mut sheet, &a, 0, (height - a.height()) as i64);
        image::imageops::overlay(&mut sheet, &b, (a.width() + gap) as i64, (height - b.height()) as i64);
        sheet.save(&out).unwrap();
        b.save(Path::new(&out).with_file_name("cowboy_default_only.png")).unwrap();
        let text: Vec<String> = (0..cowboy_default().h).map(|y| format!("{y:2} {}", cowboy_default().row(y).iter().collect::<String>())).collect();
        std::fs::write(Path::new(&out).with_extension("txt"), text.join("\n")).unwrap();
        println!("wrote {out}");
        return;
    }

    // `--soul-bare <out.png>`: the Gunslinger without the long hair at the sides, at 1x and 12x.
    if first == "--soul-bare" {
        let out = args.next().unwrap_or_else(|| "gunslinger_no_hair.png".into());
        let bare = soul_with_hair(false);
        let mut one = bare.render(1);
        for p in one.pixels_mut() {
            if p.0 == [255, 255, 255, 255] {
                *p = image::Rgba([0, 0, 0, 0]);
            }
        }
        one.save(&out).unwrap();
        let big = Path::new(&out).with_file_name("gunslinger_no_hair_x12.png");
        bare.render(12).save(&big).unwrap();
        println!("wrote {out} and {}", big.display());
        return;
    }

    // `--soul <out.png>`: the Gunslinger's default pose next to the Professor's, for review.
    if first == "--soul" {
        let out = args.next().unwrap_or_else(|| "soul_default.png".into());
        let scale = 12;
        let (a, b) = (ayz_default().render(scale), soul_default().render(scale));
        let gap = 3 * scale;
        let height = a.height().max(b.height());
        let mut sheet = image::RgbaImage::from_pixel(a.width() + gap + b.width(), height, image::Rgba([255, 255, 255, 255]));
        image::imageops::overlay(&mut sheet, &a, 0, (height - a.height()) as i64);
        image::imageops::overlay(&mut sheet, &b, (a.width() + gap) as i64, (height - b.height()) as i64);
        sheet.save(&out).unwrap();
        b.save(Path::new(&out).with_file_name("soul_default_only.png")).unwrap();
        let text: Vec<String> = (0..soul_default().h).map(|y| soul_default().row(y).iter().collect()).collect();
        std::fs::write(Path::new(&out).with_extension("txt"), text.join("
")).unwrap();
        println!("wrote {out}");
        return;
    }

    let out = first;
    let original = Grid::parse(REF);
    let ayz = ayz_default();
    let scale = 12;
    let (a, b) = (original.render(scale), ayz.render(scale));
    let gap = 3 * scale;
    let height = a.height().max(b.height());
    let mut sheet = image::RgbaImage::from_pixel(a.width() + gap + b.width(), height, image::Rgba([255, 255, 255, 255]));
    image::imageops::overlay(&mut sheet, &a, 0, (height - a.height()) as i64);
    image::imageops::overlay(&mut sheet, &b, (a.width() + gap) as i64, (height - b.height()) as i64);
    sheet.save(Path::new(&out)).unwrap();
    // The pixel letters as text, for checking the shapes row by row.
    let text: Vec<String> = (0..ayz.h).map(|y| ayz.row(y).iter().collect()).collect();
    std::fs::write(Path::new(&out).with_extension("txt"), text.join("\n")).unwrap();
    // The new default pose on its own, for reference.
    let alone = Path::new(&out).with_file_name("ayz_default_only.png");
    b.save(&alone).unwrap();
    println!("wrote {out} and {}", alone.display());
}
