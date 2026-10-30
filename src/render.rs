//! Draws the fighters from whole-body frames (see `t2f::anim`), plus the summon crescent and the star.

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use t2f::anim::{frame_for, Frame, CRESCENT_FRAMES, FRAME_W, FULL_H, STAR_FRAMES};
use t2f::sim::{Fighter, FighterState, GameState, MarkKind, Phase, ProjectileData, PX};

use crate::{AppState, Roster, Selection, Sim, GROUND_Y};

/// Image handles of every drawing: one map per fighter, keyed by file name.
#[derive(Resource, Default)]
pub struct FrameHandles(pub Vec<HashMap<String, Handle<Image>>>);

/// A fighter's root, at their feet.
#[derive(Component)]
pub struct Body(pub usize);
/// The sprite showing a fighter's current drawing.
#[derive(Component)]
pub struct SpriteOf(pub usize);
#[derive(Component)]
pub struct CrescentOf(pub usize);
/// One of up to `STAR_SLOTS` projectile sprites per fighter (a volley has several in flight).
#[derive(Component)]
pub struct StarOf(pub usize, pub usize);
pub const STAR_SLOTS: usize = 4;
/// Overlay for the special meter (the glowing mark or the hat).
#[derive(Component)]
pub struct MarkOf(pub usize);

/// One pixel of a whip's lash, drawn by the game (`animate_whips`), not an image.
#[derive(Component)]
pub struct WhipPixel(pub usize, pub usize);

const LASH_PIXELS: usize = 220;

/// A fighter's raven while it flies (when perched it's part of the body drawing), and its rose.
#[derive(Component)]
pub struct RavenOf(pub usize);
#[derive(Component)]
pub struct RoseOf(pub usize);

/// The whip layer over a fighter's body.
#[derive(Component)]
pub struct WhipOf(pub usize);
/// The moon the Professor hangs in the sky (world space, not on his body) and the star beside it.
#[derive(Component)]
pub struct MoonOf(pub usize);
#[derive(Component)]
pub struct MoonStarOf(pub usize);
/// Speech bubble over a lecturing fighter (world space, so the text isn't mirrored).
#[derive(Component)]
pub struct BubbleOf(pub usize);
#[derive(Component)]
pub struct BubbleText(pub usize);

/// The curse round a fighter: a root that follows them and its four drawings.
#[derive(Component)]
pub struct CurseRoot(pub usize);
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum CursePart {
    RingLeft,
    RingRight,
    PetalsLeft,
    PetalsRight,
}
#[derive(Component)]
pub struct CurseOf(pub usize);

/// What the lecturing fighter says.
pub const LECTURE_LINE: &str = "here's the thing...";

/// Every drawing's file name for a character.
fn all_stems(frames: &[Frame], effects: &[String], mark: bool, whip: bool, raven: bool) -> Vec<String> {
    let mut stems: Vec<String> = frames.iter().map(|f| f.file_stem()).collect();
    if raven {
        stems.extend(frames.iter().map(|f| format!("{}_nr", f.file_stem())));
        stems.extend(t2f::anim::raven_stems());
    }
    if whip {
        stems.extend(t2f::anim::whip_stems());
    }
    if mark {
        stems.extend(frames.iter().map(|f| format!("{}_mark", f.file_stem())));
    }
    for base in effects {
        let count = if base == "goblin" { 5 } else { CRESCENT_FRAMES.max(STAR_FRAMES) };
        stems.extend((0..count).map(|i| format!("{base}_{i}")));
    }
    stems
}

/// Spawn one fighter and its effects from `assets/sprites/<name>/`; without drawings it falls back to a flat colour box.
pub fn spawn_fighter(
    commands: &mut Commands,
    asset_server: &AssetServer,
    assets_dir: &Path,
    handles: &mut FrameHandles,
    fighter: usize,
    name: &str,
    color: Color,
    summon: Option<&ProjectileData>,
    effects: &[String],
    frames: &[Frame],
    mark: bool,
    whip: bool,
    raven: bool,
) {
    let folder = name.to_lowercase();
    let has_art = assets_dir.join(format!("sprites/{folder}/idle_0.png")).exists();
    let mut map = HashMap::new();
    if has_art {
        for stem in all_stems(frames, effects, mark, whip, raven) {
            map.insert(stem.clone(), asset_server.load(format!("sprites/{folder}/{stem}.png")));
        }
    }
    let get = |stem: &str| map.get(stem).cloned();

    let root = commands
        .spawn((Body(fighter), DespawnOnExit(AppState::Fight), Transform::from_xyz(0.0, 0.0, 1.0 + fighter as f32), Visibility::default()))
        .id();

    // drawings are FRAME_W wide with the body centre left of the middle, so the sprites are shifted to put the centre on the fighter
    let shift = FRAME_W as f32 / 2.0 - t2f::anim::CENTRE_X as f32;
    if let Some(image) = get("idle_0") {
        let body = commands
            .spawn((SpriteOf(fighter), Sprite::from_image(image), Transform::from_xyz(shift, FULL_H as f32 / 2.0, 0.0)))
            .id();
        commands.entity(root).add_child(body);
        if mark {
            let overlay = commands
                .spawn((
                    MarkOf(fighter),
                    Sprite::from_image(get("idle_0_mark").expect("loaded with the rest")),
                    Transform::from_xyz(shift, FULL_H as f32 / 2.0, 0.05),
                ))
                .id();
            commands.entity(root).add_child(overlay);
        }
        if raven {
            // out of the body drawing the raven is its own sprite, flying in the world
            commands.spawn((
                RavenOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("raven_fly_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, 2.6),
                Visibility::Hidden,
            ));
            commands.spawn((
                RoseOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("rose_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, 2.7),
                Visibility::Hidden,
            ));
        }
        if whip {
            // the whip layer is wider than the body drawing but has the same left edge
            let overlay = commands
                .spawn((
                    WhipOf(fighter),
                    Sprite::from_image(get("punch_1_whip").expect("loaded with the rest")),
                    Transform::from_xyz(shift + (t2f::anim::WHIP_W - FRAME_W) as f32 / 2.0, FULL_H as f32 / 2.0, 0.06),
                    Visibility::Hidden,
                ))
                .id();
            commands.entity(root).add_child(overlay);
            for index in 0..LASH_PIXELS {
                let pixel = commands
                    .spawn((
                        WhipPixel(fighter, index),
                        Sprite::from_color(Color::WHITE, Vec2::ONE),
                        Transform::from_xyz(0.0, 0.0, 0.07),
                        Visibility::Hidden,
                    ))
                    .id();
                commands.entity(root).add_child(pixel);
            }
        }
        if let Some(p) = summon.filter(|p| p.lob) {
            // the moon and star hang in the sky at a fixed spot, so they're not children of the body
            commands.spawn((
                MoonOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("crescent_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, -7.0),
                Visibility::Hidden,
            ));
            commands.spawn((
                MoonStarOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("moon_star_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, -6.9),
                Visibility::Hidden,
            ));
            let _ = p;
        }
        if frames.iter().any(|f| matches!(f, Frame::Lecture(_) | Frame::Grab(_))) {
            let width = LECTURE_LINE.len() as f32 * 5.0 + 10.0;
            commands
                .spawn((BubbleOf(fighter), DespawnOnExit(AppState::Fight), Transform::from_xyz(0.0, 0.0, 7.0), Visibility::Hidden))
                .with_children(|b| {
                    b.spawn((Sprite::from_color(Color::BLACK, Vec2::new(width + 2.0, 18.0)), Transform::from_xyz(0.0, 0.0, 0.0)));
                    b.spawn((Sprite::from_color(Color::WHITE, Vec2::new(width, 16.0)), Transform::from_xyz(0.0, 0.0, 0.1)));
                    // the tail pointing at the speaker
                    b.spawn((Sprite::from_color(Color::BLACK, Vec2::new(6.0, 6.0)), Transform::from_xyz(-6.0, -9.0, 0.0).with_rotation(Quat::from_rotation_z(0.785))));
                    b.spawn((Sprite::from_color(Color::WHITE, Vec2::new(4.0, 4.0)), Transform::from_xyz(-6.0, -8.0, 0.2).with_rotation(Quat::from_rotation_z(0.785))));
                    b.spawn((
                        BubbleText(fighter),
                        Text2d::new(""),
                        TextFont { font_size: FontSize::Px(8.0), ..default() },
                        TextColor(Color::BLACK),
                        Transform::from_xyz(0.0, 0.0, 0.3),
                    ));
                });
        }
        if let Some(p) = summon.filter(|p| p.summon_name() != "none" && !p.lob) {
            let crescent = commands
                .spawn((
                    CrescentOf(fighter),
                    Sprite::from_image(get(&format!("{}_0", p.summon_name())).expect("loaded with the rest")),
                    Transform::from_xyz(p.x as f32, p.y as f32, 0.3),
                    Visibility::Hidden,
                ))
                .id();
            commands.entity(root).add_child(crescent);
        }
        for slot in 0..if effects.is_empty() { 0 } else { STAR_SLOTS } {
            commands.spawn((
                StarOf(fighter, slot),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get(&format!("{}_0", effects.first().map(String::as_str).unwrap_or("star"))).expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, 3.0),
                Visibility::Hidden,
            ));
        }
    } else {
        let body = commands
            .spawn((Sprite::from_color(color, Vec2::new(30.0, 52.0)), Transform::from_xyz(0.0, 26.0, 0.0)))
            .id();
        commands.entity(root).add_child(body);
    }
    if handles.0.len() <= fighter {
        handles.0.resize_with(fighter + 1, HashMap::new);
    }
    handles.0[fighter] = map;
}

/// Whether a fighter is speaking: both at the start of a round, the winner at the end.
fn speaking(s: &GameState, fighter: usize) -> bool {
    match s.phase {
        Phase::Fighting => s.frame < 90 || (s.round > 1 && s.timer > GameState::ROUND_FRAMES - 60),
        Phase::RoundEnd(_) => false,
        Phase::MatchOver => s.match_winner() == Some(fighter),
    }
}

/// Tint for a fighter's sprites by state, so hits and blocks read clearly.
fn tint(state: FighterState) -> Color {
    match state {
        FighterState::Hitstun => Color::srgb(1.0, 0.65, 0.65),
        FighterState::Blockstun => Color::srgb(0.7, 0.78, 1.0),
        FighterState::Guard => Color::srgb(0.82, 0.9, 1.0),
        FighterState::Ko => Color::srgb(0.75, 0.75, 0.8),
        _ => Color::WHITE,
    }
}

/// Place each fighter, face the opponent, and knock them over when KO'd.
pub fn pose_bodies(sim: Res<Sim>, mut bodies: Query<(&Body, &mut Transform)>) {
    for (Body(i), mut tf) in &mut bodies {
        let f = &sim.0.fighters[*i];
        tf.translation.x = (f.x / PX) as f32;
        tf.translation.y = GROUND_Y + (f.y / PX) as f32;
        tf.scale.x = f.facing as f32;
    }
}

/// Pick each fighter's drawing.
pub fn animate_body(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    mut sprites: Query<(&SpriteOf, &mut Sprite, &mut Transform)>,
) {
    for (SpriteOf(i), mut sprite, mut tf) in &mut sprites {
        let f = &sim.0.fighters[*i];
        let frame = frame_for(f, &roster.0[sel.chars[*i]], speaking(&sim.0, *i), sim.0.frame);
        // while the raven is out, use the body drawing without it on the shoulder
        let mut stem = frame.file_stem();
        if sim.0.ravens[*i].active {
            stem.push_str("_nr");
        }
        if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&stem)) {
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        }
        sprite.color = tint(f.state);
        // a roll spins the crouched body once round, forwards or backwards
        tf.rotation = if f.state == FighterState::Roll {
            let turns = f.state_frame as f32 / Fighter::ROLL_FRAMES as f32;
            let forward = if f.move_dir * f.facing > 0 { -1.0 } else { 1.0 };
            Quat::from_rotation_z(forward * turns.min(1.0) * std::f32::consts::TAU)
        } else {
            Quat::IDENTITY
        };
    }
}

/// Show the crescent while its owner summons: it appears as the arms go up, twinkles, and fades once the star is sent.
pub fn animate_crescent(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    mut crescents: Query<(&CrescentOf, &mut Sprite, &mut Visibility, &mut Transform)>,
) {
    for (CrescentOf(i), mut sprite, mut visibility, mut tf) in &mut crescents {
        let f = &sim.0.fighters[*i];
        let frame = frame_for(f, &roster.0[sel.chars[*i]], false, sim.0.frame);
        match frame {
            Frame::Summon(n) if n >= 1 => {
                *visibility = Visibility::Inherited;
                let twinkle = (sim.0.frame / 6 % CRESCENT_FRAMES as u32) as usize;
                let base = roster.0[sel.chars[*i]].moves.iter().find_map(|m| m.projectile.as_ref().filter(|_| m.anim == t2f::sim::AnimKind::Summon)).map_or("crescent", |p| p.summon_name());
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("{base}_{twinkle}"))) {
                    sprite.image = image.clone();
                }
                sprite.color = Color::srgba(1.0, 1.0, 1.0, if n >= 5 { 0.5 } else { 1.0 });
                // swells while charging, flashes when fully charged
                let data = &roster.0[sel.chars[*i]];
                let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
                let grow = if mv.charge_frames > 0 { f.charge as f32 / mv.charge_frames as f32 } else { 0.0 };
                tf.scale = Vec3::splat(1.0 + grow * 0.5);
                if mv.charge_frames > 0 && f.charge >= mv.charge_frames && sim.0.frame / 3 % 2 == 0 {
                    sprite.color = Color::srgb(1.6, 1.6, 1.3);
                }
            }
            _ => *visibility = Visibility::Hidden,
        }
    }
}

/// Draw each projectile in flight, turned to face where it's going (or spinning, for a thrown hat).
pub fn animate_stars(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    mut stars: Query<(&StarOf, &mut Sprite, &mut Transform, &mut Visibility)>,
) {
    for (StarOf(i, slot), mut sprite, mut tf, mut visibility) in &mut stars {
        match sim.0.projectiles.iter().filter(|p| p.owner == *i).nth(*slot) {
            Some(p) => {
                *visibility = Visibility::Inherited;
                tf.translation.x = (p.x / PX) as f32;
                tf.translation.y = GROUND_Y + (p.y / PX) as f32;
                let data = roster.0[sel.chars[*i]].moves[p.move_index].projectile.as_ref();
                if data.is_some_and(|d| d.spin) {
                    tf.rotation = Quat::from_rotation_z(-(p.age as f32) * 0.5 * p.vx.signum() as f32);
                    sprite.flip_y = false;
                } else if data.is_some_and(|d| d.upright) {
                    tf.rotation = Quat::IDENTITY;
                    sprite.flip_y = false;
                } else {
                    tf.rotation = Quat::from_rotation_z((p.vy as f32).atan2(p.vx as f32));
                    sprite.flip_y = p.vx < 0;
                }
                // a charged star is bigger
                tf.scale = Vec3::splat(p.power as f32 / 100.0);
                let flicker = (p.age / 3 % STAR_FRAMES as u32) as usize;
                let base = data.map_or("star", |d| d.image_name());
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("{base}_{flicker}"))) {
                    sprite.image = image.clone();
                }
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// The special meter on the fighter: a mark on the skin that fades in then glows, or a hat that appears once the special is ready.
pub fn animate_marks(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    time: Res<Time>,
    handles: Res<FrameHandles>,
    mut marks: Query<(&MarkOf, &mut Sprite)>,
) {
    for (MarkOf(i), mut sprite) in &mut marks {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let frame = frame_for(f, data, speaking(&sim.0, *i), sim.0.frame);
        if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("{}_mark", frame.file_stem()))) {
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        }
        let frac = (f.meter as f32 / Fighter::METER_MAX as f32).clamp(0.0, 1.0);
        let ready = f.meter >= Fighter::METER_MAX;
        sprite.color = match &data.special_mark {
            Some(MarkKind::Scar { color, glow }) => {
                if ready {
                    let pulse = 0.78 + 0.22 * (time.elapsed_secs() * 7.0).sin();
                    Color::srgba(glow.0, glow.1, glow.2, pulse)
                } else {
                    Color::srgba(color.0, color.1, color.2, frac * 0.85)
                }
            }
            Some(MarkKind::Hat) => Color::srgba(1.0, 1.0, 1.0, if ready { 1.0 } else { 0.0 }),
            None => Color::srgba(1.0, 1.0, 1.0, 0.0),
        };
    }
}

/// The last lash a fighter threw, kept so it stays on screen a little after the move: it holds, then slackens and is drawn back in.
#[derive(Clone, Copy)]
pub struct LashMemory {
    hand: (f32, f32),
    tip: (f32, f32),
    age: u32,
}

const LASH_HOLD: u32 = 8;
const LASH_FADE: u32 = 14;

/// Fly each raven that's off its owner's shoulder, and show the rose it carries or drops.
pub fn animate_ravens(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    mut ravens: Query<(&RavenOf, &mut Sprite, &mut Transform, &mut Visibility), Without<RoseOf>>,
    mut roses: Query<(&RoseOf, &mut Sprite, &mut Transform, &mut Visibility), Without<RavenOf>>,
) {
    for (RavenOf(i), mut sprite, mut tf, mut visibility) in &mut ravens {
        let r = &sim.0.ravens[*i];
        let name = t2f::anim::raven_frame(r, &roster.0[sel.chars[*i]], sim.0.frame);
        match name.and_then(|n| handles.0.get(*i).and_then(|m| m.get(&n))) {
            Some(image) => {
                if sprite.image != *image {
                    sprite.image = image.clone();
                }
                tf.translation.x = (r.x / PX) as f32;
                tf.translation.y = GROUND_Y + (r.y / PX) as f32;
                tf.scale.x = r.facing as f32;
                *visibility = Visibility::Inherited;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    for (RoseOf(i), mut sprite, mut tf, mut visibility) in &mut roses {
        match sim.0.ravens[*i].rose {
            Some(rose) => {
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("rose_{}", (sim.0.frame / 6) % 2))) {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
                tf.translation.x = (rose.x / PX) as f32;
                tf.translation.y = GROUND_Y + (rose.y / PX) as f32;
                *visibility = Visibility::Inherited;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// Show the whip layer of the move being thrown, if it has one. The held end is in the body drawing;
/// while the whip lashes out the rest is generated here, a curve of pixels from the hand to the enemy
/// (or the end of its reach), so it always meets them.
pub fn animate_whips(
    mut memory: Local<[Option<LashMemory>; 2]>,
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    mut whips: Query<(&WhipOf, &mut Sprite, &mut Visibility), Without<WhipPixel>>,
    mut pixels: Query<(&WhipPixel, &mut Sprite, &mut Transform, &mut Visibility), Without<WhipOf>>,
) {
    let shift = FRAME_W as f32 / 2.0 - t2f::anim::CENTRE_X as f32;
    for (WhipOf(i), mut sprite, mut visibility) in &mut whips {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let layer = t2f::anim::whip_layer(f, data);
        // lashing pose: no image, the lash is generated instead
        let lash = layer.as_deref().and_then(|name| t2f::anim::lash_hand(name));
        let mut path: Vec<(i32, i32)> = Vec::new();
        if let Some(hand) = lash {
            let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
            let special = mv.anim == t2f::sim::AnimKind::Summon;
            let hb = if f.aim_low { mv.low_hitbox.unwrap_or(mv.hitbox) } else { mv.hitbox };
            let natural = (hb.x + hb.w + if f.boosted { mv.boost_reach } else { 0 } + mv.assist) as f32;
            let foe = &sim.0.fighters[1 - *i];
            let ahead = ((foe.x - f.x) * f.facing) as f32 / PX as f32;
            let tip_fwd = if ahead > 0.0 { (ahead - 6.0).clamp(24.0, natural) } else { natural };
            // height above the feet: the special keeps its fixed high or low line, the others go for the enemy's body
            let (lo, hi) = (hb.y as f32, (hb.y + hb.h) as f32);
            let tip_up = if special {
                (lo + hi) / 2.0 - if mv.world_y { f.y as f32 / PX as f32 } else { 0.0 }
            } else {
                let foe_mid = (foe.y + foe.height() * PX / 2) as f32 / PX as f32 - f.y as f32 / PX as f32;
                foe_mid.clamp(lo - mv.assist as f32, hi + mv.assist as f32)
            }
            .max(3.0);
            // the lash snaps out over its first frames
            let ext = (f.state_frame.saturating_sub(mv.startup) as f32 / 3.0).clamp(0.35, 1.0);
            let tip_full = (t2f::anim::CENTRE_X as f32 + tip_fwd, FULL_H as f32 - tip_up);
            let tip = (hand.0 + (tip_full.0 - hand.0) * ext, hand.1 + (tip_full.1 - hand.1) * ext);
            path = lash_path(hand, tip, sim.0.frame as f32 * 0.9, 2.4 - 1.6 * ext);
            memory[*i] = Some(LashMemory { hand, tip: tip_full, age: 0 });
        } else if let Some(mem) = memory[*i].as_mut() {
            // after the lash: it hangs for a moment, then droops and is drawn back in
            let other_layer = layer.as_deref().is_some_and(|n| !n.starts_with("hard_3") && !n.starts_with("summon_5"));
            mem.age += 1;
            if other_layer || mem.age > LASH_HOLD + LASH_FADE || matches!(f.state, FighterState::Hitstun | FighterState::Ko) {
                memory[*i] = None;
            } else {
                let back = mem.age.saturating_sub(LASH_HOLD) as f32 / LASH_FADE as f32;
                let ext = 1.0 - back;
                let tip = (mem.hand.0 + (mem.tip.0 - mem.hand.0) * ext, mem.hand.1 + (mem.tip.1 - mem.hand.1) * ext + back * 14.0);
                path = lash_path(mem.hand, tip, sim.0.frame as f32 * 0.9, 0.8 + 1.8 * back);
            }
        }
        if lash.is_some() || !path.is_empty() {
            *visibility = Visibility::Hidden;
        } else {
            match layer.and_then(|name| handles.0.get(*i).and_then(|m| m.get(&name))) {
                Some(image) => {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                    sprite.color = tint(f.state);
                    *visibility = Visibility::Inherited;
                }
                None => *visibility = Visibility::Hidden,
            }
        }
        for (WhipPixel(owner, index), mut px_sprite, mut tf, mut vis) in &mut pixels {
            if owner != i {
                continue;
            }
            match path.get(*index) {
                Some(&(x, y)) => {
                    px_sprite.color = lash_colour(*index, path.len(), tint(f.state));
                    tf.translation.x = shift - FRAME_W as f32 / 2.0 + x as f32 + 0.5;
                    tf.translation.y = FULL_H as f32 - y as f32 - 0.5;
                    *vis = Visibility::Inherited;
                }
                None => *vis = Visibility::Hidden,
            }
        }
    }
}

/// The pixels of a whip from `hand` to `tip` (picture pixels): a quadratic curve with a ripple running
/// down it. `ripple` is how far it wobbles sideways, `phase` moves the ripple along.
fn lash_path(hand: (f32, f32), tip: (f32, f32), phase: f32, ripple: f32) -> Vec<(i32, i32)> {
    let (dx, dy) = (tip.0 - hand.0, tip.1 - hand.1);
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let (nx, ny) = (-dy / len, dx / len);
    let mid = ((hand.0 + tip.0) / 2.0, (hand.1 + tip.1) / 2.0 - 3.0);
    let mut path: Vec<(i32, i32)> = Vec::new();
    let steps = (len * 4.0) as usize + 4;
    for k in 0..=steps {
        let t = k as f32 / steps as f32;
        let (a, b, c) = ((1.0 - t) * (1.0 - t), 2.0 * (1.0 - t) * t, t * t);
        let (mut x, mut y) = (a * hand.0 + b * mid.0 + c * tip.0, a * hand.1 + b * mid.1 + c * tip.1);
        let wave = (t * 9.0 - phase).sin() * ripple * (t * 3.0).min(1.0) * (1.0 - t * 0.5);
        x += nx * wave;
        y += ny * wave;
        let p = (x.round() as i32, y.round() as i32);
        if path.last() != Some(&p) {
            path.push(p);
        }
    }
    path.truncate(LASH_PIXELS - 8);
    // thicker handle: its first pixels are doubled just below
    let thick: Vec<(i32, i32)> = path.iter().take(8).map(|&(x, y)| (x, y + 1)).collect();
    path.extend(thick);
    path
}

/// Dark leather with a lighter glint every few pixels and a pale cracker at the tip; the doubled handle
/// pixels at the end are dark.
fn lash_colour(index: usize, len: usize, tint: Color) -> Color {
    let body = len.saturating_sub(8);
    let c = if index >= body {
        [86, 52, 30]
    } else if index + 3 >= body {
        [236, 218, 180]
    } else if index % 5 == 2 {
        [182, 128, 78]
    } else if index % 5 == 4 {
        [128, 80, 46]
    } else {
        [86, 52, 30]
    };
    let base = Color::srgb_u8(c[0], c[1], c[2]);
    if tint == Color::WHITE { base } else { base.mix(&tint, 0.5) }
}

/// The moon and its star hang where the special was summoned and fade after a while.
pub fn animate_moons(
    sim: Res<Sim>,
    handles: Res<FrameHandles>,
    time: Res<Time>,
    mut moons: Query<(&MoonOf, &mut Sprite, &mut Transform, &mut Visibility), Without<MoonStarOf>>,
    mut stars: Query<(&MoonStarOf, &mut Sprite, &mut Transform, &mut Visibility), Without<MoonOf>>,
) {
    for (MoonOf(i), mut sprite, mut tf, mut visibility) in &mut moons {
        match sim.0.fighters[*i].moon {
            Some((x, y, age)) => {
                *visibility = Visibility::Inherited;
                tf.translation.x = (x / PX) as f32;
                tf.translation.y = GROUND_Y + (y / PX) as f32;
                tf.scale = Vec3::splat((age as f32 / 12.0).min(1.0).max(0.1));
                let fade = ((Fighter::MOON_LIFE - age.min(Fighter::MOON_LIFE)) as f32 / 25.0).min(1.0);
                sprite.color = Color::srgba(1.0, 1.0, 1.0, fade);
                let twinkle = (sim.0.frame / 8 % CRESCENT_FRAMES as u32) as usize;
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("crescent_{twinkle}"))) {
                    sprite.image = image.clone();
                }
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    for (MoonStarOf(i), mut sprite, mut tf, mut visibility) in &mut stars {
        match sim.0.fighters[*i].moon {
            Some((x, y, age)) => {
                *visibility = Visibility::Inherited;
                let bob = (time.elapsed_secs() * 3.0).sin() * 2.0;
                tf.translation.x = (x / PX) as f32 + 22.0;
                tf.translation.y = GROUND_Y + (y / PX) as f32 + 8.0 + bob;
                tf.scale = Vec3::splat(((age as f32 - 6.0) / 12.0).clamp(0.1, 1.0));
                let fade = ((Fighter::MOON_LIFE - age.min(Fighter::MOON_LIFE)) as f32 / 25.0).min(1.0);
                sprite.color = Color::srgba(1.0, 1.0, 1.0, fade);
                let twinkle = (sim.0.frame / 6 % STAR_FRAMES as u32) as usize;
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("moon_star_{twinkle}"))) {
                    sprite.image = image.clone();
                }
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// The speech bubble over a lecturing fighter, with the words typed out as the lecture winds up.
pub fn animate_bubbles(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    mut roots: Query<(&BubbleOf, &mut Transform, &mut Visibility)>,
    mut texts: Query<(&BubbleText, &mut Text2d)>,
) {
    for (BubbleOf(i), mut tf, mut visibility) in &mut roots {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
        // a grab says it while he holds the enemy (after the pull); the others while they wind up
        let grabbing = mv.hits > 1;
        let lecturing = f.state == FighterState::Attack
            && matches!(mv.anim, t2f::sim::AnimKind::Lecture | t2f::sim::AnimKind::Grab)
            && f.state_frame + 3 < mv.total_frames()
            && (!grabbing || f.state_frame > mv.startup);
        *visibility = if lecturing { Visibility::Inherited } else { Visibility::Hidden };
        tf.translation.x = (f.x / PX) as f32;
        tf.translation.y = GROUND_Y + (f.y / PX) as f32 + 84.0;
        if lecturing {
            let progress = if grabbing {
                (f.state_frame.saturating_sub(mv.startup) as f32 / (mv.hit_gap as f32 * 0.6).max(1.0)).min(1.0)
            } else {
                (f.state_frame as f32 / (mv.startup as f32 * 0.85).max(1.0)).min(1.0)
            };
            let shown = ((LECTURE_LINE.len() as f32 * progress).ceil() as usize).min(LECTURE_LINE.len());
            for (BubbleText(owner), mut text) in &mut texts {
                if owner == i && text.0.len() != shown {
                    text.0 = LECTURE_LINE[..shown].to_string();
                }
            }
        }
    }
}

/// Spawn the hidden curse drawings for both fighters; whoever is cursed shows them.
pub fn spawn_curses(commands: &mut Commands, asset_server: &AssetServer) {
    for fighter in 0..2usize {
        let root = commands
            .spawn((CurseRoot(fighter), DespawnOnExit(AppState::Fight), Transform::from_xyz(0.0, 0.0, 0.8), Visibility::Hidden))
            .id();
        for (part, file, z) in [
            (CursePart::PetalsLeft, "effects/curse_petals_left.png", 0.0),
            (CursePart::PetalsRight, "effects/curse_petals_right.png", 0.0),
            (CursePart::RingLeft, "effects/curse_ring_left.png", 0.1),
            (CursePart::RingRight, "effects/curse_ring_right.png", 0.1),
        ] {
            let child = commands
                .spawn((part, CurseOf(fighter), Sprite::from_image(asset_server.load(file)), Transform::from_xyz(0.0, 0.0, z)))
                .id();
            commands.entity(root).add_child(child);
        }
    }
}

/// Show the curse on the cursed fighter: each half's glow disappears when that half is hit, and with both
/// gone the ring squeezes shut.
pub fn animate_curses(
    sim: Res<Sim>,
    time: Res<Time>,
    mut roots: Query<(&CurseRoot, &mut Transform, &mut Visibility)>,
    mut parts: Query<(&CursePart, &CurseOf, &mut Sprite, &mut Visibility), Without<CurseRoot>>,
) {
    for (CurseRoot(i), mut tf, mut visibility) in &mut roots {
        let f = &sim.0.fighters[*i];
        match f.curse {
            Some(c) => {
                *visibility = Visibility::Inherited;
                tf.translation.x = (f.x / PX) as f32;
                tf.translation.y = GROUND_Y + (f.y / PX) as f32 + 38.0;
                let grow = (c.age as f32 / 14.0).min(1.0);
                let pulse = 1.0 + 0.03 * (time.elapsed_secs() * 5.0).sin();
                let squeeze = if c.squeeze > 0 { c.squeeze as f32 / c.squeeze_frames.max(1) as f32 } else { 0.0 };
                // the ring compresses towards the middle (never quite to nothing)
                let s = grow * pulse * (1.0 - 0.9 * squeeze * squeeze);
                tf.scale = Vec3::new(s, s, 1.0);
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    for (part, CurseOf(i), mut sprite, mut visibility) in &mut parts {
        let Some(c) = sim.0.fighters[*i].curse else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let shown = match part {
            CursePart::RingLeft | CursePart::RingRight => true,
            CursePart::PetalsLeft => c.left,
            CursePart::PetalsRight => c.right,
        };
        *visibility = if shown { Visibility::Inherited } else { Visibility::Hidden };
        let glow = 0.72 + 0.28 * (time.elapsed_secs() * 6.0 + c.age as f32 * 0.02).sin();
        sprite.color = match part {
            CursePart::PetalsLeft | CursePart::PetalsRight => Color::srgba(1.0, 1.0, 1.0, glow),
            _ => {
                // the ring goes white-hot as it squeezes shut
                let squeeze = if c.squeeze > 0 { c.squeeze as f32 / c.squeeze_frames.max(1) as f32 } else { 0.0 };
                Color::srgb(1.0 + squeeze, 1.0 + squeeze * 0.6, 1.0 + squeeze * 0.2)
            }
        };
    }
}

/// One of the three squares under a fighter's feet showing their light combo.
#[derive(Component)]
pub struct ComboSquare(pub usize, pub usize);
/// The broken-shield icon over a fighter whose block is disabled.
#[derive(Component)]
pub struct BlockIcon(pub usize);

pub fn spawn_combo_ui(commands: &mut Commands, asset_server: &AssetServer) {
    for fighter in 0..2usize {
        for i in 0..3usize {
            commands.spawn((
                ComboSquare(fighter, i),
                DespawnOnExit(AppState::Fight),
                Sprite::from_color(Color::srgba(0.15, 0.15, 0.2, 0.85), Vec2::splat(6.0)),
                Transform::from_xyz(0.0, GROUND_Y - 9.0, 4.0),
            ));
        }
        commands.spawn((
            BlockIcon(fighter),
            DespawnOnExit(AppState::Fight),
            Sprite::from_image(asset_server.load("effects/block_broken.png")),
            Transform::from_xyz(0.0, 0.0, 6.0),
            Visibility::Hidden,
        ));
    }
}

/// The combo squares fill as lights connect (the third lights up when the finisher lands and flashes while
/// it's ready to be thrown); the broken shield shows while block is disabled.
pub fn animate_combo_ui(
    sim: Res<Sim>,
    time: Res<Time>,
    mut squares: Query<(&ComboSquare, &mut Sprite, &mut Transform), Without<BlockIcon>>,
    mut icons: Query<(&BlockIcon, &mut Sprite, &mut Transform, &mut Visibility), Without<ComboSquare>>,
) {
    for (ComboSquare(i, k), mut sprite, mut tf) in &mut squares {
        let f = &sim.0.fighters[*i];
        tf.translation.x = (f.x / PX) as f32 + (*k as f32 - 1.0) * 9.0;
        let lit = (*k as u8) < f.combo;
        sprite.color = if lit {
            if *k == 2 { Color::srgb(1.0, 0.4, 0.15) } else { Color::srgb(1.0, 0.85, 0.25) }
        } else if *k == 2 && f.combo == 2 {
            // the finisher is next: the last square flashes
            let pulse = 0.5 + 0.5 * (time.elapsed_secs() * 14.0).sin();
            Color::srgb(0.25 + 0.75 * pulse, 0.12 + 0.2 * pulse, 0.1)
        } else {
            Color::srgba(0.15, 0.15, 0.2, 0.85)
        };
    }
    for (BlockIcon(i), mut sprite, mut tf, mut visibility) in &mut icons {
        let f = &sim.0.fighters[*i];
        let disabled = f.block_disabled > 0;
        // blinks near the end so you can see the guard is coming back
        let blinking_off = f.block_disabled < 40 && (f.block_disabled / 5) % 2 == 0;
        *visibility = if disabled && !blinking_off { Visibility::Inherited } else { Visibility::Hidden };
        tf.translation.x = (f.x / PX) as f32;
        tf.translation.y = GROUND_Y + (f.y / PX) as f32 + 74.0 + (time.elapsed_secs() * 4.0).sin() * 1.5;
        sprite.color = Color::WHITE;
    }
}

/// One of a fighter's summoned goblins (up to `MINION_SLOTS` each).
#[derive(Component)]
pub struct MinionSprite(pub usize, pub usize);
pub const MINION_SLOTS: usize = 4;

pub fn spawn_minion_sprites(commands: &mut Commands) {
    for owner in 0..2usize {
        for slot in 0..MINION_SLOTS {
            commands.spawn((
                MinionSprite(owner, slot),
                DespawnOnExit(AppState::Fight),
                Sprite::default(),
                Transform::from_xyz(0.0, 0.0, 1.5).with_scale(Vec3::splat(1.5)),
                Visibility::Hidden,
            ));
        }
    }
}

/// Draw each goblin: bobbing beside its summoner, running at the opponent, snapping its jaws, running home.
pub fn animate_minions(
    sim: Res<Sim>,
    handles: Res<FrameHandles>,
    mut sprites: Query<(&MinionSprite, &mut Sprite, &mut Transform, &mut Visibility)>,
) {
    for (MinionSprite(owner, slot), mut sprite, mut tf, mut visibility) in &mut sprites {
        let Some(m) = sim.0.minions.iter().find(|m| m.owner == *owner && m.slot as usize == *slot) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = Visibility::Inherited;
        let frame = match m.mode {
            _ => {
                let foe_x = sim.0.fighters[1 - *owner].x;
                if m.bit && m.timer < 8 {
                    4
                } else if (foe_x - m.x).abs() < 30 * PX {
                    // on top of the enemy and snapping at them
                    ((sim.0.frame / 8) % 2) as usize * 4
                } else {
                    2 + ((sim.0.frame / 5) % 2) as usize
                }
            }
        };
        if let Some(image) = handles.0.get(*owner).and_then(|h| h.get(&format!("goblin_{frame}"))) {
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        }
        sprite.flip_x = m.facing < 0;
        tf.translation.x = (m.x / PX) as f32;
        tf.translation.y = GROUND_Y + 12.0;
        // fades out in its last half second
        let left = 480u32.saturating_sub(m.age);
        let alpha = if left < 30 { left as f32 / 30.0 } else { 1.0 };
        sprite.color = if m.hurt > 0 { Color::srgba(1.0, 0.35, 0.35, alpha) } else { Color::srgba(1.0, 1.0, 1.0, alpha) };
    }
}
