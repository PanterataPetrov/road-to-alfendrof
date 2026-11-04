//! Does the CPU's learning help? The brain (with and without learning) plays scripted opponents with patterns of their own,
//! and the CPU brain against the CPU brain, one learning and one not.
//!
//!     cargo run --release --example learning [matches]

use t2f::brain::{pattern_player, Brain};
use t2f::sim::{CharacterData, Fighter, FighterState, GameState, Input, Phase, PX};

const CHAMPIONS: [&str; 7] = ["ayz", "anaran", "cowboy", "gray_persona", "junkhead", "katmata", "scribe"];

fn load(name: &str) -> CharacterData {
    CharacterData::from_ron(&std::fs::read_to_string(format!("assets/characters/{name}.ron")).unwrap()).unwrap()
}

/// One round of the brain against a bot; returns the brain's lead in health (positive: the brain is ahead).
fn versus_bot(brain: &CharacterData, other: &CharacterData, kind: usize, learning: bool, seed: u32) -> (i32, f32, f32) {
    let mut s = GameState::new();
    s.frame = seed * 13;
    s.set_health([other.health.unwrap_or(Fighter::MAX_HEALTH), brain.health.unwrap_or(Fighter::MAX_HEALTH)]);
    let mut b = if learning { Brain::new() } else { Brain::without_learning() };
    for _ in 0..3600 {
        let p0 = pattern_player(kind, &s);
        let p1 = b.input(&s, 1, [other, brain]);
        s.step([p0, p1], [other, brain]);
        if s.phase != Phase::Fighting {
            break;
        }
    }
    let (right, default, _) = b.accuracy();
    (s.fighters[1].health - s.fighters[0].health, right, default)
}

/// The brain with learning against the brain without, one round; returns the learner's lead.
fn learner_versus_not(a: &CharacterData, b: &CharacterData, learner_is_zero: bool, seed: u32) -> i32 {
    let mut s = GameState::new();
    s.frame = seed * 13;
    s.set_health([a.health.unwrap_or(Fighter::MAX_HEALTH), b.health.unwrap_or(Fighter::MAX_HEALTH)]);
    let mut brains = if learner_is_zero { [Brain::new(), Brain::without_learning()] } else { [Brain::without_learning(), Brain::new()] };
    for _ in 0..3600 {
        let p0 = brains[0].input(&s, 0, [a, b]);
        let p1 = brains[1].input(&s, 1, [a, b]);
        s.step([p0, p1], [a, b]);
        if s.phase != Phase::Fighting {
            break;
        }
    }
    let lead = s.fighters[0].health - s.fighters[1].health;
    if learner_is_zero { lead } else { -lead }
}

fn main() {
    let reps: u32 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(6);
    let chars: Vec<CharacterData> = CHAMPIONS.iter().map(|n| load(n)).collect();
    println!("## The brain against scripted opponents with patterns (health lead per round, more is better)\n");
    println!("| pattern | lead without learning | lead with learning | guess right with learning | guess right without |");
    println!("|---|---|---|---|---|");
    let names = ["jumps in when I attack", "blocks when close, heavy after", "the same 48-frame cycle", "backs off when I come close"];
    for kind in 0..4 {
        let (mut off, mut on, mut n, mut acc, mut base) = (0i64, 0i64, 0i64, 0.0f64, 0.0f64);
        for (i, a) in chars.iter().enumerate() {
            for (j, b) in chars.iter().enumerate() {
                if i == j {
                    continue;
                }
                for r in 0..reps {
                    off += versus_bot(a, b, kind, false, r).0 as i64;
                    let (lead, right, default) = versus_bot(a, b, kind, true, r);
                    on += lead as i64;
                    acc += right as f64;
                    base += default as f64;
                    n += 1;
                }
            }
        }
        println!("| {} | {:.0} | {:.0} | {:.0}% | {:.0}% |", names[kind], off as f64 / n as f64, on as f64 / n as f64, 100.0 * acc / n as f64, 100.0 * base / n as f64);
    }
    println!("\n## The brain with learning against the brain without (every pairing, both sides)\n");
    let (mut wins, mut games, mut lead) = (0, 0, 0i64);
    for (i, a) in chars.iter().enumerate() {
        for (j, b) in chars.iter().enumerate() {
            if i == j {
                continue;
            }
            for r in 0..reps {
                for zero in [true, false] {
                    let l = learner_versus_not(a, b, zero, r);
                    wins += (l > 0) as i32;
                    lead += l as i64;
                    games += 1;
                }
            }
        }
    }
    println!("the learner won {wins} of {games} rounds ({:.0}%), average health lead {:.0}", 100.0 * wins as f64 / games as f64, lead as f64 / games as f64);
}
