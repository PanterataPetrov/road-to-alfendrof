//! Animation: one whole-body drawing per frame, so torso, arms and legs move together.
//! No Bevy in here, so the sprite generator and the game agree on sizes and file names.

use crate::sim::{AnimKind, CharacterData, Fighter, FighterState, Raven, RavenMove};

/// Every drawing is this size. The body centre is at `CENTRE_X`, with room on the right for arms and on the left for hair. Feet are on the bottom row.
pub const FRAME_W: i32 = 84;
pub const FULL_H: i32 = 63;
pub const CENTRE_X: i32 = 33;
/// Height of the torso part while drawing (head to hips).
pub const UPPER_H: i32 = 50;

pub const IDLE_FRAMES: usize = 2;
pub const WALK_FRAMES: usize = 6;
pub const HURT_FRAMES: usize = 2;
pub const SOFT_FRAMES: usize = 2;
pub const HARD_FRAMES: usize = 4;
pub const SUMMON_FRAMES: usize = 6;
pub const SPEAK_FRAMES: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Frame {
    Idle(usize),
    Walk(usize),
    Crouch,
    CrouchSoft(usize),
    CrouchHard(usize),
    AirSoft(usize),
    AirHard(usize),
    JumpUp,
    JumpTop,
    JumpFall,
    Hurt(usize),
    Block,
    Ko,
    Soft(usize),
    Hard(usize),
    Summon(usize),
    Speak(usize),
    /// Alternating punches: right wind-up and strike, then left.
    Punch(usize),
    /// The leap: crouch, tucked in the air, then a kick with each leg.
    Kick(usize),
    /// Lecture: breath, finger up, the lecture, finger down.
    Lecture(usize),
    /// Finger-gun volley: ready, left, right, left, recovery.
    Gun(usize),
    /// Commanding the raven: finger up, arm out, recovery.
    Point(usize),
    /// Collar grab: beckon, pull, hold (talking), hold (quiet), shove, recover.
    Grab(usize),
    /// Reading a scroll: unrolling it, reading, reading (another way), closing it.
    Read(usize),
    /// The staff slam: raised, the top of the swing, the slam, recovery.
    Staff(usize),
    /// The Gun Boost: guns down ready, firing at the ground, launched up, shooting down from the air.
    Boost(usize),
    /// Leaning back as he shoots in the air: a little, more, all the way.
    Lean(usize),
    /// Writing in a book: it opens, two strokes alternate while he writes, a flourish at the end.
    Write(usize),
    /// The roll: tuck in, a tight ball turning in eighths of a turn (1..=8), then rise (9).
    Roll(usize),
    /// The portal special: stepping back, leaping into the portal, the bite, recovery.
    Portal(usize),
}

impl Frame {
    /// File name without extension, in `assets/sprites/<character>/`.
    pub fn file_stem(self) -> String {
        match self {
            Frame::Idle(i) => format!("idle_{i}"),
            Frame::Walk(i) => format!("walk_{i}"),
            Frame::Crouch => "crouch".into(),
            Frame::CrouchSoft(i) => format!("crouch_soft_{i}"),
            Frame::CrouchHard(i) => format!("crouch_hard_{i}"),
            Frame::AirSoft(i) => format!("air_soft_{i}"),
            Frame::AirHard(i) => format!("air_hard_{i}"),
            Frame::JumpUp => "jump_up".into(),
            Frame::JumpTop => "jump_top".into(),
            Frame::JumpFall => "jump_fall".into(),
            Frame::Hurt(i) => format!("hurt_{i}"),
            Frame::Block => "block".into(),
            Frame::Ko => "ko".into(),
            Frame::Soft(i) => format!("soft_{i}"),
            Frame::Hard(i) => format!("hard_{i}"),
            Frame::Summon(i) => format!("summon_{i}"),
            Frame::Speak(i) => format!("speak_{i}"),
            Frame::Gun(i) => format!("gun_{i}"),
            Frame::Lecture(i) => format!("lecture_{i}"),
            Frame::Punch(i) => format!("punch_{i}"),
            Frame::Kick(i) => format!("kick_{i}"),
            Frame::Point(i) => format!("point_{i}"),
            Frame::Grab(i) => format!("grab_{i}"),
            Frame::Read(i) => format!("read_{i}"),
            Frame::Staff(i) => format!("staff_{i}"),
            Frame::Roll(i) => format!("roll_{i}"),
            Frame::Boost(i) => format!("boost_{i}"),
            Frame::Write(i) => format!("write_{i}"),
            Frame::Lean(i) => format!("lean_{i}"),
            Frame::Portal(i) => format!("portal_{i}"),
        }
    }

    /// Every drawing a character needs.
    pub fn all() -> Vec<Frame> {
        let mut v = Vec::new();
        v.extend((0..IDLE_FRAMES).map(Frame::Idle));
        v.extend((0..WALK_FRAMES).map(Frame::Walk));
        v.extend([Frame::Crouch, Frame::CrouchSoft(0), Frame::CrouchSoft(1), Frame::CrouchHard(0), Frame::CrouchHard(1)]);
        v.extend([Frame::JumpUp, Frame::JumpTop, Frame::JumpFall]);
        v.extend([Frame::AirSoft(0), Frame::AirSoft(1), Frame::AirHard(0), Frame::AirHard(1)]);
        v.extend((0..HURT_FRAMES).map(Frame::Hurt));
        v.extend([Frame::Block, Frame::Ko]);
        v.extend((0..SOFT_FRAMES).map(Frame::Soft));
        v.extend((0..HARD_FRAMES).map(Frame::Hard));
        v.extend((0..SUMMON_FRAMES).map(Frame::Summon));
        v.extend((0..SPEAK_FRAMES).map(Frame::Speak));
        v
    }
}

/// Punch and kick drawings.
pub const PUNCH_FRAMES: usize = 4;
pub const KICK_FRAMES: usize = 4;

/// Roll drawings: the tuck, eight turns of the ball, the rise.
pub const ROLL_DRAWINGS: usize = 10;

/// Lecture drawings.
pub const LECTURE_FRAMES: usize = 4;

/// Raven command and collar grab drawings.
pub const POINT_FRAMES: usize = 3;
pub const GRAB_FRAMES: usize = 6;

/// The drawings of reading a scroll, and of the staff slam.
pub const READ_FRAMES: usize = 4;
pub const STAFF_FRAMES: usize = 4;

/// The Gun Boost drawings, and the stages of leaning back while shooting in the air.
pub const BOOST_FRAMES: usize = 4;
/// Book-writing drawings.
pub const WRITE_FRAMES: usize = 4;
pub const LEAN_FRAMES: usize = 3;

/// Finger-gun volley drawings.
pub const GUN_FRAMES: usize = 5;

/// Portal special drawings.
pub const PORTAL_FRAMES: usize = 4;

/// Everything this character needs: the common set plus what its moves use.
pub fn frames_for(data: &CharacterData) -> Vec<Frame> {
    let mut v = Frame::all();
    v.extend((0..ROLL_DRAWINGS).map(Frame::Roll));
    if data.moves.iter().any(|m| m.anim == AnimKind::Volley) {
        v.extend((0..GUN_FRAMES).map(Frame::Gun));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Punch) {
        v.extend((0..PUNCH_FRAMES).map(Frame::Punch));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Kicks) {
        v.extend((0..KICK_FRAMES).map(Frame::Kick));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Lecture) {
        v.extend((0..LECTURE_FRAMES).map(Frame::Lecture));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Point) {
        v.extend((0..POINT_FRAMES).map(Frame::Point));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Grab) {
        v.extend((0..GRAB_FRAMES).map(Frame::Grab));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Read) {
        v.extend((0..READ_FRAMES).map(Frame::Read));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Portal) {
        v.extend((0..PORTAL_FRAMES).map(Frame::Portal));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Staff) {
        v.extend((0..STAFF_FRAMES).map(Frame::Staff));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Boost) {
        v.extend((0..BOOST_FRAMES).map(Frame::Boost));
    }
    if data.moves.iter().any(|m| m.anim == AnimKind::Write) {
        v.extend((0..WRITE_FRAMES).map(Frame::Write));
    }
    if data.air_shot_tilt {
        v.extend((0..LEAN_FRAMES).map(Frame::Lean));
    }
    v
}

/// Effect drawing counts (crescent, star).
pub const CRESCENT_FRAMES: usize = 2;
pub const STAR_FRAMES: usize = 2;

/// Ticks each walking drawing is held (six make a stride).
const WALK_TICKS: u32 = 5;

/// Which roll drawing to show: tuck in, a ball turning an eighth every two frames, then rise. A
/// backward roll turns the other way.
fn roll_drawing(f: &Fighter) -> usize {
    let t = f.state_frame;
    if t <= 3 {
        0
    } else if t >= 21 {
        ROLL_DRAWINGS - 1
    } else {
        let step = ((t - 4) / 2) as usize % 8;
        if f.move_dir * f.facing > 0 { 1 + step } else { 8 - step }
    }
}

/// Picks the drawing for a fighter. `speaking` swaps idle for the talking pose; `tick` drives breathing and the mouth.
pub fn frame_for(f: &Fighter, data: &CharacterData, speaking: bool, tick: u32) -> Frame {
    match f.state {
        FighterState::Ko => return Frame::Ko,
        FighterState::Hitstun => return Frame::Hurt(if f.state_frame < 8 { 0 } else { 1 }),
        FighterState::Blockstun | FighterState::Guard => return Frame::Block,
        FighterState::Attack => return attack_frame(f, data),
        // a dash is a quick run
        FighterState::Dash => return Frame::Walk(((f.state_frame / 3) % WALK_FRAMES as u32) as usize),
        FighterState::Roll => return Frame::Roll(roll_drawing(f)),
        _ => {}
    }
    if !f.grounded() {
        return if f.vy > 300 {
            Frame::JumpUp
        } else if f.vy < -300 {
            Frame::JumpFall
        } else {
            Frame::JumpTop
        };
    }
    if f.state == FighterState::Crouch {
        return Frame::Crouch;
    }

    if f.vx != 0 && f.state == FighterState::Walk {
        return Frame::Walk(((f.state_frame / WALK_TICKS) % WALK_FRAMES as u32) as usize);
    }
    if speaking {
        Frame::Speak(((tick / 10) % 2) as usize)
    } else {
        Frame::Idle(((tick / 40) % 2) as usize)
    }
}

/// Drawings for a flying raven and its rose (each body drawing also has a `_nr` version with no raven on the shoulder).
pub fn raven_stems() -> Vec<String> {
    let mut v = Vec::new();
    for base in ["raven_fly", "raven_peck", "raven_claw", "rose"] {
        v.extend((0..2).map(|i| format!("{base}_{i}")));
    }
    v
}

/// The raven when it is away from the shoulder: flapping, clawing on the way out, pecking. `None` while perched.
pub fn raven_frame(r: &Raven, data: &CharacterData, tick: u32) -> Option<String> {
    if !r.active {
        return None;
    }
    let flap = (tick / 4) % 2;
    let kind = data.moves.get(r.move_index).and_then(|m| m.raven_move);
    Some(match kind {
        Some(RavenMove::Claw) if r.step == 0 => format!("raven_claw_{flap}"),
        Some(RavenMove::Storm { pecks, .. }) if r.step < 2 * pecks && r.step % 2 == 1 => format!("raven_peck_{}", (r.timer >= 4) as usize),
        _ => format!("raven_fly_{flap}"),
    })
}

/// Width of a whip layer; its left edge matches the body drawing's.
pub const WHIP_W: i32 = 176;

/// Every whip layer a whip character can show (file names, no extension).
pub fn whip_stems() -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for k in 0..4 {
        v.push(format!("punch_{k}_whip"));
    }
    v.extend(["punch_1_whipx".into(), "punch_3_whipx".into(), "crouch_soft_0_whip".into(), "crouch_soft_1_whip".into()]);
    v.extend((0..4).map(|k| format!("hard_{k}_whip")));
    v.extend(["air_hard_0_whip".into(), "air_hard_1_whip".into()]);
    v.extend((0..4).map(|k| format!("whip_ring_{k}")));
    v.extend(["summon_4_whip_hi".into(), "summon_4_whip_lo".into(), "summon_5_whip".into()]);
    for k in 0..2 {
        v.push(format!("air_soft_{k}_whip"));
        v.push(format!("air_soft_{k}_whipb"));
    }
    v
}

/// Where Junkhead's whip hand is in each lashing pose (pixels, from the drawing's left and top). The lash itself is drawn by the game from there to the enemy.
pub fn lash_hand(layer: &str) -> Option<(f32, f32)> {
    match layer.split("_whip").next()? {
        "punch_1" => Some((72.0, 27.5)),
        "punch_3" => Some((60.0, 26.5)),
        "crouch_soft_1" => Some((67.0, 47.5)),
        "hard_2" => Some((75.0, 26.5)),
        "air_hard_1" => Some((73.0, 29.5)),
        "summon_4" => Some((72.0, 24.5)),
        _ => None,
    }
}

/// The whip layer to draw over the body now, if any. Named after the body drawing (`punch_1` -> `punch_1_whip`): cocked back in the wind-up, out on the strike (`x` after a finisher), slack in recovery.
pub fn whip_layer(f: &Fighter, data: &CharacterData) -> Option<String> {
    if f.state != FighterState::Attack {
        return None;
    }
    let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
    if !(data.whip || mv.rope) || mv.anim == AnimKind::Kicks {
        return None;
    }
    let stem = attack_frame(f, data).file_stem();
    let frame = f.state_frame;
    let last_active = mv.startup + (mv.hits.max(1) - 1) * mv.hit_gap + mv.active;
    match mv.anim {
        AnimKind::Summon => {
            if frame <= mv.startup {
                // the arm comes up in the first quarter, then the whip spins
                (frame > mv.startup / 4).then(|| format!("whip_ring_{}", (frame / 3) % 4))
            } else if frame <= last_active + 8 {
                Some(format!("{stem}_whip_{}", if f.aim_low { "lo" } else { "hi" }))
            } else {
                Some(format!("{stem}_whip"))
            }
        }
        AnimKind::Soft if mv.air => Some(format!("{stem}_whip{}", if (frame / 3) % 2 == 1 { "b" } else { "" })),
        _ => {
            let strike = frame > mv.startup && frame <= last_active;
            let longer = f.boosted && strike && mv.boost_reach > 0;
            Some(format!("{stem}_whip{}", if longer { "x" } else { "" }))
        }
    }
}

/// The move's own drawings across startup, active and recovery.
fn attack_frame(f: &Fighter, data: &CharacterData) -> Frame {
    let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
    let frame = f.state_frame;
    let startup = mv.startup.max(1);
    // shooting in the air leans him back: a little as he aims, all the way as the shot goes, and back up
    if data.air_shot_tilt && mv.air && mv.projectile.is_some() {
        let shot = startup + mv.volley_delay;
        let hold = shot + mv.active + 4;
        return Frame::Lean(if frame + 8 <= shot {
            0
        } else if frame < shot {
            1
        } else if frame <= hold {
            2
        } else if frame <= hold + 6 {
            1
        } else {
            0
        });
    }
    if mv.air && mv.anim != AnimKind::Summon {
        let strike = (frame > mv.startup) as usize;
        return match mv.anim {
            AnimKind::Soft => Frame::AirSoft(strike),
            _ => Frame::AirHard(strike),
        };
    }
    if mv.crouching {
        let strike = (frame > mv.startup) as usize;
        return match mv.anim {
            AnimKind::Soft => Frame::CrouchSoft(strike),
            _ => Frame::CrouchHard(strike),
        };
    }
    match mv.anim {
        AnimKind::Punch => Frame::Punch(f.punch_side as usize * 2 + (frame > mv.startup) as usize),
        AnimKind::Kicks => {
            if frame <= mv.startup {
                Frame::Kick(0)
            } else if let Some(k) = mv.window(frame) {
                Frame::Kick(2 + (k % 2) as usize)
            } else {
                Frame::Kick(1)
            }
        }
        AnimKind::Boost => {
            // crouched with the guns down, then firing at the ground; once launched, up in the air until
            // the volley begins, which is shot leaning back with the guns pointing down
            let first_shot = mv.startup + mv.volley_delay;
            if frame * 2 <= mv.startup {
                Frame::Boost(0)
            } else if frame <= mv.startup + 3 {
                Frame::Boost(1)
            } else if frame + 2 < first_shot {
                Frame::Boost(2)
            } else {
                Frame::Boost(3)
            }
        }
        AnimKind::Lecture => {
            let third = (startup / 3).max(1);
            Frame::Lecture(if frame <= third {
                0
            } else if frame <= mv.startup {
                1
            } else if frame <= mv.startup + mv.active {
                2
            } else {
                3
            })
        }
        AnimKind::Volley => {
            let total = mv.volley.max(1);
            if f.shots == 0 {
                Frame::Gun(0)
            } else if f.shots >= total && frame > mv.startup + (total - 1) * mv.volley_gap + 5 {
                Frame::Gun(GUN_FRAMES - 1)
            } else {
                // hands alternate, left first
                Frame::Gun(f.shots.min(3) as usize)
            }
        }
        AnimKind::Speak => Frame::Speak(if frame <= mv.startup { 0 } else { 1 }),
        AnimKind::Read => {
            // Unrolling the scroll, then reading: the two reading drawings alternate every 12
            // frames he has been held there (`charge`); the scroll closes in recovery.
            Frame::Read(if f.charge > 0 && frame + 1 == mv.startup {
                1 + ((f.charge / 12) % 2) as usize
            } else if frame < mv.startup {
                (frame * 100 > mv.startup * 60) as usize
            } else if frame <= mv.startup + mv.active {
                1
            } else {
                3
            })
        }
        // the book opens, two strokes alternate while he writes, then the flourish
        AnimKind::Write => Frame::Write(if frame * 100 <= mv.startup * 18 {
            0
        } else if frame <= mv.startup {
            1 + ((frame / 6) % 2) as usize
        } else {
            3
        }),
        AnimKind::Staff => Frame::Staff(if frame * 2 <= mv.startup {
            0
        } else if frame <= mv.startup {
            1
        } else if frame <= mv.startup + mv.active + 5 {
            2
        } else {
            3
        }),
        AnimKind::Point => {
            // finger up for half the startup, then the arm goes out
            let cock = (mv.startup * 55 / 100).max(1);
            Frame::Point(if frame <= cock {
                0
            } else if frame <= mv.startup + mv.active + 3 {
                1
            } else {
                2
            })
        }
        AnimKind::Grab => {
            // beckon, pull until the first hit, hold while talking, shove on the last hit, recover
            let second = mv.startup + (mv.hits.max(1) - 1) * mv.hit_gap;
            Frame::Grab(if frame <= startup / 2 {
                0
            } else if frame <= mv.startup + mv.active {
                1
            } else if frame <= second {
                2 + ((frame / 8) % 2) as usize
            } else if frame <= second + mv.active + 4 {
                4
            } else {
                5
            })
        }
        AnimKind::Portal => {
            // step back, leap into the portal (unseen until she comes out), then the bite if the grab held
            let vanish = mv.portal.map_or(mv.startup, |p| p.vanish);
            Frame::Portal(if frame * 2 <= vanish {
                0
            } else if frame <= mv.startup + mv.active && !f.grabbed {
                1
            } else if f.grabbed && frame <= mv.startup + mv.hit_gap + mv.active + 6 {
                2
            } else {
                3
            })
        }
        AnimKind::Soft => Frame::Soft(if frame <= mv.startup { 0 } else { 1 }),
        AnimKind::Hard => {
            let half = (startup / 2).max(1);
            Frame::Hard(if frame <= half {
                0
            } else if frame <= mv.startup {
                1
            } else if frame <= mv.startup + mv.active {
                2
            } else {
                3
            })
        }
        AnimKind::Summon => Frame::Summon(if frame <= mv.startup {
            ((frame * 4 / startup) as usize).min(3)
        } else if frame <= mv.startup + mv.active + 8 {
            4
        } else {
            5
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ayz() -> CharacterData {
        CharacterData::from_ron(include_str!("sim/classic_fighter.ron")).unwrap()
    }

    fn attacking(move_name: &str, frame: u32) -> (Fighter, CharacterData) {
        let data = ayz();
        let mut f = Fighter::new(0, 1);
        f.state = FighterState::Attack;
        f.attack = data.moves.iter().position(|m| m.name == move_name).unwrap();
        f.state_frame = frame;
        (f, data)
    }

    #[test]
    fn standing_still_uses_an_idle_drawing() {
        let f = Fighter::new(0, 1);
        assert!(matches!(frame_for(&f, &ayz(), false, 0), Frame::Idle(_)));
    }

    #[test]
    fn light_attacks_alternate_arms_and_the_crouching_one_has_its_own_drawings() {
        let (mut f, d) = attacking("Jab", 2);
        assert_eq!(frame_for(&f, &d, false, 0), Frame::Punch(0), "right arm winding up");
        let (g, d2) = attacking("Jab", 6);
        assert_eq!(frame_for(&g, &d2, false, 0), Frame::Punch(1), "right arm striking");
        f.punch_side = 1;
        assert_eq!(frame_for(&f, &d, false, 0), Frame::Punch(2), "left arm winding up");
        f.state_frame = 6;
        assert_eq!(frame_for(&f, &d, false, 0), Frame::Punch(3), "left arm striking");
        let (c, d3) = attacking("Low Jab", 2);
        assert_eq!(frame_for(&c, &d3, false, 0), Frame::CrouchSoft(0));
    }

    #[test]
    fn the_lecture_walks_through_four_drawings() {
        let seen: Vec<_> = [1, 6, 15, 28, 40]
            .into_iter()
            .map(|frame| {
                let (f, d) = attacking("Lecture", frame);
                frame_for(&f, &d, false, 0)
            })
            .collect();
        assert_eq!(seen, [Frame::Lecture(0), Frame::Lecture(0), Frame::Lecture(1), Frame::Lecture(2), Frame::Lecture(3)]);
    }

    fn junkhead() -> CharacterData {
        CharacterData::from_ron(include_str!("../assets/characters/junkhead.ron")).unwrap()
    }

    #[test]
    fn every_whip_layer_the_game_can_ask_for_exists_in_the_list_and_on_disk() {
        let j = junkhead();
        let stems = whip_stems();
        let mut used = std::collections::BTreeSet::new();
        for (index, mv) in j.moves.iter().enumerate() {
            for boosted in [false, true] {
                for aim_low in [false, true] {
                    for frame in 1..=mv.total_frames() + 2 {
                        let mut f = Fighter::new(0, 1);
                        f.state = FighterState::Attack;
                        f.attack = index;
                        f.state_frame = frame;
                        f.boosted = boosted;
                        f.aim_low = aim_low;
                        if let Some(name) = whip_layer(&f, &j) {
                            used.insert(name);
                        }
                    }
                }
            }
        }
        assert!(!used.is_empty());
        for name in &used {
            assert!(stems.contains(name), "{name} is shown but is not in whip_stems()");
        }
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/sprites/junkhead");
        for name in &stems {
            assert!(folder.join(format!("{name}.png")).exists(), "{name}.png is missing: run gen_ayz --junkhead-frames");
        }
    }

    #[test]
    fn the_whip_follows_the_moves_phases() {
        let j = junkhead();
        let layer = |name: &str, frame: u32, boosted: bool| {
            let mut f = Fighter::new(0, 1);
            f.state = FighterState::Attack;
            f.attack = j.moves.iter().position(|m| m.name == name).unwrap();
            f.state_frame = frame;
            f.boosted = boosted;
            whip_layer(&f, &j)
        };
        assert_eq!(layer("Whip", 2, false).as_deref(), Some("punch_0_whip"), "cocked back");
        assert_eq!(layer("Whip", 6, false).as_deref(), Some("punch_1_whip"), "out straight");
        assert_eq!(layer("Whip", 6, true).as_deref(), Some("punch_1_whipx"), "longer after a finisher");
        assert_eq!(layer("Whip", 12, true).as_deref(), Some("punch_1_whip"), "back to normal in recovery");
        assert_eq!(layer("Jump Kick", 8, false), None, "the kick has no whip");
        assert!(layer("Whip Storm", 20, false).unwrap().starts_with("whip_ring_"), "spinning");
        assert_eq!(layer("Whip Storm", 38, false).as_deref(), Some("summon_4_whip_hi"));
        assert_eq!(layer("Air Whip Storm", 20, false).map(|s| s.starts_with("whip_ring_")), Some(true), "the air special spins too");
        assert!(layer("Whip Spin", 7, false).unwrap().starts_with("air_soft_1_whip"));
    }

    fn professor() -> CharacterData {
        CharacterData::from_ron(include_str!("../assets/characters/ayz.ron")).unwrap()
    }

    #[test]
    fn the_raven_has_a_drawing_for_each_thing_it_does() {
        let p = professor();
        let storm = p.moves.iter().position(|m| m.name == "Raven Storm").unwrap();
        let claw = p.moves.iter().position(|m| m.name == "Raven Claw").unwrap();
        let mut r = Raven::idle();
        assert_eq!(raven_frame(&r, &p, 0), None, "perched: it is part of the body drawing");
        r.active = true;
        r.move_index = claw;
        assert_eq!(raven_frame(&r, &p, 0).as_deref(), Some("raven_claw_0"), "flying in to claw");
        r.step = 1;
        assert_eq!(raven_frame(&r, &p, 4).as_deref(), Some("raven_fly_1"), "flying home");
        r.move_index = storm;
        r.step = 1;
        r.timer = 1;
        assert_eq!(raven_frame(&r, &p, 0).as_deref(), Some("raven_peck_0"), "cocked");
        r.timer = 6;
        assert_eq!(raven_frame(&r, &p, 0).as_deref(), Some("raven_peck_1"), "the strike");
        r.step = 2;
        assert_eq!(raven_frame(&r, &p, 0).as_deref(), Some("raven_fly_0"), "between pecks it flies");
        for name in raven_stems() {
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn the_professors_moves_have_their_drawings() {
        assert!(frames_for(&professor()).contains(&Frame::Grab(5)) && frames_for(&professor()).contains(&Frame::Point(2)));
        let p = professor();
        let at = |name: &str, frame: u32| {
            let mut f = Fighter::new(0, 1);
            f.state = FighterState::Attack;
            f.attack = p.moves.iter().position(|m| m.name == name).unwrap();
            f.state_frame = frame;
            frame_for(&f, &p, false, 0)
        };
        assert_eq!(at("Uh", 3), Frame::Speak(0), "breath");
        assert_eq!(at("Uh", 12), Frame::Speak(1), "the word comes out");
        assert_eq!(at("Here's the Thing", 5), Frame::Grab(0), "beckoning");
        assert_eq!(at("Here's the Thing", 18), Frame::Grab(1), "pulling him in");
        assert!(matches!(at("Here's the Thing", 40), Frame::Grab(2 | 3)), "holding him by the collar");
        assert_eq!(at("Here's the Thing", 104), Frame::Grab(4), "the shove");
        assert_eq!(at("Here's the Thing", 125), Frame::Grab(5), "recovering");
        assert_eq!(at("Raven Claw", 2), Frame::Point(0), "finger up");
        assert_eq!(at("Raven Claw", 9), Frame::Point(1), "thrust at the enemy");
        assert!(matches!(at("Raven Storm", 10), Frame::Summon(_)));
    }

    #[test]
    fn guarding_uses_the_block_drawing() {
        let mut f = Fighter::new(0, 1);
        f.state = FighterState::Guard;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::Block);
    }

    #[test]
    fn summon_uses_all_six_drawings_in_order() {
        let mut last = 0;
        let mut seen = std::collections::BTreeSet::new();
        for frame in 1..=95 {
            let (f, d) = attacking("Crescent Call", frame);
            let Frame::Summon(i) = frame_for(&f, &d, false, 0) else { panic!("not a summon frame") };
            assert!(i >= last, "drawings went backwards at frame {frame}");
            last = i;
            seen.insert(i);
        }
        assert_eq!(seen.len(), SUMMON_FRAMES);
    }

    #[test]
    fn crouching_attacks_use_the_crouching_drawings() {
        let (f, d) = attacking("Low Jab", 2);
        assert_eq!(frame_for(&f, &d, false, 0), Frame::CrouchSoft(0));
        let (f, d) = attacking("Uppercut", 12);
        assert_eq!(frame_for(&f, &d, false, 0), Frame::CrouchHard(1));
    }

    #[test]
    fn air_attacks_use_the_air_drawings() {
        let (f, d) = attacking("Air Kick", 2);
        assert_eq!(frame_for(&f, &d, false, 0), Frame::AirSoft(0));
        let (f, d) = attacking("Air Slam", 12);
        assert_eq!(frame_for(&f, &d, false, 0), Frame::AirHard(1));
    }

    #[test]
    fn a_jump_goes_up_hangs_and_falls() {
        let mut f = Fighter::new(0, 1);
        f.state = FighterState::Jump;
        f.y = 500;
        f.vy = 800;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::JumpUp);
        f.vy = 0;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::JumpTop);
        f.vy = -800;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::JumpFall);
    }

    #[test]
    fn hit_block_crouch_and_ko_each_have_their_own_drawing() {
        let mut f = Fighter::new(0, 1);
        f.state = FighterState::Hitstun;
        f.state_frame = 2;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::Hurt(0));
        f.state_frame = 12;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::Hurt(1));
        f.state = FighterState::Blockstun;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::Block);
        f.state = FighterState::Crouch;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::Crouch);
        f.state = FighterState::Ko;
        assert_eq!(frame_for(&f, &ayz(), false, 0), Frame::Ko);
    }

    #[test]
    fn walking_cycles_through_all_six_drawings() {
        let mut f = Fighter::new(0, 1);
        f.state = FighterState::Walk;
        f.vx = 200;
        let mut seen = std::collections::BTreeSet::new();
        for frame in 0..60 {
            f.state_frame = frame;
            if let Frame::Walk(i) = frame_for(&f, &ayz(), false, 0) {
                seen.insert(i);
            }
        }
        assert_eq!(seen.len(), WALK_FRAMES);
    }

    #[test]
    fn speaking_swaps_the_idle_drawings_for_the_speaking_ones() {
        let f = Fighter::new(0, 1);
        assert_eq!(frame_for(&f, &ayz(), true, 0), Frame::Speak(0));
        assert_eq!(frame_for(&f, &ayz(), true, 10), Frame::Speak(1));
    }

    #[test]
    fn frame_names_are_unique() {
        let mut names: Vec<String> = Frame::all().into_iter().map(|f| f.file_stem()).collect();
        let count = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), count);
        assert_eq!(count, 2 + 6 + 5 + 3 + 4 + 2 + 2 + 2 + 4 + 6 + 2);
    }

    fn cowboy() -> CharacterData {
        CharacterData::from_ron(include_str!("../assets/characters/cowboy.ron")).unwrap()
    }

    #[test]
    fn the_air_shot_leans_back_in_three_separate_drawings_and_straightens_up() {
        let c = cowboy();
        let shot = c.moves.iter().position(|m| m.name == "Air Shot").unwrap();
        let mut f = Fighter::new(0, 1);
        f.state = FighterState::Attack;
        f.attack = shot;
        f.y = 40 * 100;
        let at = |f: &mut Fighter, frame: u32| {
            f.state_frame = frame;
            frame_for(f, &c, false, 0)
        };
        let m = &c.moves[shot];
        assert_eq!(at(&mut f, 1), Frame::Lean(0));
        assert_eq!(at(&mut f, m.startup - 3), Frame::Lean(1));
        assert_eq!(at(&mut f, m.startup + 1), Frame::Lean(2));
        assert_eq!(at(&mut f, m.startup + m.active + 8), Frame::Lean(1), "coming back up");
        assert_eq!(at(&mut f, m.total_frames()), Frame::Lean(0));
        let needed = frames_for(&c);
        assert!((0..LEAN_FRAMES).all(|i| needed.contains(&Frame::Lean(i))), "his folder needs the three leaning drawings");
    }

    #[test]
    fn the_gun_boost_is_drawn_guns_down_then_firing_then_launched_then_shooting_down() {
        let c = cowboy();
        let boost = c.moves.iter().position(|m| m.name == "Gun Boost").unwrap();
        let m = &c.moves[boost];
        let mut f = Fighter::new(0, 1);
        f.state = FighterState::Attack;
        f.attack = boost;
        let at = |f: &mut Fighter, frame: u32| {
            f.state_frame = frame;
            frame_for(f, &c, false, 0)
        };
        assert_eq!(at(&mut f, 1), Frame::Boost(0));
        assert_eq!(at(&mut f, m.startup), Frame::Boost(1), "the guns go off at the ground");
        assert_eq!(at(&mut f, m.startup + m.active + 1), Frame::Boost(2), "launched up");
        assert_eq!(at(&mut f, m.startup + m.volley_delay + 1), Frame::Boost(3), "the volley");
        let needed = frames_for(&c);
        assert!((0..BOOST_FRAMES).all(|i| needed.contains(&Frame::Boost(i))));
    }
}
