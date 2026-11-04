//! What the CPU learns about how a player fights. Every `DECISION_FRAMES` frames a player's play is boiled down to one of a
//! few kinds of action (stood about, blocked, a light, a heavy, the special, walked in, backed off, jumped), and the CPU
//! counts what came after what: in this situation (how far apart they are, what the player did last time, what the CPU did
//! last time) what did the player do next? "Jumps in whenever I attack", "backs off when I come close" and "light, light,
//! heavy, over and over" all turn up as counts, and the counts become the odds the CPU uses to guess the next action.
//! Old counts fade, so a player who changes their ways is followed.

use crate::sim::{Button, CharacterData, Fighter, FighterState};

/// The kinds of action: standing about, blocking, a light, a heavy, the special, walking in, backing off, jumping.
pub const CLASSES: usize = 8;
pub const IDLE: usize = 0;
pub const BLOCK: usize = 1;
pub const LIGHT: usize = 2;
pub const HEAVY: usize = 3;
pub const SPECIAL: usize = 4;
pub const APPROACH: usize = 5;
pub const RETREAT: usize = 6;
pub const JUMP: usize = 7;

/// How far apart they are: close, middling, far.
const BUCKETS: usize = 3;

/// The situation an action came in: distance bucket, the player's last action, and what the CPU did (in the same window,
/// when the odds are asked for a plan it might choose: players answer what they see).
pub type Context = (usize, usize, usize);

/// The bucket for a gap in pixels.
pub fn bucket(gap_px: i32) -> usize {
    if gap_px < 60 {
        0
    } else if gap_px < 140 {
        1
    } else {
        2
    }
}

/// What has followed what, at three levels of detail (and overall).
#[derive(Clone)]
pub struct Habits {
    /// How often each action is taken at all.
    prior: [f32; CLASSES],
    /// [distance][their last][what the CPU did] -> counts of what they did.
    full: [[[[f32; CLASSES]; CLASSES]; CLASSES]; BUCKETS],
    /// [distance][their last] -> counts.
    mid: [[[f32; CLASSES]; CLASSES]; BUCKETS],
    /// [distance] -> counts.
    low: [[f32; CLASSES]; BUCKETS],
    /// [their action before last][their last] -> counts: what comes next in a sequence ("light, light, heavy, ...").
    seq: [[[f32; CLASSES]; CLASSES]; CLASSES],
    /// How many actions have been seen.
    pub seen: u32,
}

impl Default for Habits {
    fn default() -> Self {
        Habits::new()
    }
}

impl Habits {
    pub fn new() -> Self {
        Habits {
            prior: [0.33, 0.08, 0.16, 0.05, 0.02, 0.2, 0.06, 0.1],
            full: Default::default(),
            mid: Default::default(),
            low: Default::default(),
            seq: Default::default(),
            seen: 0,
        }
    }

    /// The odds of each action the player does next, in this situation. With nothing seen it is the usual mix; the more
    /// that has followed this exact situation before, the more it is what the odds say.
    pub fn predict(&self, ctx: Context, before_last: usize) -> [f32; CLASSES] {
        let (b, t, m) = ctx;
        // each level counts for more the more it has seen (a row with a couple of observations is only a hint)
        let level = |row: &[f32; CLASSES], weight: f32| -> [f32; CLASSES] {
            let n: f32 = row.iter().sum();
            let w = weight * n / (n + 3.0);
            let mut out = [0.0; CLASSES];
            if n > 0.0 {
                for c in 0..CLASSES {
                    out[c] = w * row[c] / n;
                }
            }
            out
        };
        let (low, mid, full, seq) = (level(&self.low[b], 4.0), level(&self.mid[b][t], 6.0), level(&self.full[b][t][m], 14.0), level(&self.seq[before_last][t], 8.0));
        let mut p = [0.0; CLASSES];
        for c in 0..CLASSES {
            p[c] = 3.0 * self.prior[c] + low[c] + mid[c] + full[c] + seq[c];
        }
        let total: f32 = p.iter().sum();
        for v in &mut p {
            *v /= total;
        }
        p
    }

    /// The player did `class` in situation `ctx`.
    pub fn update(&mut self, ctx: Context, before_last: usize, class: usize) {
        let (b, t, m) = ctx;
        let fade = |row: &mut [f32; CLASSES], keep: f32| {
            for v in row.iter_mut() {
                *v *= keep;
            }
            row[class] += 1.0;
        };
        fade(&mut self.full[b][t][m], 0.95);
        fade(&mut self.mid[b][t], 0.97);
        fade(&mut self.low[b], 0.985);
        fade(&mut self.seq[before_last][t], 0.96);
        for (c, v) in self.prior.iter_mut().enumerate() {
            *v += ((c == class) as u32 as f32 - *v) * 0.03;
        }
        self.seen += 1;
    }
}

/// One fighter's play over a decision window, as it is watched frame by frame.
#[derive(Clone, Default)]
pub struct Watch {
    attack: Option<usize>,
    guard: u32,
    air: u32,
    toward: u32,
    away: u32,
    /// What the fighter was last frame, to tell an attack that begins from one that goes on.
    prev: Option<(FighterState, usize, u32)>,
}

impl Watch {
    /// Takes one frame of fighter `f`, `foe` being the other one.
    pub fn frame(&mut self, f: &Fighter, foe: &Fighter, data: &CharacterData) {
        let began = f.state == FighterState::Attack
            && match self.prev {
                Some((state, attack, frame)) => state != FighterState::Attack || attack != f.attack || f.state_frame < frame,
                None => true,
            };
        if began {
            self.attack = data.moves.get(f.attack).map(|m| match m.button {
                Button::Light => LIGHT,
                Button::Heavy => HEAVY,
                Button::Special => SPECIAL,
            });
        }
        if matches!(f.state, FighterState::Guard | FighterState::Blockstun) {
            self.guard += 1;
        }
        if !f.grounded() || f.state == FighterState::Jump {
            self.air += 1;
        }
        let toward = (foe.x - f.x).signum();
        if matches!(f.state, FighterState::Walk | FighterState::Dash | FighterState::Roll) && f.vx != 0 && toward != 0 {
            if f.vx.signum() == toward {
                self.toward += 1;
            } else {
                self.away += 1;
            }
        }
        self.prev = Some((f.state, f.attack, f.state_frame));
    }

    /// What the window added up to, and a fresh start for the next one.
    pub fn finish(&mut self) -> usize {
        let class = if let Some(a) = self.attack {
            a
        } else if self.guard >= 3 {
            BLOCK
        } else if self.air >= 3 {
            JUMP
        } else if self.toward >= 3 && self.toward >= self.away {
            APPROACH
        } else if self.away >= 3 {
            RETREAT
        } else {
            IDLE
        };
        let prev = self.prev;
        *self = Watch::default();
        self.prev = prev;
        class
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_nothing_seen_the_odds_are_the_usual_mix() {
        let h = Habits::new();
        let p = h.predict((1, IDLE, IDLE), IDLE);
        assert!((p.iter().sum::<f32>() - 1.0).abs() < 1e-4);
        assert!(p[IDLE] > p[SPECIAL] && p[APPROACH] > p[HEAVY]);
    }

    #[test]
    fn a_pattern_in_one_situation_is_learned_and_other_situations_are_left_alone() {
        let mut h = Habits::new();
        // close in, after a light of theirs while the CPU blocked: they always jump
        for _ in 0..10 {
            h.update((0, LIGHT, BLOCK), IDLE, JUMP);
        }
        let learned = h.predict((0, LIGHT, BLOCK), IDLE);
        assert!(learned[JUMP] > 0.7, "the jump is expected, got {:.2}", learned[JUMP]);
        let elsewhere = h.predict((2, IDLE, APPROACH), IDLE);
        assert!(elsewhere[JUMP] < learned[JUMP] - 0.3, "elsewhere it is not expected as much, got {:.2}", elsewhere[JUMP]);
    }

    #[test]
    fn a_sequence_is_learned_from_what_came_before() {
        // light, light, heavy, over and over
        let mut h = Habits::new();
        let cycle = [LIGHT, LIGHT, HEAVY];
        let (mut last, mut before_last) = (HEAVY, LIGHT);
        for i in 0..60 {
            let next = cycle[i % 3];
            h.update((1, last, IDLE), before_last, next);
            before_last = last;
            last = next;
        }
        assert!(h.predict((1, HEAVY, IDLE), LIGHT)[LIGHT] > 0.6, "a light follows a heavy");
        assert!(h.predict((1, LIGHT, IDLE), LIGHT)[HEAVY] > 0.45 && h.predict((1, LIGHT, IDLE), HEAVY)[LIGHT] > 0.45, "light, light then the heavy, and a light after the heavy");
    }

    #[test]
    fn a_player_who_changes_their_ways_is_followed() {
        let mut h = Habits::new();
        for _ in 0..30 {
            h.update((1, IDLE, IDLE), IDLE, BLOCK);
        }
        assert!(h.predict((1, IDLE, IDLE), IDLE)[BLOCK] > 0.7);
        for _ in 0..30 {
            h.update((1, IDLE, IDLE), IDLE, LIGHT);
        }
        let p = h.predict((1, IDLE, IDLE), IDLE);
        assert!(p[LIGHT] > p[BLOCK], "the old habit faded: light {:.2}, block {:.2}", p[LIGHT], p[BLOCK]);
    }
}
