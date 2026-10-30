//! CPU opponent. It produces the same `Input` a player does, using only the game state, so it is
//! deterministic: the same state always gives the same input.

use crate::sim::{Button, CharacterData, Fighter, FighterState, GameState, Input, MoveData, Phase, PX};

/// Centre-to-centre distance in pixels at which the CPU starts attacking.
const ATTACK_RANGE: i32 = 50;
/// Distance at which it starts worrying about the opponent's attacks.
const DANGER_RANGE: i32 = 90;
/// How many frames each decision lasts. Buttons are only pressed on the first frame of a window.
const DECISION_FRAMES: u32 = 8;

fn noise(window: u32, me: usize) -> u32 {
    let mut x = window.wrapping_mul(2_654_435_761) ^ (me as u32 + 1).wrapping_mul(40_503);
    x ^= x >> 15;
    x = x.wrapping_mul(2_246_822_519);
    x ^= x >> 13;
    x
}

/// How far from the fighter's centre a move can reach an opponent standing in front of it, in pixels.
fn reach(mv: &MoveData, boosted: bool) -> i32 {
    mv.hitbox.x + mv.hitbox.w + if boosted { mv.boost_reach } else { 0 } + Fighter::HALF_WIDTH / PX
}

pub fn cpu_input(state: &GameState, me: usize, data: &CharacterData) -> Input {
    if state.phase != Phase::Fighting {
        return Input::default();
    }
    let (m, o) = (&state.fighters[me], &state.fighters[1 - me]);
    // Goblins summoned by the opponent are fought first when one is nearer than the opponent. Fighters
    // always face the opponent, so only goblins on that side can be swung at.
    let target_x = state
        .minions
        .iter()
        .filter(|g| g.owner != me && g.health > 0 && (g.x - m.x) * (o.x - m.x) > 0)
        .map(|g| g.x)
        .min_by_key(|x| (x - m.x).abs())
        .filter(|x| (x - m.x).abs() < (o.x - m.x).abs())
        .unwrap_or(o.x);
    let dx = target_x - m.x;
    let dist = dx.abs() / PX;
    let opponent_dist = (o.x - m.x).abs() / PX;
    let (toward, away) = if dx > 0 { (Input::RIGHT, Input::LEFT) } else { (Input::LEFT, Input::RIGHT) };

    let window = state.frame / DECISION_FRAMES;
    let first_frame = state.frame % DECISION_FRAMES == 0;
    let roll = noise(window, me) % 100;

    // A zoner stays out of reach: near a wall it cannot back off any further.
    let wall = (GameState::STAGE_HALF_WIDTH - Fighter::HALF_WIDTH) / PX;
    let cornered = (away == Input::RIGHT && m.x / PX > wall - 25) || (away == Input::LEFT && m.x / PX < -(wall - 25));

    // Guard (sometimes crouching, to get under shots) while the opponent is attacking up close.
    if o.state == FighterState::Attack && opponent_dist < DANGER_RANGE && roll < 65 {
        // A zoner would rather hop away than stand and block.
        if data.keeps_distance && roll < 35 && m.is_neutral() {
            return Input(if cornered { toward } else { away } | Input::UP);
        }
        return Input(if roll < 20 { Input::DOWN } else { Input::BLOCK });
    }

    // Keep pressing forward during an attack: moves that allow it will advance with the hit.
    if m.state == FighterState::Attack {
        // Junkhead: once a heavy (the pull) or the special (the drag) has connected, go straight into
        // light attacks (tapped fast, so that each press registers).
        if data.light_after_heavy && m.hit_done && data.moves[m.attack].button != Button::Light {
            return Input(if state.frame % 2 == 0 { Input::LIGHT } else { 0 });
        }
        // A move that connected can be cancelled into a heavier one: build a combo.
        if m.hit_done && m.hitstop == 0 && roll < 55 {
            return Input(if roll < 40 { Input::HEAVY } else { Input::SPECIAL });
        }
        return Input(if dist > 20 { toward } else { 0 });
    }
    // In the air close to the opponent: kick or slam, once.
    if m.state == FighterState::Jump {
        if dist < 45 && !m.air_used && roll < 45 {
            return Input(if roll < 30 { Input::LIGHT } else { Input::HEAVY } | toward);
        }
        return Input(toward);
    }
    if !m.is_neutral() {
        return Input::default();
    }

    // A curse of mine on the opponent needs a hit from each side: walk through them to the side
    // that is still missing.
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

    // A zoner keeps its distance: it backs away when the opponent gets close (hopping over them if
    // it is against a wall), pokes with the short shot when they are right on top of it, and
    // shoots from a distance with the volley and the special.
    if data.keeps_distance {
        let button = |flag: u16| Input(if first_frame { flag } else { 0 });
        if dist < 70 {
            if cornered {
                return Input(toward | Input::UP);
            }
            if dist < 50 && roll < 22 {
                return button(Input::LIGHT);
            }
            return Input(away);
        }
        if dist <= 175 {
            if m.meter >= Fighter::METER_MAX && roll < 35 {
                return button(Input::SPECIAL);
            }
            if roll < 32 {
                return button(if roll < 16 { Input::HEAVY } else { Input::LIGHT });
            }
            // Keep adjusting the gap: back off if it is a little short, otherwise stand and wait.
            return Input(if dist < 115 && roll < 70 { away } else { 0 });
        }
        return Input(toward);
    }

    // Buttons go down on one frame only.
    let button = |flag: u16| Input(if first_frame { flag } else { 0 });
    // Junkhead keeps tapping Light while the quick lights after a heavy or special last, or a light
    // combo is going, as long as the opponent is in front of him.
    if data.light_after_heavy && (m.quick > 0 || m.combo > 0) && dist <= data.ai_range.unwrap_or(ATTACK_RANGE) + 10 {
        return Input(if state.frame % 2 == 0 { Input::LIGHT } else { 0 });
    }
    // A long-reach fighter starts attacking from further away, and a whip that just got its extra
    // reach is used from that distance.
    let light = data.find_move(Button::Light, false, false).map(|i| &data.moves[i]);
    let mut range = data.ai_range.unwrap_or(ATTACK_RANGE);
    if let Some(l) = light.filter(|l| m.whip_boost && l.boost_reach > 0) {
        range = range.max(reach(l, true) - 15);
    }
    if dist > range {
        // A lash that drags the opponent in, or pulls the fighter to them, works from a distance.
        let special = data.find_move(Button::Special, false, false).map(|i| &data.moves[i]);
        if special.is_some_and(|s| s.pull_foe && dist <= reach(s, false) - 25) && m.meter >= Fighter::METER_MAX && roll < 60 {
            return button(Input::SPECIAL);
        }
        let heavy = data.find_move(Button::Heavy, false, false).map(|i| &data.moves[i]);
        if heavy.is_some_and(|h| h.pull_self && dist <= reach(h, false) - 15) && roll < 50 {
            return button(Input::HEAVY);
        }
        let hop = if roll < 10 && dist > 70 { Input::UP } else { 0 };
        return Input(toward | hop);
    }

    // In range: pick an attack, retreat, or wait.
    // With the special ready, use it often.
    if m.meter >= crate::sim::Fighter::METER_MAX && roll < 45 {
        return button(Input::SPECIAL);
    }
    match roll {
        0..=34 => button(Input::LIGHT),
        35..=54 => button(Input::HEAVY),
        55..=61 => button(if m.meter >= crate::sim::Fighter::METER_MAX { Input::SPECIAL } else { Input::LIGHT }),
        62..=74 => button(Input::LIGHT | Input::DOWN),
        75..=84 => Input(away),
        _ => Input::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::CharacterData;

    fn chars() -> [CharacterData; 2] {
        [
            CharacterData::from_ron(include_str!("sim/classic_fighter.ron")).unwrap(),
            CharacterData::from_ron(include_str!("../assets/characters/evilayz.ron")).unwrap(),
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
    fn a_zoner_backs_away_when_the_opponent_is_close_and_shoots_from_a_distance() {
        let g = CharacterData::from_ron(include_str!("../assets/characters/gunslinger.ron")).unwrap();
        assert!(g.keeps_distance);
        // The opponent (P1) is 40 px away: P2 backs off, to the right.
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 40 * PX;
        let i = cpu_input(&s, 1, &g);
        assert!(i.has(Input::RIGHT) || i.has(Input::LIGHT), "should back off or poke, got {i:?}");
        assert!(!i.has(Input::LEFT), "must not walk into them");
        // And over time it ends up further away than a normal CPU would be.
        let c = chars();
        let mut s = GameState::new();
        s.fighters[0].x = -40 * PX;
        s.fighters[1].x = 20 * PX;
        for _ in 0..300 {
            let cpu = cpu_input(&s, 1, &g);
            s.step([Input::default(), cpu], [&c[0], &g]);
        }
        let gap = (s.fighters[1].x - s.fighters[0].x) / PX;
        assert!(gap > 70, "the zoner should have made some room, gap {gap}");
    }

    #[test]
    fn cpu_turns_on_a_goblin_that_is_nearer_than_the_player() {
        use crate::sim::{Minion, MinionMode};
        let c = chars();
        let mut s = GameState::new();
        s.fighters[0].x = -200 * PX;
        s.fighters[1].x = 100 * PX;
        // Without goblins the CPU (P2) walks left towards the far player; with a goblin 30 px in front
        // of it, it stops and attacks that instead.
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
}
