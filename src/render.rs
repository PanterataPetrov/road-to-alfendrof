//! Draws the fighters from whole-body frames (see `t2f::anim`), plus the summon crescent and the star.

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use t2f::anim::{frame_for, Frame, CRESCENT_FRAMES, FRAME_W, FULL_H, STAR_FRAMES};
use t2f::sim::{Fighter, FighterState, GameState, MarkKind, Phase, ProjectileData, PX};

use crate::{AppState, Roster, Selection, Sim, GROUND_Y};

/// Assign a component field only when the value is different. Every write through `DerefMut` marks the component
/// changed, which makes Bevy redo its transform, visibility and render work for it, so values that are set every frame
/// (positions, colours, images) go through this.
macro_rules! set_ne {
    ($place:expr, $value:expr) => {{
        let value = $value;
        if $place != value {
            $place = value;
        }
    }};
}


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
/// Which drawing a moon uses (`<name>_0`, `<name>_1`): the crescent, or a portal.
#[derive(Component)]
pub struct MoonImage(pub String);
/// The black border (true) and the white fill (false) of a speech bubble, resized to fit its words.
#[derive(Component)]
pub struct BubbleBack(pub usize, pub bool);
/// Speech bubble over a lecturing or writing fighter (world space, so the text isn't mirrored).
#[derive(Component)]
pub struct BubbleOf(pub usize);
/// The two layers of a speech bubble's tail (black border, white fill), which sits on its bottom edge.
#[derive(Component)]
pub struct BubbleTail(pub usize, pub bool);
/// One of the three red worry lines round the head of a futile fighter.
#[derive(Component)]
pub struct WorryLine(pub usize, pub usize);
/// Jesus in the sky, and the two layers (glow, core) of his beam.
#[derive(Component)]
pub struct JesusOf(pub usize);
#[derive(Component)]
pub struct BeamPart(pub usize, pub bool);
/// The blue bubble of plot armour round a fighter, and its label.
#[derive(Component)]
pub struct ShieldOf(pub usize);
#[derive(Component)]
pub struct ShieldLabel(pub usize);
/// The pool and the whale's head of a fighter whose beam is a whale.
#[derive(Component)]
pub struct PoolOf(pub usize);
#[derive(Component)]
pub struct WhaleOf(pub usize);
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
fn all_stems(frames: &[Frame], effects: &[(String, usize)], mark: bool, whip: bool, raven: bool) -> Vec<String> {
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
    for (base, drawings) in effects {
        let count = if *drawings > 0 { *drawings } else if base == "goblin" { 8 } else if base == "agent" { 9 } else if base == "glitch" { 4 } else if base == "whale" { 6 } else { CRESCENT_FRAMES.max(STAR_FRAMES) };
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
    effects: &[(String, usize)],
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
            // the moon (or portal) and star hang in the sky at a fixed spot, so they're not children of the body
            let base = p.summon_name().to_string();
            commands.spawn((
                MoonOf(fighter),
                MoonImage(base.clone()),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get(&format!("{base}_0")).expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, -7.0),
                Visibility::Hidden,
            ));
            if !p.over_target {
                commands.spawn((
                    MoonStarOf(fighter),
                    DespawnOnExit(AppState::Fight),
                    Sprite::from_image(get("moon_star_0").expect("loaded with the rest")),
                    Transform::from_xyz(0.0, 0.0, -6.9),
                    Visibility::Hidden,
                ));
            }
        }
        if frames.iter().any(|f| matches!(f, Frame::Lecture(_) | Frame::Grab(_) | Frame::Write(_))) {
            let width = LECTURE_LINE.len() as f32 * 5.0 + 10.0;
            commands
                .spawn((BubbleOf(fighter), DespawnOnExit(AppState::Fight), Transform::from_xyz(0.0, 0.0, 7.0), Visibility::Hidden))
                .with_children(|b| {
                    b.spawn((BubbleBack(fighter, true), Sprite::from_color(Color::BLACK, Vec2::new(width + 2.0, 18.0)), Transform::from_xyz(0.0, 0.0, 0.0)));
                    b.spawn((BubbleBack(fighter, false), Sprite::from_color(Color::WHITE, Vec2::new(width, 16.0)), Transform::from_xyz(0.0, 0.0, 0.1)));
                    // the tail pointing at the speaker
                    b.spawn((BubbleTail(fighter, true), Sprite::from_color(Color::BLACK, Vec2::new(6.0, 6.0)), Transform::from_xyz(-6.0, -9.0, 0.0).with_rotation(Quat::from_rotation_z(0.785))));
                    b.spawn((BubbleTail(fighter, false), Sprite::from_color(Color::WHITE, Vec2::new(4.0, 4.0)), Transform::from_xyz(-6.0, -8.0, 0.2).with_rotation(Quat::from_rotation_z(0.785))));
                    b.spawn((
                        BubbleText(fighter),
                        Text2d::new(""),
                        bevy::text::TextLayout::new(bevy::text::Justify::Center, bevy::text::LineBreak::NoWrap),
                        crate::pixel_font(8.0),
                        TextColor(Color::BLACK),
                        Transform::from_xyz(0.0, 0.0, 0.3),
                    ));
                });
        }
        // red worry lines round the head, shown while futile
        for index in 0..3 {
            let line = commands
                .spawn((WorryLine(fighter, index), Sprite::from_color(Color::srgb(0.92, 0.1, 0.14), Vec2::new(1.0, 5.0)), Transform::from_xyz(0.0, 0.0, 0.4), Visibility::Hidden))
                .id();
            commands.entity(root).add_child(line);
        }
        // the bubble of plot armour, and the words on its top left
        if effects.iter().any(|e| e.0 == "shield") {
            commands.spawn((
                ShieldOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("shield_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, 2.0),
                Visibility::Hidden,
            ));
            commands.spawn((
                ShieldLabel(fighter),
                DespawnOnExit(AppState::Fight),
                Text2d::new("Plot Armour"),
                crate::pixel_font(8.0),
                TextColor(Color::srgb(0.7, 0.9, 1.0)),
                bevy::sprite::Anchor::BOTTOM_LEFT,
                Transform::from_xyz(0.0, 0.0, 2.2),
                Visibility::Hidden,
            ));
        }
        // the pool and the whale that comes up out of it
        if effects.iter().any(|e| e.0 == "whale") {
            commands.spawn((
                PoolOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("pool_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, 1.0),
                Visibility::Hidden,
            ));
            commands.spawn((
                WhaleOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("whale_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, 1.2),
                Visibility::Hidden,
            ));
        }
        // Jesus and his beam, in the sky
        if effects.iter().any(|e| e.0 == "jesus") {
            commands.spawn((
                JesusOf(fighter),
                DespawnOnExit(AppState::Fight),
                Sprite::from_image(get("jesus_0").expect("loaded with the rest")),
                Transform::from_xyz(0.0, 0.0, -6.5),
                Visibility::Hidden,
            ));
            for core in [false, true] {
                commands.spawn((
                    BeamPart(fighter, core),
                    DespawnOnExit(AppState::Fight),
                    Sprite::from_color(if core { Color::srgb(1.0, 1.0, 0.95) } else { Color::srgba(1.0, 0.88, 0.45, 0.55) }, Vec2::new(10.0, 10.0)),
                    Transform::from_xyz(0.0, 0.0, if core { 0.45 } else { 0.4 }),
                    Visibility::Hidden,
                ));
            }
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
                Sprite::from_image(get(&format!("{}_0", effects.first().map(|e| e.0.as_str()).unwrap_or("star"))).expect("loaded with the rest")),
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
        Phase::Briefing(_) => false,
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

/// A futile fighter is drawn washed over with blue.
fn futile_tint(c: Color) -> Color {
    let s = c.to_srgba();
    Color::srgba(s.red * 0.6, s.green * 0.74, s.blue, s.alpha)
}

/// Place each fighter, face the opponent, and knock them over when KO'd.
pub fn pose_bodies(sim: Res<Sim>, mut bodies: Query<(&Body, &mut Transform)>) {
    for (Body(i), mut tf) in &mut bodies {
        let f = &sim.0.fighters[*i];
        set_ne!(tf.translation.x, (f.x / PX) as f32);
        set_ne!(tf.translation.y, GROUND_Y + (f.y / PX) as f32);
        tf.scale.x = f.heading as f32;
    }
}

/// Pick each fighter's drawing.
pub fn animate_body(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    mut sprites: Query<(&SpriteOf, &mut Sprite)>,
) {
    for (SpriteOf(i), mut sprite) in &mut sprites {
        let f = &sim.0.fighters[*i];
        let frame = frame_for(f, &roster.0[sel.chars[*i]], speaking(&sim.0, *i), sim.0.frame);
        // while the raven is out, use the body drawing without it on the shoulder
        let mut stem = frame.file_stem();
        if sim.0.ravens[*i].active {
            stem.push_str("_nr");
        }
        if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&stem)) {
            if sprite.image != *image {
                if sprite.image != *image {
                    sprite.image = image.clone();
                }
            }
        }
        set_ne!(sprite.color, tint(f.state));
        // a fighter who can't attack (the staff's shockwave) is tinted violet
        if f.silenced > 0 && f.state != FighterState::Hitstun {
            set_ne!(sprite.color, Color::srgb(0.78, 0.68, 1.0));
        }
        if f.futile > 0 {
            set_ne!(sprite.color, futile_tint(sprite.color));
        }
        // bleeding: washed over with red
        if f.bleed_ticks > 0 {
            let c = sprite.color.to_srgba();
            set_ne!(sprite.color, Color::srgba(c.red, c.green * 0.65, c.blue * 0.65, c.alpha));
        }
        // inside a portal there is nothing to see
        if f.hidden {
            set_ne!(sprite.color, Color::srgba(1.0, 1.0, 1.0, 0.0));
        }
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
                set_ne!(*visibility, Visibility::Inherited);
                let twinkle = (sim.0.frame / 6 % CRESCENT_FRAMES as u32) as usize;
                let base = roster.0[sel.chars[*i]].moves.iter().find_map(|m| m.projectile.as_ref().filter(|_| m.anim == t2f::sim::AnimKind::Summon)).map_or("crescent", |p| p.summon_name());
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("{base}_{twinkle}"))) {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
                sprite.color = Color::srgba(1.0, 1.0, 1.0, if n >= 5 { 0.5 } else { 1.0 });
                // swells while charging, flashes when fully charged
                let data = &roster.0[sel.chars[*i]];
                let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
                let grow = if mv.charge_frames > 0 { f.charge as f32 / mv.charge_frames as f32 } else { 0.0 };
                tf.scale = Vec3::splat(1.0 + grow * 0.5);
                if mv.charge_frames > 0 && f.charge >= mv.charge_frames && sim.0.frame / 3 % 2 == 0 {
                    set_ne!(sprite.color, Color::srgb(1.6, 1.6, 1.3));
                }
            }
            _ => set_ne!(*visibility, Visibility::Hidden),
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
                set_ne!(*visibility, Visibility::Inherited);
                set_ne!(tf.translation.x, (p.x / PX) as f32);
                set_ne!(tf.translation.y, GROUND_Y + (p.y / PX) as f32);
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
                let flicker = (p.age / 3 % data.map_or(STAR_FRAMES, |d| d.drawing_count()) as u32) as usize;
                let base = data.map_or("star", |d| d.image_name());
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("{base}_{flicker}"))) {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
            }
            None => set_ne!(*visibility, Visibility::Hidden),
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
                if sprite.image != *image {
                    sprite.image = image.clone();
                }
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
            Some(MarkKind::GlowingHat) => {
                // gone while the throw recharges, glowing gold once it can be thrown again
                let throw = data.moves.iter().position(|m| m.button == t2f::sim::Button::Special);
                if throw.is_some_and(|i| f.on_cooldown(i, data)) {
                    Color::srgba(1.0, 1.0, 1.0, 0.0)
                } else if ready {
                    let pulse = 0.5 + 0.5 * (time.elapsed_secs() * 7.0).sin();
                    Color::srgb(1.3 + 0.4 * pulse, 1.15 + 0.35 * pulse, 0.55 + 0.25 * pulse)
                } else {
                    Color::WHITE
                }
            }
            None => Color::srgba(1.0, 1.0, 1.0, 0.0),
        };
        if f.hidden {
            set_ne!(sprite.color, Color::srgba(1.0, 1.0, 1.0, 0.0));
        }
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
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
                set_ne!(tf.translation.x, (r.x / PX) as f32);
                set_ne!(tf.translation.y, GROUND_Y + (r.y / PX) as f32);
                tf.scale.x = r.facing as f32;
                set_ne!(*visibility, Visibility::Inherited);
            }
            None => set_ne!(*visibility, Visibility::Hidden),
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
                set_ne!(tf.translation.x, (rose.x / PX) as f32);
                set_ne!(tf.translation.y, GROUND_Y + (rose.y / PX) as f32);
                set_ne!(*visibility, Visibility::Inherited);
            }
            None => set_ne!(*visibility, Visibility::Hidden),
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
            set_ne!(*visibility, Visibility::Hidden);
        } else {
            match layer.and_then(|name| handles.0.get(*i).and_then(|m| m.get(&name))) {
                Some(image) => {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                    set_ne!(sprite.color, tint(f.state));
                    set_ne!(*visibility, Visibility::Inherited);
                }
                None => set_ne!(*visibility, Visibility::Hidden),
            }
        }
        for (WhipPixel(owner, index), mut px_sprite, mut tf, mut vis) in &mut pixels {
            if owner != i {
                continue;
            }
            match path.get(*index) {
                Some(&(x, y)) => {
                    set_ne!(px_sprite.color, lash_colour(*index, path.len(), tint(f.state)));
                    set_ne!(tf.translation.x, shift - FRAME_W as f32 / 2.0 + x as f32 + 0.5);
                    set_ne!(tf.translation.y, FULL_H as f32 - y as f32 - 0.5);
                    set_ne!(*vis, Visibility::Inherited);
                }
                None => set_ne!(*vis, Visibility::Hidden),
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
    mut moons: Query<(&MoonOf, &MoonImage, &mut Sprite, &mut Transform, &mut Visibility), Without<MoonStarOf>>,
    mut stars: Query<(&MoonStarOf, &mut Sprite, &mut Transform, &mut Visibility), Without<MoonOf>>,
) {
    for (MoonOf(i), MoonImage(base), mut sprite, mut tf, mut visibility) in &mut moons {
        match sim.0.fighters[*i].moon {
            Some((x, y, age)) => {
                set_ne!(*visibility, Visibility::Inherited);
                set_ne!(tf.translation.x, (x / PX) as f32);
                set_ne!(tf.translation.y, GROUND_Y + (y / PX) as f32);
                let grow = (age as f32 / 12.0).min(1.0).max(0.1);
                // a portal over the opponent is wide
                tf.scale = if base == "portal" { Vec3::new(grow * 5.6, grow * 1.1, 1.0) } else { Vec3::splat(grow) };
                let fade = ((Fighter::MOON_LIFE - age.min(Fighter::MOON_LIFE)) as f32 / 25.0).min(1.0);
                set_ne!(sprite.color, Color::srgba(1.0, 1.0, 1.0, fade));
                let twinkle = (sim.0.frame / 8 % CRESCENT_FRAMES as u32) as usize;
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("{base}_{twinkle}"))) {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
            }
            None => set_ne!(*visibility, Visibility::Hidden),
        }
    }
    for (MoonStarOf(i), mut sprite, mut tf, mut visibility) in &mut stars {
        match sim.0.fighters[*i].moon {
            Some((x, y, age)) => {
                set_ne!(*visibility, Visibility::Inherited);
                let bob = (time.elapsed_secs() * 3.0).sin() * 2.0;
                set_ne!(tf.translation.x, (x / PX) as f32 + 22.0);
                set_ne!(tf.translation.y, GROUND_Y + (y / PX) as f32 + 8.0 + bob);
                tf.scale = Vec3::splat(((age as f32 - 6.0) / 12.0).clamp(0.1, 1.0));
                let fade = ((Fighter::MOON_LIFE - age.min(Fighter::MOON_LIFE)) as f32 / 25.0).min(1.0);
                set_ne!(sprite.color, Color::srgba(1.0, 1.0, 1.0, fade));
                let twinkle = (sim.0.frame / 6 % STAR_FRAMES as u32) as usize;
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("moon_star_{twinkle}"))) {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
            }
            None => set_ne!(*visibility, Visibility::Hidden),
        }
    }
}

/// The speech bubble over a lecturing or writing fighter, with the words typed out as the move winds up.
pub fn animate_bubbles(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    mut roots: Query<(&BubbleOf, &mut Transform, &mut Visibility)>,
    mut texts: Query<(&BubbleText, &mut Text2d)>,
    mut backs: Query<(&BubbleBack, &mut Sprite), Without<BubbleTail>>,
    mut tails: Query<(&BubbleTail, &mut Transform), (Without<BubbleOf>, Without<BubbleText>)>,
    mut spoken: Local<[Option<usize>; 2]>,
) {
    for (BubbleOf(i), mut tf, mut visibility) in &mut roots {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let attacking = f.state == FighterState::Attack;
        // a line that was written stays up, whole, while what it made (the orb, the knives) is still out
        let lingering = spoken[*i].filter(|&m| !attacking && sim.0.projectiles.iter().any(|p| p.owner == *i && p.move_index == m));
        let index = lingering.unwrap_or(f.attack.min(data.moves.len() - 1));
        let mv = &data.moves[index];
        // a writing move says its line (with the enemy's name in it) while it plays
        // (a move can have a second line, said once its last hit has landed)
        let last_hit = mv.startup + (mv.hits.max(1) - 1) * mv.hit_gap;
        let said_after = attacking && mv.say_after.is_some() && f.state_frame > last_hit;
        let line = if said_after { mv.say_after.as_ref() } else { mv.say.as_ref() }.map(|say| say.replace("{enemy}", &roster.0[sel.chars[1 - *i]].name));
        spoken[*i] = if attacking && line.is_some() { Some(index) } else { lingering };
        // a grab says it while he holds the enemy (after the pull); the others while they wind up
        let grabbing = mv.hits > 1;
        let lecturing = lingering.is_some()
            || (attacking
                && (line.is_some() || matches!(mv.anim, t2f::sim::AnimKind::Lecture | t2f::sim::AnimKind::Grab))
                && (line.is_some() || f.state_frame + 3 < mv.total_frames())
                && (!grabbing || f.state_frame > mv.startup));
        set_ne!(*visibility, if lecturing { Visibility::Inherited } else { Visibility::Hidden });
        set_ne!(tf.translation.x, (f.x / PX) as f32);
        set_ne!(tf.translation.y, GROUND_Y + (f.y / PX) as f32 + 84.0);
        if lecturing {
            // a long line goes onto several rows
            let full = wrap_words(&line.clone().unwrap_or_else(|| LECTURE_LINE.to_string()), BUBBLE_LINE);
            let progress = if lingering.is_some() {
                1.0
            } else if line.is_some() {
                // a written line types at its own pace (30 letters a second), whatever the move is doing
                let typed_for = if said_after { f.state_frame - last_hit } else { f.state_frame };
                (typed_for as f32 * if said_after { 1.5 } else { 0.5 } / full.chars().count().max(1) as f32).min(1.0)
            } else if grabbing {
                (f.state_frame.saturating_sub(mv.startup) as f32 / (mv.hit_gap as f32 * 0.6).max(1.0)).min(1.0)
            } else {
                (f.state_frame as f32 / (mv.startup as f32 * 0.85).max(1.0)).min(1.0)
            };
            let total = full.chars().count();
            let shown = ((total as f32 * progress).ceil() as usize).min(total);
            let typed: String = full.chars().take(shown).collect();
            for (BubbleText(owner), mut text) in &mut texts {
                if owner == i && text.0 != typed {
                    text.0 = typed.clone();
                }
            }
            // the bubble is as big as the whole line, so it doesn't grow while it is typed; its bottom edge stays put
            let rows = full.lines().count().max(1);
            let width = full.lines().map(|l| l.chars().count()).max().unwrap_or(1) as f32 * 5.0 + 10.0;
            let height = if rows > 1 { rows as f32 * 10.0 + 6.0 } else { 16.0 };
            tf.translation.y += (height - 16.0) / 2.0;
            for (BubbleBack(owner, border), mut sprite) in &mut backs {
                if owner == i {
                    sprite.custom_size = Some(Vec2::new(if *border { width + 2.0 } else { width }, if *border { height + 2.0 } else { height }));
                }
            }
            for (BubbleTail(owner, border), mut tail) in &mut tails {
                if owner == i {
                    set_ne!(tail.translation.y, -(height / 2.0) - if *border { 1.0 } else { 0.0 });
                }
            }
        }
    }
}

/// The skill bar in the bottom right: Light, Heavy and Special as icons with their key (Options > Controls)
/// in the corner. An attack that can't be used right now (cooling down, the opponent's shockwave has
/// silenced you, the raven is away, or the special meter isn't full) is greyed out with the time left;
/// the special fills up from the bottom as its meter charges, and glows when it's ready.
#[derive(Component)]
pub struct SkillIcon(pub usize);
#[derive(Component)]
pub struct SkillShade(pub usize);
#[derive(Component)]
pub struct SkillTimer(pub usize);
#[derive(Component)]
pub struct SkillGlow(pub usize);

const SKILL_SIZE: f32 = 24.0;
/// Centres of the three icons (the bar is part of the HUD, which is laid out in view units).
const SKILL_X: [f32; 3] = [118.0, 146.0, 174.0];
const SKILL_Y: f32 = -91.0;

pub fn spawn_skill_bar(
    mut commands: Commands,
    hud: Single<Entity, With<crate::HudRoot>>,
    asset_server: Res<AssetServer>,
    settings: Res<crate::settings::Settings>,
) {
    use t2f::sim::Input;
    let keys = [Input::LIGHT, Input::HEAVY, Input::SPECIAL].map(|flag| crate::settings::key_name(settings.key(flag)));
    commands.entity(*hud).with_children(|h| {
        for (k, key) in keys.into_iter().enumerate() {
            // the badge grows with the key's name ("Space"), from the icon's left edge
            let badge_w = key.chars().count() as f32 * 4.0 + 4.0;
            let badge_x = SKILL_X[k] - SKILL_SIZE / 2.0 + 1.0 + badge_w / 2.0;
            let (x, y) = (SKILL_X[k], SKILL_Y);
            h.spawn((SkillGlow(k), Sprite::from_color(Color::srgba(1.0, 0.85, 0.3, 0.0), Vec2::splat(SKILL_SIZE + 4.0)), Transform::from_xyz(x, y, 9.8)));
            h.spawn((SkillIcon(k), Sprite::from_image(asset_server.load("icons/ayz/light.png")), Transform::from_xyz(x, y, 10.0)));
            h.spawn((SkillShade(k), Sprite::from_color(Color::srgba(0.04, 0.04, 0.07, 0.74), Vec2::splat(SKILL_SIZE)), Transform::from_xyz(x, y, 10.1), Visibility::Hidden));
            h.spawn((
                SkillTimer(k),
                Text2d::new(""),
                crate::pixel_font(9.0),
                TextColor(Color::WHITE),
                Transform::from_xyz(x, y - 1.0, 10.3),
            ));
            // the key in the top-left corner, on a little dark badge
            h.spawn((Sprite::from_color(Color::srgba(0.04, 0.03, 0.08, 0.9), Vec2::new(badge_w, 9.0)), Transform::from_xyz(badge_x, y + SKILL_SIZE / 2.0 - 5.5, 10.4)));
            h.spawn((
                Text2d::new(key),
                crate::pixel_font(7.0),
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
                Transform::from_xyz(badge_x, y + SKILL_SIZE / 2.0 - 5.5, 10.5),
            ));
        }
    });
}

/// Keep the skill bar in step with the fighter you control: their icons, what's available and the timers.
pub fn animate_skill_bar(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    time: Res<Time>,
    net: Option<Res<crate::online::NetSession>>,
    asset_server: Res<AssetServer>,
    mut icons: Query<(&SkillIcon, &mut Sprite), (Without<SkillShade>, Without<SkillGlow>)>,
    mut shades: Query<(&SkillShade, &mut Sprite, &mut Transform, &mut Visibility), (Without<SkillIcon>, Without<SkillGlow>)>,
    mut glows: Query<(&SkillGlow, &mut Sprite), (Without<SkillIcon>, Without<SkillShade>)>,
    mut timers: Query<(&SkillTimer, &mut Text2d)>,
    fresh: Query<(), Added<SkillIcon>>,
    // which icon each slot shows now, so the image is only asked for when it changes
    mut shown: Local<(String, [&'static str; 3])>,
) {
    use t2f::sim::Button;
    if !fresh.is_empty() {
        *shown = Default::default();
    }
    let side = net.as_ref().map_or(0, |n| n.local);
    let f = &sim.0.fighters[side.min(1)];
    let data = &roster.0[sel.chars[side.min(1)]];
    let folder = data.sprites.clone().unwrap_or_else(|| data.name.to_lowercase());
    let buttons = [Button::Light, Button::Heavy, Button::Special];
    let names = ["light", "heavy", "special"];
    // what each slot looks like right now: (available, frames of timer, how much of the icon is greyed from the top, 0..1)
    let state = |k: usize| -> (bool, u32, f32) {
        let index = data.find_move(buttons[k], false, false);
        let cooling = index.is_some_and(|i| f.on_cooldown(i, data));
        let raven_away = index.is_some_and(|i| data.moves[i].raven_move.is_some() && f.raven_away);
        let waiting_for_meter = buttons[k] == Button::Special && f.meter < Fighter::METER_MAX;
        let silenced = f.silenced > 0;
        // the heavy turns into a bat (and back) whatever its cooldown; a bat can't use the others
        let bat_slot = buttons[k] == Button::Heavy && (f.bat_ready > 0 || f.bat_form > 0);
        let cooling = cooling && !bat_slot;
        let bat_blocked = f.bat_form > 0 && buttons[k] != Button::Heavy;
        let resting = buttons[k] == Button::Light && f.light_lock > 0;
        let frames = if cooling { index.map_or(0, |i| f.cooldown_left(i)) } else if silenced { f.silenced } else if resting { f.light_lock } else { 0 };
        let shade = if waiting_for_meter && !cooling && !silenced && !bat_blocked { 1.0 - f.meter as f32 / Fighter::METER_MAX as f32 } else { 1.0 };
        (!(cooling || silenced || raven_away || waiting_for_meter || bat_blocked || resting), frames, shade)
    };
    for (SkillIcon(k), mut sprite) in &mut icons {
        let name = match names[*k] {
            "heavy" if f.bat_form > 0 => "poof",
            "heavy" if f.bat_ready > 0 => "bat",
            name => name,
        };
        if shown.0 != folder || shown.1[*k] != name {
            sprite.image = asset_server.load(format!("icons/{folder}/{name}.png"));
            shown.1[*k] = name;
        }
    }
    if shown.0 != folder {
        shown.0 = folder.clone();
    }
    for (SkillShade(k), mut sprite, mut tf, mut visibility) in &mut shades {
        let (available, _, shade) = state(*k);
        set_ne!(*visibility, if available { Visibility::Hidden } else { Visibility::Inherited });
        let h = SKILL_SIZE * shade;
        sprite.custom_size = Some(Vec2::new(SKILL_SIZE, h));
        // the grey covers the top of the icon down to `h`
        set_ne!(tf.translation.y, SKILL_Y + SKILL_SIZE / 2.0 - h / 2.0);
    }
    for (SkillGlow(k), mut sprite) in &mut glows {
        let (available, _, _) = state(*k);
        let pulse = 0.55 + 0.45 * (time.elapsed_secs() * 7.0).sin();
        // the special shouts when it's ready; the others have a quiet edge
        let alpha = if available && buttons[*k] == Button::Special { pulse } else { 0.0 };
        set_ne!(sprite.color, Color::srgba(1.0, 0.85, 0.3, alpha));
    }
    for (SkillTimer(k), mut text) in &mut timers {
        let (_, frames, _) = state(*k);
        let shown = if frames > 0 { format!("{:.1}", frames as f32 / 60.0) } else { String::new() };
        if text.0 != shown {
            text.0 = shown;
        }
    }
}

/// One of the zones on the map (glitched areas and spills), shown by the first four.
#[derive(Component)]
pub struct ZoneSprite(pub usize);

pub fn spawn_zone_sprites(mut commands: Commands) {
    for i in 0..4 {
        commands.spawn((ZoneSprite(i), DespawnOnExit(AppState::Fight), Sprite::default(), Transform::from_xyz(0.0, 0.0, 0.3), Visibility::Hidden));
    }
}

/// Draw each zone: its drawings cycle, and it fades in as it appears and out as it ends.
pub fn animate_zones(sim: Res<Sim>, handles: Res<FrameHandles>, mut sprites: Query<(&ZoneSprite, &mut Sprite, &mut Transform, &mut Visibility)>) {
    use t2f::sim::ZoneKind;
    for (ZoneSprite(i), mut sprite, mut tf, mut visibility) in &mut sprites {
        let Some(z) = sim.0.zones.get(*i) else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        set_ne!(*visibility, Visibility::Inherited);
        let (name, count, y, depth) = match z.kind {
            ZoneKind::Glitch => ("glitch", 4, 39.0, 0.3),
            ZoneKind::Spill => ("spill", 2, 3.0, 0.2),
        };
        let frame = (sim.0.frame / if z.kind == ZoneKind::Glitch { 3 } else { 14 }) as usize % count;
        if let Some(image) = handles.0.get(z.owner).and_then(|m| m.get(&format!("{name}_{frame}"))) {
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        }
        set_ne!(tf.translation.x, (z.x / PX) as f32);
        set_ne!(tf.translation.y, GROUND_Y + y);
        set_ne!(tf.translation.z, depth);
        let alpha = (z.age as f32 / 12.0).min(1.0) * (z.life.saturating_sub(z.age) as f32 / 20.0).min(1.0);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

/// One cigarette in a fighter's row under the special bar (the first is the lit one in his mouth).
#[derive(Component)]
pub struct CigaretteIcon(pub usize, pub usize);

/// The row of cigarettes under each health bar, for a character who carries them.
pub fn spawn_cigarette_hud(mut commands: Commands, hud: Single<Entity, With<crate::HudRoot>>) {
    commands.entity(*hud).with_children(|h| {
        for side in 0..2usize {
            let dir = if side == 0 { -1.0 } else { 1.0 };
            let left = dir * (crate::BAR_EDGE - crate::BAR_W / 2.0) - crate::BAR_W / 2.0;
            let y = crate::BAR_Y - crate::BAR_H / 2.0 - 12.0;
            for k in 0..7usize {
                let x = left + 5.0 + k as f32 * 9.0;
                h.spawn((CigaretteIcon(side, k), Sprite::from_color(Color::srgb(0.96, 0.95, 0.9), Vec2::new(7.0, 2.0)), Transform::from_xyz(x, y, 10.1), Visibility::Hidden));
                // the filter end, and an ember on the lit one
                h.spawn((CigaretteIcon(side, k + 10), Sprite::from_color(if k == 0 { Color::srgb(1.0, 0.45, 0.1) } else { Color::srgb(0.85, 0.55, 0.25) }, Vec2::new(2.0, 2.0)), Transform::from_xyz(x + 4.5, y, 10.2), Visibility::Hidden));
            }
        }
    });
}

/// Show as many of the row as the fighter has cigarettes left; hidden for a character who has none.
pub fn animate_cigarettes(sim: Res<Sim>, roster: Res<Roster>, sel: Res<Selection>, mut icons: Query<(&CigaretteIcon, &mut Sprite, &mut Visibility)>) {
    for (CigaretteIcon(side, k), mut sprite, mut visibility) in &mut icons {
        let data = &roster.0[sel.chars[*side]];
        let Some(c) = data.cigarettes else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        let index = *k % 10;
        set_ne!(*visibility, if index < c.max as usize { Visibility::Inherited } else { Visibility::Hidden });
        let left = sim.0.fighters[*side].cigs(data) as usize;
        let alpha = if index < left { 1.0 } else { 0.18 };
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

/// One of a fighter's two portals (0: the one behind her that she leaps into, 1: the one behind the enemy).
#[derive(Component)]
pub struct GateOf(pub usize, pub usize);
/// The hand that reaches out of the far portal.
#[derive(Component)]
pub struct HandOf(pub usize);

/// Spawn the hidden portals and hand for a fighter whose moves open portals (drawings `gate_0`, `gate_1` and
/// `hand_0` in their sprite folder).
pub fn spawn_portals(commands: &mut Commands, asset_server: &AssetServer, fighter: usize, folder: &str) {
    for which in 0..2 {
        commands.spawn((
            GateOf(fighter, which),
            DespawnOnExit(AppState::Fight),
            Sprite::from_image(asset_server.load(format!("sprites/{folder}/gate_0.png"))),
            Transform::from_xyz(0.0, 0.0, 0.6),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        HandOf(fighter),
        DespawnOnExit(AppState::Fight),
        Sprite::from_image(asset_server.load(format!("sprites/{folder}/hand_0.png"))),
        Transform::from_xyz(0.0, 0.0, 2.6),
        Visibility::Hidden,
    ));
}

/// Open the portal behind her as she steps back, the far one behind the enemy once she is gone, and reach the
/// hand out of the far one during the grab (holding on if it caught them). Both close once she is out.
pub fn animate_portals(
    mut start: Local<[f32; 2]>,
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    asset_server: Res<AssetServer>,
    mut gates: Query<(&GateOf, &mut Sprite, &mut Transform, &mut Visibility), Without<HandOf>>,
    mut hands: Query<(&HandOf, &mut Sprite, &mut Transform, &mut Visibility), Without<GateOf>>,
) {
    for (GateOf(i, which), mut sprite, mut tf, mut visibility) in &mut gates {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
        let (Some(pd), true) = (mv.portal, f.state == FighterState::Attack) else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        let frame = f.state_frame;
        if frame <= 1 {
            start[*i] = (f.x / PX) as f32;
        }
        let end = mv.total_frames();
        // (near) from the first frame until a little after she is gone; (far) from then until she is out
        let (from, to, x) = if *which == 0 {
            (1, pd.vanish + 10, start[*i] - f.facing as f32 * 14.0)
        } else {
            (pd.aim_frame(mv.startup), if f.hidden { end } else { (mv.startup + mv.active + 10).min(end) }, f.portal_x.map_or(0.0, |p| (p / PX) as f32))
        };
        if frame < from || frame > to || (*which == 1 && f.portal_x.is_none()) {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        }
        set_ne!(*visibility, Visibility::Inherited);
        // it swirls open, then shrinks shut over its last frames
        let age = (frame - from) as f32;
        let left = (to - frame) as f32;
        let size = (age / 6.0).min(1.0).min(left / 6.0).max(0.1);
        set_ne!(tf.translation.x, x);
        set_ne!(tf.translation.y, GROUND_Y + 22.0);
        tf.scale = Vec3::new(size, size, 1.0);
        let folder = data.sprites.clone().unwrap_or_else(|| data.name.to_lowercase());
        sprite.image = asset_server.load(format!("sprites/{folder}/gate_{}.png", (sim.0.frame / 5) % 2));
    }
    for (HandOf(i), mut sprite, mut tf, mut visibility) in &mut hands {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
        let (Some(x), true) = (f.portal_x, f.state == FighterState::Attack && f.hidden && mv.portal.is_some()) else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        let frame = f.state_frame;
        if frame + 8 < mv.startup {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        }
        set_ne!(*visibility, Visibility::Inherited);
        // it creeps out of the portal, then lunges over the grab
        let reach = if frame <= mv.startup { (frame + 8 - mv.startup) as f32 * 0.8 } else { 6.0 + (frame - mv.startup).min(4) as f32 * 4.0 };
        set_ne!(tf.translation.x, (x / PX) as f32 + f.facing as f32 * reach);
        set_ne!(tf.translation.y, GROUND_Y + 30.0);
        tf.scale.x = f.facing as f32;
        set_ne!(sprite.color, Color::WHITE);
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
                set_ne!(*visibility, Visibility::Inherited);
                set_ne!(tf.translation.x, (f.x / PX) as f32);
                set_ne!(tf.translation.y, GROUND_Y + (f.y / PX) as f32 + 38.0);
                let grow = (c.age as f32 / 14.0).min(1.0);
                let pulse = 1.0 + 0.03 * (time.elapsed_secs() * 5.0).sin();
                let squeeze = if c.squeeze > 0 { c.squeeze as f32 / c.squeeze_frames.max(1) as f32 } else { 0.0 };
                // the ring compresses towards the middle (never quite to nothing)
                let s = grow * pulse * (1.0 - 0.9 * squeeze * squeeze);
                tf.scale = Vec3::new(s, s, 1.0);
            }
            None => set_ne!(*visibility, Visibility::Hidden),
        }
    }
    for (part, CurseOf(i), mut sprite, mut visibility) in &mut parts {
        let Some(c) = sim.0.fighters[*i].curse else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        let shown = match part {
            CursePart::RingLeft | CursePart::RingRight => true,
            CursePart::PetalsLeft => c.left,
            CursePart::PetalsRight => c.right,
        };
        set_ne!(*visibility, if shown { Visibility::Inherited } else { Visibility::Hidden });
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
        set_ne!(tf.translation.x, (f.x / PX) as f32 + (*k as f32 - 1.0) * 9.0);
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
        set_ne!(*visibility, if disabled && !blinking_off { Visibility::Inherited } else { Visibility::Hidden });
        set_ne!(tf.translation.x, (f.x / PX) as f32);
        set_ne!(tf.translation.y, GROUND_Y + (f.y / PX) as f32 + 74.0 + (time.elapsed_secs() * 4.0).sin() * 1.5);
        set_ne!(sprite.color, Color::WHITE);
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
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        set_ne!(*visibility, Visibility::Inherited);
        let frame = match m.mode {
            _ if m.agent => {
                // 0, 1 walking; 2 winding up a punch; 3 punching; 4 dashing; 5 rolling
                use t2f::sim::MinionMode;
                match m.mode {
                    MinionMode::Dash => 4,
                    MinionMode::Roll => 5 + ((m.timer / 3) % 4) as usize,
                    MinionMode::Punch if (4..9).contains(&m.timer) || (14..19).contains(&m.timer) => 3,
                    MinionMode::Punch => 2,
                    _ => ((sim.0.frame / 7) % 2) as usize,
                }
            }
            _ => {
                // 0, 1 waiting; 2 to 5 running; 6 jaws wide; 7 snapping shut as it lunges
                let foe_x = sim.0.fighters[1 - *owner].x;
                if m.bit && m.timer < 8 {
                    7
                } else if (foe_x - m.x).abs() < 30 * PX {
                    // on top of the enemy and working its jaws
                    6 + ((sim.0.frame / 6) % 2) as usize
                } else if m.mode == t2f::sim::MinionMode::Wait {
                    ((sim.0.frame / 12) % 2) as usize
                } else {
                    2 + ((sim.0.frame / 4) % 4) as usize
                }
            }
        };
        let name = if m.agent { "agent" } else { "goblin" };
        if let Some(image) = handles.0.get(*owner).and_then(|h| h.get(&format!("{name}_{frame}"))) {
            if sprite.image != *image {
                if sprite.image != *image {
                    sprite.image = image.clone();
                }
            }
        }
        sprite.flip_x = m.facing < 0;
        // a goblin is drawn at 1x (32 x 24, feet on the bottom row); an agent like a fighter (84 x 63, body centre left of
        // the middle, feet on the bottom row)
        let shift = if m.agent { (FRAME_W as f32 / 2.0 - t2f::anim::CENTRE_X as f32) * m.facing as f32 } else { 0.0 };
        set_ne!(tf.translation.x, (m.x / PX) as f32 + shift);
        set_ne!(tf.translation.y, GROUND_Y + if m.agent { FULL_H as f32 / 2.0 } else { 12.0 });
        set_ne!(tf.scale, Vec3::splat(1.0));
        // fades out in its last half second
        let left = 480u32.saturating_sub(m.age);
        let alpha = if left < 30 { left as f32 / 30.0 } else { 1.0 };
        sprite.color = if m.hurt > 0 { Color::srgba(1.0, 0.35, 0.35, alpha) } else { Color::srgba(1.0, 1.0, 1.0, alpha) };
    }
}

/// The most characters on one row of a speech bubble.
const BUBBLE_LINE: usize = 22;

/// Break `text` into rows of at most `width` characters, at spaces.
fn wrap_words(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut row = 0;
    for word in text.split_whitespace() {
        let len = word.chars().count();
        if row > 0 && row + 1 + len > width {
            out.push('\n');
            row = 0;
        } else if row > 0 {
            out.push(' ');
            row += 1;
        }
        out.push_str(word);
        row += len;
    }
    out
}

/// The red worry lines round the head of a futile fighter (they tremble), and Jesus with his beam.
pub fn animate_scribe_effects(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    time: Res<Time>,
    mut worry: Query<(&WorryLine, &mut Transform, &mut Visibility), (Without<JesusOf>, Without<BeamPart>)>,
    mut jesus: Query<(&JesusOf, &mut Sprite, &mut Transform, &mut Visibility), (Without<WorryLine>, Without<BeamPart>)>,
    mut beams: Query<(&BeamPart, &mut Sprite, &mut Transform, &mut Visibility), (Without<WorryLine>, Without<JesusOf>)>,
) {
    for (WorryLine(i, index), mut tf, mut visibility) in &mut worry {
        let f = &sim.0.fighters[*i];
        set_ne!(*visibility, if f.futile > 0 { Visibility::Inherited } else { Visibility::Hidden });
        // three short strokes fanning out above the head, trembling
        let shake = if (sim.0.frame / 4 + *index as u32) % 2 == 0 { 0.0 } else { 1.0 };
        let (x, y, angle) = match index {
            0 => (-10.0, 60.0 + shake, 0.55),
            1 => (0.0, 64.0 - shake, 0.0),
            _ => (10.0, 60.0 + shake, -0.55),
        };
        // the head is a little right of the body centre in the drawings
        tf.translation = Vec3::new(x + 2.0, y, 0.4);
        tf.rotation = Quat::from_rotation_z(angle);
    }
    for (JesusOf(i), mut sprite, mut tf, mut visibility) in &mut jesus {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
        match (f.state == FighterState::Attack, mv.beam.filter(|b| !b.whale), f.beam_lock) {
            (true, Some(b), Some(lock)) => {
                set_ne!(*visibility, Visibility::Inherited);
                let age = f.state_frame.saturating_sub(b.lock_frame);
                let rise = (age as f32 / 14.0).min(1.0);
                let leave = if f.state_frame > mv.startup + mv.active { ((mv.total_frames() - f.state_frame.min(mv.total_frames())) as f32 / 12.0).min(1.0) } else { 1.0 };
                set_ne!(tf.translation.x, (lock / PX) as f32 - f.facing as f32 * b.lead as f32);
                set_ne!(tf.translation.y, GROUND_Y + 170.0 + (1.0 - rise) * 24.0 + (time.elapsed_secs() * 2.0).sin() * 1.5);
                set_ne!(sprite.color, Color::srgba(1.0, 1.0, 1.0, rise * leave));
                let frame = (sim.0.frame / 8 % 2) as usize;
                if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("jesus_{frame}"))) {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
            }
            _ => set_ne!(*visibility, Visibility::Hidden),
        }
    }
    for (BeamPart(i, core), mut sprite, mut tf, mut visibility) in &mut beams {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
        let (Some(b), Some(lock)) = (mv.beam.filter(|b| !b.whale), f.beam_lock) else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        // charging before it fires, then the sweep, then it fades
        let firing = f.state == FighterState::Attack && f.state_frame > mv.startup && f.state_frame <= mv.startup + mv.active + 6;
        if !firing {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        }
        set_ne!(*visibility, Visibility::Inherited);
        let progress = (f.state_frame.saturating_sub(mv.startup) as f32).min(mv.active as f32) / mv.active.max(1) as f32;
        let lock_x = (lock / PX) as f32;
        let (start, end) = (lock_x - f.facing as f32 * b.lead as f32, lock_x + f.facing as f32 * b.sweep as f32);
        let from = Vec2::new(lock_x - f.facing as f32 * b.lead as f32, GROUND_Y + 160.0);
        let to = Vec2::new(start + (end - start) * progress, GROUND_Y + 4.0);
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        let length = (dx * dx + dy * dy).sqrt().max(1.0);
        let fade = if f.state_frame > mv.startup + mv.active { 1.0 - (f.state_frame - mv.startup - mv.active) as f32 / 6.0 } else { 1.0 };
        let pulse = 1.0 + 0.15 * (time.elapsed_secs() * 40.0).sin();
        let thick = if *core { 9.0 } else { 20.0 } * pulse * fade.max(0.1);
        sprite.custom_size = Some(Vec2::new(thick, length));
        let mid = (from + to) / 2.0;
        set_ne!(tf.translation.x, mid.x);
        set_ne!(tf.translation.y, mid.y);
        // a rectangle is tall by default: turn it to lie along the beam, so its top stays at Jesus and only the
        // bottom sweeps
        tf.rotation = Quat::from_rotation_z(dx.atan2(-dy));
        sprite.color = if *core { Color::srgba(1.0, 1.0, 0.95, fade) } else { Color::srgba(1.0, 0.88, 0.45, 0.55 * fade) };
    }
}

/// The Scribe's whale: a pool opens in the ground where the enemy stood, the whale's head rises out of it, opens wide, snaps
/// shut on the spot, and sinks again. Every stage is its own drawing (`whale_0..5`), so nothing here moves it.
pub fn animate_whale(
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    handles: Res<FrameHandles>,
    mut pools: Query<(&PoolOf, &mut Sprite, &mut Transform, &mut Visibility), Without<WhaleOf>>,
    mut whales: Query<(&WhaleOf, &mut Sprite, &mut Transform, &mut Visibility), Without<PoolOf>>,
) {
    // which drawing of the whale at which frame of the move: rising until the bite, the bite, then sinking
    let drawing = |mv: &t2f::sim::MoveData, frame: u32, lock_frame: u32| -> Option<usize> {
        let bite_end = mv.startup + mv.active;
        if frame < lock_frame + 4 {
            None
        } else if frame < mv.startup.saturating_sub(10) {
            Some(0)
        } else if frame < mv.startup.saturating_sub(6) {
            Some(1)
        } else if frame < mv.startup.saturating_sub(2) {
            Some(2)
        } else if frame <= mv.startup {
            Some(3)
        } else if frame <= bite_end + 4 {
            Some(4)
        } else if frame <= bite_end + 12 {
            Some(5)
        } else {
            None
        }
    };
    for (PoolOf(i), mut sprite, mut tf, mut visibility) in &mut pools {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
        let (Some(b), Some(lock), true) = (mv.beam.filter(|b| b.whale), f.beam_lock, f.state == FighterState::Attack) else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        // the pool is there from the lock until the whale has gone down again
        if f.state_frame > mv.startup + mv.active + 16 {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        }
        let _ = b;
        set_ne!(*visibility, Visibility::Inherited);
        set_ne!(tf.translation.x, (lock / PX) as f32);
        set_ne!(tf.translation.y, GROUND_Y - 2.0);
        let frame = (sim.0.frame / 6 % 2) as usize;
        if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("pool_{frame}"))) {
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        }
    }
    for (WhaleOf(i), mut sprite, mut tf, mut visibility) in &mut whales {
        let f = &sim.0.fighters[*i];
        let data = &roster.0[sel.chars[*i]];
        let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
        let (Some(b), Some(lock), true) = (mv.beam.filter(|b| b.whale), f.beam_lock, f.state == FighterState::Attack) else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        let Some(which) = drawing(mv, f.state_frame, b.lock_frame) else {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        };
        set_ne!(*visibility, Visibility::Inherited);
        // the drawings stand on the water line, 4 px up from the bottom of the picture
        set_ne!(tf.translation.x, (lock / PX) as f32);
        set_ne!(tf.translation.y, GROUND_Y + 28.0);
        // it faces the one who called it
        set_ne!(tf.scale.x, f.facing as f32);
        if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("whale_{which}"))) {
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        }
    }
}

/// The bubble of plot armour round a fighter who has it (it flickers in its last half second), with its label above the top
/// left of it.
pub fn animate_shield(
    sim: Res<Sim>,
    handles: Res<FrameHandles>,
    mut bubbles: Query<(&ShieldOf, &mut Sprite, &mut Transform, &mut Visibility), Without<ShieldLabel>>,
    mut labels: Query<(&ShieldLabel, &mut Transform, &mut Visibility, &mut TextColor), Without<ShieldOf>>,
) {
    for (ShieldOf(i), mut sprite, mut tf, mut visibility) in &mut bubbles {
        let f = &sim.0.fighters[*i];
        if f.plot_armour == 0 {
            set_ne!(*visibility, Visibility::Hidden);
            continue;
        }
        // blinking as it runs out
        let shown = f.plot_armour > 30 || (f.plot_armour / 3) % 2 == 0;
        set_ne!(*visibility, if shown { Visibility::Inherited } else { Visibility::Hidden });
        set_ne!(tf.translation.x, (f.x / PX) as f32);
        set_ne!(tf.translation.y, GROUND_Y + (f.y / PX) as f32 + 33.0);
        let frame = (sim.0.frame / 10 % 2) as usize;
        if let Some(image) = handles.0.get(*i).and_then(|m| m.get(&format!("shield_{frame}"))) {
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        }
    }
    for (ShieldLabel(i), mut tf, mut visibility, mut color) in &mut labels {
        let f = &sim.0.fighters[*i];
        let shown = f.plot_armour > 30 || (f.plot_armour / 3) % 2 == 0;
        set_ne!(*visibility, if f.plot_armour > 0 && shown { Visibility::Inherited } else { Visibility::Hidden });
        // on the top left of the bubble, just outside it
        set_ne!(tf.translation.x, (f.x / PX) as f32 - 36.0);
        set_ne!(tf.translation.y, GROUND_Y + (f.y / PX) as f32 + 68.0);
        let tone = Color::srgb(0.7, 0.9, 1.0);
        if color.0 != tone {
            color.0 = tone;
        }
    }
}
