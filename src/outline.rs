//! A slight pastel stroke and glow around each fighter: blue for you, red for your opponent (online each side sees
//! itself in blue). It is a second sprite behind the body whose picture is the body's current drawing, grown into a
//! ring; the rings are made once per drawing and colour and kept.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::online::NetSession;
use crate::render::SpriteOf;

/// The ring sprite of a fighter.
#[derive(Component)]
pub struct OutlineOf(pub usize);

const BLUE: [f32; 3] = [0.62, 0.78, 1.0];
const RED: [f32; 3] = [1.0, 0.62, 0.68];
/// How far the glow reaches, in pixels of the drawing (the first of them is the solid stroke).
const REACH: i32 = 4;
const STROKE: f32 = 1.0;

/// The ring pictures made so far: (drawing, whether blue) -> ring. Cleared when a fight is left.
#[derive(Resource, Default)]
pub struct Rings(pub HashMap<(AssetId<Image>, bool), Handle<Image>>);

/// A picture's solid pixels grown into a ring of `rgb`: stroke next to the art, a fading glow beyond it, nothing on
/// the art itself (the body covers it).
fn ring(source: &Image, rgb: [f32; 3]) -> Option<Image> {
    if source.texture_descriptor.format != TextureFormat::Rgba8UnormSrgb {
        return None;
    }
    let data = source.data.as_ref()?;
    let (w, h) = (source.width() as i32, source.height() as i32);
    if data.len() != (w * h * 4) as usize {
        return None;
    }
    let solid = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && data[((y * w + x) * 4 + 3) as usize] > 40;
    let to_byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut out = vec![0u8; data.len()];
    for y in 0..h {
        for x in 0..w {
            if solid(x, y) {
                continue;
            }
            // distance to the nearest solid pixel within reach
            let mut nearest = f32::MAX;
            for dy in -REACH..=REACH {
                for dx in -REACH..=REACH {
                    if (dx != 0 || dy != 0) && solid(x + dx, y + dy) {
                        nearest = nearest.min(((dx * dx + dy * dy) as f32).sqrt());
                    }
                }
            }
            let alpha = if nearest <= STROKE {
                0.95
            } else if nearest <= REACH as f32 {
                let fade = 1.0 - (nearest - STROKE) / (REACH as f32 - STROKE + 0.5);
                0.42 * fade * fade
            } else {
                continue;
            };
            let at = ((y * w + x) * 4) as usize;
            out[at] = to_byte(rgb[0]);
            out[at + 1] = to_byte(rgb[1]);
            out[at + 2] = to_byte(rgb[2]);
            out[at + 3] = to_byte(alpha);
        }
    }
    Some(Image::new(
        Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        out,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

/// Keep each ring sprite on its fighter's current drawing, in the colour of its side.
pub fn animate_outline(
    net: Option<Res<NetSession>>,
    mut images: ResMut<Assets<Image>>,
    bodies: Query<(&SpriteOf, &Sprite), Without<OutlineOf>>,
    mut rings: Query<(&OutlineOf, &mut Sprite, &mut Visibility)>,
    mut made: ResMut<Rings>,
) {
    // you are the fighter you control: the left one against the CPU, your own side online
    let me = net.map_or(0, |n| n.local);
    for (OutlineOf(i), mut ring_sprite, mut visibility) in &mut rings {
        let Some((_, body)) = bodies.iter().find(|(SpriteOf(j), _)| j == i) else { continue };
        let blue = *i == me;
        let key = (body.image.id(), blue);
        if !made.0.contains_key(&key) {
            let Some(source) = images.get(&body.image) else { continue };
            let Some(image) = ring(source, if blue { BLUE } else { RED }) else { continue };
            made.0.insert(key, images.add(image));
        }
        let handle = &made.0[&key];
        if ring_sprite.image != *handle {
            ring_sprite.image = handle.clone();
        }
        // it fades and hides with the body (a portal, a knock-out)
        let alpha = body.color.to_srgba().alpha;
        let color = Color::srgba(1.0, 1.0, 1.0, alpha);
        if ring_sprite.color != color {
            ring_sprite.color = color;
        }
        let wanted = if alpha > 0.0 { Visibility::Inherited } else { Visibility::Hidden };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}
