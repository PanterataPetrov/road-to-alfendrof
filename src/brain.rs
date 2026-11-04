//! A CPU that looks ahead. Every few frames it tries a handful of things it could do (step in, step out, block, each
//! attack, jump, roll, dash, or just carry on with the rules in `cpu.rs`), plays each one out in a copy of the match against
//! guesses of what the player does next, and does whichever comes out best. The guesses are weighed by what this player
//! has done in the same situation before (see `habits.rs`: it learns what follows what, and what they do against what it
//! does), and it keeps an eye on its own habits too, so it doesn't repeat itself into being easy to read. It reacts like a
//! person: an attack they have only just started isn't seen for a few frames.

use std::collections::VecDeque;

use crate::cpu::{cpu_input_vs, style_of, Style, DECISION_FRAMES};
use crate::habits::{self, Context, Habits, Watch, CLASSES};
use crate::sim::{CharacterData, Fighter, FighterState, GameState, Input, Phase, PX};

/// Frames played out for every try: the first `DECISION_FRAMES` are the try itself, the rest the rules carrying on.
const HORIZON: u32 = 28;
/// How long the guessed player actions last before the rules take over for them too.
const GUESS_FRAMES: u32 = 14;
/// Frames before the CPU notices an attack the player has just started.
const REACTION: u32 = 9;
/// Out of a hundred decisions, how many are a human-like slip: the second best choice instead of the best.
const SLIP_PERCENT: u32 = 10;
/// How much better than the rest something has to be to beat the rules' own choice.
const RULES_EDGE: f32 = 10.0;

/// What the CPU does for the next `DECISION_FRAMES`.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Plan {
    /// The rules in `cpu.rs`, frame by frame.
    Rules,
    /// This input: buttons on the first frame, held directions after.
    Hold(Input),
    /// A dash: the direction tapped twice (down, up, down), which is what the game takes for a dash.
    Dash(u16),
}

/// The input of a double tap of `flag` on frame `frame` of a plan.
fn dash_input(flag: u16, frame: u32) -> Input {
    Input(if frame == 0 || frame == 2 { flag } else { 0 })
}

/// The kind of action a plan is, in the terms `habits.rs` uses.
fn plan_class(plan: Plan) -> usize {
    match plan {
        Plan::Rules => habits::IDLE,
        Plan::Dash(_) => habits::APPROACH,
        Plan::Hold(i) => {
            if i.has(Input::SPECIAL) {
                habits::SPECIAL
            } else if i.has(Input::HEAVY) {
                habits::HEAVY
            } else if i.has(Input::LIGHT) {
                habits::LIGHT
            } else if i.has(Input::BLOCK) {
                habits::BLOCK
            } else if i.has(Input::UP) {
                habits::JUMP
            } else {
                habits::IDLE
            }
        }
    }
}

/// How much it counts against an attack that it is the third time running (a CPU that does the same thing again and again
/// is easy to read).
const REPEAT_PENALTY: f32 = 2.5;

/// How much it counts against a plan that its own habits say it is about to do (see `Brain::self_habits`): it is easy to
/// punish a fighter whose next move can be read.
const SELF_PENALTY: f32 = 8.0;

/// What it costs a plan that its own habits say it will choose with probability `likely`: nothing up to even odds, then more
/// the more certain it is.
fn self_penalty(likely: f32) -> f32 {
    SELF_PENALTY * (likely - 0.5).max(0.0)
}

/// The CPU's memory between frames.
pub struct Brain {
    plan: Option<Plan>,
    /// The opponent's last few frames, oldest first (for the reaction time).
    seen: VecDeque<Fighter>,
    /// What this player does in which situation.
    habits: Habits,
    /// The two fighters' play over the window in progress.
    watches: [Watch; 2],
    /// The situation the window in progress started in, and whether there has been a whole window yet to learn from.
    ctx: Context,
    /// What the player did the window before the last one.
    before_last: usize,
    ctx_ready: bool,
    /// Whether it learns at all (off, it only has the usual mix to go on).
    learning: bool,
    /// The kinds of plan it chose last, newest first (its own habits).
    own: [usize; 2],
    /// The CPU's own habits, learned the same way as the player's: what it tends to do in which situation. A plan it is
    /// very likely to choose anyway counts against it, so it keeps itself hard to read.
    self_habits: Habits,
    mine_before_last: usize,
    self_aware: bool,
    /// How often its own next action could have been guessed from its habits, and out of how many windows.
    self_right: u32,
    self_windows: u32,
    /// How often its guess at the player's next action was right, against how often the usual mix would have been, and how
    /// many windows that was out of.
    right: u32,
    right_by_default: u32,
    guessed_windows: u32,
    last_frame: u32,
}

fn noise(a: u32, b: u32) -> u32 {
    let mut x = a.wrapping_mul(2_654_435_761) ^ b.wrapping_mul(40_503).wrapping_add(17);
    x ^= x >> 15;
    x = x.wrapping_mul(2_246_822_519);
    x ^ (x >> 13)
}

impl Default for Brain {
    fn default() -> Self {
        Brain::new()
    }
}

impl Brain {
    pub fn new() -> Self {
        Brain {
            plan: None,
            seen: VecDeque::new(),
            habits: Habits::new(),
            watches: [Watch::default(), Watch::default()],
            ctx: (1, habits::IDLE, habits::IDLE),
            before_last: habits::IDLE,
            ctx_ready: false,
            learning: true,
            own: [usize::MAX; 2],
            self_habits: Habits::new(),
            mine_before_last: habits::IDLE,
            self_aware: true,
            self_right: 0,
            self_windows: 0,
            right: 0,
            right_by_default: 0,
            guessed_windows: 0,
            last_frame: 0,
        }
    }

    /// How often the CPU's guess at the player's next action was right, and how often the usual mix (what it would guess
    /// with no learning) was, over all the windows so far.
    pub fn accuracy(&self) -> (f32, f32, u32) {
        let n = self.guessed_windows.max(1) as f32;
        (self.right as f32 / n, self.right_by_default as f32 / n, self.guessed_windows)
    }

    /// A brain that learns nothing about the player (for comparing).
    pub fn without_learning() -> Self {
        Brain { learning: false, ..Brain::new() }
    }

    /// A brain that learns about the player but pays no attention to its own habits (for comparing).
    pub fn without_self_awareness() -> Self {
        Brain { self_aware: false, ..Brain::new() }
    }

    /// How often its next action could have been guessed from its own habits (lower is harder to read).
    pub fn own_predictability(&self) -> f32 {
        self.self_right as f32 / self.self_windows.max(1) as f32
    }

    /// Whether it learns.
    pub fn learns(&self) -> bool {
        self.learning
    }

    /// The odds it gives to the player's next action in the situation it is in (for tests and for looking at).
    pub fn expects(&self) -> [f32; CLASSES] {
        self.habits.predict(self.ctx, self.before_last)
    }

    /// The CPU's input for fighter `me` this frame. `chars[i]` is fighter `i`'s character.
    pub fn input(&mut self, state: &GameState, me: usize, chars: [&CharacterData; 2]) -> Input {
        if state.frame < self.last_frame {
            *self = Brain { learning: self.learning, self_aware: self.self_aware, ..Brain::new() };
        }
        self.last_frame = state.frame;
        let rules = |s: &GameState| cpu_input_vs(s, me, chars[me], chars[1 - me]);
        if state.phase != Phase::Fighting {
            return rules(state);
        }
        self.watch(state, 1 - me, chars);
        if state.frame % DECISION_FRAMES == 0 {
            self.end_window(state, me);
        }
        if state.frame % DECISION_FRAMES == 0 || self.plan.is_none() {
            let plan = self.decide(state, me, chars);
            self.own = [plan_class(plan), self.own[0]];
            self.plan = Some(plan);
        }
        match self.plan {
            Some(Plan::Hold(i)) => {
                let first = state.frame % DECISION_FRAMES == 0;
                if first { i } else { held(i) }
            }
            Some(Plan::Dash(flag)) => dash_input(flag, state.frame % DECISION_FRAMES),
            _ => rules(state),
        }
    }

    /// Notes what both fighters are doing, for the habits and (the opponent's last few frames) for the reaction time.
    fn watch(&mut self, state: &GameState, opp: usize, chars: [&CharacterData; 2]) {
        self.seen.push_back(state.fighters[opp].clone());
        while self.seen.len() > REACTION as usize + 1 {
            self.seen.pop_front();
        }
        let me = 1 - opp;
        let (o, m) = (&state.fighters[opp], &state.fighters[me]);
        self.watches[opp].frame(o, m, chars[opp]);
        self.watches[me].frame(m, o, chars[me]);
    }

    /// A decision window is over: what the player did in it is learned against the situation it started in, and the next
    /// window's situation is read off.
    fn end_window(&mut self, state: &GameState, me: usize) {
        let opp = 1 - me;
        let theirs = self.watches[opp].finish();
        let mine = self.watches[me].finish();
        // what they did is learned against what the CPU did in the very same window: they answer what it does
        if self.learning && self.ctx_ready {
            let ctx = (self.ctx.0, self.ctx.1, mine);
            let best = |p: [f32; CLASSES]| (0..CLASSES).max_by(|&a, &b| p[a].total_cmp(&p[b])).unwrap_or(0);
            self.right += (best(self.habits.predict(ctx, self.before_last)) == theirs) as u32;
            self.right_by_default += (best(Habits::new().predict(ctx, self.before_last)) == theirs) as u32;
            self.guessed_windows += 1;
            self.habits.update(ctx, self.before_last, theirs);
        }
        // and what the CPU itself did is learned the same way, against what the player had done: that is its own habit
        if self.ctx_ready {
            let own_ctx = (self.ctx.0, self.ctx.2, self.ctx.1);
            let best = |p: [f32; CLASSES]| (0..CLASSES).max_by(|&a, &b| p[a].total_cmp(&p[b])).unwrap_or(0);
            self.self_right += (best(self.self_habits.predict(own_ctx, self.mine_before_last)) == mine) as u32;
            self.self_windows += 1;
            self.self_habits.update(own_ctx, self.mine_before_last, mine);
            self.mine_before_last = self.ctx.2;
        }
        let gap = (state.fighters[0].x - state.fighters[1].x).abs() / PX;
        self.before_last = self.ctx.1;
        self.ctx = (habits::bucket(gap), theirs, mine);
        self.ctx_ready = true;
    }

    fn decide(&self, state: &GameState, me: usize, chars: [&CharacterData; 2]) -> Plan {
        let opp = 1 - me;
        let (m, o) = (&state.fighters[me], &state.fighters[opp]);
        // sub-agents and goblins are the rules' business (they are fought first, and one light kills an agent)
        let minion_near = state.minions.iter().any(|g| g.owner != me && g.health > 0 && (g.agent || (g.x - m.x).abs() <= 140 * PX));
        if !m.is_neutral() || minion_near {
            return Plan::Rules;
        }
        // what the CPU sees: an attack that has only just started isn't there yet
        let mut base = state.clone();
        if o.state == FighterState::Attack && o.state_frame < REACTION {
            if let Some(old) = self.seen.front().filter(|f| f.state != FighterState::Attack) {
                base.fighters[opp] = old.clone();
            }
        }
        let toward = if o.x > m.x { Input::RIGHT } else { Input::LEFT };
        let away = if o.x > m.x { Input::LEFT } else { Input::RIGHT };
        let roll_toward = if o.x > m.x { Input::ROLL_RIGHT } else { Input::ROLL_LEFT };
        let roll_away = if o.x > m.x { Input::ROLL_LEFT } else { Input::ROLL_RIGHT };
        let mut tries = vec![
            Plan::Rules,
            Plan::Hold(Input(0)),
            Plan::Hold(Input(toward)),
            Plan::Hold(Input(away)),
            Plan::Hold(Input(Input::BLOCK)),
            Plan::Hold(Input(Input::BLOCK | Input::DOWN)),
            Plan::Hold(Input(Input::LIGHT)),
            Plan::Hold(Input(Input::LIGHT | Input::DOWN)),
            Plan::Hold(Input(Input::HEAVY)),
            Plan::Hold(Input(Input::UP | toward)),
            Plan::Hold(Input(Input::UP | away)),
            Plan::Hold(Input(roll_toward)),
            Plan::Hold(Input(roll_away)),
            Plan::Dash(toward),
            Plan::Dash(away),
        ];
        if m.meter >= Fighter::METER_MAX {
            tries.push(Plan::Hold(Input(Input::SPECIAL)));
        }
        let mut scored: Vec<(f32, Plan)> = Vec::with_capacity(tries.len());
        for plan in tries {
            // guesses at what the player does next, and how likely each is: what they have done before in this situation
            // when the CPU did this kind of thing (the rules' plan counts as what it did last time)
            let mine = if plan == Plan::Rules { self.ctx.2 } else { plan_class(plan) };
            let odds = self.habits.predict((self.ctx.0, self.ctx.1, mine), self.before_last);
            let guesses: Vec<(usize, f32)> = (0..CLASSES).map(|c| (c, odds[c])).collect();
            let total: f32 = guesses.iter().map(|g| g.1).sum();
            let mut value = 0.0;
            for &(guess, weight) in &guesses {
                if weight / total < 0.04 {
                    continue;
                }
                value += weight * play_out(&base, me, chars, plan, guess);
            }
            let used: f32 = guesses.iter().filter(|g| g.1 / total >= 0.04).map(|g| g.1).sum();
            let mut value = value / used.max(0.001);
            if plan == Plan::Rules {
                value += RULES_EDGE;
            }
            // its own habits: what it is likely to do anyway counts against it, and so does the third attack of a kind in a row
            let class = plan_class(plan);
            if self.self_aware {
                let own = self.self_habits.predict((self.ctx.0, self.ctx.2, self.ctx.1), self.mine_before_last);
                // (the rules' plan has no kind of its own: it counts as the habit it most plays into)
                let likely = if plan == Plan::Rules { own.iter().cloned().fold(0.0, f32::max) } else { own[class] };
                value -= self_penalty(likely);
            }
            if plan != Plan::Rules && matches!(class, habits::LIGHT | habits::HEAVY | habits::SPECIAL) && self.own == [class, class] {
                value -= REPEAT_PENALTY;
            }
            scored.push((value, plan));
        }
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        // now and then a slip
        let slip = noise(state.frame / DECISION_FRAMES, me as u32 + 9) % 100 < SLIP_PERCENT;
        scored[if slip && scored.len() > 1 { 1 } else { 0 }].1
    }
}

/// A scripted player with a pattern of its own, as fighter 0, for testing how well the CPU learns (`kind` 0..4): 0 jumps in
/// whenever the other attacks and pokes otherwise; 1 blocks when the other is close and attacking and answers with a heavy
/// when it stops; 2 does the same forty-eight frames over and over (in, light, light, heavy, out); 3 backs off whenever the
/// other comes close and throws lights from a distance.
pub fn pattern_player(kind: usize, s: &GameState) -> Input {
    let (me, foe) = (&s.fighters[0], &s.fighters[1]);
    let toward = if foe.x > me.x { Input::RIGHT } else { Input::LEFT };
    let away = if foe.x > me.x { Input::LEFT } else { Input::RIGHT };
    let gap = (foe.x - me.x).abs() / PX;
    let t = s.frame % 48;
    let first = s.frame % 8 == 0;
    Input(match kind {
        0 => {
            if foe.state == FighterState::Attack {
                Input::UP | toward
            } else if gap > 55 {
                toward
            } else if first {
                Input::LIGHT
            } else {
                0
            }
        }
        1 => {
            if gap < 90 && foe.state == FighterState::Attack {
                Input::BLOCK
            } else if gap < 90 && first && foe.state != FighterState::Attack {
                Input::HEAVY
            } else if gap > 90 {
                toward
            } else {
                0
            }
        }
        2 => match t {
            0..=15 => toward,
            16 | 24 => Input::LIGHT,
            32 => Input::HEAVY,
            40..=47 => away,
            _ => 0,
        },
        _ => {
            if gap < 70 {
                away
            } else if first && gap < 160 {
                Input::LIGHT
            } else {
                0
            }
        }
    })
}

/// The directions and block held on from a first frame; buttons, jumps and rolls are only pressed once.
fn held(i: Input) -> Input {
    Input(i.0 & (Input::LEFT | Input::RIGHT | Input::DOWN | Input::BLOCK))
}

/// What the player does for a guess (a kind of action, see `habits.rs`).
fn guessed(guess: usize, frame: u32, s: &GameState, who: usize) -> Input {
    let toward = if s.fighters[1 - who].x > s.fighters[who].x { Input::RIGHT } else { Input::LEFT };
    let away = if toward == Input::RIGHT { Input::LEFT } else { Input::RIGHT };
    Input(match (guess, frame) {
        (habits::BLOCK, _) => Input::BLOCK,
        (habits::LIGHT, 0) => Input::LIGHT,
        (habits::HEAVY, 0) => Input::HEAVY,
        (habits::SPECIAL, 0) => Input::SPECIAL,
        (habits::APPROACH, _) => toward,
        (habits::RETREAT, _) => away,
        (habits::JUMP, 0) => Input::UP | toward,
        _ => 0,
    })
}

/// Plays `plan` out in a copy of `base` against the guess and says how good the end of it is for fighter `me`.
fn play_out(base: &GameState, me: usize, chars: [&CharacterData; 2], plan: Plan, guess: usize) -> f32 {
    let opp = 1 - me;
    let mut s = base.clone();
    let before = (s.fighters[me].health, s.fighters[opp].health, s.fighters[me].meter, s.fighters[opp].meter);
    for frame in 0..HORIZON {
        let mine = match plan {
            Plan::Hold(i) if frame < DECISION_FRAMES => {
                if frame == 0 { i } else { held(i) }
            }
            Plan::Dash(flag) if frame < DECISION_FRAMES => dash_input(flag, frame),
            _ => cpu_input_vs(&s, me, chars[me], chars[opp]),
        };
        let theirs = if frame < GUESS_FRAMES { guessed(guess, frame, &s, opp) } else { cpu_input_vs(&s, opp, chars[opp], chars[me]) };
        let mut inputs = [Input(0); 2];
        inputs[me] = mine;
        inputs[opp] = theirs;
        s.step(inputs, chars);
        if s.phase != Phase::Fighting {
            break;
        }
    }
    score(&s, me, chars[me], before)
}

/// How the position looks for `me`: health lost and dealt (taking damage counts a little more), the special meter,
/// who is in hit-stun, the gap, and sub-agents and zones in play.
fn score(s: &GameState, me: usize, data: &CharacterData, before: (i32, i32, i32, i32)) -> f32 {
    let opp = 1 - me;
    let (m, o) = (&s.fighters[me], &s.fighters[opp]);
    let dealt = (before.1 - o.health) as f32;
    let taken = (before.0 - m.health) as f32;
    let mut v = dealt - taken * 1.15;
    v += ((m.meter - before.2) - (o.meter - before.3)) as f32 * 0.25;
    let stunned = |f: &Fighter| matches!(f.state, FighterState::Hitstun) as i32;
    v += (stunned(o) - stunned(m)) as f32 * 12.0;
    // a fighter who is out of the round has lost it
    if o.health <= 0 {
        v += 400.0;
    }
    if m.health <= 0 {
        v -= 400.0;
    }
    // the gap: a shooter wants some room, everyone else wants to be in reach
    let gap = ((o.x - m.x).abs() / PX) as f32;
    // (Katmata with the lighter in his hand fights up close)
    let style = if m.lighter_mode { Style::Melee } else { style_of(data) };
    let (near, far) = match style {
        Style::Ranged => (80.0, 170.0),
        Style::Melee => (0.0, data.cpu_range.unwrap_or(50) as f32 + 15.0),
    };
    v -= ((near - gap).max(0.0) + (gap - far).max(0.0)).min(150.0) * 0.12;
    // cornered is bad
    let wall = ((GameState::STAGE_HALF_WIDTH - Fighter::HALF_WIDTH) / PX) as f32;
    let at = (m.x / PX) as f32;
    if at > wall - 20.0 || at < -(wall - 20.0) {
        v -= 4.0;
    }
    let mine = s.minions.iter().filter(|g| g.owner == me && g.health > 0).count() as f32;
    let theirs = s.minions.iter().filter(|g| g.owner == opp && g.health > 0).count() as f32;
    v += (mine - theirs) * 12.0;
    v += s.zones.iter().filter(|z| z.owner == me).count() as f32 * 20.0;
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(name: &str) -> CharacterData {
        CharacterData::from_ron(&std::fs::read_to_string(format!("{}/assets/characters/{name}.ron", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap()
    }

    const ALL: [&str; 7] = ["ayz", "anaran", "cowboy", "gray_persona", "junkhead", "katmata", "scribe"];

    /// Fighter 1 plays with the brain (or the rules) against fighter 0's rules for one round; returns (health 1, health 0).
    fn fight(a: &CharacterData, b: &CharacterData, brain: bool) -> (i32, i32) {
        let mut s = GameState::new();
        s.set_health([a.health.unwrap_or(Fighter::MAX_HEALTH), b.health.unwrap_or(Fighter::MAX_HEALTH)]);
        let mut brain_state = Brain::new();
        for _ in 0..3600 {
            let p0 = cpu_input_vs(&s, 0, a, b);
            let p1 = if brain { brain_state.input(&s, 1, [a, b]) } else { cpu_input_vs(&s, 1, b, a) };
            s.step([p0, p1], [a, b]);
            if s.phase != Phase::Fighting {
                break;
            }
        }
        (s.fighters[1].health, s.fighters[0].health)
    }

    #[test]
    fn the_brain_plays_every_champion_without_breaking() {
        let chars: Vec<CharacterData> = ALL.iter().map(|n| load(n)).collect();
        for a in &chars {
            for b in &chars {
                fight(a, b, true);
            }
        }
    }

    #[test]
    fn the_brain_beats_the_rules_it_is_built_on() {
        let chars: Vec<CharacterData> = ALL.iter().map(|n| load(n)).collect();
        let (mut brain_lead, mut rules_lead, mut brain_wins, mut games) = (0, 0, 0, 0);
        for a in &chars {
            for b in &chars {
                // the same fight twice: fighter 1 with the brain, then with the rules
                let (bh, oh) = fight(a, b, true);
                let (rh, ro) = fight(a, b, false);
                brain_lead += bh - oh;
                rules_lead += rh - ro;
                brain_wins += (bh > oh) as i32;
                games += 1;
                println!("{:>12} vs {:>12}: brain {bh:>4}-{oh:<4} rules {rh:>4}-{ro:<4}", a.name, b.name);
            }
        }
        println!("brain lead {brain_lead}, rules lead {rules_lead}, brain won {brain_wins}/{games}");
        assert!(brain_lead > rules_lead, "the brain isn't better");
    }

    /// The brain as fighter 1 against scripted pattern `kind` as fighter 0, for one round of 3600 frames; returns the brain, and
    /// the brain's lead in health.
    fn versus_pattern(brain_char: &CharacterData, bot_char: &CharacterData, kind: usize, mut brain: Brain, seed: u32) -> (Brain, i32) {
        let mut s = GameState::new();
        s.frame = seed * 13;
        s.set_health([bot_char.health.unwrap_or(Fighter::MAX_HEALTH), brain_char.health.unwrap_or(Fighter::MAX_HEALTH)]);
        for _ in 0..3600 {
            let p0 = pattern_player(kind, &s);
            let p1 = brain.input(&s, 1, [bot_char, brain_char]);
            s.step([p0, p1], [bot_char, brain_char]);
            if s.phase != Phase::Fighting {
                break;
            }
        }
        let lead = s.fighters[1].health - s.fighters[0].health;
        (brain, lead)
    }

    const PAIRS: [(&str, &str); 4] = [("junkhead", "cowboy"), ("ayz", "anaran"), ("katmata", "scribe"), ("gray_persona", "junkhead")];

    #[test]
    fn the_brain_picks_up_every_scripted_pattern_and_guesses_the_next_action_better_than_the_usual_mix() {
        for kind in 0..4 {
            let (mut right, mut default, mut n) = (0.0, 0.0, 0.0);
            for (a, b) in PAIRS {
                let (brain, _) = versus_pattern(&load(a), &load(b), kind, Brain::new(), 1);
                let (r, d, _) = brain.accuracy();
                right += r;
                default += d;
                n += 1.0;
            }
            let (right, default) = (right / n, default / n);
            println!("pattern {kind}: guessed right {:.0}%, the usual mix {:.0}%", right * 100.0, default * 100.0);
            assert!(right > default + 0.03, "pattern {kind}: {right:.2} against {default:.2}");
            assert!(right > 0.5, "pattern {kind}: only {right:.2}");
        }
        // a player who answers what the CPU does (jumps in whenever it attacks) is read far better than the usual mix would
        let (mut right, mut default) = (0.0, 0.0);
        for (a, b) in PAIRS {
            let (brain, _) = versus_pattern(&load(a), &load(b), 0, Brain::new(), 2);
            right += brain.accuracy().0;
            default += brain.accuracy().1;
        }
        assert!(right > default + 4.0 * 0.3, "{right:.2} against {default:.2}");
    }

    #[test]
    fn learning_wins_more_against_a_player_who_answers_what_it_does() {
        let (mut with, mut without) = (0, 0);
        for (a, b) in PAIRS {
            for seed in 0..3 {
                with += versus_pattern(&load(a), &load(b), 0, Brain::new(), seed).1;
                without += versus_pattern(&load(a), &load(b), 0, Brain::without_learning(), seed).1;
            }
        }
        println!("health lead over 12 rounds: with learning {with}, without {without}");
        assert!(with > without, "learning didn't help: {with} against {without}");
    }

    #[test]
    fn the_brain_learns_its_own_habits_and_marks_down_what_it_is_about_to_do_anyway() {
        // nothing is held against a plan it is only about as likely to choose as any other; a near certainty is
        assert_eq!(self_penalty(0.3), 0.0);
        assert_eq!(self_penalty(0.5), 0.0);
        assert!(self_penalty(0.9) > self_penalty(0.7) && self_penalty(0.7) > 0.0);
        // and it keeps count of how readable it is: a brain left playing a pattern bot has habits that can be learned
        let (brain, _) = versus_pattern(&load("junkhead"), &load("cowboy"), 2, Brain::new(), 1);
        let readable = brain.own_predictability();
        println!("its own next action could be guessed {:.0}% of the time", readable * 100.0);
        assert!(readable > 0.0 && readable <= 1.0);
        // learning off or self-awareness off still plays (they are the variants the comparisons use)
        for b in [Brain::without_learning(), Brain::without_self_awareness()] {
            versus_pattern(&load("katmata"), &load("ayz"), 0, b, 1);
        }
    }

    #[test]
    fn a_dash_plan_double_taps_and_the_game_takes_it_for_a_dash() {
        let (a, b) = (load("junkhead"), load("katmata"));
        let mut s = GameState::new();
        s.fighters[0].x = -100 * PX;
        s.fighters[1].x = 100 * PX;
        let mut dashed = false;
        for frame in 0..8 {
            let p0 = dash_input(Input::RIGHT, frame);
            s.step([p0, Input(0)], [&a, &b]);
            dashed |= s.fighters[0].state == FighterState::Dash;
        }
        assert!(dashed, "right, nothing, right is a dash");
    }

    #[test]
    fn a_decision_is_quick_enough() {
        let (a, b) = (load("katmata"), load("junkhead"));
        let mut s = GameState::new();
        let mut brain = Brain::new();
        let start = std::time::Instant::now();
        let mut decisions = 0;
        for _ in 0..600 {
            let p1 = brain.input(&s, 1, [&a, &b]);
            decisions += (s.frame % DECISION_FRAMES == 0) as u32;
            s.step([Input(0), p1], [&a, &b]);
        }
        let per_frame = start.elapsed().as_micros() as f32 / 600.0;
        println!("{per_frame:.0} microseconds a frame over {decisions} decisions");
        assert!(per_frame < 6000.0, "{per_frame} microseconds a frame is too slow");
    }
}

