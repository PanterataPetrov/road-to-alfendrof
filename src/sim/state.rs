use super::data::{CharacterData, MoveData};
use super::fighter::{Curse, Fighter, FighterState, PX};
use super::input::Input;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    Fighting,
    /// Frames since the round ended (KO or time up).
    RoundEnd(u32),
    MatchOver,
}

/// A projectile in flight. Positions and velocities are in subpixels.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Projectile {
    pub owner: usize,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    /// Frames since launch.
    pub age: u32,
    /// Which of the owner's moves launched it (for damage and stun).
    pub move_index: usize,
    /// Where a lob lands (subpixels).
    pub land_x: i32,
    /// Damage as a percentage of the move's (100 = uncharged).
    pub power: i32,
}

/// What a minion is doing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MinionMode {
    /// Waiting beside the summoner.
    Wait,
    /// Rushing the opponent.
    Charge,
    /// Running home.
    Return,
}

/// A summoned minion (a goblin).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Minion {
    pub owner: usize,
    pub slot: u32,
    pub x: i32,
    pub mode: MinionMode,
    /// Frames in the current mode, and since it was summoned.
    pub timer: u32,
    pub age: u32,
    /// Which of the owner's moves summoned it (for the bite's damage), and whether it has bitten this rush.
    pub move_index: usize,
    pub bit: bool,
    /// 1 faces right, -1 faces left.
    pub facing: i32,
    /// Hit points left; the minion dies at 0.
    pub health: i32,
    /// Frames until the same swing can hit it again (also drives the hit flash).
    pub hurt: u32,
}

/// A raven sent out by a move (see `RavenMove`). While `active` it follows its script one step at a
/// time; otherwise it sits on the owner's shoulder.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Raven {
    pub active: bool,
    /// The move that sent it (index into the owner's moves) and where it is in the script.
    pub move_index: usize,
    pub step: u32,
    /// Frames in this step.
    pub timer: u32,
    /// Position in subpixels, and 1 if it faces right.
    pub x: i32,
    pub y: i32,
    pub facing: i32,
    /// 100, or 150 if a combo finisher sent it.
    pub power: i32,
    /// The rose it carries or has dropped.
    pub rose: Option<Rose>,
}

impl Raven {
    pub fn idle() -> Self {
        Raven { active: false, move_index: 0, step: 0, timer: 0, x: 0, y: 0, facing: 1, power: 100, rose: None }
    }
}

/// A rose: carried by the raven, then falling onto its owner.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Rose {
    pub x: i32,
    pub y: i32,
    pub falling: bool,
}

/// The whole simulation. `Clone` + `Hash` make rollback and desync checks cheap. Character data
/// isn't part of the state: it's constant and passed to `step`.
#[derive(Clone, Hash, Debug)]
pub struct GameState {
    pub frame: u32,
    pub fighters: [Fighter; 2],
    /// Every projectile in flight.
    pub projectiles: Vec<Projectile>,
    /// Every minion on the field.
    pub minions: Vec<Minion>,
    /// Each fighter's raven.
    pub ravens: [Raven; 2],
    /// Where projectiles were just destroyed (subpixels) and on which frame, kept a few frames so the
    /// screen can show a puff.
    pub cleared: Vec<(i32, i32, u32)>,
    /// Where shots aimed at the floor just landed (subpixels) and on which frame, for the sparks.
    pub impacts: Vec<(i32, i32, u32)>,
    /// Frames left on the round clock.
    pub timer: u32,
    pub wins: [u8; 2],
    /// 1-based round number.
    pub round: u8,
    pub phase: Phase,
}

impl GameState {
    /// The stage spans `-STAGE_HALF_WIDTH..=STAGE_HALF_WIDTH` around x = 0.
    pub const STAGE_HALF_WIDTH: i32 = 400 * PX;
    pub const ROUND_FRAMES: u32 = 99 * 60;
    pub const ROUND_END_FRAMES: u32 = 150;
    pub const ROUNDS_TO_WIN: u8 = 2;
    /// A blocked hit takes 1/CHIP_DIVISOR of its damage but never kills.
    const CHIP_DIVISOR: i32 = 10;
    /// A blocked whip pulls its user only this far (percent).
    const PULL_BLOCKED_PCT: i32 = 50;

    pub fn new() -> Self {
        GameState {
            frame: 0,
            fighters: Self::starting_fighters(),
            projectiles: Vec::new(),
            minions: Vec::new(),
            ravens: [Raven::idle(); 2],
            cleared: Vec::new(),
            impacts: Vec::new(),
            timer: Self::ROUND_FRAMES,
            wins: [0; 2],
            round: 1,
            phase: Phase::Fighting,
        }
    }

    /// Give each fighter their own starting health.
    pub fn set_health(&mut self, health: [i32; 2]) {
        for (f, h) in self.fighters.iter_mut().zip(health) {
            f.max_health = h;
            f.health = h;
        }
    }

    fn starting_fighters() -> [Fighter; 2] {
        [Fighter::new(-80 * PX, 1), Fighter::new(80 * PX, -1)]
    }

    /// Advance one frame (60 a second). Fighters are processed in index order, so the result depends
    /// only on the state and the inputs.
    pub fn step(&mut self, inputs: [Input; 2], chars: [&CharacterData; 2]) {
        self.frame += 1;
        let now = self.frame;
        self.cleared.retain(|c| c.2 + 6 > now);
        self.impacts.retain(|c| c.2 + 6 > now);
        let fighting = self.phase == Phase::Fighting;
        let inputs = if fighting { inputs } else { [Input::default(); 2] };

        for i in 0..2 {
            self.fighters[i].raven_away = self.ravens[i].active;
            self.fighters[i].step(inputs[i], chars[i]);
        }

        self.clamp_to_stage();
        self.face_each_other();

        match self.phase {
            Phase::Fighting => {
                self.start_lashes(chars);
                self.update_portals(chars);
                self.apply_poofs(chars);
                self.apply_slams(inputs);
                self.lock_beams(chars);
                self.launch_projectiles(chars);
                self.fire_channels(chars);
                self.apply_curses(chars);
                self.summon_minions(chars);
                self.update_minions(chars, inputs);
                self.launch_ravens(chars);
                self.update_ravens(chars, inputs);
                self.resolve_hits(inputs, chars);
                self.update_curses();
                self.clear_projectiles(chars);
                self.move_projectiles(inputs, chars);
                if self.fighters.iter().all(|f| f.hitstop == 0) {
                    self.timer = self.timer.saturating_sub(1);
                }
                if self.fighters.iter().any(|f| f.health <= 0) || self.timer == 0 {
                    self.end_round();
                }
            }
            Phase::RoundEnd(n) => {
                if n + 1 >= Self::ROUND_END_FRAMES {
                    if self.wins.iter().any(|&w| w >= Self::ROUNDS_TO_WIN) {
                        self.phase = Phase::MatchOver;
                    } else {
                        self.start_next_round();
                    }
                } else {
                    self.phase = Phase::RoundEnd(n + 1);
                }
            }
            Phase::MatchOver => {}
        }
    }

    /// Who won the match once it's over; `None` if it ended level.
    pub fn match_winner(&self) -> Option<usize> {
        if self.phase != Phase::MatchOver || self.wins[0] == self.wins[1] {
            return None;
        }
        Some(if self.wins[0] > self.wins[1] { 0 } else { 1 })
    }

    fn end_round(&mut self) {
        let (h0, h1) = (self.fighters[0].health.max(0), self.fighters[1].health.max(0));
        // equal health (double KO or a time-up tie) gives both a point
        if h0 >= h1 {
            self.wins[0] += 1;
        }
        if h1 >= h0 {
            self.wins[1] += 1;
        }
        self.phase = Phase::RoundEnd(0);
    }

    fn start_next_round(&mut self) {
        let health = [self.fighters[0].max_health, self.fighters[1].max_health];
        self.fighters = Self::starting_fighters();
        self.set_health(health);
        self.projectiles.clear();
        self.minions.clear();
        self.ravens = [Raven::idle(); 2];
        self.cleared.clear();
        self.impacts.clear();
        self.timer = Self::ROUND_FRAMES;
        self.round += 1;
        self.phase = Phase::Fighting;
    }

    /// Whether the defender is guarding: free to act (or already blocking) and holding block.
    fn can_block(&self, defender: usize, input: Input) -> bool {
        let f = &self.fighters[defender];
        let blockable_state = f.is_neutral() || f.state == FighterState::Blockstun;
        blockable_state && input.has(Input::BLOCK) && f.block_disabled == 0
    }

    /// World-space box `[x0, y0, x1, y1]` (subpixels) of `attacker`'s hitbox, if it's active.
    pub fn hitbox(&self, attacker: usize, data: &CharacterData) -> Option<[i32; 4]> {
        let f = &self.fighters[attacker];
        if f.state != FighterState::Attack || f.hit_done || f.hitstop > 0 {
            return None;
        }
        let mv = &data.moves[f.attack];
        if !mv.is_active(f.state_frame) {
            return None;
        }
        // a beam of light is a column that sweeps from in front of where the opponent stood to behind it
        if let (Some(b), Some(lock)) = (mv.beam, f.beam_lock) {
            let progress = (f.state_frame.saturating_sub(mv.startup)).min(mv.active) as i32;
            let (start, end) = (lock - f.facing * b.lead * PX, lock + f.facing * b.sweep * PX);
            let cx = start + (end - start) * progress / mv.active.max(1) as i32;
            return Some([cx - b.width * PX / 2, 0, cx + b.width * PX / 2, b.height * PX]);
        }
        let r = if f.aim_low { mv.low_hitbox.unwrap_or(mv.hitbox) } else { mv.hitbox };
        if r.w <= 0 {
            return None; // a summon has no melee hit
        }
        let w = r.w + if f.boosted { mv.boost_reach } else { 0 };
        let (x0, x1) = if f.facing > 0 {
            (f.x + r.x * PX, f.x + (r.x + w) * PX)
        } else {
            (f.x - (r.x + w) * PX, f.x - r.x * PX)
        };
        let base = if mv.world_y { 0 } else { f.y };
        let (mut x0, mut x1, mut y0, mut y1) = (x0, x1, base + r.y * PX, base + (r.y + r.h) * PX);
        if mv.assist > 0 {
            let a = mv.assist * PX;
            if f.facing > 0 {
                x1 += a;
            } else {
                x0 -= a;
            }
            // a move with a high and a low version (the special) keeps its height so crouching and jumping still dodge it
            if mv.low_hitbox.is_none() {
                let foe = &self.fighters[1 - attacker];
                let foe_mid = foe.y + foe.height() * PX / 2;
                let slide = (foe_mid - (y0 + y1) / 2).clamp(-2 * a, 2 * a);
                y0 += slide - a;
                y1 += slide + a;
            }
        }
        Some([x0, y0, x1, y1])
    }

    /// World-space hurtbox `[x0, y0, x1, y1]` in subpixels.
    pub fn hurtbox(&self, defender: usize) -> [i32; 4] {
        let f = &self.fighters[defender];
        // mid-roll nothing can touch them, so the box is moved out of the world
        if f.invulnerable() {
            return [i32::MAX / 2; 4];
        }
        [f.x - Fighter::HALF_WIDTH, f.y, f.x + Fighter::HALF_WIDTH, f.y + f.height() * PX]
    }

    fn resolve_hits(&mut self, inputs: [Input; 2], chars: [&CharacterData; 2]) {
        // decide everything from the start-of-frame state so simultaneous hits (trades) are symmetric
        let mut landed = [false; 2];
        for a in 0..2 {
            if let Some(h) = self.hitbox(a, chars[a]) {
                let d = self.hurtbox(1 - a);
                landed[a] = h[0] < d[2] && d[0] < h[2] && h[1] < d[3] && d[1] < h[3];
            }
        }
        let blocks = [self.can_block(0, inputs[0]), self.can_block(1, inputs[1])];

        for a in 0..2 {
            // a swing can also cut down goblins it reaches, even if it misses the fighter
            if let Some(h) = self.hitbox(a, chars[a]) {
                let mv = &chars[a].moves[self.fighters[a].attack];
                let power = if self.fighters[a].finisher { 150 } else { 100 };
                if self.hit_minion(1 - a, h, mv.damage * power / 100) && !landed[a] {
                    self.fighters[a].hit_done = true;
                    self.fighters[a].hitstop = mv.hitstop;
                }
            }
            if !landed[a] {
                continue;
            }
            let base = &chars[a].moves[self.fighters[a].attack];
            // a multi-hit move may hit differently in each window (a grab, then a shove)
            let adjusted = base.for_window(base.window(self.fighters[a].state_frame));
            let mv = &*adjusted;
            let dir = if mv.radial {
                if self.fighters[1 - a].x >= self.fighters[a].x { 1 } else { -1 }
            } else {
                self.fighters[a].facing
            };
            self.fighters[a].hit_done = true;
            self.fighters[a].hitstop = mv.hitstop;
            let power = if self.fighters[a].finisher { 150 } else { 100 };
            // a hand out of a portal can't be blocked, only dodged
            let can_block = blocks[1 - a] && base.portal.is_none();
            let blocked = self.apply_hit(1 - a, mv, dir, can_block, power);
            // the first window of a grab (it does no damage, a later hit does) holds them where they are,
            // even in the middle of a jump
            if !blocked && base.hits > 1 && mv.damage == 0 && base.window(self.fighters[a].state_frame) == Some(0) && self.fighters[1 - a].state == FighterState::Hitstun {
                let d = &mut self.fighters[1 - a];
                d.held = true;
                d.vx = 0;
                d.vy = 0;
            }
            if base.portal.is_some() && !blocked && base.window(self.fighters[a].state_frame) == Some(0) {
                // the grab held: she comes out of the portal for the bite
                self.fighters[a].grabbed = true;
                self.fighters[a].hidden = false;
            }
            if mv.heal_pct > 0 && mv.damage > 0 && !blocked {
                let f = &mut self.fighters[a];
                f.health = (f.health + f.max_health * mv.heal_pct / 100).min(f.max_health);
            }
            if mv.light_lock > 0 && !blocked {
                self.fighters[a].light_lock = mv.light_lock;
            }
            if mv.silence > 0 && !blocked {
                self.fighters[1 - a].silenced = mv.silence;
            }
            self.apply_pulls(a, mv, blocked);
            self.gain_meter(a, mv, mv.damage * power / 100, blocked, chars[a].meter_gain_pct.unwrap_or(100));
            self.register_light_hit(a, mv, blocked);
            self.mark_curse(a);
        }
    }

    /// A fighter who just plunged to the floor hits everything in reach of the landing: the opponent,
    /// if they're low enough, and their goblins.
    fn apply_slams(&mut self, inputs: [Input; 2]) {
        for a in 0..2 {
            if !self.fighters[a].slam_landed {
                continue;
            }
            self.fighters[a].slam_landed = false;
            // (a floor smash deals its own damage; the plain ground pound the usual amount)
            let slam_damage = std::mem::replace(&mut self.fighters[a].slam_damage, Fighter::SLAM_DAMAGE);
            let d = 1 - a;
            let ax = self.fighters[a].x;
            let radius = Fighter::SLAM_RADIUS * PX;
            let foe = &self.fighters[d];
            if (foe.x - ax).abs() <= radius && foe.y <= 30 * PX && foe.state != FighterState::Ko && !foe.invulnerable() {
                let dir = if foe.x >= ax { 1 } else { -1 };
                let blocked = self.can_block(d, inputs[d]);
                let foe = &mut self.fighters[d];
                if blocked {
                    foe.health = (foe.health - slam_damage / Self::CHIP_DIVISOR).max(1);
                    foe.start_stun(FighterState::Blockstun, 10, dir * 2 * PX);
                } else {
                    foe.health -= slam_damage;
                    foe.combo = 0;
                    if foe.health <= 0 {
                        foe.start_stun(FighterState::Ko, 0, dir * 4 * PX);
                    } else {
                        foe.start_stun(FighterState::Hitstun, 22, dir * 3 * PX);
                    }
                }
            }
            self.hit_minion(d, [ax - radius, 0, ax + radius, 40 * PX], slam_damage);
        }
    }

    /// Hurt every goblin of `owner` overlapping `area` (a world box in subpixels).
    fn hit_minion(&mut self, owner: usize, area: [i32; 4], damage: i32) -> bool {
        let mut any = false;
        // goblins can be hit anywhere along a fighter's height, and one swing hits every goblin it reaches
        for m in self.minions.iter_mut().filter(|m| m.owner == owner && m.health > 0) {
            let (x0, x1) = (m.x - 8 * PX, m.x + 8 * PX);
            if area[0] < x1 && x0 < area[2] && area[1] < 56 * PX && 0 < area[3] {
                m.health -= damage;
                m.hurt = 8;
                any = true;
            }
        }
        any
    }

    /// A move that sends the raven out does it when its startup ends (if the raven is home); the special
    /// spends the meter.
    fn launch_ravens(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            if f.state != FighterState::Attack || self.ravens[owner].active {
                continue;
            }
            let (Some(rd), mv) = (chars[owner].raven, &chars[owner].moves[f.attack]) else { continue };
            if mv.raven_move.is_none() || f.state_frame != mv.startup + 1 {
                continue;
            }
            self.ravens[owner] = Raven {
                active: true,
                move_index: f.attack,
                step: 0,
                timer: 0,
                x: f.x + f.facing * rd.home.0 * PX,
                y: f.y + rd.home.1 * PX,
                facing: f.facing,
                power: if f.finisher { 150 } else { 100 },
                rose: None,
            };
            if mv.button == super::data::Button::Special {
                self.fighters[owner].meter = 0;
            }
        }
    }

    /// Fly each raven along its script: the claw (out, one hit, home) or the storm (peck from the left, the
    /// right, the left, then above the owner to drop a rose, then home). A raven goes home when the opponent is beaten.
    fn update_ravens(&mut self, chars: [&CharacterData; 2], inputs: [Input; 2]) {
        for owner in 0..2 {
            let mut r = self.ravens[owner];
            if !r.active {
                continue;
            }
            let (Some(rd), Some(kind)) = (chars[owner].raven, chars[owner].moves[r.move_index].raven_move) else {
                self.ravens[owner] = Raven::idle();
                continue;
            };
            let (me, foe) = (&self.fighters[owner], &self.fighters[1 - owner]);
            let speed = rd.speed * PX;
            let home = (me.x + me.facing * rd.home.0 * PX, me.y + rd.home.1 * PX);
            let home_step = match kind {
                super::data::RavenMove::Claw => 1,
                super::data::RavenMove::Storm { pecks, .. } => 2 * pecks + 2,
            };
            if (foe.health <= 0 || self.phase != Phase::Fighting) && r.step < home_step {
                r.step = home_step;
                r.timer = 0;
                r.rose = None;
            }
            r.timer += 1;
            let mut hit: Option<i32> = None;
            let mut heal = 0;
            match kind {
                super::data::RavenMove::Claw => {
                    if r.step == 0 {
                        if Self::raven_toward(&mut r, (foe.x, foe.y + 30 * PX), speed) {
                            hit = Some(if r.x <= foe.x { 1 } else { -1 });
                            r.step = 1;
                            r.timer = 0;
                        }
                    } else if Self::raven_toward(&mut r, home, speed) {
                        r = Raven::idle();
                    }
                }
                super::data::RavenMove::Storm { pecks, heal_pct } => {
                    if r.step < 2 * pecks {
                        if r.step % 2 == 0 {
                            // fly to the side the next peck comes from: left, right, left...
                            let side = if (r.step / 2) % 2 == 0 { -1 } else { 1 };
                            if Self::raven_toward(&mut r, (foe.x + side * 26 * PX, foe.y + 38 * PX), speed) {
                                r.step += 1;
                                r.timer = 0;
                            }
                        } else {
                            // peck: faces the opponent, strikes after a few frames and stays a moment
                            if foe.x != r.x {
                                r.facing = if foe.x > r.x { 1 } else { -1 };
                            }
                            if r.timer == 4 {
                                hit = Some(if r.x <= foe.x { 1 } else { -1 });
                            }
                            if r.timer >= 10 {
                                r.step += 1;
                                r.timer = 0;
                            }
                        }
                    } else if r.step == 2 * pecks {
                        // back above its owner with a rose in its beak
                        r.rose.get_or_insert(Rose { x: r.x, y: r.y, falling: false });
                        if Self::raven_toward(&mut r, (me.x, me.y + 100 * PX), speed) {
                            r.step += 1;
                            r.timer = 0;
                        }
                    } else if r.step == 2 * pecks + 1 {
                        // lets go of the rose and waits for it to land
                        if r.timer == 1 {
                            if let Some(rose) = r.rose.as_mut() {
                                rose.falling = true;
                            }
                        }
                        if r.rose.is_none() || r.timer > 60 {
                            r.step += 1;
                            r.timer = 0;
                        }
                    } else if Self::raven_toward(&mut r, home, speed) {
                        r = Raven::idle();
                    }
                    // the rose: carried under the raven, then falling onto the owner's head
                    if let Some(mut rose) = r.rose {
                        if rose.falling {
                            rose.x = me.x;
                            rose.y -= 4 * PX;
                            if rose.y <= me.y + 56 * PX {
                                heal = self.fighters[owner].max_health * heal_pct / 100;
                                r.rose = None;
                            } else {
                                r.rose = Some(rose);
                            }
                        } else if r.active {
                            rose.x = r.x;
                            rose.y = r.y - 6 * PX;
                            r.rose = Some(rose);
                        }
                    }
                }
            }
            // a rose still falling when the raven goes home keeps falling on its own
            self.ravens[owner] = r;
            if heal > 0 {
                let f = &mut self.fighters[owner];
                f.health = (f.health + heal).min(f.max_health);
            }
            if let Some(dir) = hit {
                let mv = &chars[owner].moves[r.move_index];
                let defender = 1 - owner;
                let blocked = self.can_block(defender, inputs[defender]);
                let blocked = self.apply_hit(defender, mv, dir, blocked, r.power);
                self.gain_meter(owner, mv, mv.damage * r.power / 100, blocked, chars[owner].meter_gain_pct.unwrap_or(100));
                self.register_light_hit_as(owner, mv, blocked, r.power == 150);
            }
        }
    }

    /// Integer square root (no floats in the simulation).
    fn isqrt(n: u64) -> u64 {
        if n < 2 {
            return n;
        }
        let mut x = n;
        let mut y = (x + 1) / 2;
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        x
    }

    /// Move `r` towards a point at `speed` subpixels a frame; true once it's there.
    fn raven_toward(r: &mut Raven, target: (i32, i32), speed: i32) -> bool {
        let (dx, dy) = ((target.0 - r.x) as i64, (target.1 - r.y) as i64);
        let dist = Self::isqrt((dx * dx + dy * dy) as u64) as i64;
        if dx != 0 {
            r.facing = if dx > 0 { 1 } else { -1 };
        }
        if dist <= speed as i64 {
            r.x = target.0;
            r.y = target.1;
            return true;
        }
        r.x += (dx * speed as i64 / dist) as i32;
        r.y += (dy * speed as i64 / dist) as i32;
        false
    }

    /// A move with a `low_hitbox` picks high or low at random when it starts (from the frame and where
    /// everyone stands, so it's the same on every machine). The special spends the meter when the lash comes out.
    fn start_lashes(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            if f.state != FighterState::Attack {
                continue;
            }
            let mv = &chars[owner].moves[f.attack];
            if mv.low_hitbox.is_none() {
                continue;
            }
            if f.state_frame == 0 {
                let mut h = (self.frame).wrapping_mul(2_654_435_761)
                    ^ (self.fighters[0].x as u32).wrapping_mul(40_503)
                    ^ (self.fighters[1].x as u32).wrapping_mul(97_001)
                    ^ (self.fighters[owner].health as u32).wrapping_mul(31)
                    ^ owner as u32;
                h = (h ^ (h >> 15)).wrapping_mul(2_246_822_519);
                self.fighters[owner].aim_low = (h >> 13) & 1 == 1;
            } else if f.state_frame == mv.startup + 1 && mv.button == super::data::Button::Special {
                self.fighters[owner].meter = 0;
            }
        }
    }

    /// The portal special (see `MoveData::portal`): at `vanish` she is gone, the meter is spent and the far portal
    /// opens behind the opponent; when the startup ends she is moved there (still unseen) facing them, so the grab
    /// comes out of the far portal. If the grab misses she steps out when its window is over.
    fn update_portals(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            let portal = if f.state == FighterState::Attack { chars[owner].moves[f.attack].portal } else { None };
            let Some(pd) = portal else {
                let f = &mut self.fighters[owner];
                f.hidden = false;
                f.portal_x = None;
                continue;
            };
            let mv = &chars[owner].moves[f.attack];
            let (frame, foe_x) = (f.state_frame, self.fighters[1 - owner].x);
            let f = &mut self.fighters[owner];
            if frame == pd.vanish {
                f.hidden = true;
                f.meter = 0;
            }
            if frame == pd.aim_frame(mv.startup) {
                let dir = if foe_x >= f.x { 1 } else { -1 };
                let limit = Self::STAGE_HALF_WIDTH - Fighter::HALF_WIDTH;
                f.portal_x = Some((foe_x + dir * pd.gap * PX).clamp(-limit, limit));
            }
            if frame == mv.startup {
                if let Some(x) = f.portal_x {
                    if x != foe_x {
                        f.facing = if foe_x > x { 1 } else { -1 };
                    }
                    f.x = x;
                    f.y = 0;
                    f.vx = 0;
                }
            }
            if f.hidden && frame > mv.startup + mv.active {
                f.hidden = false;
            }
        }
    }

    /// A bat turning back bursts into smoke that stuns the opponent if they are close.
    fn apply_poofs(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let (Some(bat), true) = (chars[owner].bat_form(), self.fighters[owner].poof) else { continue };
            let x = self.fighters[owner].x;
            let foe = &mut self.fighters[1 - owner];
            let hit = (foe.x - x).abs() <= bat.radius * PX && !foe.invulnerable() && foe.state != FighterState::Ko;
            if hit {
                foe.start_stun(FighterState::Hitstun, bat.stun, 0);
            }
            if let Some(left) = chars[owner].bat_move().and_then(|i| self.fighters[owner].cooldowns.get_mut(i)) {
                *left = if hit { bat.hit_cooldown } else { bat.miss_cooldown };
            }
        }
    }

    /// A whip that connects pulls the attacker towards the opponent, or the opponent towards the attacker,
    /// until they are `pull_gap` pixels apart. A blocked hit only pulls the attacker half way.
    fn apply_pulls(&mut self, attacker: usize, mv: &MoveData, blocked: bool) {
        if !mv.pull_self && !mv.pull_foe {
            return;
        }
        let (ax, dx) = (self.fighters[attacker].x, self.fighters[1 - attacker].x);
        let travel = (ax - dx).abs() - mv.pull_gap * PX;
        if travel <= 0 {
            return;
        }
        let towards_foe = if dx > ax { 1 } else { -1 };
        if mv.pull_self {
            let amount = if blocked { travel * Self::PULL_BLOCKED_PCT / 100 } else { travel };
            self.fighters[attacker].pull = towards_foe * amount;
        }
        if mv.pull_foe && !blocked {
            self.fighters[1 - attacker].pull = -towards_foe * travel;
        }
    }

    /// Damage and stun `defender` with `mv`, pushing them in direction `dir`.
    fn apply_hit(&mut self, defender: usize, mv: &MoveData, dir: i32, blocked: bool, power: i32) -> bool {
        let d = &mut self.fighters[defender];
        d.hitstop = mv.hitstop;
        let push = dir * mv.knockback * PX;
        let damage = mv.damage * power / 100;
        // a guard holds against lights for good; heavier moves break it after so many hits
        let mut blocked = blocked;
        if blocked && mv.guard_need() > 0 {
            d.shield_hits += 1;
            d.shield_timer = 40;
            if d.shield_hits >= mv.guard_need() {
                blocked = false;
                d.shield_hits = 0;
                d.block_disabled = Fighter::BLOCK_DISABLED_FRAMES;
            }
        }
        if blocked {
            d.health = (d.health - damage / Self::CHIP_DIVISOR).max(1);
            d.start_stun(FighterState::Blockstun, mv.blockstun, push);
        } else {
            d.health -= damage;
            d.combo = 0;
            if d.health <= 0 {
                d.start_stun(FighterState::Ko, 0, push * 2);
            } else {
                d.start_stun(FighterState::Hitstun, mv.hitstun, push);
            }
        }
        blocked
    }

    /// Launch the next shot of any attack whose startup is over. A volley fires `volley_gap` frames apart;
    /// the special spends the meter with its first shot.
    fn launch_projectiles(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            if self.fighters[owner].state != FighterState::Attack {
                self.fighters[owner].lob_target = None;
                continue;
            }
            // a lobbing special locks in where it will land well before it launches
            {
                let f = &self.fighters[owner];
                let mv = &chars[owner].moves[f.attack];
                if let Some(p) = mv.projectile.as_ref().filter(|p| p.lob) {
                    if f.lob_target.is_none() && f.state_frame >= p.lock_frame {
                        let target_x = self.fighters[1 - owner].x;
                        self.fighters[owner].lob_target = Some(target_x);
                        // a portal over the opponent opens where they stand
                        if p.over_target {
                            self.fighters[owner].moon = Some((target_x, p.moon_height * PX, 0));
                            self.fighters[owner].portal = true;
                        }
                    }
                }
            }
            let f = &self.fighters[owner];
            let mv = &chars[owner].moves[f.attack];
            let Some(p) = &mv.projectile else { continue };
            if mv.channel.is_some() {
                continue; // fired by fire_channels
            }
            let total = mv.volley.max(1);
            if f.shots >= total || f.state_frame <= mv.startup + mv.volley_delay + f.shots * mv.volley_gap {
                continue;
            }
            let (mut x, mut y) = (f.x + f.facing * p.x * PX, f.y + p.y * PX);
            let (move_index, frames) = (f.attack, p.travel_frames.max(1) as i32);
            let target = &self.fighters[1 - owner];
            let mut land_x = f.lob_target.unwrap_or(target.x);
            // each shot of a volley lands somewhere else near the locked spot
            if p.lob && p.spread > 0 {
                land_x += ((f.shots as i32 * 37 + 11) % (2 * p.spread + 1) - p.spread) * PX;
            }
            // a lob starts at the moon and comes down where the opponent is standing
            if p.lob {
                if let Some((mx, my, _)) = f.moon {
                    (x, y) = (mx, my);
                    // from a portal overhead it falls straight down
                    if p.over_target {
                        x = land_x;
                    }
                }
            }
            let (mut vx, mut vy) = ((if p.lob { land_x } else { target.x } - x) / frames, (if p.lob { 0 } else if p.aim_ground { 0 } else { target.y + p.aim_y * PX } - y) / frames);
            // straight shots fly ahead at a fixed speed; their lifetime limits the range
            if !p.aimed && !p.lob {
                vx = if p.speed_tenths > 0 { f.facing * p.speed_tenths * PX / 10 } else { f.facing * if p.speed > 0 { p.speed } else { 5 } * PX };
                vy = 0;
            }
            if p.homing {
                (vx, vy) = Self::missile_launch_velocity(p, f.facing);
            }
            let mut power = if mv.charge_frames > 0 { 100 + f.charge as i32 * mv.charge_bonus_pct / mv.charge_frames as i32 } else { 100 };
            if f.finisher {
                power = power * 150 / 100;
            }
            self.projectiles.push(Projectile { owner, x, y, vx, vy, age: 0, move_index, land_x, power });
            let f = &mut self.fighters[owner];
            f.shots += 1;
            f.launched = true;
            if mv.button == super::data::Button::Special {
                f.meter = 0;
            }
        }
    }

    /// A move that clears projectiles (the staff's shockwave) destroys every enemy projectile in its
    /// horizontal reach while it is active, whatever the height up to a little above the head, but
    /// not `melee` ones.
    fn clear_projectiles(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            if f.state != FighterState::Attack {
                continue;
            }
            let mv = &chars[owner].moves[f.attack];
            if !mv.clears_projectiles || !mv.is_active(f.state_frame) {
                continue;
            }
            let r = mv.hitbox;
            let (x0, x1) = if f.facing > 0 { (f.x + r.x * PX, f.x + (r.x + r.w) * PX) } else { (f.x - (r.x + r.w) * PX, f.x - r.x * PX) };
            let top = f.y + 100 * PX;
            let bottom = f.y - 10 * PX;
            let others = chars[1 - owner];
            let frame = self.frame;
            let mut gone = Vec::new();
            self.projectiles.retain(|p| {
                if p.owner == owner {
                    return true;
                }
                let melee = others.moves[p.move_index].projectile.as_ref().is_some_and(|d| d.melee);
                let inside = p.x >= x0 && p.x <= x1 && p.y >= bottom && p.y <= top;
                if inside && !melee {
                    gone.push((p.x, p.y, frame));
                }
                melee || !inside
            });
            self.cleared.extend(gone);
        }
    }

    /// A heat-seeking missile's speed in subpixels a frame, and how it leaves the launcher: up and
    /// forward, to curve round towards its target.
    fn missile_speed(p: &super::data::ProjectileData) -> i32 {
        if p.speed_tenths > 0 {
            p.speed_tenths * PX / 10
        } else {
            p.speed.max(1) * PX
        }
    }

    fn missile_launch_velocity(p: &super::data::ProjectileData, facing: i32) -> (i32, i32) {
        let s = Self::missile_speed(p);
        (facing * s / 2, s * 87 / 100)
    }

    /// A held move (reading a scroll) fires its projectile at held frame `first` and then every `every`
    /// frames, for as long as the button stays down.
    /// A beam of light locks in where the opponent stands at its lock frame, and forgets it when the
    /// move is over.
    fn lock_beams(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            let beam = (f.state == FighterState::Attack).then(|| chars[owner].moves[f.attack].beam).flatten();
            match beam {
                Some(b) if f.beam_lock.is_none() && f.state_frame >= b.lock_frame => {
                    let x = self.fighters[1 - owner].x;
                    self.fighters[owner].beam_lock = Some(x);
                    // the special spends the meter when the beam locks in
                    if chars[owner].moves[self.fighters[owner].attack].button == super::data::Button::Special {
                        self.fighters[owner].meter = 0;
                    }
                }
                None => self.fighters[owner].beam_lock = None,
                _ => {}
            }
        }
    }

    fn fire_channels(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            if f.state != FighterState::Attack {
                continue;
            }
            let mv = &chars[owner].moves[f.attack];
            let (Some(ch), Some(p)) = (mv.channel, mv.projectile.as_ref()) else { continue };
            // `charge` counts the frames the move has been held at the end of its startup.
            // (while it waits there the move sits one frame before the end of its startup)
            if f.state_frame + 1 != mv.startup || f.charge < ch.first + f.shots * ch.every {
                continue;
            }
            let (x, y) = (f.x + f.facing * p.x * PX, f.y + p.y * PX);
            let (vx, vy) = if p.homing { Self::missile_launch_velocity(p, f.facing) } else { (f.facing * Self::missile_speed(p), 0) };
            let move_index = f.attack;
            self.projectiles.push(Projectile { owner, x, y, vx, vy, age: 0, move_index, land_x: 0, power: 100 });
            self.fighters[owner].shots += 1;
            self.fighters[owner].launched = true;
        }
    }

    /// A curse move puts its curse on the opponent when its startup ends, spending the meter.
    fn apply_curses(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            if f.state != FighterState::Attack {
                continue;
            }
            let mv = &chars[owner].moves[f.attack];
            let Some(c) = mv.curse else { continue };
            // a curse that rides on a projectile (the thrown hat) is put on when that lands
            if mv.projectile.is_some() || f.state_frame != mv.startup + 1 || self.fighters[1 - owner].curse.is_some() {
                continue;
            }
            self.fighters[1 - owner].curse = Some(Curse {
                owner,
                left: true,
                right: true,
                age: 0,
                squeeze: 0,
                squeeze_frames: c.squeeze,
                duration: c.duration,
                damage: mv.damage,
            });
            self.fighters[owner].meter = 0;
        }
    }

    /// A summoning move brings out its minions when its startup ends, spending the meter.
    fn summon_minions(&mut self, chars: [&CharacterData; 2]) {
        for owner in 0..2 {
            let f = &self.fighters[owner];
            if f.state != FighterState::Attack {
                continue;
            }
            let mv = &chars[owner].moves[f.attack];
            let Some(md) = mv.minions else { continue };
            if f.state_frame != mv.startup + 1 {
                continue;
            }
            // slots whose goblin died are filled again
            for slot in 0..md.count {
                if self.minions.iter().any(|m| m.owner == owner && m.slot == slot) {
                    continue;
                }
                self.minions.push(Minion {
                    health: md.health,
                    hurt: 0,
                    owner,
                    slot,
                    x: f.x + f.facing * (24 + 16 * slot as i32) * PX,
                    mode: MinionMode::Wait,
                    timer: slot * md.interval / md.count.max(1),
                    age: 0,
                    move_index: f.attack,
                    bit: false,
                    facing: f.facing,
                });
            }
            self.fighters[owner].meter = 0;
        }
    }

    /// Minions swarm the opponent and bite on a timer; they leave when their time is up.
    fn update_minions(&mut self, chars: [&CharacterData; 2], inputs: [Input; 2]) {
        let old = std::mem::take(&mut self.minions);
        // (the survivors go back into a list that already has room, instead of growing it from nothing every step)
        self.minions.reserve(old.len());
        for mut m in old {
            let mv = &chars[m.owner].moves[m.move_index];
            let Some(md) = mv.minions else { continue };
            m.age += 1;
            m.timer += 1;
            m.hurt = m.hurt.saturating_sub(1);
            if m.age > md.lifetime || m.health <= 0 {
                continue;
            }
            let owner = &self.fighters[m.owner];
            let foe = &self.fighters[1 - m.owner];
            let _ = owner;
            let step = |from: i32, to: i32, speed: i32| from + (to - from).clamp(-speed * PX, speed * PX);
            // the swarm rushes the opponent and stays on them, each goblin taking its own spot and biting every `interval` frames
            match m.mode {
                MinionMode::Wait | MinionMode::Return | MinionMode::Charge => {
                    m.mode = MinionMode::Charge;
                    let spot = foe.x + (m.slot as i32 * 2 - md.count as i32 + 1) * 5 * PX;
                    m.x = step(m.x, spot, md.speed);
                    m.facing = if foe.x >= m.x { 1 } else { -1 };
                    let close = (foe.x - m.x).abs() <= md.reach * PX && foe.y < 30 * PX;
                    if close && m.timer >= md.interval && foe.state != FighterState::Ko && !foe.invulnerable() {
                        m.bit = true;
                        m.timer = 0;
                        let dir = m.facing;
                        let blocked = self.can_block(1 - m.owner, inputs[1 - m.owner]);
                        self.apply_hit(1 - m.owner, mv, dir, blocked, 100);
                    }
                }
            }
            self.minions.push(m);
        }
    }

    /// A hit on a cursed fighter removes the half of the curse on the attacker's side.
    fn mark_curse(&mut self, attacker: usize) {
        let ax = self.fighters[attacker].x;
        let d = &mut self.fighters[1 - attacker];
        let dx = d.x;
        if let Some(c) = d.curse.as_mut() {
            if c.owner == attacker && c.squeeze == 0 {
                if ax < dx {
                    c.left = false;
                } else {
                    c.right = false;
                }
            }
        }
    }

    /// Age curses, squeeze finished ones shut and deal their damage, let old ones fade.
    fn update_curses(&mut self) {
        for d in 0..2 {
            let Some(mut c) = self.fighters[d].curse else { continue };
            c.age += 1;
            if c.squeeze > 0 {
                c.squeeze += 1;
            } else if !c.left && !c.right {
                c.squeeze = 1;
            }
            if c.squeeze > c.squeeze_frames {
                self.fighters[d].curse = None;
                let victim = &mut self.fighters[d];
                victim.health -= c.damage;
                victim.hitstop = 10;
                let dir = if c.owner == 1 - d && self.fighters_x_of(c.owner) < self.fighters_x_of(d) { 1 } else { -1 };
                let victim = &mut self.fighters[d];
                if victim.health <= 0 {
                    victim.start_stun(FighterState::Ko, 0, dir * 6 * PX);
                } else {
                    victim.start_stun(FighterState::Hitstun, 26, dir * 3 * PX);
                }
            } else if c.squeeze == 0 && c.age > c.duration {
                self.fighters[d].curse = None;
            } else {
                self.fighters[d].curse = Some(c);
            }
        }
    }

    fn fighters_x_of(&self, i: usize) -> i32 {
        self.fighters[i].x
    }

    /// A light attack that connects (hit or blocked) advances the combo; the third is the finisher. A
    /// blocked finisher stuns its thrower.
    fn register_light_hit(&mut self, attacker: usize, mv: &MoveData, blocked: bool) {
        let was_finisher = self.fighters[attacker].finisher;
        self.register_light_hit_as(attacker, mv, blocked, was_finisher);
    }

    /// `register_light_hit`, told whether it was a finisher (the raven's claw lands some time after the
    /// move that sent it).
    fn register_light_hit_as(&mut self, attacker: usize, mv: &MoveData, blocked: bool, was_finisher: bool) {
        if mv.button != super::data::Button::Light {
            return;
        }
        {
            let f = &mut self.fighters[attacker];
            f.combo = if was_finisher { 3 } else { (f.combo + 1).min(2) };
            f.combo_timer = 0;
        }
        if was_finisher && !blocked {
            self.fighters[attacker].whip_boost = true;
        }
        if was_finisher && blocked {
            let dir = if self.fighters[attacker].x <= self.fighters[1 - attacker].x { -1 } else { 1 };
            let f = &mut self.fighters[attacker];
            f.finisher = false;
            f.combo = 0;
            f.start_stun(FighterState::Hitstun, Fighter::FINISHER_STUN, dir * 2 * PX);
        }
    }

    /// Fill a fighter's special meter for a hit they landed (half for a blocked one). Specials don't fill it.
    fn gain_meter(&mut self, owner: usize, mv: &MoveData, damage: i32, blocked: bool, rate_pct: i32) {
        if mv.button == super::data::Button::Special {
            return;
        }
        let gain = (damage / 3 + 4) * rate_pct / 100;
        let f = &mut self.fighters[owner];
        f.meter = (f.meter + if blocked { gain / 2 } else { gain }).min(Fighter::METER_MAX);
    }

    /// Fly projectiles forward: land on the opponent or fizzle out.
    fn move_projectiles(&mut self, inputs: [Input; 2], chars: [&CharacterData; 2]) {
        let old = std::mem::take(&mut self.projectiles);
        self.projectiles.reserve(old.len());
        for mut p in old {
            let owner = p.owner;
            let mv = &chars[owner].moves[p.move_index];
            let Some(data) = &mv.projectile else { continue };
            // a heat-seeking missile turns towards the opponent's chest before it moves
            let mut close = false;
            if data.homing {
                let foe = &self.fighters[1 - owner];
                let (dx, dy) = ((foe.x - p.x) as i64, (foe.y + 30 * PX - p.y) as i64);
                let dist = (Self::isqrt((dx * dx + dy * dy) as u64) as i64).max(1);
                let speed = Self::missile_speed(data) as i64;
                let turn = data.homing_turn.clamp(1, 100) as i64;
                p.vx += ((dx * speed / dist - p.vx as i64) * turn / 100) as i32;
                p.vy += ((dy * speed / dist - p.vy as i64) * turn / 100) as i32;
                let now = (Self::isqrt((p.vx as i64 * p.vx as i64 + p.vy as i64 * p.vy as i64) as u64) as i64).max(1);
                p.vx = (p.vx as i64 * speed / now) as i32;
                p.vy = (p.vy as i64 * speed / now) as i32;
                close = dist <= (data.detonate * PX) as i64;
            }
            p.x += p.vx;
            p.y += p.vy;
            p.age += 1;

            let defender = 1 - owner;
            let hurt = self.hurtbox(defender);
            // a lob comes down and splashes on the ground around where it lands
            if data.lob {
                if p.age >= data.travel_frames.max(1) {
                    let r = data.splash_radius * PX;
                    let low = self.fighters[defender].y < 36 * PX;
                    if low && hurt[0] < p.land_x + r && p.land_x - r < hurt[2] {
                        let dir = if self.fighters[defender].x >= p.land_x { 1 } else { -1 };
                        let blocked = self.can_block(defender, inputs[defender]) && !data.unblockable;
                        let blocked = self.apply_hit(defender, mv, dir, blocked, p.power);
                        self.gain_meter(owner, mv, mv.damage * p.power / 100, blocked, chars[owner].meter_gain_pct.unwrap_or(100));
                    }
                } else {
                    self.projectiles.push(p);
                }
                continue;
            }
            let (hw, hh) = (data.size.0 * PX / 2, data.size.1 * PX / 2);
            let hit = close || (p.x - hw < hurt[2] && hurt[0] < p.x + hw && p.y - hh < hurt[3] && hurt[1] < p.y + hh);
            let floor = data.aim_ground && p.y <= 0;
            let gone = floor || p.age >= data.lifetime || p.x.abs() > Self::STAGE_HALF_WIDTH + 80 * PX || p.y < -10 * PX;

            if hit {
                let blocked = self.can_block(defender, inputs[defender]) && !data.unblockable;
                let dir = if p.vx >= 0 { 1 } else { -1 };
                if let Some(c) = mv.curse {
                    // a thrown hat curses whoever it lands on instead of hurting them (the damage comes when the curse closes)
                    let mut landed = mv.clone();
                    landed.damage = 0;
                    let blocked = self.apply_hit(defender, &landed, dir, blocked, p.power);
                    if !blocked && self.fighters[defender].curse.is_none() {
                        self.fighters[defender].curse = Some(Curse {
                            owner,
                            left: true,
                            right: true,
                            age: 0,
                            squeeze: 0,
                            squeeze_frames: c.squeeze,
                            duration: c.duration,
                            damage: mv.damage,
                        });
                    }
                } else {
                    let shot = mv.as_shot();
                    let blocked = self.apply_hit(defender, &shot, dir, blocked, p.power);
                    if mv.bat_form.is_some() && !blocked {
                        // she can turn into a bat until the heavy's cooldown runs out
                        let f = &mut self.fighters[owner];
                        f.bat_ready = f.cooldown_left(p.move_index).max(1);
                    }
                    if !blocked && mv.futility > 0 {
                        self.fighters[defender].futile = mv.futility;
                    }
                    self.gain_meter(owner, &shot, shot.damage * p.power / 100, blocked, chars[owner].meter_gain_pct.unwrap_or(100));
                    self.register_light_hit(owner, mv, blocked);
                    // a shot counts as a hit from its owner's side for the opponent's curse
                    self.mark_curse(owner);
                }
            } else if self.hit_minion(defender, [p.x - hw, p.y - hh, p.x + hw, p.y + hh], mv.shot_damage.unwrap_or(mv.damage) * p.power / 100) {
                // the shot is spent on the goblin
            } else if !gone {
                self.projectiles.push(p);
            } else if floor {
                self.impacts.push((p.x, 0, self.frame));
            }
        }
    }

    fn clamp_to_stage(&mut self) {
        let limit = Self::STAGE_HALF_WIDTH - Fighter::HALF_WIDTH;
        for f in &mut self.fighters {
            f.x = f.x.clamp(-limit, limit);
        }
    }

    /// Free fighters turn to face the opponent; everyone else keeps their facing.
    fn face_each_other(&mut self) {
        let (x0, x1) = (self.fighters[0].x, self.fighters[1].x);
        if x0 == x1 {
            return;
        }
        let toward = |from: i32, to: i32| if to > from { 1 } else { -1 };
        if self.fighters[0].is_neutral() {
            self.fighters[0].facing = toward(x0, x1);
        }
        if self.fighters[1].is_neutral() {
            self.fighters[1].facing = toward(x1, x0);
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::data::Button;
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn ayz() -> CharacterData {
        CharacterData::from_ron(include_str!("classic_fighter.ron")).unwrap()
    }

    fn evilayz() -> CharacterData {
        CharacterData::from_ron(include_str!("classic_opponent.ron")).unwrap()
    }

    fn step(s: &mut GameState, a: u16, b: u16) {
        s.step([Input(a), Input(b)], [&ayz(), &evilayz()]);
    }

    fn hash(state: &GameState) -> u64 {
        let mut h = DefaultHasher::new();
        state.hash(&mut h);
        h.finish()
    }

    /// Small deterministic random generator so the tests need no extra crates.
    fn lcg(seed: &mut u64) -> u16 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*seed >> 33) as u16
    }

    fn run_random(frames: u32, mut seed: u64) -> GameState {
        let mut state = GameState::new();
        for _ in 0..frames {
            let a = lcg(&mut seed) & 0x7f;
            let b = lcg(&mut seed) & 0x7f;
            step(&mut state, a, b);
        }
        state
    }

    /// Two fighters a few pixels apart, so a normal reaches.
    fn close_up() -> GameState {
        let mut s = GameState::new();
        s.fighters[0].x = -18 * PX;
        s.fighters[1].x = 18 * PX;
        s
    }

    #[test]
    fn character_files_parse_and_cover_every_button() {
        for c in [ayz(), evilayz()] {
            for b in [Button::Light, Button::Heavy, Button::Special] {
                assert!(c.find_move(b, false, false).is_some(), "{} lacks {:?}", c.name, b);
            }
        }
    }

    #[test]
    fn same_inputs_give_same_state() {
        assert_eq!(hash(&run_random(10_000, 7)), hash(&run_random(10_000, 7)));
    }

    #[test]
    fn clone_then_step_matches_original() {
        let mut a = run_random(500, 3);
        let mut b = a.clone();
        for i in 0..200u16 {
            step(&mut a, i & 0x7f, (i * 7) & 0x7f);
            step(&mut b, i & 0x7f, (i * 7) & 0x7f);
        }
        assert_eq!(hash(&a), hash(&b));
    }

    #[test]
    fn walking_moves_toward_input_direction() {
        let mut s = GameState::new();
        let start = s.fighters[0].x;
        for _ in 0..10 {
            step(&mut s, Input::RIGHT, 0);
        }
        assert!(s.fighters[0].x > start);
        assert_eq!(s.fighters[0].state, FighterState::Walk);
    }

    #[test]
    fn jump_leaves_ground_and_lands() {
        let mut s = GameState::new();
        step(&mut s, Input::UP, 0);
        assert!(s.fighters[0].y > 0);
        assert_eq!(s.fighters[0].state, FighterState::Jump);
        for _ in 0..120 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].y, 0);
        assert_eq!(s.fighters[0].state, FighterState::Idle);
    }

    #[test]
    fn fighters_stay_inside_stage() {
        for seed in 0..20 {
            let mut state = GameState::new();
            let mut rng = seed;
            let limit = GameState::STAGE_HALF_WIDTH - Fighter::HALF_WIDTH;
            for _ in 0..3_000 {
                let (a, b) = (lcg(&mut rng) & 0x7f, lcg(&mut rng) & 0x7f);
                step(&mut state, a, b);
                for f in &state.fighters {
                    assert!(f.x.abs() <= limit);
                }
            }
        }
    }

    #[test]
    fn fighters_can_walk_through_each_other() {
        let mut s = GameState::new();
        for _ in 0..200 {
            step(&mut s, Input::RIGHT, 0);
        }
        assert!(s.fighters[0].x > s.fighters[1].x, "P1 should have passed P2");
        step(&mut s, 0, 0);
        assert_eq!(s.fighters[0].facing, -1, "after crossing over they turn to face back");
    }

    #[test]
    fn inputs_only_affect_their_own_fighter_when_apart() {
        for (p1, p2) in [(Input::RIGHT, 0), (Input::UP, 0), (Input::DOWN, 0), (0, Input::LEFT), (0, Input::UP)] {
            let mut s = GameState::new();
            let before = s.clone();
            step(&mut s, p1, p2);
            let other = if p1 != 0 { 1 } else { 0 };
            assert_eq!(s.fighters[other].x, before.fighters[other].x);
            assert_eq!(s.fighters[other].y, before.fighters[other].y);
        }
    }

    #[test]
    fn grounded_fighters_face_each_other() {
        let mut s = GameState::new();
        step(&mut s, 0, 0);
        assert_eq!(s.fighters[0].facing, 1);
        assert_eq!(s.fighters[1].facing, -1);
    }

    #[test]
    fn jab_hits_for_its_damage_and_causes_hitstun() {
        let mut s = close_up();
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..8 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH - 30);
        assert!(matches!(s.fighters[1].state, FighterState::Hitstun | FighterState::Idle));
        assert_eq!(s.fighters[0].health, Fighter::MAX_HEALTH);
    }

    #[test]
    fn a_jab_that_connects_can_be_cancelled_into_the_cross_but_a_whiff_cannot() {
        let mut s = close_up();
        step(&mut s, Input::LIGHT, 0);
        let mut connected = false;
        for _ in 0..12 {
            step(&mut s, 0, 0);
            if s.fighters[0].hit_done {
                connected = true;
                break;
            }
        }
        assert!(connected, "the jab never connected");
        // The press lands during hitstop and is remembered until it ends.
        step(&mut s, Input::HEAVY, 0);
        for _ in 0..12 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].state, FighterState::Attack);
        assert_eq!(s.fighters[0].attack, 2, "should be the Cross");

        let mut whiff = GameState::new();
        step(&mut whiff, Input::LIGHT, 0);
        for _ in 0..6 {
            step(&mut whiff, 0, 0);
        }
        step(&mut whiff, Input::HEAVY, 0);
        assert_eq!(whiff.fighters[0].attack, 0, "a whiffed jab must not cancel");
    }

    fn cowboy() -> CharacterData {
        CharacterData::from_ron(include_str!("../../assets/characters/cowboy.ron")).unwrap()
    }

    #[test]
    fn the_cowboys_combo_is_two_finger_shots_and_then_a_lasso_that_drags_and_stuns() {
        let c = cowboy();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 90 * PX;
        let mut thrown = Vec::new();
        for _ in 0..3 {
            s.step([Input(Input::LIGHT), Input(0)], [&c, &evilayz()]);
            thrown.push(c.moves[s.fighters[0].attack].name.clone());
            // let it land and the foe recover a little (the next light has to come inside the combo window)
            for _ in 0..28 {
                s.step([Input(0), Input(0)], [&c, &evilayz()]);
            }
            s.fighters[1].x = 90 * PX;
            s.fighters[1].stun = 0;
        }
        assert_eq!(thrown, ["Finger Shot", "Finger Shot 2", "Lasso"]);
    }

    #[test]
    fn the_lasso_pulls_the_opponent_in_a_little_and_stuns_them_for_a_long_time() {
        let c = cowboy();
        let lasso = c.moves.iter().find(|m| m.name == "Lasso").unwrap();
        assert!(lasso.pull_foe && lasso.hitstun >= 40 && lasso.rope);
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 100 * PX;
        s.fighters[0].combo = 2;
        s.fighters[0].combo_timer = 20;
        s.step([Input(Input::LIGHT), Input(0)], [&c, &evilayz()]);
        assert_eq!(c.moves[s.fighters[0].attack].name, "Lasso");
        for _ in 0..24 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        let gap = (s.fighters[1].x - s.fighters[0].x).abs() / PX;
        assert!(gap <= lasso.pull_gap + 2, "the lasso should drag him in to about {} px, he is {gap} px away", lasso.pull_gap);
        assert_eq!(s.fighters[1].state, FighterState::Hitstun, "and he is stunned");
    }

    #[test]
    fn the_gun_boost_splashes_the_ground_launches_him_and_then_shoots_three_weak_bullets() {
        let c = cowboy();
        let boost = c.moves.iter().find(|m| m.name == "Gun Boost").unwrap();
        assert_eq!(boost.shot_damage, Some(11), "his bullets are weak");
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 40 * PX;
        s.step([Input(Input::HEAVY), Input(0)], [&c, &evilayz()]);
        let (mut highest, mut splash_taken) = (0, None);
        for _ in 0..80 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
            highest = highest.max(s.fighters[0].y);
            if splash_taken.is_none() && s.fighters[1].health < Fighter::MAX_HEALTH && s.fighters[0].shots == 0 {
                splash_taken = Some(Fighter::MAX_HEALTH - s.fighters[1].health);
            }
        }
        assert!(highest > 150 * PX, "he should be launched way up, got {} px", highest / PX);
        assert_eq!(splash_taken, Some(boost.damage), "the splash on the ground comes first, before any bullet");
        assert_eq!(s.fighters[0].shots, 3, "then the volley of three");
        let taken = Fighter::MAX_HEALTH - s.fighters[1].health;
        assert!((taken - boost.damage) % boost.shot_damage.unwrap() == 0 && taken >= boost.damage, "the rest is bullets of {:?} each, took {taken}", boost.shot_damage);
    }

    #[test]
    fn finger_shots_take_the_curse_halves_away_like_any_other_hit() {
        let c = cowboy();
        let mut s = GameState::new();
        s.fighters[0].x = -100 * PX;
        s.fighters[1].x = 60 * PX;
        s.fighters[1].curse = Some(Curse { owner: 0, left: true, right: true, age: 0, squeeze: 0, squeeze_frames: 24, duration: 420, damage: 200 });
        s.step([Input(Input::LIGHT), Input(0)], [&c, &evilayz()]);
        for _ in 0..60 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        assert!(s.fighters[1].health < Fighter::MAX_HEALTH, "the shot should have landed");
        let curse = s.fighters[1].curse.expect("still cursed");
        assert!(!curse.left && curse.right, "the shot came from the left, so the left half goes");
    }

    #[test]
    fn the_gun_boost_bullets_go_for_the_floor_under_the_enemy_so_jumping_dodges_them() {
        let c = cowboy();
        let run = |jump: bool| {
            let mut s = GameState::new();
            s.fighters[0].x = -50 * PX;
            s.fighters[1].x = 70 * PX;
            s.step([Input(Input::HEAVY), Input(0)], [&c, &evilayz()]);
            let mut volley_started = false;
            for frame in 0..120 {
                // the enemy jumps just before the bullets come out (they leave at frames 39, 47 and 55)
                let up = jump && (30..34).contains(&frame);
                s.step([Input(0), Input(if up { Input::UP } else { 0 })], [&c, &evilayz()]);
                volley_started |= s.fighters[0].shots > 0;
            }
            assert!(volley_started, "the volley went out");
            assert_eq!(s.fighters[0].shots, 3);
            Fighter::MAX_HEALTH - s.fighters[1].health
        };
        assert!(run(false) > 0, "a standing enemy gets hit");
        assert_eq!(run(true), 0, "an enemy in the air is missed");
    }

    #[test]
    fn the_gun_boost_floats_down_slowly_and_lands_the_floor_shots_as_sparks() {
        let c = cowboy();
        let mut s = GameState::new();
        s.fighters[0].x = -50 * PX;
        s.fighters[1].x = 70 * PX;
        s.step([Input(Input::HEAVY), Input(0)], [&c, &evilayz()]);
        let (mut apex_frame, mut apex, mut fall_speed, mut impacts) = (0, 0, 0, 0);
        for frame in 0..140 {
            // the enemy jumps over the bullets, so they land on the floor
            let up = (30..34).contains(&frame);
            s.step([Input(0), Input(if up { Input::UP } else { 0 })], [&c, &evilayz()]);
            let f = &s.fighters[0];
            if f.y > apex {
                apex = f.y;
                apex_frame = frame;
            } else if frame > apex_frame + 5 && f.y > 0 && f.state == FighterState::Attack {
                fall_speed = fall_speed.max(-f.vy);
            }
            impacts += s.impacts.iter().filter(|i| i.2 == s.frame).count();
        }
        assert!(apex >= 150 * PX, "{} px", apex / PX);
        assert!(fall_speed <= 600, "he floats down, fastest fall {fall_speed} subpixels a frame");
        assert_eq!(impacts, 3, "each of the three bullets lands on the floor");
    }

    #[test]
    fn the_gun_boost_can_be_used_while_being_hit_but_not_other_moves_and_not_on_cooldown() {
        let c = cowboy();
        let boost = c.moves.iter().position(|m| m.name == "Gun Boost").unwrap();
        let mut s = GameState::new();
        s.fighters[0].start_stun(FighterState::Hitstun, 40, 0);
        s.step([Input(Input::LIGHT), Input(0)], [&c, &evilayz()]);
        assert_eq!(s.fighters[0].state, FighterState::Hitstun, "a light doesn't get out of it");
        s.step([Input(Input::HEAVY), Input(0)], [&c, &evilayz()]);
        assert_eq!(s.fighters[0].state, FighterState::Attack);
        assert_eq!(s.fighters[0].attack, boost, "the boost takes him out of the stun");
        // on cooldown it stays unusable
        let mut s = GameState::new();
        s.fighters[0].cooldowns[boost] = 100;
        s.fighters[0].start_stun(FighterState::Hitstun, 40, 0);
        s.step([Input(Input::HEAVY), Input(0)], [&c, &evilayz()]);
        assert_eq!(s.fighters[0].state, FighterState::Hitstun);
    }

    #[test]
    fn every_cowboy_heavy_and_the_hat_throw_have_cooldowns() {
        let c = cowboy();
        for m in c.moves.iter().filter(|m| m.button != Button::Light) {
            assert!(m.cooldown > 0, "{} needs a cooldown", m.name);
        }
    }

    #[test]
    fn the_thrown_hat_curses_and_the_curse_needs_a_hit_from_each_side_and_then_squeezes_shut_for_the_damage() {
        let c = cowboy();
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        s.fighters[1].x = s.fighters[0].x + 30 * PX;
        s.step([Input(Input::SPECIAL), Input(0)], [&c, &evilayz()]);
        for _ in 0..30 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        let curse = s.fighters[1].curse.expect("the opponent should be cursed");
        assert!(curse.left && curse.right);
        assert_eq!(s.fighters[0].meter, 0, "the curse spends the meter");
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH, "casting it does no damage by itself");

        // A punch from the left takes the left half away.
        for _ in 0..20 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        s.fighters[0].x = s.fighters[1].x - 20 * PX;
        // (the low slap: a finger shot would start past someone this close)
        s.step([Input(Input::LIGHT | Input::DOWN), Input(0)], [&c, &evilayz()]);
        for _ in 0..30 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        let curse = s.fighters[1].curse.expect("still cursed");
        assert!(!curse.left && curse.right, "only the left half should be gone");
        let before = s.fighters[1].health;

        // A punch from the right takes the other half; the ring squeezes and then the damage lands.
        s.fighters[0].x = s.fighters[1].x + 20 * PX;
        s.step([Input(0), Input(0)], [&c, &evilayz()]); // he turns to face the other way
        s.step([Input(Input::LIGHT | Input::DOWN), Input(0)], [&c, &evilayz()]);
        for _ in 0..16 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        let curse = s.fighters[1].curse.expect("squeezing");
        assert!(curse.squeeze > 0, "both halves are gone, the ring should be squeezing");
        for _ in 0..40 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        assert!(s.fighters[1].curse.is_none());
        assert!(before - s.fighters[1].health >= 200, "the curse should deal its 200 damage, took {}", before - s.fighters[1].health);
    }

    #[test]
    fn the_cowboy_has_a_third_less_health_and_keeps_it_between_rounds() {
        let c = cowboy();
        assert_eq!(c.health, Some(667));
        let mut s = GameState::new();
        s.set_health([c.health.unwrap(), Fighter::MAX_HEALTH]);
        assert_eq!(s.fighters[0].health, 667);
        s.fighters[1].health = 0;
        s.phase = Phase::RoundEnd(GameState::ROUND_END_FRAMES - 1);
        s.step([Input(0), Input(0)], [&c, &evilayz()]);
        assert_eq!(s.fighters[0].health, 667, "his health resets to 667, not 1000");
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH);
    }

    fn scribe() -> CharacterData {
        CharacterData::from_ron(include_str!("../../assets/characters/scribe.ron")).unwrap()
    }

    /// P1 throws `n` jabs at P2 (standing right next to them), waiting for each to finish.
    fn jabs(s: &mut GameState, p2: u16, n: u32) {
        for _ in 0..n {
            s.fighters[1].x = s.fighters[0].x + 30 * PX;
            step(s, Input::LIGHT, p2);
            for _ in 0..30 {
                step(s, 0, p2);
            }
        }
    }

    #[test]
    fn the_third_light_attack_of_a_combo_is_a_finisher_that_hits_harder() {
        let mut s = close_up();
        let mut taken = Vec::new();
        let mut last = s.fighters[1].health;
        for _ in 0..3 {
            s.fighters[1].x = s.fighters[0].x + 30 * PX; // the knockback pushes them away each time
            step(&mut s, Input::LIGHT, 0);
            for _ in 0..30 {
                step(&mut s, 0, 0);
            }
            taken.push(last - s.fighters[1].health);
            last = s.fighters[1].health;
        }
        assert_eq!(taken, [30, 30, 45], "the finisher does 150%");
        assert_eq!(s.fighters[0].combo, 3, "three squares are lit when the finisher lands");
        // The display goes out soon after, and a new combo starts.
        for _ in 0..40 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].combo, 0);
    }

    #[test]
    fn a_combo_runs_out_if_you_wait_and_is_broken_by_getting_hit() {
        let mut s = close_up();
        jabs(&mut s, 0, 1);
        assert_eq!(s.fighters[0].combo, 1);
        for _ in 0..90 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].combo, 0, "too slow: the combo is gone");
        jabs(&mut s, 0, 1);
        assert_eq!(s.fighters[0].combo, 1);
        s.fighters[0].health = 1000;
        s.fighters[0].x = s.fighters[1].x - 10 * PX;
        // P2 hits P1 back.
        step(&mut s, 0, Input::LIGHT);
        for _ in 0..10 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].combo, 0, "being hit breaks the combo");
    }

    #[test]
    fn a_blocked_finisher_stuns_the_attacker_for_one_and_a_fifth_seconds_and_blocks_let_ten_percent_through() {
        let mut s = close_up();
        let before = s.fighters[1].health;
        jabs(&mut s, Input::BLOCK, 2);
        // Two blocked jabs: 10% of 30 each.
        assert_eq!(before - s.fighters[1].health, 3 + 3, "a block lets 10% of the damage through");
        // The third (the finisher) is blocked too, and the attacker is stunned.
        s.fighters[1].x = s.fighters[0].x + 30 * PX;
        step(&mut s, Input::LIGHT, Input::BLOCK);
        let mut stunned = 0;
        for _ in 0..100 {
            step(&mut s, 0, Input::BLOCK);
            if s.fighters[0].state == FighterState::Hitstun {
                stunned += 1;
            }
        }
        assert!(stunned >= 70 && stunned <= 80, "stunned for about 72 frames, got {stunned}");
    }

    #[test]
    fn a_broken_guard_cannot_block_for_two_and_a_half_seconds() {
        let mut s = close_up();
        // P1's lecture-like heavy breaks P2's guard.
        step(&mut s, Input::HEAVY, Input::BLOCK);
        for _ in 0..40 {
            step(&mut s, 0, Input::BLOCK);
        }
        assert!(s.fighters[1].block_disabled > 0, "the guard is broken and disabled");
        // Trying to guard now does nothing.
        step(&mut s, 0, Input::BLOCK);
        assert_ne!(s.fighters[1].state, FighterState::Guard);
        for _ in 0..140 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].block_disabled, 0, "about 2.4 seconds later it works again");
        step(&mut s, 0, Input::BLOCK);
        step(&mut s, 0, Input::BLOCK);
        assert_eq!(s.fighters[1].state, FighterState::Guard);
    }

    #[test]
    fn the_cowboys_special_meter_fills_much_more_slowly() {
        let c = cowboy();
        let fill = |data: &CharacterData| {
            let mut s = close_up();
            s.fighters[1].x = s.fighters[0].x + 30 * PX;
            s.step([Input(Input::LIGHT), Input(0)], [data, &evilayz()]);
            for _ in 0..20 {
                s.step([Input(0), Input(0)], [data, &evilayz()]);
            }
            s.fighters[0].meter
        };
        let (cowboy_meter, other) = (fill(&c), fill(&ayz()));
        assert!(cowboy_meter > 0 && cowboy_meter * 2 < other + 1, "cowboy {cowboy_meter} vs {other}");
    }

    fn anaran() -> CharacterData {
        CharacterData::from_ron(include_str!("../../assets/characters/anaran.ron")).unwrap()
    }

    #[test]
    fn anarans_goblins_stay_for_eight_seconds_and_bite_on_their_own() {
        let a = anaran();
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        s.fighters[1].x = s.fighters[0].x + 100 * PX;
        s.step([Input(Input::SPECIAL), Input(0)], [&a, &evilayz()]);
        let mut most = 0;
        let mut bitten = 0;
        let mut last = s.fighters[1].health;
        for _ in 0..600 {
            s.step([Input(0), Input(0)], [&a, &evilayz()]);
            most = most.max(s.minions.len());
            if s.fighters[1].health < last {
                bitten += 1;
            }
            last = s.fighters[1].health;
            // Keep the opponent standing in the same place so that every bite can land.
            s.fighters[1].x = s.fighters[0].x + 100 * PX;
        }
        assert_eq!(most, 4, "four goblins should come out");
        assert!(bitten >= 3, "they should bite several times on their own, bit {bitten}");
        assert!(s.minions.is_empty(), "and leave after about eight seconds");
        assert_eq!(s.fighters[0].meter, 0, "the summon spends the meter");
    }

    #[test]
    fn junkheads_heavy_whip_follows_a_jumping_opponent_a_little() {
        let j = junkhead();
        let mut s = GameState::new();
        // Far out of reach, so only the shape of the lash is looked at; the opponent's feet are
        // 45 px up, above where the plain lash (22..38 px) reaches.
        s.fighters[1].x = s.fighters[0].x + 300 * PX;
        s.step([Input(Input::HEAVY), Input(0)], [&j, &evilayz()]);
        let mut found = None;
        for _ in 0..30 {
            s.fighters[1].y = 45 * PX;
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
            if let Some(h) = s.hitbox(0, &j) {
                found = Some(h);
                break;
            }
        }
        let h = found.expect("the lash should become active");
        assert!(h[3] > 45 * PX, "the lash should have followed them up: {h:?}");
        assert!(h[3] < 45 * PX + 60 * PX, "but only a little");
    }

    #[test]
    fn after_a_connected_heavy_light_comes_out_at_once_and_the_next_lights_are_quick() {
        let e = junkhead();
        let mut s = GameState::new();
        s.fighters[1].x = s.fighters[0].x + 60 * PX;
        s.step([Input(Input::HEAVY), Input(0)], [&e, &ayz()]);
        for _ in 0..60 {
            if s.fighters[0].hit_done {
                break;
            }
            s.step([Input(0), Input(0)], [&e, &ayz()]);
        }
        assert!(s.fighters[0].hit_done, "the heavy should connect");
        let heavy = s.fighters[0].attack;
        // the very next frame the light button is pressed it must start (no recovery to wait out)
        s.step([Input(Input::LIGHT), Input(0)], [&e, &ayz()]);
        for _ in 0..15 {
            if s.fighters[0].attack != heavy {
                break;
            }
            s.step([Input(Input::LIGHT), Input(0)], [&e, &ayz()]);
        }
        let f = &s.fighters[0];
        assert!(f.attack != heavy && f.state == FighterState::Attack, "light should have cancelled the heavy");
        assert!(f.fast, "and be a quick one");
        assert_eq!(f.quick, 2, "leaving two more quick lights");
    }

    // ---- mobility: dash, double jump, rolls, ground pound

    fn tap(s: &mut GameState, flag: u16, with: u16) {
        s.step([Input(flag | with), Input(0)], [&ayz(), &evilayz()]);
        s.step([Input(with), Input(0)], [&ayz(), &evilayz()]);
    }

    #[test]
    fn double_tapping_a_direction_dashes() {
        let mut s = GameState::new();
        s.fighters[1].x = 900 * PX; // far away
        tap(&mut s, Input::RIGHT, 0);
        s.step([Input(Input::RIGHT), Input(0)], [&ayz(), &evilayz()]);
        assert_eq!(s.fighters[0].state, FighterState::Dash, "the second tap should dash");
        let x0 = s.fighters[0].x;
        for _ in 0..10 {
            s.step([Input(0), Input(0)], [&ayz(), &evilayz()]);
        }
        assert!(s.fighters[0].x - x0 > 30 * PX, "a dash covers real ground");
    }

    #[test]
    fn one_tap_only_walks() {
        let mut s = GameState::new();
        s.step([Input(Input::RIGHT), Input(0)], [&ayz(), &evilayz()]);
        assert_eq!(s.fighters[0].state, FighterState::Walk);
    }

    #[test]
    fn you_can_jump_twice_but_not_three_times() {
        let mut s = GameState::new();
        s.step([Input(Input::UP), Input(0)], [&ayz(), &evilayz()]);
        for _ in 0..12 {
            s.step([Input(0), Input(0)], [&ayz(), &evilayz()]);
        }
        let before = s.fighters[0].vy;
        s.step([Input(Input::UP), Input(0)], [&ayz(), &evilayz()]);
        assert!(s.fighters[0].vy > before, "the second press should launch again");
        assert!(s.fighters[0].double_jumped);
        s.step([Input(0), Input(0)], [&ayz(), &evilayz()]);
        let vy = s.fighters[0].vy;
        s.step([Input(Input::UP), Input(0)], [&ayz(), &evilayz()]);
        assert!(s.fighters[0].vy < vy, "a third press does nothing");
    }

    #[test]
    fn a_roll_is_untouchable_in_the_middle_and_works_in_the_air_once() {
        let mut s = GameState::new();
        s.fighters[1].x = 900 * PX;
        s.step([Input(Input::ROLL_RIGHT), Input(0)], [&ayz(), &evilayz()]);
        assert_eq!(s.fighters[0].state, FighterState::Roll);
        let mut safe = false;
        for _ in 0..8 {
            s.step([Input(0), Input(0)], [&ayz(), &evilayz()]);
            safe |= s.fighters[0].invulnerable();
        }
        assert!(safe, "part of the roll cannot be hit");
        // in the air
        let mut a = GameState::new();
        a.fighters[1].x = 900 * PX;
        a.step([Input(Input::UP), Input(0)], [&ayz(), &evilayz()]);
        for _ in 0..8 {
            a.step([Input(0), Input(0)], [&ayz(), &evilayz()]);
        }
        a.step([Input(Input::ROLL_LEFT), Input(0)], [&ayz(), &evilayz()]);
        assert_eq!(a.fighters[0].state, FighterState::Roll);
        assert!(a.fighters[0].y > 0, "still in the air");
        assert!(a.fighters[0].air_roll_used);
    }

    #[test]
    fn holding_down_in_the_air_plunges_and_splashes_for_half_a_goblins_health() {
        let mut s = GameState::new();
        s.fighters[1].x = s.fighters[0].x + 20 * PX;
        s.step([Input(Input::UP), Input(0)], [&ayz(), &evilayz()]);
        for _ in 0..6 {
            s.step([Input(0), Input(0)], [&ayz(), &evilayz()]);
        }
        s.fighters[0].x = s.fighters[1].x - 10 * PX;
        let before = s.fighters[1].health;
        for _ in 0..40 {
            s.step([Input(Input::DOWN), Input(0)], [&ayz(), &evilayz()]);
        }
        assert_eq!(before - s.fighters[1].health, Fighter::SLAM_DAMAGE, "28 damage, half of a goblin's 56");
        assert_eq!(Fighter::SLAM_DAMAGE * 2, 56);
    }

    #[test]
    fn being_hit_does_not_stop_a_ground_pound_from_landing() {
        let mut s = GameState::new();
        s.fighters[1].x = s.fighters[0].x + 20 * PX;
        s.step([Input(Input::UP), Input(0)], [&ayz(), &evilayz()]);
        for _ in 0..6 {
            s.step([Input(0), Input(0)], [&ayz(), &evilayz()]);
        }
        s.fighters[0].x = s.fighters[1].x - 10 * PX;
        s.step([Input(Input::DOWN), Input(0)], [&ayz(), &evilayz()]);
        assert!(s.fighters[0].slamming);
        // The opponent hits him mid-plunge.
        let mut mv = ayz().moves[0].clone();
        mv.knockback = 3;
        s.apply_hit(0, &mv, 1, false, 100);
        assert!(s.fighters[0].slamming, "the hit hurt but did not stop the plunge");
        let (mine, theirs) = (s.fighters[0].health, s.fighters[1].health);
        for _ in 0..40 {
            s.step([Input(Input::DOWN), Input(0)], [&ayz(), &evilayz()]);
        }
        assert!(mine < Fighter::MAX_HEALTH, "he took the damage");
        assert!(s.fighters[1].health < theirs, "and still landed the splash");
    }

    #[test]
    fn shots_fly_straight_ahead_and_do_not_home_in() {
        let a = professor();
        let mut s = GameState::new();
        s.fighters[1].x = s.fighters[0].x + 150 * PX;
        s.fighters[1].y = 40 * PX; // up in the air
        s.step([Input(Input::LIGHT), Input(0)], [&a, &evilayz()]);
        let mut seen = false;
        for _ in 0..30 {
            s.step([Input(0), Input(0)], [&a, &evilayz()]);
            for p in &s.projectiles {
                seen = true;
                assert_eq!(p.vy, 0, "a shot must not aim at the opponent's height");
                assert_eq!(p.vx.signum(), 1);
            }
        }
        assert!(seen);
    }

    #[test]
    fn goblins_can_be_killed_by_the_opponent() {
        let a = anaran();
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        s.fighters[1].x = s.fighters[0].x + 200 * PX;
        s.step([Input(Input::SPECIAL), Input(0)], [&a, &evilayz()]);
        for _ in 0..40 {
            s.step([Input(0), Input(0)], [&a, &evilayz()]);
        }
        assert_eq!(s.minions.len(), 4);
        // A standing jab's height is within a goblin's reach; two jabs kill one.
        let x = s.minions[0].x;
        let jab = [x - 10 * PX, 29 * PX, x + 10 * PX, 40 * PX];
        assert!(s.hit_minion(0, jab, 35) && s.hit_minion(0, jab, 35));
        s.step([Input(0), Input(0)], [&a, &evilayz()]);
        assert!(s.minions.len() < 4, "a goblin that took two jabs should be dead");
    }

    #[test]
    fn jumping_is_high_enough_that_ground_attacks_cannot_reach() {
        let mut s = GameState::new();
        step(&mut s, Input::UP, 0);
        let mut peak = 0;
        for _ in 0..80 {
            step(&mut s, 0, 0);
            peak = peak.max(s.fighters[0].y);
        }
        // Higher than the tallest ground hitbox (the uppercut, 56 px).
        assert!(peak > 70 * PX, "peak was only {} px", peak / PX);
    }

    #[test]
    fn an_air_attack_hits_a_grounded_opponent_once_per_jump() {
        let mut s = close_up();
        step(&mut s, Input::UP, 0);
        // Wait for the way down: a kick thrown near the top of the jump goes over the head.
        for _ in 0..32 {
            step(&mut s, 0, 0);
        }
        s.fighters[0].x = s.fighters[1].x - 20 * PX;
        step(&mut s, Input::LIGHT, 0);
        assert_eq!(s.fighters[0].state, FighterState::Attack);
        assert!(ayz().moves[s.fighters[0].attack].air);
        for _ in 0..60 {
            step(&mut s, 0, 0);
        }
        assert!(s.fighters[1].health < Fighter::MAX_HEALTH, "the air attack never hit");
        assert_eq!(s.fighters[0].state, FighterState::Idle, "should have landed");
        // A second press in the same jump does nothing.
        let mut t = GameState::new();
        step(&mut t, Input::UP, 0);
        step(&mut t, Input::LIGHT, 0);
        for _ in 0..30 {
            step(&mut t, 0, 0);
        }
        step(&mut t, Input::LIGHT, 0);
        step(&mut t, 0, 0);
        assert!(t.fighters[0].air_used);
    }

    #[test]
    fn a_chargeable_move_hits_harder_the_longer_the_button_is_held() {
        let mut data = ayz();
        let special = data.moves.iter().position(|m| m.button == Button::Special).unwrap();
        data.moves[special].charge_frames = 45;
        data.moves[special].charge_bonus_pct = 80;
        let damage = |hold: u32| {
            let mut s = GameState::new();
            s.fighters[0].meter = Fighter::METER_MAX;
            s.fighters[1].x = 60 * PX;
            let enemy = evilayz();
            s.step([Input(Input::SPECIAL), Input(0)], [&data, &enemy]);
            for _ in 0..hold {
                s.step([Input(Input::SPECIAL), Input(0)], [&data, &enemy]);
            }
            for _ in 0..120 {
                s.step([Input(0), Input(0)], [&data, &enemy]);
            }
            Fighter::MAX_HEALTH - s.fighters[1].health
        };
        let (quick, charged) = (damage(0), damage(60));
        assert!(quick > 0);
        assert!(charged > quick, "charged {charged} vs quick {quick}");
    }

    #[test]
    fn the_special_needs_a_full_meter_which_hits_fill_and_the_special_spends() {
        let mut s = GameState::new();
        step(&mut s, Input::SPECIAL, 0);
        assert_ne!(s.fighters[0].state, FighterState::Attack, "an empty meter must not allow the special");

        // Landing hits fills the meter.
        let mut s = close_up();
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..12 {
            step(&mut s, 0, 0);
        }
        assert!(s.fighters[0].meter > 0, "a landed jab should fill the meter");
        assert!(s.fighters[1].meter == 0, "the one who was hit gains nothing");

        // A full meter allows the special, and throwing it spends the meter.
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        step(&mut s, Input::SPECIAL, 0);
        assert_eq!(s.fighters[0].state, FighterState::Attack);
        for _ in 0..70 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].meter, 0);
    }

    #[test]
    fn an_attack_hits_at_most_once() {
        let mut s = close_up();
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..30 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH - 30);
    }

    #[test]
    fn attack_out_of_range_whiffs() {
        let mut s = GameState::new(); // 160px apart, a jab reaches ~34px
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..30 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH);
    }

    #[test]
    fn the_block_button_stops_light_attacks_for_good_with_only_chip_damage() {
        let mut s = close_up();
        // P2 guards; P1 jabs again and again. The jab can never break the guard.
        for _ in 0..6 {
            step(&mut s, Input::LIGHT, Input::BLOCK);
            for _ in 0..16 {
                step(&mut s, 0, Input::BLOCK);
            }
        }
        let taken = Fighter::MAX_HEALTH - s.fighters[1].health;
        assert!(taken > 0 && taken <= 6 * 3, "only chip damage, took {taken}");
        assert!(matches!(s.fighters[1].state, FighterState::Guard | FighterState::Blockstun));
    }

    #[test]
    fn a_heavy_attack_breaks_the_guard_but_a_light_one_does_not() {
        let mut s = close_up();
        step(&mut s, Input::HEAVY, Input::BLOCK);
        for _ in 0..40 {
            step(&mut s, 0, Input::BLOCK);
        }
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH - 70, "the lecture breaks the guard and lands in full");
    }

    #[test]
    fn without_the_block_button_nothing_is_blocked() {
        let mut s = close_up();
        // Holding away used to block; now only the block button does.
        step(&mut s, Input::LIGHT, Input::RIGHT);
        for _ in 0..10 {
            step(&mut s, 0, Input::RIGHT);
        }
        assert!(s.fighters[1].health < Fighter::MAX_HEALTH - 3);
    }

    #[test]
    fn hitstop_freezes_both_fighters() {
        let mut s = close_up();
        step(&mut s, Input::LIGHT, 0);
        let mut saw = false;
        for _ in 0..10 {
            step(&mut s, 0, 0);
            if s.fighters[0].hitstop > 0 {
                let frame = s.fighters[0].state_frame;
                step(&mut s, 0, 0);
                assert_eq!(s.fighters[0].state_frame, frame, "attacker advanced during hitstop");
                saw = true;
                break;
            }
        }
        assert!(saw, "never reached hitstop");
    }

    #[test]
    fn ko_ends_the_round_and_awards_a_win() {
        let mut s = close_up();
        s.fighters[1].health = 20;
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..8 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].state, FighterState::Ko);
        assert!(matches!(s.phase, Phase::RoundEnd(_)));
        assert_eq!(s.wins, [1, 0]);
    }

    #[test]
    fn next_round_resets_positions_and_health() {
        let mut s = close_up();
        s.fighters[1].health = 20;
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..(GameState::ROUND_END_FRAMES + 20) {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.phase, Phase::Fighting);
        assert_eq!(s.round, 2);
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH);
        assert_eq!(s.fighters[1].x, 80 * PX);
    }

    #[test]
    fn winning_two_rounds_ends_the_match() {
        let mut s = GameState::new();
        s.wins = [1, 0];
        s.fighters = close_up().fighters;
        s.fighters[1].health = 20;
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..(GameState::ROUND_END_FRAMES + 20) {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.phase, Phase::MatchOver);
        assert_eq!(s.match_winner(), Some(0));
    }

    #[test]
    fn time_up_goes_to_the_healthier_fighter() {
        let mut s = GameState::new();
        s.timer = 1;
        s.fighters[0].health = 500;
        step(&mut s, 0, 0);
        assert_eq!(s.wins, [0, 1]);
    }

    #[test]
    fn crouching_jab_uses_the_low_move() {
        let mut s = close_up();
        step(&mut s, Input::LIGHT | Input::DOWN, 0);
        assert_eq!(ayz().moves[s.fighters[0].attack].name, "Low Jab");
    }

    /// Run the summon from a standing start until the star is gone, returning the star's
    /// furthest-travelled position and the defender's health at the end.
    fn summon_vs(defender_input: u16, defender_x: i32) -> GameState {
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        s.fighters[1].x = defender_x;
        step(&mut s, Input::SPECIAL, defender_input);
        for _ in 0..150 {
            step(&mut s, 0, defender_input);
        }
        s
    }

    #[test]
    fn the_summon_launches_a_star_that_flies_at_the_opponent_and_hits() {
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        step(&mut s, Input::SPECIAL, 0);
        let mut launched_at = None;
        for frame in 0..80 {
            step(&mut s, 0, 0);
            if let (None, Some(p)) = (launched_at, s.projectiles.first().copied()) {
                launched_at = Some((frame, p));
            }
        }
        let (frame, p) = launched_at.expect("no star was ever launched");
        assert!(frame > 15, "the star came out during the startup, at frame {frame}");
        assert!(p.vx > 0, "the star should head toward the opponent on the right");
        assert!(p.vy < 0, "the star should fall toward the opponent from above");
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH - 110, "the star should hit once for 110");
        assert!(s.projectiles.is_empty(), "a star that has hit is gone");
    }

    #[test]
    fn the_moon_hangs_in_the_sky_and_the_fish_splashes_where_the_opponent_stood() {
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        step(&mut s, Input::SPECIAL, 0);
        step(&mut s, 0, 0);
        let (mx, my, _) = s.fighters[0].moon.expect("the moon should appear when the special starts");
        assert!(my > 50 * PX, "the moon is high in the sky");
        // He walks off; the moon stays where it was summoned.
        for _ in 0..10 {
            step(&mut s, Input::LEFT, 0);
        }
        let now = s.fighters[0].moon.unwrap();
        assert_eq!((now.0, now.1), (mx, my), "the moon must not follow him");

        // The splash hurts an opponent who stays put.
        let mut hurt = GameState::new();
        hurt.fighters[0].meter = Fighter::METER_MAX;
        step(&mut hurt, Input::SPECIAL, 0);
        for _ in 0..100 {
            step(&mut hurt, 0, 0);
        }
        assert_eq!(hurt.fighters[1].health, Fighter::MAX_HEALTH - 110);
    }

    #[test]
    fn the_landing_spot_is_shown_early_and_the_fish_is_not_launched_at_once() {
        let mut s = GameState::new();
        s.fighters[0].meter = Fighter::METER_MAX;
        step(&mut s, Input::SPECIAL, 0);
        let mut locked_at = None;
        let mut launched_at = None;
        for frame in 1..90 {
            step(&mut s, 0, 0);
            if locked_at.is_none() && s.fighters[0].lob_target.is_some() {
                locked_at = Some(frame);
            }
            if launched_at.is_none() && !s.projectiles.is_empty() {
                launched_at = Some(frame);
            }
        }
        let (locked, launched) = (locked_at.expect("no landing spot was shown"), launched_at.expect("no fish was launched"));
        assert!(launched - locked >= 30, "the opponent needs time to react: spot at {locked}, fish at {launched}");
    }

    #[test]
    fn walking_out_of_the_marked_spot_in_time_dodges_the_fish_but_not_after_it_is_launched() {
        // Starts walking as soon as the spot is shown: out of the zone long before the fish.
        let mut early = GameState::new();
        early.fighters[0].meter = Fighter::METER_MAX;
        step(&mut early, Input::SPECIAL, 0);
        for _ in 0..15 {
            step(&mut early, 0, 0);
        }
        for _ in 0..90 {
            step(&mut early, 0, Input::RIGHT);
        }
        assert_eq!(early.fighters[1].health, Fighter::MAX_HEALTH, "there is time to walk out of the zone");

        // Starts walking only once the fish is in the air: no chance.
        let mut late = GameState::new();
        late.fighters[0].meter = Fighter::METER_MAX;
        step(&mut late, Input::SPECIAL, 0);
        for _ in 0..90 {
            if !late.projectiles.is_empty() {
                break;
            }
            step(&mut late, 0, 0);
        }
        for _ in 0..40 {
            step(&mut late, 0, Input::RIGHT);
        }
        assert_eq!(late.fighters[1].health, Fighter::MAX_HEALTH - 110, "once the fish is shot it cannot be outrun");
    }

    #[test]
    fn the_summon_has_no_melee_hit() {
        // Standing right next to the opponent, the move itself (before the star is launched)
        // must not hurt them.
        let mut s = close_up();
        s.fighters[0].meter = Fighter::METER_MAX;
        step(&mut s, Input::SPECIAL, 0);
        for _ in 0..21 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH);
        assert!(s.projectiles.is_empty());
    }

    #[test]
    fn a_star_that_hits_nothing_fizzles_out() {
        let mut s = GameState::new();
        s.projectiles.push(Projectile { owner: 0, x: 0, y: 100 * PX, vx: 0, vy: 0, age: 0, move_index: 6, land_x: 0, power: 100 });
        // Park the opponent far away so the star never reaches them.
        for _ in 0..100 {
            step(&mut s, 0, 0);
        }
        assert!(s.projectiles.is_empty(), "the star should have run out of life");
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH);
    }

    #[test]
    fn the_star_is_cleared_between_rounds() {
        let mut s = summon_vs(0, 80 * PX);
        s.projectiles.push(Projectile { owner: 0, x: 0, y: 50 * PX, vx: 0, vy: 0, age: 0, move_index: 6, land_x: 0, power: 100 });
        s.fighters[1].health = 1;
        s.phase = Phase::RoundEnd(GameState::ROUND_END_FRAMES - 1);
        step(&mut s, 0, 0);
        assert_eq!(s.phase, Phase::Fighting);
        assert!(s.projectiles.is_empty());
    }

    #[test]
    fn jab_lets_you_keep_walking_but_the_uppercut_does_not() {
        // Jab while holding right: still moves, just slower than walking.
        let mut walking = GameState::new();
        let mut jabbing = GameState::new();
        step(&mut walking, Input::RIGHT, 0);
        step(&mut jabbing, Input::RIGHT | Input::LIGHT, 0);
        for _ in 0..5 {
            step(&mut walking, Input::RIGHT, 0);
            step(&mut jabbing, Input::RIGHT, 0);
        }
        assert_eq!(jabbing.fighters[0].state, FighterState::Attack);
        assert!(jabbing.fighters[0].x > GameState::new().fighters[0].x, "jab froze the fighter");
        assert!(jabbing.fighters[0].x < walking.fighters[0].x, "jab should be slower than walking");

        // Crouching heavy is rooted.
        let mut s = GameState::new();
        let start = s.fighters[0].x;
        step(&mut s, Input::RIGHT | Input::DOWN | Input::HEAVY, 0);
        for _ in 0..5 {
            step(&mut s, Input::RIGHT | Input::DOWN, 0);
        }
        assert_eq!(s.fighters[0].x, start);
    }

    #[test]
    fn both_players_can_move_and_attack_in_the_same_frame() {
        let mut s = GameState::new();
        s.fighters[0].x = -20 * PX;
        s.fighters[1].x = 20 * PX;
        for _ in 0..3 {
            step(&mut s, Input::LIGHT | Input::LEFT, Input::LIGHT | Input::RIGHT);
        }
        assert_eq!(s.fighters[0].state, FighterState::Attack);
        assert_eq!(s.fighters[1].state, FighterState::Attack);
    }

    #[test]
    fn rolled_back_replay_matches() {
        // Save a snapshot mid-fight, replay the same inputs from it and compare: this is what
        // rollback netcode relies on.
        let mut seed = 42;
        let mut live = GameState::new();
        let mut inputs = Vec::new();
        let mut snapshot = None;
        for f in 0..2_000 {
            let i = (lcg(&mut seed) & 0x7f, lcg(&mut seed) & 0x7f);
            if f == 700 {
                snapshot = Some(live.clone());
            }
            if f >= 700 {
                inputs.push(i);
            }
            step(&mut live, i.0, i.1);
        }
        let mut replay = snapshot.unwrap();
        for (a, b) in inputs {
            step(&mut replay, a, b);
        }
        assert_eq!(hash(&live), hash(&replay));
    }

    fn junkhead() -> CharacterData {
        CharacterData::from_ron(include_str!("../../assets/characters/junkhead.ron")).unwrap()
    }

    #[test]
    fn junkheads_third_light_is_a_jump_kick_and_a_landed_finisher_gives_one_long_whip() {
        let j = junkhead();
        let step = |s: &mut GameState, a: u16, b: u16| s.step([Input(a), Input(b)], [&j, &evilayz()]);
        let mut s = close_up();
        let mut names = Vec::new();
        for _ in 0..3 {
            s.fighters[1].x = s.fighters[0].x + 30 * PX;
            step(&mut s, Input::LIGHT, 0);
            names.push(j.moves[s.fighters[0].attack].name.clone());
            for _ in 0..40 {
                step(&mut s, 0, 0);
            }
        }
        assert_eq!(names, ["Whip", "Whip", "Jump Kick"]);
        assert!(s.fighters[0].whip_boost, "the finisher landed, so the next whip is boosted");

        // 95 px away is out of a normal whip's reach (60 px) but inside the boosted one's (105 px).
        for _ in 0..40 {
            step(&mut s, 0, 0);
        }
        let before = s.fighters[1].health;
        s.fighters[0].x = 0;
        s.fighters[1].x = 95 * PX;
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..30 {
            step(&mut s, 0, 0);
        }
        assert_eq!(before - s.fighters[1].health, 28, "the boosted whip reaches");
        assert!(!s.fighters[0].whip_boost, "and the boost is spent");

        for _ in 0..80 {
            step(&mut s, 0, 0);
        }
        let before = s.fighters[1].health;
        s.fighters[0].x = 0;
        s.fighters[1].x = 95 * PX;
        step(&mut s, Input::LIGHT, 0);
        for _ in 0..30 {
            step(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].health, before, "the next whip is back to normal reach");
    }

    #[test]
    fn the_whip_pull_drags_him_to_the_opponent_but_only_half_way_if_it_is_blocked() {
        let j = junkhead();
        let gap_after = |foe_input: u16| {
            let mut s = GameState::new();
            s.fighters[0].x = 0;
            s.fighters[1].x = 100 * PX;
            for _ in 0..80 {
                let a = if s.frame == 0 { Input::HEAVY } else { 0 };
                s.step([Input(a), Input(foe_input)], [&j, &evilayz()]);
            }
            ((s.fighters[1].x - s.fighters[0].x) / PX, Fighter::MAX_HEALTH - s.fighters[1].health)
        };
        let (gap, taken) = gap_after(0);
        assert!((22..=26).contains(&gap), "pulled right up to the opponent, gap {gap}");
        assert_eq!(taken, 60);
        let (gap, taken) = gap_after(Input::BLOCK);
        assert!((55..=70).contains(&gap), "blocked: only about half of the 76 px, gap {gap}");
        assert!(taken < 60, "a blocked whip only chips, took {taken}");
    }

    #[test]
    fn the_whip_storm_goes_high_or_low_at_random_and_drags_a_victim_in_and_stuns_them() {
        let j = junkhead();
        // Both outcomes happen, depending on the moment.
        let mut seen = [false; 2];
        for frame in 0..60 {
            let mut s = GameState::new();
            s.frame = frame;
            s.fighters[0].meter = Fighter::METER_MAX;
            s.step([Input(Input::SPECIAL), Input(0)], [&j, &evilayz()]);
            seen[s.fighters[0].aim_low as usize] = true;
        }
        assert_eq!(seen, [true, true], "it should sometimes go high and sometimes low");

        // Plays the special with the aim forced; the opponent presses `foe_input` (and jumps if `jump`).
        let storm = |aim_low: bool, foe_input: u16, jump: bool| {
            let mut s = GameState::new();
            s.fighters[0].x = 0;
            s.fighters[1].x = 80 * PX;
            s.fighters[0].meter = Fighter::METER_MAX;
            for _ in 0..60 {
                let a = if s.frame == 0 { Input::SPECIAL } else { 0 };
                s.step([Input(a), Input(foe_input)], [&j, &evilayz()]);
                if s.frame == 1 {
                    s.fighters[0].aim_low = aim_low;
                }
                // The opponent jumps during the spins, so they are in the air when the lash comes out.
                if jump && s.frame == 8 {
                    s.fighters[1].vy = 1050;
                    s.fighters[1].state = FighterState::Jump;
                }
            }
            s
        };
        let hit = storm(false, 0, false);
        assert_eq!(Fighter::MAX_HEALTH - hit.fighters[1].health, 150);
        assert_eq!(hit.fighters[1].state, FighterState::Hitstun, "stunned");
        let gap = (hit.fighters[1].x - hit.fighters[0].x) / PX;
        assert!((20..=24).contains(&gap), "dragged right up to him, gap {gap}");
        assert_eq!(hit.fighters[0].meter, 0, "the special spends the meter");

        assert_eq!(storm(false, Input::DOWN, false).fighters[1].health, Fighter::MAX_HEALTH, "crouching ducks the high lash");
        assert_eq!(storm(true, 0, false).fighters[1].health, Fighter::MAX_HEALTH - 150, "the low lash hits a standing fighter");
        assert_eq!(storm(true, Input::DOWN, false).fighters[1].health, Fighter::MAX_HEALTH - 150, "and a crouching one");
        assert_eq!(storm(true, 0, true).fighters[1].health, Fighter::MAX_HEALTH, "jumping clears the low lash");
    }

    #[test]
    fn the_air_whip_spin_covers_both_sides_of_him() {
        let j = junkhead();
        let mut s = GameState::new();
        s.step([Input(Input::UP), Input(0)], [&j, &evilayz()]);
        for _ in 0..5 {
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
        }
        s.step([Input(Input::LIGHT), Input(0)], [&j, &evilayz()]);
        let mut box_seen = None;
        for _ in 0..20 {
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
            if let Some(b) = s.hitbox(0, &j) {
                box_seen = Some(b);
                break;
            }
        }
        let b = box_seen.expect("the spin should become active");
        let x = s.fighters[0].x;
        assert!(b[0] < x - 40 * PX && b[2] > x + 40 * PX, "reaches both in front of and behind him");
        assert!(b[3] - b[1] >= 70 * PX, "from his head to his legs");
    }

    /// Junkhead jumps over a passive opponent and presses `button` whenever the move he is doing has
    /// just connected (and once to start). Returns the state and the names of the air moves he used.
    fn air_combo(first: u16, then: &[u16]) -> (GameState, Vec<String>) {
        let j = junkhead();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 24 * PX;
        s.step([Input(Input::UP), Input(0)], [&j, &evilayz()]);
        for _ in 0..4 {
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
        }
        let mut names = Vec::new();
        let mut next = then.iter();
        let mut button = Some(first);
        let mut pressed_last = false;
        for _ in 0..70 {
            let f = &s.fighters[0];
            let ready = f.state == FighterState::Jump || (f.state == FighterState::Attack && f.hit_done);
            // Tap (never hold) the next button once the move in progress has connected.
            let press = match button {
                Some(b) if ready && !pressed_last => b,
                _ => 0,
            };
            pressed_last = press != 0;
            let before = (s.fighters[0].attack, s.fighters[0].air_count);
            s.step([Input(press), Input(0)], [&j, &evilayz()]);
            let f = &s.fighters[0];
            if f.state == FighterState::Attack && f.state_frame == 0 && (f.attack, f.air_count) != before {
                names.push(j.moves[f.attack].name.clone());
                button = next.next().copied();
            }
        }
        (s, names)
    }

    #[test]
    fn junkhead_can_chain_three_air_lights_in_one_jump() {
        let (s, names) = air_combo(Input::LIGHT, &[Input::LIGHT, Input::LIGHT, Input::LIGHT]);
        assert_eq!(names, ["Whip Spin", "Whip Spin", "Whip Spin"], "three air lights, and no fourth");
        assert!(Fighter::MAX_HEALTH - s.fighters[1].health >= 26 * 3, "they hit, took {}", Fighter::MAX_HEALTH - s.fighters[1].health);
    }

    #[test]
    fn a_connected_air_light_can_be_cancelled_into_the_air_heavy_which_ends_the_chain() {
        let (_, names) = air_combo(Input::LIGHT, &[Input::HEAVY, Input::LIGHT]);
        assert_eq!(names, ["Whip Spin", "Air Whip Pull"], "the heavy ends the chain");
    }

    #[test]
    fn the_air_heavy_works_on_its_own_and_pulls_him_towards_the_opponent() {
        let j = junkhead();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 90 * PX;
        s.step([Input(Input::UP), Input(0)], [&j, &evilayz()]);
        for _ in 0..4 {
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
        }
        s.step([Input(Input::HEAVY), Input(0)], [&j, &evilayz()]);
        assert_eq!(j.moves[s.fighters[0].attack].name, "Air Whip Pull");
        for _ in 0..40 {
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
        }
        assert!(Fighter::MAX_HEALTH - s.fighters[1].health >= 55, "it hit");
        let gap = (s.fighters[1].x - s.fighters[0].x).abs() / PX;
        // (the floor smash shoves them back a little, so the gap ends up a bit wider than the 24 px he was pulled to)
        assert!(gap < 50, "he was pulled in, gap {gap}");
    }

    #[test]
    fn the_air_whip_catches_for_half_then_pulls_him_all_the_way_and_smashes_the_floor_for_the_other_half() {
        let j = junkhead();
        let pull = j.moves.iter().find(|m| m.name == "Air Whip Pull").unwrap();
        assert_eq!(pull.damage + pull.smash_damage, 55, "the same total as before, in two halves");
        assert!(pull.damage.abs_diff(pull.smash_damage) <= 1, "{} and {}", pull.damage, pull.smash_damage);
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 110 * PX;
        s.step([Input(Input::UP), Input(0)], [&j, &evilayz()]);
        for _ in 0..4 {
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
        }
        s.step([Input(Input::HEAVY), Input(0)], [&j, &evilayz()]);
        let (mut after_catch, mut after_smash, mut landed_at_gap, mut smashed_frame) = (None, None, None, None);
        for frame in 0..80 {
            s.step([Input(0), Input(0)], [&j, &evilayz()]);
            let lost = Fighter::MAX_HEALTH - s.fighters[1].health;
            if after_catch.is_none() && lost > 0 {
                after_catch = Some(lost);
            }
            if after_catch.is_some() && after_smash.is_none() && lost > after_catch.unwrap() {
                after_smash = Some(lost);
                smashed_frame = Some(frame);
                landed_at_gap = Some((s.fighters[1].x - s.fighters[0].x).abs() / PX);
            }
        }
        assert_eq!(after_catch, Some(pull.damage), "the catch deals the first half");
        assert_eq!(after_smash, Some(55), "the landing deals the rest");
        let gap = landed_at_gap.unwrap();
        assert!(gap <= 40, "he arrived at the enemy before smashing, gap {gap}");
        assert!(smashed_frame.is_some());
        assert_eq!(s.fighters[0].y, 0, "and he is on the floor");
    }

    #[test]
    fn a_floor_smash_hits_by_its_own_damage_and_the_plain_ground_pound_still_deals_its_usual_amount() {
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 20 * PX;
        s.fighters[0].slam_landed = true;
        s.fighters[0].slam_damage = 17;
        s.step([Input(0), Input(0)], [&junkhead(), &evilayz()]);
        assert_eq!(Fighter::MAX_HEALTH - s.fighters[1].health, 17);
        assert_eq!(s.fighters[0].slam_damage, Fighter::SLAM_DAMAGE, "back to the usual for the next plunge");
    }

    #[test]
    fn the_special_works_in_the_air_hangs_there_and_still_lashes_high_or_low_at_the_floor() {
        let j = junkhead();
        let storm = |aim_low: bool, foe_input: u16| {
            let mut s = GameState::new();
            s.fighters[0].x = 0;
            s.fighters[1].x = 80 * PX;
            s.fighters[0].meter = Fighter::METER_MAX;
            s.step([Input(Input::UP), Input(foe_input)], [&j, &evilayz()]);
            for _ in 0..4 {
                s.step([Input(0), Input(foe_input)], [&j, &evilayz()]);
            }
            s.step([Input(Input::SPECIAL), Input(foe_input)], [&j, &evilayz()]);
            assert_eq!(j.moves[s.fighters[0].attack].name, "Air Whip Storm");
            s.fighters[0].aim_low = aim_low;
            let height = s.fighters[0].y;
            let mut lowest = height;
            for _ in 0..60 {
                s.step([Input(0), Input(foe_input)], [&j, &evilayz()]);
                if s.fighters[0].state == FighterState::Attack && s.fighters[0].state_frame < 40 {
                    lowest = lowest.min(s.fighters[0].y);
                }
            }
            assert!(lowest > height - 3 * PX, "he hangs in the air while he spins, {lowest} vs {height}");
            s
        };
        let hit = storm(false, 0);
        assert_eq!(Fighter::MAX_HEALTH - hit.fighters[1].health, 150);
        assert_eq!(hit.fighters[0].meter, 0);
        assert_eq!(storm(false, Input::DOWN).fighters[1].health, Fighter::MAX_HEALTH, "crouching ducks the high lash");
        assert_eq!(storm(true, Input::DOWN).fighters[1].health, Fighter::MAX_HEALTH - 150, "the low lash gets a crouching fighter");
    }

    #[test]
    fn other_characters_still_get_one_air_attack_per_jump() {
        let c = cowboy();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 24 * PX;
        s.step([Input(Input::UP), Input(0)], [&c, &evilayz()]);
        for _ in 0..4 {
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
        }
        let mut used = 0;
        for _ in 0..70 {
            let before = s.fighters[0].air_count;
            s.step([Input(Input::LIGHT), Input(0)], [&c, &evilayz()]);
            s.step([Input(0), Input(0)], [&c, &evilayz()]);
            used += (s.fighters[0].air_count > before) as u32;
        }
        assert_eq!(used, 1, "the cowboy gets a single air attack");
    }

    fn professor() -> CharacterData {
        CharacterData::from_ron(include_str!("../../assets/characters/ayz.ron")).unwrap()
    }

    fn pstep(s: &mut GameState, a: u16, b: u16) {
        s.step([Input(a), Input(b)], [&professor(), &evilayz()]);
    }

    #[test]
    fn the_professors_light_combo_is_uh_then_meh_then_the_ravens_claw() {
        let p = professor();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 90 * PX;
        let mut names = Vec::new();
        let mut taken = Vec::new();
        for _ in 0..3 {
            let before = s.fighters[1].health;
            pstep(&mut s, Input::LIGHT, 0);
            names.push(p.moves[s.fighters[0].attack].name.clone());
            for _ in 0..70 {
                pstep(&mut s, 0, 0);
            }
            taken.push(before - s.fighters[1].health);
            // The words push the opponent back a little: put them where they were.
            s.fighters[1].x = 90 * PX;
        }
        assert_eq!(names, ["Uh", "Meh", "Raven Claw"]);
        assert_eq!(taken, [24, 28, 69], "the claw is the finisher: 46 x 150%");
        assert!(!s.ravens[0].active, "the raven is back on his shoulder");
    }

    #[test]
    fn while_the_raven_is_away_the_claw_and_the_storm_cannot_be_sent_again() {
        let p = professor();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 150 * PX;
        s.fighters[0].meter = Fighter::METER_MAX;
        pstep(&mut s, Input::SPECIAL, 0);
        for _ in 0..40 {
            pstep(&mut s, 0, 0);
        }
        assert!(s.ravens[0].active, "the raven is out");
        s.fighters[0].meter = Fighter::METER_MAX;
        s.fighters[0].combo = 2;
        pstep(&mut s, Input::LIGHT, 0);
        assert_ne!(p.moves[s.fighters[0].attack].name, "Raven Claw", "no claw while the raven is away");
        for _ in 0..30 {
            pstep(&mut s, 0, 0);
        }
        let before = s.ravens[0].move_index;
        pstep(&mut s, Input::SPECIAL, 0);
        assert!(s.fighters[0].state != FighterState::Attack || s.fighters[0].attack != before || s.ravens[0].step > 0, "no second storm");
        assert_eq!(s.ravens[0].move_index, before);
    }

    #[test]
    fn here_is_the_thing_pulls_the_enemy_in_holds_them_and_then_shoves_them_away_for_the_damage() {
        let p = professor();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 130 * PX;
        pstep(&mut s, Input::HEAVY, 0);
        assert_eq!(p.moves[s.fighters[0].attack].name, "Here's the Thing");
        // Pulled in and held, with no damage yet.
        for _ in 0..60 {
            pstep(&mut s, 0, 0);
        }
        let gap = (s.fighters[1].x - s.fighters[0].x) / PX;
        assert!((22..=30).contains(&gap), "pulled next to him, gap {gap}");
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH, "no damage while he is held");
        assert_eq!(s.fighters[1].state, FighterState::Hitstun, "held");
        // Then the shove.
        for _ in 0..90 {
            pstep(&mut s, 0, 0);
        }
        assert_eq!(Fighter::MAX_HEALTH - s.fighters[1].health, 95);
        let gap = (s.fighters[1].x - s.fighters[0].x) / PX;
        assert!(gap > 70, "shoved away, gap {gap}");
    }

    #[test]
    fn here_is_the_thing_does_nothing_if_the_enemy_is_out_of_range() {
        let mut s = GameState::new();
        s.fighters[0].x = -30 * PX;
        s.fighters[1].x = 150 * PX; // 180 px apart: beyond the 158 px the grab reaches
        pstep(&mut s, Input::HEAVY, 0);
        for _ in 0..150 {
            pstep(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH);
        assert_eq!(s.fighters[1].x, 150 * PX, "not moved an inch");
    }

    #[test]
    fn the_ravens_storm_pecks_from_the_left_then_the_right_then_the_left_and_drops_a_healing_rose() {
        let mut s = GameState::new();
        s.fighters[0].x = -150 * PX;
        s.fighters[1].x = 100 * PX;
        s.fighters[0].health = 500;
        s.fighters[0].meter = Fighter::METER_MAX;
        pstep(&mut s, Input::SPECIAL, 0);
        let mut sides = Vec::new();
        let mut last = s.fighters[1].health;
        let mut rose_seen = false;
        let mut longest_away = 0;
        let mut away = 0;
        for _ in 0..400 {
            pstep(&mut s, 0, 0);
            let (r, foe) = (&s.ravens[0], &s.fighters[1]);
            if foe.health < last {
                sides.push(if r.x < foe.x { "left" } else { "right" });
                last = foe.health;
            }
            rose_seen |= r.rose.is_some();
            away = if r.active { away + 1 } else { 0 };
            longest_away = longest_away.max(away);
        }
        assert_eq!(sides, ["left", "right", "left"], "three pecks, switching sides");
        assert_eq!(Fighter::MAX_HEALTH - s.fighters[1].health, 3 * 42);
        assert!(rose_seen, "it carried a rose");
        assert_eq!(s.fighters[0].health, 550, "the rose heals 5% of his 1000 health");
        assert_eq!(s.fighters[0].meter, 0, "the special spent the meter");
        assert!(!s.ravens[0].active, "and the raven came home");
        assert!(longest_away > 60, "it was away for a good while, {longest_away} frames");
    }

    #[test]
    fn the_ravens_storm_reaches_an_enemy_anywhere_on_the_map() {
        let mut s = GameState::new();
        s.fighters[0].x = -180 * PX;
        s.fighters[1].x = 180 * PX;
        s.fighters[0].meter = Fighter::METER_MAX;
        pstep(&mut s, Input::SPECIAL, 0);
        for _ in 0..400 {
            pstep(&mut s, 0, 0);
        }
        assert_eq!(Fighter::MAX_HEALTH - s.fighters[1].health, 3 * 42, "all three pecks landed across the whole stage");
    }

    #[test]
    fn the_healing_rose_never_goes_over_full_health() {
        let mut s = GameState::new();
        s.fighters[0].x = -100 * PX;
        s.fighters[1].x = 100 * PX;
        s.fighters[0].health = 990;
        s.fighters[0].meter = Fighter::METER_MAX;
        pstep(&mut s, Input::SPECIAL, 0);
        for _ in 0..400 {
            pstep(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].health, 1000);
    }

    #[test]
    fn the_professor_with_a_raven_still_plays_deterministically() {
        let run = |seed: u64| {
            let p = professor();
            let mut s = GameState::new();
            let mut seed = seed;
            for _ in 0..4_000 {
                let a = lcg(&mut seed) & 0xff;
                let b = lcg(&mut seed) & 0x7f;
                s.step([Input(a), Input(b)], [&p, &evilayz()]);
            }
            hash(&s)
        };
        assert_eq!(run(5), run(5));
    }

    #[test]
    fn here_is_the_thing_works_from_a_good_distance_but_not_from_the_far_side_of_the_stage() {
        let mut s = GameState::new();
        s.fighters[0].x = -70 * PX;
        s.fighters[1].x = 70 * PX;
        let gap_at_start = (s.fighters[1].x - s.fighters[0].x) / PX;
        pstep(&mut s, Input::HEAVY, 0);
        for _ in 0..60 {
            pstep(&mut s, 0, 0);
        }
        let gap = (s.fighters[1].x - s.fighters[0].x) / PX;
        assert!(gap < gap_at_start - 60, "pulled in from {gap_at_start} px to {gap} px");
    }

    #[test]
    fn the_negaprofessor_is_the_professor_exactly_move_for_move_and_number_for_number() {
        let a = professor();
        let n = CharacterData::from_ron(include_str!("../../assets/characters/evilayz.ron")).unwrap();
        assert_ne!(a.name, n.name);
        assert_eq!(format!("{:?}", a.moves), format!("{:?}", n.moves), "every move is the same");
        assert_eq!((a.walk_speed, a.jump_speed, a.health, a.meter_gain_pct), (n.walk_speed, n.jump_speed, n.health, n.meter_gain_pct));
        assert_eq!((a.keeps_distance, a.cpu_range, a.air_chain, a.light_after_heavy), (n.keeps_distance, n.cpu_range, n.air_chain, n.light_after_heavy));
        assert_eq!(format!("{:?}", a.raven), format!("{:?}", n.raven), "the raven too");
    }

    fn astep(s: &mut GameState, a: u16, b: u16) {
        s.step([Input(a), Input(b)], [&anaran(), &evilayz()]);
    }

    #[test]
    fn anaran_reads_while_the_button_is_held_and_fires_a_missile_every_second() {
        let mut s = GameState::new();
        s.fighters[0].x = -100 * PX;
        s.fighters[1].x = 150 * PX;
        let start = s.fighters[0].x;
        // Holding Light (and walking away) he keeps reading and keeps walking.
        let mut missiles_at = Vec::new();
        let mut last = 0;
        for frame in 0..320 {
            astep(&mut s, Input::LIGHT | Input::LEFT, 0);
            if s.fighters[0].shots > last {
                last = s.fighters[0].shots;
                missiles_at.push(frame);
            }
            assert_eq!(s.fighters[0].state, FighterState::Attack, "still reading at frame {frame}");
        }
        assert!(s.fighters[0].x < start, "he walks while he reads");
        assert_eq!(missiles_at.len(), 5, "five missiles in a bit over five seconds: {missiles_at:?}");
        for pair in missiles_at.windows(2) {
            assert_eq!(pair[1] - pair[0], 60, "one second apart: {missiles_at:?}");
        }
        // Letting go closes the scroll and stops the missiles.
        for _ in 0..40 {
            astep(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].state, FighterState::Idle);
        assert_eq!(s.fighters[0].shots, 5);
    }

    #[test]
    fn a_tap_of_light_opens_the_scroll_but_no_missile_comes_out() {
        let mut s = GameState::new();
        astep(&mut s, Input::LIGHT, 0);
        for _ in 0..100 {
            astep(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[0].shots, 0);
        assert!(s.projectiles.is_empty());
    }

    #[test]
    fn the_missile_curves_after_the_opponent_even_if_they_run_behind_him_and_detonates() {
        let mut s = GameState::new();
        s.fighters[0].x = -100 * PX;
        s.fighters[1].x = 80 * PX;
        // Hold the scroll until the first missile is out, then let go.
        let mut released = false;
        let mut turned = false;
        let mut hit_at = None;
        for frame in 0..500 {
            let a = if released { 0 } else { Input::LIGHT };
            astep(&mut s, a, 0);
            if !released && s.fighters[0].shots == 1 {
                released = true;
                // The opponent slips round behind him: the missile has to turn right round.
                s.fighters[1].x = -180 * PX;
            }
            if let Some(m) = s.projectiles.first() {
                turned |= m.vx < 0;
            }
            if hit_at.is_none() && s.fighters[1].health < Fighter::MAX_HEALTH {
                hit_at = Some(frame);
            }
        }
        assert!(turned, "the missile should have turned back");
        assert!(hit_at.is_some(), "and still found them");
        assert_eq!(Fighter::MAX_HEALTH - s.fighters[1].health, 70, "one missile, 70 damage");
        assert!(s.projectiles.is_empty(), "it detonated");
    }

    #[test]
    fn the_staff_slam_shoves_enemies_away_from_him_on_either_side_and_leaves_them_unable_to_attack_for_a_second_and_a_half() {
        for side in [1, -1] {
            let mut s = GameState::new();
            s.fighters[0].x = 0;
            s.fighters[1].x = side * 45 * PX;
            astep(&mut s, Input::HEAVY, 0);
            let mut taken = 0;
            for _ in 0..40 {
                astep(&mut s, 0, 0);
                taken = Fighter::MAX_HEALTH - s.fighters[1].health;
            }
            assert_eq!(taken, 80, "side {side}");
            assert!((s.fighters[1].x - 0) * side > 45 * PX, "pushed away from him, side {side}");
            assert!(s.fighters[1].silenced > 0, "silenced");
            // The opponent mashes Light for the next second: nothing comes out.
            let mut attacked = false;
            for i in 0..60 {
                astep(&mut s, 0, if i % 2 == 0 { Input::LIGHT } else { 0 });
                attacked |= s.fighters[1].state == FighterState::Attack;
            }
            assert!(!attacked, "unable to attack while silenced, side {side}");
            // After the 90 frames they can again.
            for _ in 0..40 {
                astep(&mut s, 0, 0);
            }
            astep(&mut s, 0, Input::LIGHT);
            assert_eq!(s.fighters[1].state, FighterState::Attack, "free again after a second and a half");
        }
    }

    #[test]
    fn a_blocked_shockwave_does_not_silence_and_jumping_over_it_avoids_it() {
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 45 * PX;
        astep(&mut s, Input::HEAVY, Input::BLOCK);
        for _ in 0..40 {
            astep(&mut s, 0, Input::BLOCK);
        }
        assert_eq!(s.fighters[1].silenced, 0, "blocked: no silence");
        assert!(s.fighters[1].health > Fighter::MAX_HEALTH - 20, "only chip damage");

        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 45 * PX;
        astep(&mut s, Input::HEAVY, Input::UP);
        for _ in 0..40 {
            astep(&mut s, 0, 0);
        }
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH, "jumped over it");
        assert_eq!(s.fighters[1].silenced, 0);
    }

    #[test]
    fn the_staff_slam_destroys_enemy_projectiles_in_its_reach_but_not_melee_ones_or_far_ones() {
        let run = |slam: bool, melee: bool, gap: i32| {
            let mut prof = professor();
            if melee {
                prof.moves[0].projectile.as_mut().unwrap().melee = true;
            }
            let a = anaran();
            let mut s = GameState::new();
            s.fighters[0].x = 0;
            s.fighters[1].x = gap * PX;
            for i in 0..60 {
                // the Professor speaks at once; the slam (nearly instant) comes while the word is on its way
                let (x, y) = (if slam && i == 20 { Input::HEAVY } else { 0 }, if i == 0 { Input::LIGHT } else { 0 });
                s.step([Input(x), Input(y)], [&a, &prof]);
            }
            Fighter::MAX_HEALTH - s.fighters[0].health
        };
        assert_eq!(run(false, false, 120), 24, "without the slam the word hits him");
        assert_eq!(run(true, false, 120), 0, "the slam destroys it");
        assert_eq!(run(true, true, 120), 24, "but not a melee projectile");
        // from the far side of the stage the word is still well outside the shockwave while it is active, and arrives
        assert_eq!(run(true, false, 200), 24, "out of reach: not cleared");
    }

    #[test]
    fn the_staff_slam_is_a_quick_cast_and_then_cools_down_for_two_and_a_half_seconds() {
        let a = anaran();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 60 * PX;
        // pressed once: the shockwave hits within a few frames
        astep(&mut s, Input::HEAVY, 0);
        let mut first_hit = None;
        let mut slams = 1;
        let slam = a.moves.iter().position(|m| m.name == "Staff Slam").unwrap();
        for frame in 1..400 {
            let before = s.fighters[0].cooldown_left(slam);
            // mashing Heavy the whole time
            astep(&mut s, if frame % 2 == 0 { Input::HEAVY } else { 0 }, 0);
            if first_hit.is_none() && s.fighters[1].health < Fighter::MAX_HEALTH {
                first_hit = Some(frame);
            }
            // a new cast restarts the cooldown
            if s.fighters[0].cooldown_left(slam) > before && frame > 1 {
                slams += 1;
                assert!(frame >= 150, "cast again after only {frame} frames");
            }
        }
        assert!(first_hit.is_some_and(|f| f <= 12), "the slam should land within a few frames, {first_hit:?}");
        assert!(slams >= 2 && slams <= 3, "mashing for ~6.5 seconds gives a couple of slams, got {slams}");
        assert_eq!(a.moves.iter().find(|m| m.name == "Staff Slam").unwrap().cooldown, 150);
    }

    #[test]
    fn destroyed_projectiles_are_remembered_for_a_few_frames_so_the_screen_can_puff() {
        let prof = professor();
        let a = anaran();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 120 * PX;
        let mut seen = 0;
        for i in 0..60 {
            let (x, y) = (if i == 20 { Input::HEAVY } else { 0 }, if i == 0 { Input::LIGHT } else { 0 });
            s.step([Input(x), Input(y)], [&a, &prof]);
            seen = seen.max(s.cleared.len());
        }
        assert_eq!(seen, 1, "one word was destroyed");
        assert!(s.cleared.is_empty(), "and the memory fades after a few frames");
    }

    // ---- The Scribe: the quill, the book, the hail of knives and the beam of light

    #[test]
    fn the_scribes_moves_are_set_up_as_designed() {
        let c = scribe();
        let jab = &c.moves[c.find_move(Button::Light, false, false).unwrap()];
        assert!(jab.startup <= 3 && jab.hitbox.w > 0, "very fast quill stabs");
        let fin = &c.moves[c.find_finisher().unwrap()];
        assert!(fin.futility > 0 && fin.say.as_deref().unwrap().contains("{enemy}"), "the book writes about the enemy");
        assert!(fin.projectile.as_ref().is_some_and(|p| p.homing && p.unblockable), "an unblockable homing particle");
        let heavy = &c.moves[c.find_move(Button::Heavy, false, false).unwrap()];
        assert!(heavy.sparks.is_some() && heavy.projectile.as_ref().is_some_and(|p| p.lob && p.over_target), "sparks, then knives from a portal over the enemy");
        let special = &c.moves[c.find_move(Button::Special, false, false).unwrap()];
        assert!(special.beam.is_some() && special.say.is_some());
    }

    /// The Scribe (P1) with his combo at two lights, throws the book at a passive enemy 100 px away.
    fn futility_fight() -> GameState {
        let (c, e) = (scribe(), ayz());
        let mut s = GameState::new();
        s.fighters[1].x = s.fighters[0].x + 100 * PX;
        s.fighters[0].combo = 2;
        s.step([Input(Input::LIGHT), Input(0)], [&c, &e]);
        for _ in 0..160 {
            s.step([Input(0), Input(0)], [&c, &e]);
        }
        s
    }

    #[test]
    fn the_scribes_blue_particle_always_lands_and_makes_the_enemys_attacks_cost_them_health() {
        let (c, e) = (scribe(), ayz());
        let mut s = futility_fight();
        assert!(s.fighters[1].health < Fighter::MAX_HEALTH, "the particle landed");
        assert!(s.fighters[1].futile > 0 || s.fighters[1].futile == 0 && s.frame > 0);
        // right after it lands the debuff is on; starting an attack costs health
        let mut t = GameState::new();
        t.fighters[1].x = t.fighters[0].x + 100 * PX;
        t.fighters[0].combo = 2;
        t.step([Input(Input::LIGHT), Input(0)], [&c, &e]);
        while t.fighters[1].futile == 0 && t.frame < 300 {
            t.step([Input(0), Input(0)], [&c, &e]);
        }
        assert!(t.fighters[1].futile > 100, "futile for about 2.5 seconds, got {}", t.fighters[1].futile);
        for _ in 0..40 {
            t.step([Input(0), Input(0)], [&c, &e]);
        }
        let before = t.fighters[1].health;
        t.step([Input(0), Input(Input::LIGHT)], [&c, &e]);
        assert_eq!(before - t.fighters[1].health, Fighter::FUTILITY_DAMAGE, "an attack costs a little");
        // it wears off
        for _ in 0..200 {
            t.step([Input(0), Input(0)], [&c, &e]);
        }
        assert_eq!(t.fighters[1].futile, 0);
        s.fighters[1].health = 1000;
    }

    #[test]
    fn the_hail_of_knives_pours_from_a_portal_over_where_the_enemy_stood() {
        let (c, e) = (scribe(), ayz());
        let run = |walk_away: bool| {
            let mut s = GameState::new();
            s.fighters[1].x = s.fighters[0].x + 100 * PX;
            let spot = s.fighters[1].x;
            s.step([Input(Input::HEAVY), Input(0)], [&c, &e]);
            let mut portal = None;
            for frame in 0..220 {
                if frame == 55 && walk_away {
                    s.fighters[1].x += 120 * PX;
                }
                s.step([Input(0), Input(0)], [&c, &e]);
                portal = portal.or(s.fighters[0].moon);
            }
            let (mx, my, _) = portal.expect("a portal opened");
            assert_eq!(mx, spot, "right over where the enemy stood");
            assert!(my > 80 * PX, "high in the air");
            Fighter::MAX_HEALTH - s.fighters[1].health
        };
        assert!(run(false) >= 18, "a standing enemy is hit by several knives");
        assert_eq!(run(true), 0, "one who moved away is not");
    }

    #[test]
    fn the_beam_of_light_sweeps_from_in_front_of_the_enemy_to_behind_them_and_misses_if_they_left() {
        let (c, e) = (scribe(), ayz());
        let run = |walk_away: bool| {
            let mut s = GameState::new();
            s.fighters[0].meter = Fighter::METER_MAX;
            s.fighters[1].x = s.fighters[0].x + 100 * PX;
            let spot = s.fighters[1].x;
            s.step([Input(Input::SPECIAL), Input(0)], [&c, &e]);
            let mut starts = Vec::new();
            for frame in 0..170 {
                // after Jesus has locked on (0.6 s), before the beam comes down
                if frame == 40 && walk_away {
                    s.fighters[1].x += 120 * PX;
                }
                s.step([Input(0), Input(0)], [&c, &e]);
                if let Some(b) = s.hitbox(0, &c) {
                    starts.push((b[0] + b[2]) / 2);
                }
            }
            if walk_away {
                assert!(starts.len() > 10, "the beam was active for a while");
                assert!(starts.windows(2).all(|w| w[1] > w[0]), "and it moves away from the Scribe");
                assert!(starts[0] < spot && *starts.last().unwrap() > spot, "from in front of where the enemy stood to behind it");
            }
            Fighter::MAX_HEALTH - s.fighters[1].health
        };
        assert!(run(false) >= 100, "an enemy who stands still is hit");
        // and the special spends the meter, so it can't be used again at once
        let mut t = GameState::new();
        t.fighters[0].meter = Fighter::METER_MAX;
        t.fighters[1].x = t.fighters[0].x + 100 * PX;
        t.step([Input(Input::SPECIAL), Input(0)], [&c, &e]);
        for _ in 0..70 {
            t.step([Input(0), Input(0)], [&c, &e]);
        }
        assert_eq!(t.fighters[0].meter, 0, "the meter is spent");
        assert_eq!(run(true), 0, "an enemy who moved out of the way is not");
    }

    #[test]
    fn being_hit_does_not_interrupt_the_scribes_hail_or_beam_once_they_are_firing_but_does_before() {
        let (c, e) = (scribe(), ayz());
        let hit_at = |special: bool, frame_to_hit: u32| {
            let mut s = GameState::new();
            s.fighters[0].meter = Fighter::METER_MAX;
            s.fighters[1].x = s.fighters[0].x + 100 * PX;
            s.step([Input(if special { Input::SPECIAL } else { Input::HEAVY }), Input(0)], [&c, &e]);
            for _ in 0..frame_to_hit {
                s.step([Input(0), Input(0)], [&c, &e]);
            }
            let mv = &c.moves[s.fighters[0].attack];
            let hurt = mv.hitstun;
            let mut forced = c.moves[0].clone();
            forced.hitstun = hurt.max(30);
            s.apply_hit(0, &forced, -1, false, 100);
            (s.fighters[0].state, s.fighters[0].health)
        };
        // before the portal or Jesus shows, a hit knocks him out of it
        assert_eq!(hit_at(false, 10).0, FighterState::Hitstun);
        assert_eq!(hit_at(true, 30).0, FighterState::Hitstun);
        // once they show, it doesn't
        assert_eq!(hit_at(false, 20).0, FighterState::Attack);
        assert_eq!(hit_at(true, 38).0, FighterState::Attack);
        // once the knives or the beam are going, the hit hurts but the move goes on
        let (state, health) = hit_at(false, 80);
        assert_eq!(state, FighterState::Attack, "the hail goes on");
        assert!(health < Fighter::MAX_HEALTH, "though it hurt");
        assert_eq!(hit_at(true, 90).0, FighterState::Attack, "and so does the beam");
    }

    fn gray_persona() -> CharacterData {
        CharacterData::from_ron(include_str!("../../assets/characters/gray_persona.ron")).unwrap()
    }

    #[test]
    fn gray_personas_combo_finisher_bites_and_heals_her_three_percent() {
        let g = gray_persona();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 30 * PX;
        s.fighters[0].health = 500;
        s.fighters[0].combo = 2;
        s.fighters[0].combo_timer = 20;
        s.step([Input(Input::LIGHT), Input(0)], [&g, &evilayz()]);
        assert_eq!(g.moves[s.fighters[0].attack].name, "Fang Frenzy");
        for _ in 0..30 {
            s.step([Input(0), Input(0)], [&g, &evilayz()]);
        }
        assert!(s.fighters[1].health < Fighter::MAX_HEALTH, "the bite should land");
        assert_eq!(s.fighters[0].health, 500 + Fighter::MAX_HEALTH * 3 / 100, "and heal her 3%");
    }

    #[test]
    fn a_landed_bite_rests_her_lights_for_a_second() {
        let g = gray_persona();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 30 * PX;
        s.fighters[0].combo = 2;
        s.fighters[0].combo_timer = 20;
        s.step([Input(Input::LIGHT), Input(0)], [&g, &evilayz()]);
        let mut landed_at = None;
        for frame in 0..60 {
            s.step([Input(0), Input(0)], [&g, &evilayz()]);
            if landed_at.is_none() && s.fighters[1].health < Fighter::MAX_HEALTH {
                landed_at = Some(frame);
            }
        }
        assert!(landed_at.is_some(), "the bite should land");
        assert!(s.fighters[0].light_lock > 0, "a second has not passed yet");
        // pressing light now does nothing
        s.fighters[0].state = FighterState::Idle;
        s.step([Input(Input::LIGHT), Input(0)], [&g, &evilayz()]);
        assert_ne!(s.fighters[0].state, FighterState::Attack);
        for _ in 0..30 {
            s.step([Input(0), Input(0)], [&g, &evilayz()]);
        }
        assert_eq!(s.fighters[0].light_lock, 0);
    }

    /// Gray Persona (P1) opens her portal with the opponent 60 px away; `dodge` makes them walk away from the far portal.
    fn portal_run(dodge: bool) -> GameState {
        let g = gray_persona();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 60 * PX;
        s.fighters[0].health = 700;
        s.fighters[0].meter = Fighter::METER_MAX;
        s.step([Input(Input::SPECIAL), Input(0)], [&g, &evilayz()]);
        for frame in 1..140 {
            let foe = if dodge && (25..70).contains(&frame) { Input::LEFT } else { 0 };
            s.step([Input(0), Input(foe)], [&g, &evilayz()]);
            if frame == 30 {
                assert!(s.fighters[0].hidden && s.fighters[0].invulnerable(), "inside the portal she is gone and can't be hit");
                assert_eq!(s.fighters[0].meter, 0, "the portal spends the meter");
            }
        }
        s
    }

    #[test]
    fn a_grab_holds_a_jumping_opponent_in_the_air_until_the_bite() {
        let g = gray_persona();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 60 * PX;
        s.fighters[0].meter = Fighter::METER_MAX;
        s.step([Input(Input::SPECIAL), Input(0)], [&g, &evilayz()]);
        let mut held_at = None;
        for frame in 1..120 {
            // the opponent jumps just as the hand comes out
            let foe = if frame == 50 { Input::UP } else { 0 };
            s.step([Input(0), Input(foe)], [&g, &evilayz()]);
            let f = &s.fighters[1];
            if f.held && held_at.is_none() {
                held_at = Some((f.x, f.y));
            }
            if let (Some(at), true) = (held_at, f.held) {
                assert_eq!((f.x, f.y), at, "held fast, in the air too");
            }
        }
        let (_, y) = held_at.expect("the grab caught them mid-jump");
        assert!(y > 0, "caught in the air");
        assert!(s.fighters[1].health < Fighter::MAX_HEALTH, "and bitten");
    }

    #[test]
    fn a_grab_never_pins_someone_it_did_not_stun() {
        let ayz = CharacterData::from_ron(include_str!("../../assets/characters/ayz.ron")).unwrap();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 60 * PX;
        s.step([Input(Input::HEAVY), Input(0)], [&ayz, &evilayz()]);
        // the opponent jumps and is mid-way through a move that can't be interrupted when the grab lands
        s.fighters[1].y = 30 * PX;
        s.fighters[1].vy = 400;
        s.fighters[1].armored = true;
        for _ in 0..200 {
            s.fighters[1].armored = s.fighters[1].y > 0;
            s.step([Input(0), Input(0)], [&ayz, &evilayz()]);
        }
        assert!(!s.fighters[1].held);
        assert_eq!(s.fighters[1].y, 0, "they came down");
        s.step([Input(0), Input(Input::UP)], [&ayz, &evilayz()]);
        s.step([Input(0), Input(0)], [&ayz, &evilayz()]);
        assert!(s.fighters[1].y > 0, "and can jump again");
    }

    #[test]
    fn the_portal_grab_holds_the_opponent_for_a_bite_that_heals_ten_percent() {
        let s = portal_run(false);
        assert_eq!(Fighter::MAX_HEALTH - s.fighters[1].health, 130, "the grab does nothing, the bite does the damage");
        assert_eq!(s.fighters[0].health, 700 + Fighter::MAX_HEALTH / 10, "the bite heals her 10%");
        assert!(s.fighters[0].x > s.fighters[1].x, "she came out behind them");
        assert!(!s.fighters[0].hidden);
    }

    #[test]
    fn moving_away_from_the_hand_dodges_the_portal() {
        let s = portal_run(true);
        assert_eq!(s.fighters[1].health, Fighter::MAX_HEALTH, "the hand misses and no bite follows");
        assert_eq!(s.fighters[0].health, 700, "no heal");
        assert!(!s.fighters[0].hidden, "she steps out of the portal anyway");
    }

    #[test]
    fn landed_bats_let_her_turn_into_a_bat_that_poofs_and_stuns() {
        let g = gray_persona();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 60 * PX;
        let quiet = |s: &mut GameState, n: usize| {
            for _ in 0..n {
                s.step([Input(0), Input(0)], [&g, &evilayz()]);
            }
        };
        s.step([Input(Input::HEAVY), Input(0)], [&g, &evilayz()]);
        quiet(&mut s, 60);
        assert!(s.fighters[0].bat_ready > 0, "the bats landed, so the heavy can be cast again");
        // the stun from the bats is over by now; heavy again: a bat
        s.step([Input(Input::HEAVY), Input(0)], [&g, &evilayz()]);
        assert!(s.fighters[0].bat_form > 0);
        // no lights or specials as a bat
        s.fighters[0].meter = Fighter::METER_MAX;
        for flag in [Input::LIGHT, Input::SPECIAL] {
            s.step([Input(flag), Input(0)], [&g, &evilayz()]);
            assert_ne!(s.fighters[0].state, FighterState::Attack);
        }
        // faster than walking
        let x = s.fighters[0].x;
        s.step([Input(Input::RIGHT), Input(0)], [&g, &evilayz()]);
        assert!(s.fighters[0].x - x > g.walk_speed, "a bat moves faster than she walks");
        // the bat faces where it flies, even away from the opponent
        s.step([Input(Input::LEFT), Input(0)], [&g, &evilayz()]);
        assert_eq!(s.fighters[0].heading, -1);
        assert_eq!(s.fighters[0].facing, 1, "she still faces the opponent for everything else");
        // fly up close and turn back: the smoke stuns
        s.fighters[0].x = s.fighters[1].x - 30 * PX;
        quiet(&mut s, 1);
        s.step([Input(Input::HEAVY), Input(0)], [&g, &evilayz()]);
        assert_eq!(s.fighters[0].bat_form, 0);
        assert_eq!(s.fighters[1].state, FighterState::Hitstun, "the puff of smoke stuns");
        let heavy = g.bat_move().unwrap();
        assert_eq!(s.fighters[0].cooldown_left(heavy), 360, "6 seconds after a stunning puff");
        // and by itself after three seconds
        let mut s2 = s.clone();
        s2.fighters[0].bat_form = 10;
        s2.fighters[1] = Fighter::new(1, -1);
        s2.fighters[1].x = s2.fighters[0].x + 30 * PX;
        quiet(&mut s2, 12);
        assert_eq!(s2.fighters[0].bat_form, 0);
        assert_eq!(s2.fighters[1].state, FighterState::Hitstun);
        // turning back far from anyone: the shorter cooldown
        let mut s3 = s.clone();
        s3.fighters[0].bat_form = 5;
        s3.fighters[1] = Fighter::new(1, -1);
        s3.fighters[1].x = s3.fighters[0].x + 200 * PX;
        quiet(&mut s3, 6);
        assert_eq!(s3.fighters[1].state, FighterState::Idle);
        assert!((230..=240).contains(&s3.fighters[0].cooldown_left(heavy)), "4 seconds after a missed puff");
    }

    #[test]
    fn the_bat_wave_hops_her_back() {
        let g = gray_persona();
        let mut s = GameState::new();
        s.fighters[0].x = 0;
        s.fighters[1].x = 100 * PX;
        s.step([Input(Input::HEAVY), Input(0)], [&g, &evilayz()]);
        for _ in 0..8 {
            s.step([Input(0), Input(0)], [&g, &evilayz()]);
        }
        assert!(s.fighters[0].y > 0 && s.fighters[0].x < 0, "up and back, away from the enemy");
        for _ in 0..30 {
            s.step([Input(0), Input(0)], [&g, &evilayz()]);
        }
        assert_eq!(s.fighters[0].y, 0);
        assert_eq!(s.projectiles.len() + (s.fighters[1].health < Fighter::MAX_HEALTH) as usize, 1, "the bats still flew");
    }

    #[test]
    fn the_scribes_heavy_and_special_hold_once_the_portal_or_jesus_shows() {
        let scribe = CharacterData::from_ron(include_str!("../../assets/characters/scribe.ron")).unwrap();
        for button in [Button::Heavy, Button::Special] {
            let index = scribe.find_move(button, false, false).unwrap();
            let mv = &scribe.moves[index];
            // the portal shows 0.3 s in, Jesus 0.6 s in, and from then on the move can't be stopped
            assert_eq!(mv.commit_frame(), if button == Button::Heavy { 18 } else { 36 }, "{}", mv.name);
            let mut f = Fighter::new(0, 1);
            f.state = FighterState::Attack;
            f.attack = index;
            f.state_frame = mv.commit_frame();
            f.step(Input(0), &scribe);
            assert!(f.armored, "{}", mv.name);
            f.start_stun(FighterState::Hitstun, 20, 0);
            assert_eq!(f.state, FighterState::Attack, "a hit no longer stops {}", mv.name);
        }
    }

    #[test]
    fn gray_personas_heavy_has_a_cooldown_and_sends_bats() {
        let g = gray_persona();
        let heavy = &g.moves[g.find_move(Button::Heavy, false, false).unwrap()];
        assert!(heavy.cooldown > 0);
        assert_eq!(heavy.projectile.as_ref().unwrap().image_name(), "bats");
    }
}

