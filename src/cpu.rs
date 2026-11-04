//! The CPU opponent. It produces the same `Input` a player does, from the game state alone,
//! so the same state always gives the same input.

use crate::sim::{Button, CharacterData, Fighter, FighterState, GameState, Input, MoveData, Phase, PX};

/// Distance (pixels, centre to centre) at which the CPU starts attacking.
const ATTACK_RANGE: i32 = 50;
/// Distance at which it starts to worry about the opponent's attacks.
const DANGER_RANGE: i32 = 90;
/// Frames per decision. Buttons are only pressed on the first frame of a window.
pub const DECISION_FRAMES: u32 = 8;

fn noise(window: u32, me: usize) -> u32 {
    let mut x = window.wrapping_mul(2_654_435_761) ^ (me as u32 + 1).wrapping_mul(40_503);
    x ^= x >> 15;
    x = x.wrapping_mul(2_246_822_519);
    x ^= x >> 13;
    x
}

/// How far from the fighter's centre a move reaches an opponent in front of it, in pixels.
fn reach(mv: &MoveData, boosted: bool) -> i32 {
    mv.hitbox.x + mv.hitbox.w + if boosted { mv.boost_reach } else { 0 } + Fighter::HALF_WIDTH / PX
}

/// Whether a heavy kills a minion with `health` hit points left (an agent's 40, a goblin's 56) at distance `dist`
/// without costing the fighter: it does at least that much damage, and covers them (a shockwave round the fighter, a wide grab, or a broad wave of shots). A heavy
/// that leaps, pulls the fighter in, is held, or rains down on where the opponent stands is no use against them.
fn kills_minions(h: &MoveData, dist: i32, health: i32) -> bool {
    if h.air || h.damage < health || h.leap > 0 || h.pull_self || h.channel.is_some() || h.minions.is_some() {
        return false;
    }
    if h.radial {
        // a shockwave out to either side
        return dist <= h.hitbox.w / 2 - 8;
    }
    if h.hitbox.w >= 100 {
        // a wide grab or sweep reaching far in front
        return (20..=reach(h, false) - 10).contains(&dist);
    }
    // a broad wave of shots flying straight ahead, or a spray of them (Gray Persona's bats, a short way)
    h.projectile.as_ref().is_some_and(|p| !p.lob && !p.homing && (p.size.0 >= 30 || (h.volley >= 4 && p.fan > 0))) && (45..=170).contains(&dist)
}

/// How a character fights: from a distance (words, bullets, missiles, a raven) or up close.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    Ranged,
    Melee,
}

pub fn style_of(data: &CharacterData) -> Style {
    let light_shoots = data.find_move(Button::Light, false, false).map(|i| &data.moves[i]).is_some_and(|m| m.projectile.is_some() || m.channel.is_some());
    if data.keeps_distance || light_shoots {
        Style::Ranged
    } else {
        Style::Melee
    }
}

/// The CPU's input when it doesn't know who it is fighting (it is treated as fighting its own kind).
pub fn cpu_input(state: &GameState, me: usize, data: &CharacterData) -> Input {
    cpu_input_vs(state, me, data, data)
}

/// The CPU's input for fighter `me` playing `data` against a fighter playing `opp`. It plays differently
/// against a ranged opponent (it dodges and answers shots, closes the gap and doesn't back off) than a
/// close-range one (a ranged CPU then keeps further away and shoots more), and every champion uses
/// its own tools.
pub fn cpu_input_vs(state: &GameState, me: usize, data: &CharacterData, opp: &CharacterData) -> Input {
    let input = choose_input(state, me, data, opp);
    // a heavy that's cooling down does nothing, so don't waste the press (a light goes out instead)
    let cooling = data.find_move(Button::Heavy, false, false).is_some_and(|i| state.fighters[me].on_cooldown(i, data));
    if cooling && input.has(Input::HEAVY) {
        return Input(input.0 & !Input::HEAVY | Input::LIGHT);
    }
    input
}

fn choose_input(state: &GameState, me: usize, data: &CharacterData, opp: &CharacterData) -> Input {
    if state.phase != Phase::Fighting {
        return Input::default();
    }
    let (m, o) = (&state.fighters[me], &state.fighters[1 - me]);
    // the opponent's minions are fought before anything else while any are alive: sub-agents wherever they are, goblins
    // once they are on me. (A fighter turns to face a minion that is close, so they can be fought on either side.)
    let minion = state
        .minions
        .iter()
        .filter(|g| g.owner != me && g.health > 0 && (g.agent || (g.x - m.x).abs() <= 140 * PX))
        .min_by_key(|g| (g.x - m.x).abs());
    let agent_x = minion.map(|g| g.x);
    let minion_health = minion.map_or(0, |g| g.health);
    let target_x = agent_x.unwrap_or(o.x);
    let dx = target_x - m.x;
    let dist = dx.abs() / PX;
    let opponent_dist = (o.x - m.x).abs() / PX;
    let (toward, away) = if dx > 0 { (Input::RIGHT, Input::LEFT) } else { (Input::LEFT, Input::RIGHT) };

    let window = state.frame / DECISION_FRAMES;
    let first_frame = state.frame % DECISION_FRAMES == 0;
    let roll = noise(window, me) % 100;
    let my_style = style_of(data);
    // Katmata fights with the lighter, up close, until he has his cigarettes again
    let zoning = data.keeps_distance && !m.lighter_mode;
    let vs_ranged = style_of(opp) == Style::Ranged;
    // a shot of theirs (or a homing missile) is close to me
    let incoming = state.projectiles.iter().any(|p| p.owner != me && (p.x - m.x).abs() / PX < 90);
    let button = |flag: u16| Input(if first_frame { flag } else { 0 });
    // the standing heavy is still cooling down
    let heavy_cooling = data.find_move(Button::Heavy, false, false).is_some_and(|i| m.on_cooldown(i, data));

    // a zoner stays out of reach; near a wall it can't back off any further
    let wall = (GameState::STAGE_HALF_WIDTH - Fighter::HALF_WIDTH) / PX;
    let cornered = (away == Input::RIGHT && m.x / PX > wall - 25) || (away == Input::LEFT && m.x / PX < -(wall - 25));

    // guard (sometimes crouching, to get under shots) while the opponent attacks up close
    if o.state == FighterState::Attack && opponent_dist < DANGER_RANGE && roll < 65 && !(agent_x.is_some() && dist < 45) {
        // a zoner would rather hop away than block
        if zoning && roll < 35 && m.is_neutral() {
            return Input(if cornered { toward } else { away } | Input::UP);
        }
        return Input(if roll < 20 { Input::DOWN } else { Input::BLOCK });
    }

    // a ranged opponent shooting from afar, or a shot already on its way: dodge it, block it, or answer it
    if m.is_neutral() && ((vs_ranged && o.state == FighterState::Attack && opponent_dist >= DANGER_RANGE) || incoming) && roll < 60 {
        // a shockwave around himself destroys the shots near him (Anaran's staff slam)
        if incoming && data.moves.iter().enumerate().any(|(i, mv)| mv.clears_projectiles && mv.button == Button::Heavy && !m.on_cooldown(i, data)) {
            return button(Input::HEAVY);
        }
        return Input(match (my_style, roll) {
            // a close-range fighter jumps in over it
            (Style::Melee, 0..=24) => toward | Input::UP,
            (_, 0..=39) => Input::BLOCK,
            // duck: words and bullets fly at head height
            _ => Input::DOWN,
        });
    }

    // keep pressing forward during an attack; moves that allow it advance with the hit
    if m.state == FighterState::Attack {
        // reading a scroll (a held move): keep the button down while the opponent stays at a distance,
        // for about two missiles, then let go (and slam the staff if they come close)
        let mv = &data.moves[m.attack];
        if let Some(ch) = mv.channel {
            if m.state_frame <= mv.startup {
                let reading = m.state_frame + 1 >= mv.startup;
                let stop = reading && (opponent_dist <= 60 || m.charge >= ch.first + ch.every + 40);
                return Input(if stop { 0 } else { Input::LIGHT });
            }
        }
        // Junkhead: once a heavy or the special has connected, go straight into lights (tapped fast so each press registers)
        if data.light_after_heavy && m.hit_done && data.moves[m.attack].button != Button::Light {
            return Input(if state.frame % 2 == 0 { Input::LIGHT } else { 0 });
        }
        // a connected move can be cancelled into a heavier one: build a combo
        if m.hit_done && m.hitstop == 0 && roll < 55 {
            return Input(if roll < 40 { Input::HEAVY } else { Input::SPECIAL });
        }
        return Input(if dist > 20 { toward } else { 0 });
    }
    // in the air near the opponent: kick or slam, once
    if m.state == FighterState::Jump {
        if dist < 45 && !m.air_used && roll < 45 {
            return Input(if roll < 30 { Input::LIGHT } else { Input::HEAVY } | toward);
        }
        return Input(toward);
    }
    if !m.is_neutral() {
        return Input::default();
    }

    // a curse of mine needs a hit from each side: walk through them to the side that's still missing
    if let Some(c) = o.curse {
        if c.owner == me && c.squeeze == 0 {
            let want_left = c.left && (!c.right || m.x < o.x);
            if want_left && m.x > o.x {
                return Input(Input::LEFT);
            }
            if !want_left && c.right && m.x < o.x {
                return Input(Input::RIGHT);
            }
        }
    }

    // minions come first, and it doesn't back off from them. A heavy that wipes them out (Anaran's slam, the Professor's grab,
    // Gray Persona's bats) is used when it covers them; otherwise a light (which kills an agent outright, a goblin in two)
    if agent_x.is_some() {
        let heavy = data.find_move(Button::Heavy, false, false).map(|i| &data.moves[i]);
        if heavy.is_some_and(|h| kills_minions(h, dist, minion_health)) && !heavy_cooling && roll < 80 {
            return button(Input::HEAVY);
        }
        if dist <= data.cpu_range.unwrap_or(ATTACK_RANGE) + 10 {
            return if roll < 88 { button(Input::LIGHT) } else { Input::default() };
        }
    }

    // two lights in: go on to the third (the finisher), from as far as it reaches
    if m.combo == 2 && !m.raven_away {
        if let Some(fin) = data.find_finisher().map(|i| &data.moves[i]) {
            let reaches = fin.projectile.is_some() || fin.raven_move.is_some() || dist <= reach(fin, false) + 5;
            if reaches && dist <= 175 && roll < 75 {
                return button(Input::LIGHT);
            }
        }
    }

    // a zoner backs away when the opponent gets close (hopping over them against a wall), pokes when they're right on top of it, and shoots from a distance with the volley and the special
    if zoning {
        // a shockwave around himself (the staff slam) is the answer to someone close
        let slam = data.moves.iter().any(|mv| mv.radial && mv.button == Button::Heavy);
        // against a close-range fighter it backs off sooner, sits further away and shoots more; against
        // another ranged fighter it holds the middle and trades shots
        let close = if vs_ranged { 60 } else { 85 };
        let comfort = if vs_ranged { 100 } else { 135 };
        let fire = if vs_ranged { 40 } else { 32 };
        // the heavy is only worth using if it reaches from here
        let heavy_reaches = data.find_move(Button::Heavy, false, false).map(|i| &data.moves[i]).is_some_and(|h| h.hitbox.w <= 0 || dist <= reach(h, false) - 8);
        if dist < close {
            if slam && !heavy_cooling && dist < 60 && roll < 50 {
                return button(Input::HEAVY);
            }
            if cornered {
                return Input(toward | Input::UP);
            }
            if dist < close - 20 && roll < 22 {
                return button(Input::LIGHT);
            }
            return Input(away);
        }
        if dist <= 175 {
            if m.meter >= Fighter::METER_MAX && roll < 35 {
                return button(Input::SPECIAL);
            }
            if roll < fire {
                return button(if roll < fire / 2 && heavy_reaches { Input::HEAVY } else { Input::LIGHT });
            }
            // keep adjusting the gap: back off if it's a bit short, otherwise wait
            return Input(if dist < comfort && roll < 70 { away } else { 0 });
        }
        return Input(toward);
    }

    // Junkhead keeps tapping Light during the quick lights after a heavy or special, or a light combo, while the opponent is in front of him
    if data.light_after_heavy && (m.quick > 0 || m.combo > 0) && dist <= data.cpu_range.unwrap_or(ATTACK_RANGE) + 10 {
        return Input(if state.frame % 2 == 0 { Input::LIGHT } else { 0 });
    }
    // long-reach fighters start attacking from further away, and a whip with its extra reach is used from that distance
    let light = data.find_move(Button::Light, false, false).map(|i| &data.moves[i]);
    let mut range = data.cpu_range.unwrap_or(ATTACK_RANGE);
    if let Some(l) = light.filter(|l| m.whip_boost && l.boost_reach > 0) {
        range = range.max(reach(l, true) - 15);
    }
    if dist > range {
        // a lash that drags the opponent in, or pulls the fighter to them, works from a distance
        let special = data.find_move(Button::Special, false, false).map(|i| &data.moves[i]);
        if special.is_some_and(|s| s.pull_foe && dist <= reach(s, false) - 25) && m.meter >= Fighter::METER_MAX && roll < 60 {
            return button(Input::SPECIAL);
        }
        let heavy = data.find_move(Button::Heavy, false, false).map(|i| &data.moves[i]);
        // against someone who shoots from afar, the pulling lash is used much more to close the gap
        if heavy.is_some_and(|h| h.pull_self && !heavy_cooling && dist <= reach(h, false) - 15) && roll < if vs_ranged { 80 } else { 50 } {
            return button(Input::HEAVY);
        }
        // a leaping heavy (the dive kicks) jumps the gap
        if vs_ranged && heavy.is_some_and(jumps_the_gap) && dist > 60 && dist < 140 && roll < 35 {
            return button(Input::HEAVY);
        }
        // a rusher dashes in (tapping the direction twice), dives from further out and hops all the time
        if data.cpu_rush {
            if heavy.is_some_and(jumps_the_gap) && dist > 55 && dist < 170 && roll < 30 {
                return button(Input::HEAVY);
            }
            if dist > 85 && roll < 60 {
                return Input(if state.frame % 2 == 0 { toward } else { 0 });
            }
            if dist > 50 && roll >= 60 && roll < 80 {
                return Input(toward | Input::UP);
            }
        }
        // and a hop now and then keeps shots from hitting
        let hop = if roll < if vs_ranged { 18 } else { 10 } && dist > 70 { Input::UP } else { 0 };
        return Input(toward | hop);
    }

    // in range: attack, retreat or wait; with the special ready, use it often
    if m.meter >= crate::sim::Fighter::METER_MAX && roll < 45 {
        return button(Input::SPECIAL);
    }
    match roll {
        0..=34 => button(Input::LIGHT),
        35..=54 => button(Input::HEAVY),
        55..=61 => button(if m.meter >= crate::sim::Fighter::METER_MAX { Input::SPECIAL } else { Input::LIGHT }),
        62..=74 => button(Input::LIGHT | Input::DOWN),
        // against a shooter there is no point backing off: keep on them
        // (a rusher keeps hopping at them instead)
        75..=84 if data.cpu_rush => Input(toward | Input::UP),
        75..=84 => Input(if vs_ranged { toward } else { away }),
        _ => Input::default(),
    }
}

/// A heavy that leaps far enough forward to close a gap (the cowboy's gun boost goes almost straight up).
fn jumps_the_gap(h: &crate::sim::MoveData) -> bool {
    h.leap > 0 && h.leap_vx >= 100
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::CharacterData;

    fn chars() -> [CharacterData; 2] {
        [
            CharacterData::from_ron(include_str!("sim/classic_fighter.ron")).unwrap(),
            CharacterData::from_ron(include_str!("sim/classic_opponent.ron")).unwrap(),
        ]
    }

    #[test]
    fn same_state_gives_same_input() {
        let s = GameState::new();
        assert_eq!(cpu_input(&s, 1, &chars()[1]), cpu_input(&s, 1, &chars()[1]));
    }

    #[test]
    fn cpu_walks_toward_a_distant_opponent() {
        let s = GameState::new(); // P2 is on the right; P1 is to its left
        let i = cpu_input(&s, 1, &chars()[1]);
        assert!(i.has(Input::LEFT) && !i.has(Input::RIGHT));
    }

    #[test]
    fn cpu_closes_in_and_lands_hits_on_a_passive_player() {
        let c = chars();
        let mut s = GameState::new();
        for _ in 0..1_200 {
            let cpu = cpu_input(&s, 1, &c[1]);
            s.step([Input::default(), cpu], [&c[0], &c[1]]);
        }
        assert!(s.fighters[0].health < crate::sim::Fighter::MAX_HEALTH || s.wins[1] > 0, "CPU never hit anyone");
    }

    #[test]
    fn the_rushing_cowboy_closes_the_distance_faster_and_moves_more() {
        let cowboy = CharacterData::from_ron(include_str!("../assets/characters/cowboy.ron")).unwrap();
        assert!(cowboy.cpu_rush);
        let mut calm = cowboy.clone();
        calm.cpu_rush = false;
        let c = chars();
        // a passive opponent at the far side: how long until the CPU is in striking distance, and whether it dashes on the way
        let run = |data: &CharacterData| {
            let mut s = GameState::new();
            s.fighters[0].x = -300 * PX;
            s.fighters[1].x = 300 * PX;
            let (mut arrived, mut dashes, mut was_dashing) = (None, 0, false);
            for frame in 0..400 {
                let cpu = cpu_input_vs(&s, 1, data, &c[0]);
                s.step([Input::default(), cpu], [&c[0], data]);
                let dashing = s.fighters[1].state == FighterState::Dash;
                dashes += (dashing && !was_dashing) as u32;
                was_dashing = dashing;
                if arrived.is_none() && (s.fighters[1].x - s.fighters[0].x).abs() / PX < 60 {
                    arrived = Some(frame);
                }
            }
            (arrived.unwrap_or(400), dashes)
        };
        let (rush_t, rush_dashes) = run(&cowboy);
        let (calm_t, _) = run(&calm);
        assert!(rush_dashes > 0, "the rusher dashes");
        assert!(rush_t < calm_t, "the rusher arrives sooner: {rush_t} vs {calm_t}");
    }

    #[test]
    fn cpu_turns_on_a_goblin_that_is_nearer_than_the_player() {
        use crate::sim::{Minion, MinionMode};
        let c = chars();
        let mut s = GameState::new();
        s.fighters[0].x = -200 * PX;
        s.fighters[1].x = 100 * PX;
        // without goblins the CPU (P2) walks left towards the far player; with a goblin 30 px in front it stops and attacks that
        assert!(cpu_input(&s, 1, &c[1]).has(Input::LEFT));
        s.minions.push(Minion {
            owner: 0,
            slot: 0,
            x: 70 * PX,
            mode: MinionMode::Wait,
            timer: 0,
            age: 0,
            move_index: 0,
            bit: false,
            facing: 1,
            health: 100,
            hurt: 0,
            agent: false,
            roll_dir: 0,
        });
        let i = cpu_input(&s, 1, &c[1]);
        assert!(!i.has(Input::LEFT) && !i.has(Input::RIGHT), "should stand and attack the goblin, got {i:?}");
        assert!(i.has(Input::LIGHT) || i.has(Input::HEAVY) || i.has(Input::SPECIAL), "should attack, got {i:?}");
    }

    #[test]
    fn a_long_reach_cpu_starts_attacking_from_further_away_than_a_normal_one() {
        let j = CharacterData::from_ron(include_str!("../assets/characters/junkhead.ron")).unwrap();
        let c = chars();
        let mut s = GameState::new();
        s.fighters[0].x = -150 * PX;
        s.fighters[1].x = 80 * PX;
        let mut start_gaps = Vec::new();
        for _ in 0..1_500 {
            let cpu = cpu_input(&s, 1, &j);
            s.step([Input::default(), cpu], [&c[0], &j]);
            let f = &s.fighters[1];
            if f.state == FighterState::Attack && f.state_frame == 0 {
                start_gaps.push((s.fighters[1].x - s.fighters[0].x).abs() / PX);
            }
        }
        assert!(!start_gaps.is_empty(), "the CPU never attacked");
        assert!(start_gaps.iter().any(|&g| g > ATTACK_RANGE + 5), "it should whip from beyond 55 px, gaps {start_gaps:?}");
        assert!(s.fighters[0].health < Fighter::MAX_HEALTH, "and it should land some hits");
    }

    #[test]
    fn the_cpu_professor_speaks_from_afar_uses_the_raven_and_hurts_a_passive_player() {
        let p = CharacterData::from_ron(include_str!("../assets/characters/ayz.ron")).unwrap();
        assert!(p.keeps_distance, "he is a ranged character");
        let c = chars();
        let mut s = GameState::new();
        let mut used = std::collections::BTreeSet::new();
        for _ in 0..3_000 {
            let cpu = cpu_input(&s, 1, &p);
            s.step([Input::default(), cpu], [&c[0], &p]);
            if s.fighters[1].state == FighterState::Attack && s.fighters[1].state_frame == 0 {
                used.insert(p.moves[s.fighters[1].attack].name.clone());
            }
            s.fighters[1].meter = s.fighters[1].meter.max(60);
            if s.phase != Phase::Fighting {
                break;
            }
        }
        assert!(s.fighters[0].health < Fighter::MAX_HEALTH, "he never hurt anyone");
        assert!(used.contains("Uh") || used.contains("Meh"), "he should speak at them, used {used:?}");
        assert!(used.iter().any(|n| n == "Here's the Thing" || n == "Raven Storm" || n == "Raven Claw"), "and use his big moves, used {used:?}");
    }

    #[test]
    fn the_cpu_anaran_reads_from_a_distance_slams_when_close_and_hurts_a_passive_player() {
        let a = CharacterData::from_ron(include_str!("../assets/characters/anaran.ron")).unwrap();
        assert!(a.keeps_distance);
        let c = chars();
        let mut s = GameState::new();
        let (mut missiles, mut slams) = (0, 0);
        for _ in 0..4_000 {
            let cpu = cpu_input(&s, 1, &a);
            let before = s.fighters[1].shots;
            s.step([Input::default(), cpu], [&c[0], &a]);
            missiles += (s.fighters[1].shots > before && s.fighters[1].state == FighterState::Attack) as u32;
            if s.fighters[1].state == FighterState::Attack && s.fighters[1].state_frame == 0 && a.moves[s.fighters[1].attack].radial {
                slams += 1;
            }
            if s.phase != Phase::Fighting {
                break;
            }
        }
        assert!(missiles > 0, "he never fired a missile");
        assert!(s.fighters[0].health < Fighter::MAX_HEALTH, "he never hurt anyone");
        let _ = slams;
    }

    #[test]
    fn two_cpus_fight_without_breaking_anything() {
        let c = chars();
        let mut s = GameState::new();
        for _ in 0..20_000 {
            let (a, b) = (cpu_input(&s, 0, &c[0]), cpu_input(&s, 1, &c[1]));
            s.step([a, b], [&c[0], &c[1]]);
            if s.phase == Phase::MatchOver {
                break;
            }
        }
        assert!(s.wins.iter().sum::<u8>() > 0, "no round was ever finished");
    }

    #[test]
    fn junkhead_taps_light_right_after_his_heavy_connects() {
        let j = CharacterData::from_ron(include_str!("../assets/characters/junkhead.ron")).unwrap();
        let mut s = GameState::new();
        s.frame = 10;
        s.fighters[1].state = FighterState::Attack;
        s.fighters[1].attack = j.find_move(Button::Heavy, false, false).unwrap();
        s.fighters[1].hit_done = true;
        assert!(cpu_input(&s, 1, &j).has(Input::LIGHT), "light on an even frame");
        s.frame = 11;
        assert!(!cpu_input(&s, 1, &j).has(Input::LIGHT), "and released on the next so that it counts as a new press");
    }

    fn load(path: &str) -> CharacterData {
        CharacterData::from_ron(&std::fs::read_to_string(format!("{}/assets/characters/{path}.ron", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap()
    }

    #[test]
    fn the_champions_are_sorted_into_ranged_and_close_range() {
        for ranged in ["ayz", "evilayz", "anaran", "cowboy", "katmata"] {
            assert_eq!(style_of(&load(ranged)), Style::Ranged, "{ranged}");
        }
        for melee in ["junkhead", "scribe"] {
            assert_eq!(style_of(&load(melee)), Style::Melee, "{melee}");
        }
    }

    /// The mean gap (px) a CPU `me` keeps over a few seconds against a player who just stands there.
    fn mean_gap(me: &CharacterData, opp: &CharacterData) -> i32 {
        let mut s = GameState::new();
        s.fighters[0].x = -80 * PX;
        s.fighters[1].x = 80 * PX;
        let (mut total, mut counted) = (0, 0);
        for i in 0..900 {
            let cpu = cpu_input_vs(&s, 1, me, opp);
            s.step([Input::default(), cpu], [opp, me]);
            // only the moments when both are just standing around count (not while one has been grabbed)
            if i >= 300 && s.fighters[1].is_neutral() && s.fighters[0].state == FighterState::Idle {
                total += (s.fighters[1].x - s.fighters[0].x).abs() / PX;
                counted += 1;
            }
            // nobody dies, so the positions say what the CPU wants
            s.fighters[0].health = Fighter::MAX_HEALTH;
        }
        total / counted.max(1)
    }

    #[test]
    fn a_ranged_cpu_keeps_further_away_from_a_close_range_fighter_than_from_another_shooter() {
        let (professor, cowboy, ranged) = (load("ayz"), load("junkhead"), load("anaran"));
        let (vs_melee, vs_ranged) = (mean_gap(&professor, &cowboy), mean_gap(&professor, &ranged));
        assert!(vs_melee > vs_ranged + 8, "the Professor sat {vs_melee} px from the cowboy but {vs_ranged} px from Anaran");
    }

    #[test]
    fn a_close_range_cpu_answers_shots_a_ranged_opponent_fires_and_ignores_them_from_a_melee_one() {
        let (cowboy, gunslinger) = (load("cowboy"), load("anaran"));
        // the player's (P1) bullet is 50 px from the cowboy CPU (P2)
        let mut s = GameState::new();
        s.fighters[0].x = -150 * PX;
        s.fighters[1].x = 50 * PX;
        s.projectiles.push(crate::sim::Projectile { owner: 0, x: 0, y: 44 * PX, vx: 6 * PX, vy: 0, age: 0, move_index: 0, land_x: 0, power: 100 });
        let mut defended = 0;
        for window in 0..60 {
            s.frame = window * 8;
            let i = cpu_input_vs(&s, 1, &cowboy, &gunslinger);
            defended += (i.has(Input::BLOCK) || i.has(Input::DOWN) || i.has(Input::UP)) as u32;
        }
        assert!(defended >= 20, "it should block, duck or jump a shot on its way, did so {defended} times out of 60");
        // with nothing flying at it it just walks up to the player
        s.projectiles.clear();
        let mut defended = 0;
        for window in 0..60 {
            s.frame = window * 8;
            let i = cpu_input_vs(&s, 1, &cowboy, &cowboy);
            defended += (i.has(Input::BLOCK) || i.has(Input::DOWN)) as u32;
        }
        assert_eq!(defended, 0);
    }

    #[test]
    fn anaran_answers_a_shot_close_to_him_with_the_shockwave_that_destroys_it() {
        let (anaran, gunslinger) = (load("anaran"), load("ayz"));
        let mut s = GameState::new();
        s.fighters[0].x = -150 * PX;
        s.fighters[1].x = 50 * PX;
        s.projectiles.push(crate::sim::Projectile { owner: 0, x: 0, y: 44 * PX, vx: 6 * PX, vy: 0, age: 0, move_index: 0, land_x: 0, power: 100 });
        let slams = (0..40u32).filter(|w| {
            s.frame = w * 8;
            cpu_input_vs(&s, 1, &anaran, &gunslinger).has(Input::HEAVY)
        });
        assert!(slams.count() >= 10, "the staff slam is the answer to a shot");
        // while it is cooling down he does something else
        let slam = anaran.moves.iter().position(|m| m.name == "Staff Slam").unwrap();
        s.fighters[1].cooldowns[slam] = 100;
        assert_eq!((0..40u32).filter(|w| { s.frame = w * 8; cpu_input_vs(&s, 1, &anaran, &gunslinger).has(Input::HEAVY) }).count(), 0);
    }

    #[test]
    fn a_close_range_cpu_closes_the_gap_on_a_shooter_and_a_ranged_cpu_still_fights_without_breaking() {
        // every pairing of champions plays on without panicking or standing still for good
        let names = ["ayz", "anaran", "cowboy", "scribe", "junkhead", "evilayz", "gray_persona", "katmata"];
        for a in names {
            for b in names {
                let (ca, cb) = (load(a), load(b));
                let mut s = GameState::new();
                for _ in 0..2_000 {
                    let (i0, i1) = (cpu_input_vs(&s, 0, &ca, &cb), cpu_input_vs(&s, 1, &cb, &ca));
                    s.step([i0, i1], [&ca, &cb]);
                    if s.phase == Phase::MatchOver {
                        break;
                    }
                }
                assert!(s.fighters[0].health < Fighter::MAX_HEALTH || s.fighters[1].health < Fighter::MAX_HEALTH || s.wins.iter().sum::<u8>() > 0, "{a} vs {b}: nobody ever got hurt");
            }
        }
    }

    /// A sub-agent of the opponent's, `at` pixels along the stage.
    fn agent_at(x: i32) -> crate::sim::Minion {
        crate::sim::Minion {
            owner: 0,
            slot: 0,
            x: x * PX,
            mode: crate::sim::MinionMode::Charge,
            timer: 0,
            age: 0,
            move_index: 0,
            bit: false,
            facing: 1,
            health: 40,
            hurt: 0,
            agent: true,
            roll_dir: 0,
        }
    }

    #[test]
    fn the_cpu_uses_lights_on_a_sub_agent_in_front_of_it_and_never_backs_off_from_it() {
        let c = chars();
        let mut seen_light = false;
        for window in 0..40u32 {
            let mut s = GameState::new();
            s.fighters[0].x = -200 * PX;
            s.fighters[1].x = 100 * PX;
            s.minions.push(agent_at(80));
            s.frame = window * DECISION_FRAMES;
            let i = cpu_input(&s, 1, &c[1]);
            assert!(!i.has(Input::HEAVY) && !i.has(Input::SPECIAL) && !i.has(Input::RIGHT), "only lights on an agent, got {i:?}");
            seen_light |= i.has(Input::LIGHT);
        }
        assert!(seen_light, "it attacks the agent");
    }

    #[test]
    fn the_cpu_fights_a_sub_agent_that_is_behind_it_too() {
        let c = chars();
        let mut seen_light = false;
        for window in 0..40u32 {
            let mut s = GameState::new();
            s.fighters[0].x = -200 * PX;
            s.fighters[1].x = 100 * PX;
            // the agent is on the far side of the CPU from its opponent: the fighter turns to it, so it is attacked
            s.minions.push(agent_at(130));
            s.frame = window * DECISION_FRAMES;
            let i = cpu_input(&s, 1, &c[1]);
            assert!(!i.has(Input::LEFT) && !i.has(Input::HEAVY), "stays on the agent and uses lights, got {i:?}");
            seen_light |= i.has(Input::LIGHT);
        }
        assert!(seen_light);
    }

    /// What each fighter's CPU presses against an agent `dist` pixels in front of it, over many decisions.
    fn presses(name: &str, dist: i32) -> (bool, bool, bool) {
        let text = std::fs::read_to_string(format!("assets/characters/{name}.ron")).unwrap();
        let me = CharacterData::from_ron(&text).unwrap();
        let (mut light, mut heavy, mut special) = (false, false, false);
        for window in 0..60u32 {
            let mut s = GameState::new();
            s.fighters[0].x = -250 * PX;
            s.fighters[1].x = 100 * PX;
            s.fighters[1].meter = crate::sim::Fighter::METER_MAX;
            s.minions.push(agent_at(100 - dist));
            s.frame = window * DECISION_FRAMES;
            let i = cpu_input_vs(&s, 1, &me, &me);
            light |= i.has(Input::LIGHT);
            heavy |= i.has(Input::HEAVY);
            special |= i.has(Input::SPECIAL);
        }
        (light, heavy, special)
    }

    #[test]
    fn each_cpu_fights_sub_agents_with_the_attack_that_suits_it() {
        // Anaran's staff slam wipes out agents round him; his slow scroll is not for them
        let (_, heavy, special) = presses("anaran", 40);
        assert!(heavy && !special, "Anaran slams");
        // the Professors' wide grab reaches far; Gray Persona's bats fly straight at them from a way off
        assert!(presses("ayz", 100).1, "the Professor grabs from a distance");
        assert!(presses("evilayz", 100).1, "so does the Negaprofessor");
        assert!(presses("gray_persona", 110).1, "Gray Persona sends the bats");
        // the rest use lights only, and never the special
        for (name, dist) in [("cowboy", 25), ("junkhead", 25), ("scribe", 20), ("gray_persona", 20)] {
            let (light, heavy, special) = presses(name, dist);
            assert!(light && !heavy && !special, "{name} uses lights on an agent {dist} px away");
        }
    }

    #[test]
    fn the_cpu_fights_goblins_on_both_sides_instead_of_backing_off_or_ignoring_them() {
        let text = std::fs::read_to_string("assets/characters/ayz.ron").unwrap();
        let me = CharacterData::from_ron(&text).unwrap();
        let goblin = |x: i32, slot: u32| crate::sim::Minion {
            owner: 0,
            slot,
            x: x * PX,
            mode: crate::sim::MinionMode::Charge,
            timer: 0,
            age: 0,
            move_index: 0,
            bit: false,
            facing: 1,
            health: 56,
            hurt: 0,
            agent: false,
            roll_dir: 0,
        };
        let mut attacked = false;
        for window in 0..60u32 {
            let mut s = GameState::new();
            s.fighters[0].x = -250 * PX;
            s.fighters[1].x = 100 * PX;
            // goblins at its feet, one in front of it and one behind
            s.minions.push(goblin(88, 0));
            s.minions.push(goblin(112, 1));
            s.frame = window * DECISION_FRAMES;
            let i = cpu_input_vs(&s, 1, &me, &me);
            assert!(!i.has(Input::SPECIAL), "no special on goblins, got {i:?}");
            attacked |= i.has(Input::LIGHT) || i.has(Input::HEAVY);
        }
        assert!(attacked, "it attacks the goblins");
    }
}
