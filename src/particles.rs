//! Hit sparks, dust, swing streaks, charge sparkles, star trails and screen shake. Only drawn, never part of
//! the simulation: `emit` watches how the state changed since the last tick and `update_particles` moves
//! and fades them every frame.

use bevy::prelude::*;
use t2f::anim::{frame_for, Frame, CENTRE_X, FULL_H};
use t2f::sim::{AnimKind, Fighter, FighterState, PX};

use crate::{AppState, Roster, Selection, Sim, GROUND_Y};

#[derive(Component)]
pub struct Particle {
    vel: Vec2,
    life: f32,
    max: f32,
    gravity: f32,
    size: f32,
    shrink: bool,
}

/// What a fighter looked like on the previous tick.
#[derive(Clone, Copy)]
struct Snap {
    state: FighterState,
    health: i32,
    y: i32,
    slamming: bool,
}

/// A comic-book word ("BAM!") that pops up and fades.
#[derive(Component)]
pub struct Pop {
    life: f32,
    max: f32,
    vel: Vec2,
    rgb: [f32; 3],
}

#[derive(Default)]
pub struct Seen {
    fighters: [Option<Snap>; 2],
    ready: [bool; 2],
    /// Each fighter's curse last tick: (left half there, right half there, squeezing).
    curse: [Option<(bool, bool, bool)>; 2],
    /// Lobs seen last tick: (owner, landing spot in pixels, splash radius in pixels).
    lobs: Vec<(usize, i32, i32)>,
    seed: u32,
}

/// Screen shake in pixels; decays by itself.
#[derive(Resource, Default)]
pub struct Shake(pub f32);

impl Seen {
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223) ^ 0x9e37;
        ((self.seed >> 8) & 0xffff) as f32 / 65535.0
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.rand()
    }
}

fn spawn(commands: &mut Commands, at: Vec2, vel: Vec2, color: Color, size: f32, life: f32, gravity: f32, shrink: bool) {
    commands.spawn((
        Particle { vel, life, max: life, gravity, size, shrink },
        DespawnOnExit(AppState::Fight),
        Sprite::from_color(color, Vec2::splat(size)),
        Transform::from_xyz(at.x.round(), at.y.round(), 5.0),
    ));
}

/// A floating word or symbol that drifts and fades.
fn pop(commands: &mut Commands, text: &str, at: Vec2, vel: Vec2, rgb: [f32; 3], size: f32) {
    commands.spawn((
        Pop { life: 0.7, max: 0.7, vel, rgb },
        DespawnOnExit(AppState::Fight),
        Text2d::new(text),
        TextFont { font_size: FontSize::Px(size), ..default() },
        TextColor(Color::srgba(rgb[0], rgb[1], rgb[2], 1.0)),
        Transform::from_xyz(at.x.round(), at.y.round(), 6.0),
    ));
}

/// A burst of `n` particles flying out from `at` at between `lo` and `hi` pixels a second.
fn burst(commands: &mut Commands, seen: &mut Seen, at: Vec2, n: usize, lo: f32, hi: f32, colors: &[Color], size: f32, life: f32, gravity: f32) {
    for k in 0..n {
        let angle = seen.range(0.0, std::f32::consts::TAU);
        let speed = seen.range(lo, hi);
        let color = colors[k % colors.len()];
        let s = size * seen.range(0.7, 1.4);
        spawn(commands, at, Vec2::new(angle.cos(), angle.sin()) * speed, color, s.round().max(1.0), life * seen.range(0.7, 1.2), gravity, true);
    }
}

pub fn emit(
    mut commands: Commands,
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    mut seen: Local<Seen>,
    mut shake: ResMut<Shake>,
) {
    let s = &sim.0;
    let spark = [Color::srgb(1.0, 0.95, 0.5), Color::WHITE, Color::srgb(1.0, 0.6, 0.15)];
    let guard = [Color::srgb(0.7, 0.85, 1.0), Color::WHITE];
    let dust = [Color::srgb(0.85, 0.8, 0.72), Color::srgb(0.7, 0.66, 0.6)];
    let gold = [Color::srgb(1.0, 0.85, 0.25), Color::srgb(1.0, 0.97, 0.7), Color::WHITE];
    if seen.seed == 0 {
        seen.seed = 12345;
    }

    for i in 0..2 {
        let f = &s.fighters[i];
        let data = &roster.0[sel.chars[i]];
        let feet = Vec2::new((f.x / PX) as f32, GROUND_Y + (f.y / PX) as f32);
        let towards = if s.fighters[1 - i].x >= f.x { 1.0 } else { -1.0 };
        if let Some(prev) = seen.fighters[i] {
            // hits and blocks: health dropped this tick
            if f.health < prev.health {
                let lost = prev.health - f.health;
                let at = feet + Vec2::new(towards * 10.0, 32.0);
                if f.state == FighterState::Blockstun {
                    burst(&mut commands, &mut seen, at, 6, 40.0, 110.0, &guard, 2.0, 0.25, 0.0);
                } else {
                    let n = 7 + (lost / 8) as usize;
                    burst(&mut commands, &mut seen, at, n.min(26), 50.0, 170.0, &spark, 3.0, 0.35, -60.0);
                    spawn(&mut commands, at, Vec2::ZERO, Color::WHITE, 14.0, 0.09, 0.0, true);
                    shake.0 = shake.0.max((lost as f32 / 14.0).min(6.0));
                }
            }
            if f.state == FighterState::Ko && prev.state != FighterState::Ko {
                burst(&mut commands, &mut seen, feet + Vec2::new(0.0, 30.0), 36, 60.0, 240.0, &spark, 3.0, 0.7, -120.0);
                shake.0 = 8.0;
            }
            // dust from jumping, landing and walking
            if prev.y == 0 && f.y > 0 {
                for k in 0..6 {
                    let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
                    let v = Vec2::new(dir * seen.range(20.0, 60.0), seen.range(5.0, 25.0));
                    spawn(&mut commands, feet + Vec2::new(dir * 6.0, 1.0), v, dust[k % 2], 3.0, 0.4, -20.0, true);
                }
            }
            // ground pound: speed lines while plunging, a ring of rubble and a shake on landing
            if f.slamming && s.frame % 2 == 0 {
                for dx in [-6.0, 0.0, 6.0] {
                    spawn(&mut commands, feet + Vec2::new(dx, 18.0 + seen.range(0.0, 18.0)), Vec2::new(0.0, 90.0), Color::srgb(1.0, 1.0, 1.0), 1.0, 0.18, 0.0, true);
                }
            }
            if prev.slamming && !f.slamming && f.y == 0 {
                for k in 0..22 {
                    let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
                    let v = Vec2::new(dir * seen.range(40.0, 170.0), seen.range(20.0, 90.0));
                    spawn(&mut commands, feet + Vec2::new(dir * seen.range(2.0, 10.0), 1.0), v, dust[k % 2], seen.range(2.0, 4.0), 0.55, -260.0, true);
                }
                // flat shockwave along the floor
                for k in 0..10 {
                    let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
                    spawn(&mut commands, feet + Vec2::new(0.0, 2.0), Vec2::new(dir * 150.0, 0.0), Color::srgb(1.0, 0.95, 0.8), 3.0, 0.25, 0.0, true);
                }
                shake.0 = shake.0.max(5.0);
            }
            if prev.y > 0 && f.y == 0 && f.state != FighterState::Ko {
                for k in 0..8 {
                    let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
                    let v = Vec2::new(dir * seen.range(30.0, 90.0), seen.range(5.0, 20.0));
                    spawn(&mut commands, feet + Vec2::new(dir * 8.0, 1.0), v, dust[k % 2], 3.0, 0.4, -20.0, true);
                }
            }
            // dashes and rolls kick dust up behind them
            if matches!(f.state, FighterState::Dash | FighterState::Roll) && f.y == 0 && f.state_frame % 2 == 0 {
                let back = -(f.move_dir as f32);
                let v = Vec2::new(back * seen.range(20.0, 70.0), seen.range(8.0, 28.0));
                spawn(&mut commands, feet + Vec2::new(back * 8.0, 1.0), v, dust[(f.state_frame as usize / 2) % 2], 3.0, 0.35, -40.0, true);
            }
            if f.state == FighterState::Walk && f.vx != 0 && f.state_frame % 8 == 0 {
                let v = Vec2::new(-(f.facing as f32) * seen.range(10.0, 30.0), seen.range(8.0, 24.0));
                spawn(&mut commands, feet + Vec2::new(-(f.facing as f32) * 8.0, 1.0), v, dust[0], 3.0, 0.3, -20.0, true);
            }
        }

        // the swing: streaks trailing off the front of an active hitbox
        if let Some(b) = s.hitbox(i, data) {
            let front = if f.facing > 0 { b[2] } else { b[0] };
            let at = Vec2::new((front / PX) as f32, GROUND_Y + ((b[1] + b[3]) / 2 / PX) as f32);
            for _ in 0..2 {
                let jitter = seen.range(-6.0, 6.0);
                spawn(
                    &mut commands,
                    at + Vec2::new(-(f.facing as f32) * seen.range(0.0, 14.0), jitter),
                    Vec2::new(-(f.facing as f32) * 40.0, 0.0),
                    Color::srgba(1.0, 1.0, 1.0, 0.9),
                    2.0,
                    0.14,
                    0.0,
                    true,
                );
            }
        }

        // charging: sparkles flow into the crescent over his head, more the longer he holds
        if f.state == FighterState::Attack && f.charge > 0 && !f.launched {
            if let Some(p) = &data.moves[f.attack].projectile {
                let centre = feet + Vec2::new(f.facing as f32 * p.x as f32, p.y as f32);
                for _ in 0..(1 + f.charge / 12) {
                    let angle = seen.range(0.0, std::f32::consts::TAU);
                    let radius = seen.range(22.0, 38.0);
                    let off = Vec2::new(angle.cos(), angle.sin()) * radius;
                    spawn(&mut commands, centre + off, -off / 0.3, gold[(f.charge as usize) % 3], 2.0, 0.3, 0.0, true);
                }
                if f.charge == data.moves[f.attack].charge_frames {
                    burst(&mut commands, &mut seen, centre, 10, 60.0, 120.0, &gold, 3.0, 0.3, 0.0);
                }
            }
        }

        // smoke curls up from the fingertip after the finger-gun volley
        if f.state == FighterState::Attack && data.moves[f.attack].anim == AnimKind::Volley && frame_for(f, data, false, s.frame) == Frame::Gun(4) && s.frame % 3 == 0 {
            // tip of the finger that fired last (column 63, row 22 of the drawing), in the world
            let tip = Vec2::new((f.x / PX) as f32 + f.facing as f32 * (63 - CENTRE_X) as f32, GROUND_Y + (f.y / PX) as f32 + (FULL_H - 22) as f32);
            let grey = [Color::srgb(0.88, 0.88, 0.9), Color::srgb(0.7, 0.7, 0.75)];
            let off = Vec2::new(seen.range(-1.5, 1.5), 0.0);
            let drift = Vec2::new(seen.range(-4.0, 8.0) * f.facing as f32, seen.range(14.0, 24.0));
            let size = if seen.rand() > 0.5 { 3.0 } else { 4.0 };
            spawn(&mut commands, tip + off, drift, grey[(s.frame as usize / 3) % 2], size, 1.1, 0.0, false);
        }

        // the curse: sparkles round the ring, shards when a half is hit, a collapse at the end
        let centre = feet + Vec2::new(0.0, 38.0);
        let blue = [Color::srgb(0.45, 0.72, 1.0), Color::WHITE, Color::srgb(0.2, 0.45, 0.95)];
        if let Some(c) = f.curse {
            if s.frame % 3 == 0 {
                let a = seen.range(0.0, std::f32::consts::TAU);
                let p = centre + Vec2::new(a.cos() * 58.0, a.sin() * 58.0);
                spawn(&mut commands, p, Vec2::new(0.0, seen.range(6.0, 18.0)), blue[(s.frame as usize / 3) % 3], 2.0, 0.6, 0.0, true);
            }
            if c.squeeze > 0 {
                // the ring closes in: sparks fly towards the middle
                for _ in 0..3 {
                    let a = seen.range(0.0, std::f32::consts::TAU);
                    let k = 1.0 - c.squeeze as f32 / c.squeeze_frames.max(1) as f32;
                    let p = centre + Vec2::new(a.cos() * 58.0 * k, a.sin() * 58.0 * k);
                    spawn(&mut commands, p, (centre - p) * 6.0, blue[(seen.rand() * 3.0) as usize % 3], 2.0, 0.18, 0.0, true);
                }
            }
            if let Some((was_left, was_right, _)) = seen.curse[i] {
                if was_left && !c.left {
                    burst(&mut commands, &mut seen, centre + Vec2::new(-34.0, 0.0), 14, 40.0, 120.0, &blue, 3.0, 0.5, 0.0);
                    pop(&mut commands, "HIT", centre + Vec2::new(-34.0, 30.0), Vec2::new(-8.0, 20.0), [0.5, 0.8, 1.0], 9.0);
                }
                if was_right && !c.right {
                    burst(&mut commands, &mut seen, centre + Vec2::new(34.0, 0.0), 14, 40.0, 120.0, &blue, 3.0, 0.5, 0.0);
                    pop(&mut commands, "HIT", centre + Vec2::new(34.0, 30.0), Vec2::new(8.0, 20.0), [0.5, 0.8, 1.0], 9.0);
                }
            }
            seen.curse[i] = Some((c.left, c.right, c.squeeze > 0));
        } else {
            if let Some((_, _, true)) = seen.curse[i] {
                burst(&mut commands, &mut seen, centre, 40, 80.0, 230.0, &blue, 3.0, 0.7, -60.0);
                spawn(&mut commands, centre, Vec2::ZERO, Color::WHITE, 20.0, 0.12, 0.0, true);
                shake.0 = shake.0.max(6.0);
            }
            seen.curse[i] = None;
        }

        // guard break: the guard shatters and the hit lands in full
        if let Some(prev) = seen.fighters[i] {
            if f.health < prev.health && matches!(prev.state, FighterState::Guard | FighterState::Blockstun) && f.state == FighterState::Hitstun {
                let at = feet + Vec2::new(towards * 8.0, 34.0);
                let ice = [Color::srgb(0.6, 0.9, 1.0), Color::WHITE, Color::srgb(0.35, 0.7, 1.0)];
                burst(&mut commands, &mut seen, at, 22, 60.0, 190.0, &ice, 3.0, 0.45, -80.0);
                pop(&mut commands, "BREAK!", feet + Vec2::new(0.0, 66.0), Vec2::new(0.0, 22.0), [0.55, 0.9, 1.0], 12.0);
                shake.0 = shake.0.max(4.0);
            }
        }

        // the lecture: words and symbols pour out of his mouth, a blast of them when it lands, chalk dust off the raised finger (the collar grab only talks while he holds the enemy)
        let talking = |mv: &t2f::sim::MoveData| match mv.anim {
            AnimKind::Lecture => true,
            AnimKind::Grab => f.state_frame > mv.startup && f.state_frame <= mv.startup + mv.hit_gap,
            _ => false,
        };
        if f.state == FighterState::Attack && talking(&data.moves[f.attack]) {
            let mv = &data.moves[f.attack];
            let dir = f.facing as f32;
            let mouth = feet + Vec2::new(2.0 * dir, 41.0);
            if s.frame % 4 == 0 {
                let glyphs = ["?", "!", "...", "%", "+", "=", "a", "b", "#", "2", "x"];
                let g = glyphs[(seen.rand() * glyphs.len() as f32) as usize % glyphs.len()];
                let rgb = if seen.rand() > 0.5 { [1.0, 1.0, 1.0] } else { [1.0, 0.92, 0.5] };
                let vel = Vec2::new(dir * seen.range(16.0, 34.0), seen.range(8.0, 22.0));
                pop(&mut commands, g, mouth + Vec2::new(dir * 4.0, seen.range(-3.0, 6.0)), vel, rgb, 9.0);
            }
            if s.frame % 3 == 0 && mv.anim == AnimKind::Lecture {
                let tip = feet + Vec2::new(dir * 20.0, 57.0);
                let v = Vec2::new(seen.range(-12.0, 12.0), seen.range(-6.0, 10.0));
                spawn(&mut commands, tip, v, Color::srgb(0.96, 0.96, 0.98), 2.0, 0.5, -30.0, true);
            }
            if mv.is_active(f.state_frame) && mv.anim == AnimKind::Lecture {
                // the words land: a fan of white flecks in front of him
                for _ in 0..5 {
                    let v = Vec2::new(dir * seen.range(70.0, 130.0), seen.range(-22.0, 22.0));
                    spawn(&mut commands, mouth + Vec2::new(dir * 6.0, seen.range(-3.0, 3.0)), v, Color::WHITE, 2.0, 0.28, 0.0, true);
                }
            }
        }

        // runes: golden glyphs swirl round him as he pronounces, sparks trail the wave in flight
        let runic = |m: &t2f::sim::MoveData| m.projectile.as_ref().and_then(|p| p.image.as_deref()) == Some("runes");
        if f.state == FighterState::Attack && runic(&data.moves[f.attack]) {
            let mv = &data.moves[f.attack];
            let dir = f.facing as f32;
            let centre = feet + Vec2::new(0.0, 36.0);
            if s.frame % 2 == 0 {
                let glyphs = ["*", "+", "x", "o", "#", "%", "=", "~"];
                let g = glyphs[(seen.rand() * glyphs.len() as f32) as usize % glyphs.len()];
                let angle = seen.range(0.0, std::f32::consts::TAU);
                let r = seen.range(18.0, 34.0);
                let at = centre + Vec2::new(angle.cos() * r, angle.sin() * r * 0.8);
                let vel = Vec2::new(angle.cos() * 6.0 + dir * 14.0, 14.0 + seen.range(0.0, 12.0));
                let rgb = if seen.rand() > 0.4 { [1.0, 0.86, 0.25] } else { [1.0, 0.96, 0.6] };
                pop(&mut commands, g, at, vel, rgb, 10.0);
            }
            if s.frame % 3 == 0 {
                let at = centre + Vec2::new(seen.range(-26.0, 26.0), seen.range(-22.0, 26.0));
                spawn(&mut commands, at, Vec2::new(0.0, seen.range(8.0, 24.0)), gold[(s.frame as usize / 3) % 3], 2.0, 0.55, 0.0, true);
            }
            if mv.is_active(f.state_frame) {
                burst(&mut commands, &mut seen, centre + Vec2::new(dir * 30.0, 4.0), 14, 50.0, 150.0, &gold, 3.0, 0.5, 0.0);
            }
        }
        for p in s.projectiles.iter().filter(|p| p.owner == i && runic(&data.moves[p.move_index])) {
            let at = Vec2::new((p.x / PX) as f32, GROUND_Y + (p.y / PX) as f32);
            for _ in 0..2 {
                let v = Vec2::new(seen.range(-20.0, 20.0), seen.range(-12.0, 24.0));
                spawn(&mut commands, at + Vec2::new(seen.range(-10.0, 10.0), seen.range(-8.0, 8.0)), v, gold[(s.frame as usize + 1) % 3], 2.0, 0.45, 0.0, true);
            }
            if s.frame % 3 == 0 {
                let glyphs = ["*", "+", "x", "o"];
                let g = glyphs[(seen.rand() * glyphs.len() as f32) as usize % glyphs.len()];
                pop(&mut commands, g, at + Vec2::new(seen.range(-12.0, 4.0), seen.range(-8.0, 12.0)), Vec2::new(0.0, 16.0), [1.0, 0.88, 0.3], 9.0);
            }
        }

        // the moon appearing: sparkles where it hangs
        if let Some((mx, my, age)) = f.moon {
            let at = Vec2::new((mx / PX) as f32, GROUND_Y + (my / PX) as f32);
            if age == 1 {
                burst(&mut commands, &mut seen, at, 20, 40.0, 120.0, &gold, 3.0, 0.6, 0.0);
            } else if age < 50 && s.frame % 5 == 0 {
                let off = Vec2::new(seen.range(-14.0, 14.0), seen.range(-14.0, 14.0));
                spawn(&mut commands, at + off, Vec2::ZERO, gold[(s.frame as usize / 5) % 3], 2.0, 0.5, 0.0, true);
            }
        }

        // the fish: bubbles where it will land (the warning), a bubbly trail, a splash on landing
        let blue = [Color::srgb(0.72, 0.9, 1.0), Color::WHITE, Color::srgb(0.45, 0.72, 0.98)];
        let mut lobs_now: Vec<(usize, i32, i32)> = Vec::new();
        for p in s.projectiles.iter().filter(|p| p.owner == i) {
            let Some(pd) = data.moves[p.move_index].projectile.as_ref().filter(|d| d.lob) else { continue };
            let (land, radius) = ((p.land_x / PX), pd.splash_radius);
            lobs_now.push((i, land, radius));
            for _ in 0..2 {
                let x = land as f32 + seen.range(-(radius as f32), radius as f32);
                let v = Vec2::new(seen.range(-3.0, 3.0), seen.range(8.0, 20.0));
                spawn(&mut commands, Vec2::new(x, GROUND_Y + 1.0), v, blue[(seen.rand() * 3.0) as usize % 3], if seen.rand() > 0.6 { 4.0 } else { 3.0 }, 0.8, 0.0, true);
            }
            // the edge of the zone is marked by a ring of darker bubbles
            for edge in [-1.0f32, 1.0] {
                if s.frame % 3 == 0 {
                    spawn(&mut commands, Vec2::new(land as f32 + edge * radius as f32, GROUND_Y + 1.0), Vec2::new(0.0, 6.0), Color::srgb(0.3, 0.58, 0.95), 3.0, 0.6, 0.0, true);
                }
            }
            let at = Vec2::new((p.x / PX) as f32, GROUND_Y + (p.y / PX) as f32);
            let v = Vec2::new(seen.range(-10.0, 10.0), seen.range(0.0, 14.0));
            spawn(&mut commands, at + Vec2::new(seen.range(-4.0, 4.0), seen.range(-3.0, 3.0)), v, blue[(s.frame as usize) % 3], 3.0, 0.4, 0.0, true);
        }
        // before the fish is shot, the landing spot is already marked with bubbles
        if f.state == FighterState::Attack && !f.launched {
            if let (Some(land), Some(pd)) = (f.lob_target, data.moves[f.attack].projectile.as_ref().filter(|d| d.lob)) {
                let (land, radius) = (land / PX, pd.splash_radius);
                for _ in 0..2 {
                    let x = land as f32 + seen.range(-(radius as f32), radius as f32);
                    let v = Vec2::new(seen.range(-3.0, 3.0), seen.range(8.0, 20.0));
                    spawn(&mut commands, Vec2::new(x, GROUND_Y + 1.0), v, blue[(seen.rand() * 3.0) as usize % 3], if seen.rand() > 0.6 { 4.0 } else { 3.0 }, 0.8, 0.0, true);
                }
                for edge in [-1.0f32, 1.0] {
                    if s.frame % 3 == 0 {
                        spawn(&mut commands, Vec2::new(land as f32 + edge * radius as f32, GROUND_Y + 1.0), Vec2::new(0.0, 6.0), Color::srgb(0.3, 0.58, 0.95), 3.0, 0.6, 0.0, true);
                    }
                }
            }
        }
        let landed: Vec<(usize, i32, i32)> = seen.lobs.iter().filter(|l| l.0 == i && !lobs_now.contains(l)).cloned().collect();
        seen.lobs.retain(|l| l.0 != i);
        seen.lobs.extend(lobs_now);
        for (_, land, radius) in landed {
            let at = Vec2::new(land as f32, GROUND_Y + 1.0);
            // water thrown up in a fountain
            for k in 0..40 {
                let angle = seen.range(0.35, std::f32::consts::PI - 0.35);
                let speed = seen.range(60.0, 170.0);
                let v = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                let x = at.x + seen.range(-(radius as f32) * 0.5, radius as f32 * 0.5);
                spawn(&mut commands, Vec2::new(x, at.y), v, blue[k % 3], if k % 4 == 0 { 3.0 } else { 2.0 }, 0.9, -260.0, true);
            }
            // bubbles across the splash zone, and a puddle that dries up
            for k in 0..18 {
                let x = at.x + seen.range(-(radius as f32), radius as f32);
                spawn(&mut commands, Vec2::new(x, at.y), Vec2::new(seen.range(-6.0, 6.0), seen.range(14.0, 36.0)), blue[k % 3], 4.0, 1.1, 0.0, true);
            }
            for k in 0..14 {
                let x = at.x + (k as f32 / 13.0 * 2.0 - 1.0) * radius as f32;
                spawn(&mut commands, Vec2::new(x, GROUND_Y + 1.0), Vec2::ZERO, Color::srgb(0.4, 0.68, 0.97), 4.0, 1.4, 0.0, false);
            }
            shake.0 = shake.0.max(4.0);
        }

        // projectiles: a burst and flash when thrown, a glittering trail behind; a volley shot also gets a "BAM!" at the fingertips
        for p in s.projectiles.iter().filter(|p| p.owner == i) {
            let at = Vec2::new((p.x / PX) as f32, GROUND_Y + (p.y / PX) as f32);
            let mv = &data.moves[p.move_index];
            if p.age == 1 {
                burst(&mut commands, &mut seen, at, 14, 50.0, 150.0, &gold, 3.0, 0.4, 0.0);
                if mv.anim == AnimKind::Volley {
                    spawn(&mut commands, at, Vec2::ZERO, Color::WHITE, 12.0, 0.1, 0.0, true);
                    shake.0 = shake.0.max(2.0);
                    let jitter = seen.range(-6.0, 6.0);
                    commands.spawn((
                        Pop { life: 0.45, max: 0.45, vel: Vec2::new(0.0, 28.0), rgb: [1.0, 0.9, 0.2] },
                        DespawnOnExit(AppState::Fight),
                        Text2d::new("BAM!"),
                        TextFont { font_size: FontSize::Px(14.0), ..default() },
                        TextColor(Color::srgb(1.0, 0.9, 0.2)),
                        Transform::from_xyz(at.x + towards * 14.0, at.y + 12.0 + jitter, 6.0),
                    ));
                }
            }
            let size = 2.0 + (p.power - 100) as f32 / 40.0;
            for _ in 0..2 {
                let v = Vec2::new(seen.range(-15.0, 15.0), seen.range(-15.0, 15.0));
                let off = Vec2::new(seen.range(-5.0, 5.0), seen.range(-5.0, 5.0));
                spawn(&mut commands, at + off, v, gold[(seen.rand() * 3.0) as usize % 3], size.round(), 0.3, 0.0, true);
            }
        }

        // the special becoming ready: a burst of sparkles round the head, then a trickle while it's ready
        let ready = f.meter >= Fighter::METER_MAX;
        let head = feet + Vec2::new(0.0, 50.0);
        if ready && !seen.ready[i] {
            burst(&mut commands, &mut seen, head, 18, 40.0, 130.0, &gold, 3.0, 0.5, 0.0);
            spawn(&mut commands, head, Vec2::ZERO, Color::WHITE, 16.0, 0.12, 0.0, true);
        } else if ready && f.state != FighterState::Attack && s.frame % 6 == 0 {
            let off = Vec2::new(seen.range(-10.0, 10.0), seen.range(-4.0, 8.0));
            spawn(&mut commands, head + off, Vec2::new(0.0, 18.0), gold[(s.frame as usize / 6) % 3], 2.0, 0.5, 0.0, true);
        }
        seen.ready[i] = ready;
        seen.fighters[i] = Some(Snap { state: f.state, health: f.health, y: f.y, slamming: f.slamming });
    }
}

/// Move, fade and remove particles.
pub fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite), Without<Pop>>,
    mut pops: Query<(Entity, &mut Pop, &mut Transform, &mut TextColor), Without<Particle>>,
    mut shake: ResMut<Shake>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut tf, mut sprite) in &mut particles {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let g = p.gravity;
        p.vel.y += g * dt;
        tf.translation.x += p.vel.x * dt;
        tf.translation.y += p.vel.y * dt;
        let t = p.life / p.max;
        let size = if p.shrink { (p.size * t.sqrt()).max(1.0) } else { p.size };
        sprite.custom_size = Some(Vec2::splat(size.round().max(1.0)));
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, (t * 2.0).min(1.0));
    }
    for (entity, mut pop, mut tf, mut color) in &mut pops {
        pop.life -= dt;
        if pop.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation.x += pop.vel.x * dt;
        tf.translation.y += pop.vel.y * dt;
        let t = pop.life / pop.max;
        tf.scale = Vec3::splat(1.0 + (1.0 - t) * 0.4);
        color.0 = Color::srgba(pop.rgb[0], pop.rgb[1], pop.rgb[2], (t * 2.0).min(1.0));
    }
    shake.0 = (shake.0 - 18.0 * dt).max(0.0);
}
