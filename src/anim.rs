//! Frame-by-frame animation. Each fighter is one hand-drawn whole-body sprite per frame: the
//! torso leans, bobs and swings its arms together with the legs, so movement reads as one body.
//!
//! This module is pure (no Bevy) so the generator and the game agree on the sizes and file
//! names, and so the frame choice can be tested.

use crate::sim::{AnimKind, CharacterData, Fighter, FighterState};

/// Every drawing is this wide and this tall. The fighter's body centre is at column `CENTRE_X`,
/// which leaves room on the right for outstretched arms and on the left for the long hair. The
/// feet are on the bottom row.
pub const FRAME_W: i32 = 84;
pub const FULL_H: i32 = 63;
pub const CENTRE_X: i32 = 33;
/// Height of the torso part of a drawing while generating (head to hips).
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
    /// The alternating punches: right arm (wind-up, strike), then left arm (wind-up, strike).
    Punch(usize),
    /// The leap and its kicks: the crouch, tucked in the air, a kick with one leg, a kick with the other.
    Kick(usize),
    /// The Professor's lecture: breath, the finger goes up, the lecture, the finger comes down.
    Lecture(usize),
    /// The finger-gun volley: ready, then a shot from the left hand, the right, the left, then recovery.
    Gun(usize),
}

impl Frame {
    /// File name (without extension) in `assets/sprites/<character>/`.
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

/// The drawings of the alternating punches and of the kicks.
pub const PUNCH_FRAMES: usize = 4;
pub const KICK_FRAMES: usize = 4;

/// The drawings of the lecture.
pub const LECTURE_FRAMES: usize = 4;

/// The drawings of the finger-gun volley, for characters that have one.
pub const GUN_FRAMES: usize = 5;

/// Every drawing this character needs: the common set, plus the volley's if it has a volley move.
pub fn frames_for(data: &CharacterData) -> Vec<Frame> {
    let mut v = Frame::all();
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
    v
}

/// Frames of the effect drawings (the summoned crescent, and the shooting star).
pub const CRESCENT_FRAMES: usize = 2;
pub const STAR_FRAMES: usize = 2;

/// Ticks each walking drawing is held; six drawings make one stride cycle.
const WALK_TICKS: u32 = 5;

/// The drawing for a fighter. `speaking` swaps the idle drawings for the speaking ones, and
/// `tick` drives the idle breathing and the mouth.
pub fn frame_for(f: &Fighter, data: &CharacterData, speaking: bool, tick: u32) -> Frame {
    match f.state {
        FighterState::Ko => return Frame::Ko,
        FighterState::Hitstun => return Frame::Hurt(if f.state_frame < 8 { 0 } else { 1 }),
        FighterState::Blockstun | FighterState::Guard => return Frame::Block,
        FighterState::Attack => return attack_frame(f, data),
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

/// Width of a whip layer; its left edge lines up with the body drawing's (see `render`).
pub const WHIP_W: i32 = 176;

/// Every whip layer a character with `whip` can show (file names without the extension).
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

/// Where Junkhead's whip hand is in the poses where the whip lashes out (pixels of the drawing,
/// from its left edge and from the top). The lash itself is not an image: the game draws it from
/// here to the enemy. `layer` is the name `whip_layer` gives.
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

/// The whip layer to show over the body now, if any. The layer is named after the body drawing
/// (`punch_1` -> `punch_1_whip`): cocked back while winding up, out straight on the strike (the
/// `x` one is the longer whip after a combo finisher), and slack in recovery. The spin overhead
/// is a ring that turns a quarter every 3 frames, the lash goes `_hi` or `_lo`, and the air spin
/// circles him (the `b` drawing every other 3 frames).
pub fn whip_layer(f: &Fighter, data: &CharacterData) -> Option<String> {
    if !data.whip || f.state != FighterState::Attack {
        return None;
    }
    let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
    if mv.anim == AnimKind::Kicks {
        return None;
    }
    let stem = attack_frame(f, data).file_stem();
    let frame = f.state_frame;
    let last_active = mv.startup + (mv.hits.max(1) - 1) * mv.hit_gap + mv.active;
    match mv.anim {
        AnimKind::Summon => {
            if frame <= mv.startup {
                // The raised arm comes up during the first quarter; after that the whip spins.
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

/// The move's own drawings, played along its startup, active and recovery.
fn attack_frame(f: &Fighter, data: &CharacterData) -> Frame {
    let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
    let frame = f.state_frame;
    let startup = mv.startup.max(1);
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
                // The legs alternate: right, left, right.
                Frame::Kick(2 + (k % 2) as usize)
            } else {
                Frame::Kick(1)
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
                // Hands alternate: the first shot is the left hand, the second the right, ...
                Frame::Gun(f.shots.min(3) as usize)
            }
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
        CharacterData::from_ron(include_str!("../assets/characters/ayz.ron")).unwrap()
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
}
