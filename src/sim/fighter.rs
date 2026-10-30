use super::data::{AnimKind, Button, CharacterData, MoveData};
use super::input::Input;

/// Positions and velocities are integers in subpixels (1 pixel = `PX`). No floats, so rollback stays deterministic.
pub const PX: i32 = 100;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FighterState {
    Idle,
    Walk,
    Crouch,
    Jump,
    Attack,
    Hitstun,
    Blockstun,
    /// Holding the block button.
    Guard,
    /// A quick burst along the ground, from double-tapping left or right.
    Dash,
    /// A roll (Q or E), on the ground or in the air; the middle of it can't be hit.
    Roll,
    Ko,
}

/// A curse on a fighter (see `CurseData`): which halves are left and how far along it is.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Curse {
    /// Who cast it.
    pub owner: usize,
    pub left: bool,
    pub right: bool,
    pub age: u32,
    /// Frames since both halves were hit (0 = not yet); it squeezes shut meanwhile.
    pub squeeze: u32,
    pub squeeze_frames: u32,
    pub duration: u32,
    pub damage: i32,
}

#[derive(Clone, Hash, Debug)]
pub struct Fighter {
    /// Horizontal centre; larger is further right.
    pub x: i32,
    /// Height above the ground (0 = on the floor).
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    /// 1 faces right, -1 faces left.
    pub facing: i32,
    pub state: FighterState,
    /// Frames in the current state, starting at 1.
    pub state_frame: u32,
    pub health: i32,
    /// Health at the start of each round (the length of the health bar).
    pub max_health: i32,
    pub prev_input: Input,
    /// Buttons pressed during hitstop, kept so cancels can use them.
    pub buffer: Input,
    /// Index into the character's `moves` while attacking.
    pub attack: usize,
    /// The attack has connected (a move hits once).
    pub hit_done: bool,
    /// The attack has launched its projectile (once per move).
    pub launched: bool,
    /// An air attack has been used since the last landing.
    pub air_used: bool,
    /// Frames a chargeable move has been charged.
    pub charge: u32,
    /// Special meter, filled by landing hits (0..=`METER_MAX`); the special needs it full.
    pub meter: i32,
    /// Shots of the current volley fired so far.
    pub shots: u32,
    /// Blocked hits taken lately (a volley breaks a guard only if every shot lands).
    pub shield_hits: u32,
    pub shield_timer: u32,
    /// Moon from a lobbing special: where it hangs (subpixels) and its age in frames.
    pub moon: Option<(i32, i32, u32)>,
    /// Where the lobbed projectile will land (subpixels), once locked in.
    pub lob_target: Option<i32>,
    /// A curse the opponent put on this fighter.
    pub curse: Option<Curse>,
    /// Which arm throws the next punch (0 right, 1 left) and frames since the last one.
    pub punch_side: u8,
    pub since_punch: u32,
    /// Light combo: connected lights in a row (0..=3, the third is the finisher) and frames since the
    /// last one. `finisher` is set while the finisher is thrown.
    pub combo: u8,
    pub combo_timer: u32,
    pub finisher: bool,
    /// Frames left in which blocking is impossible (after a guard break).
    pub block_disabled: u32,
    /// Frames of hitstun or blockstun left.
    pub stun: u32,
    /// Freeze frames after contact; the fighter does nothing until it hits zero.
    pub hitstop: u32,
    /// The next move with `boost_reach` gets extra reach (earned by landing a combo finisher), and
    /// whether the current move is using it.
    pub whip_boost: bool,
    pub boosted: bool,
    /// Subpixels still to be pulled along the ground (negative = left).
    pub pull: i32,
    /// The current move (if it has a `low_hitbox`) goes low instead of high.
    pub aim_low: bool,
    /// Air attacks used in this jump (see `CharacterData::air_chain`).
    pub air_count: u8,
    /// The current move hangs in the air (see `MoveData::hover`).
    pub hovering: bool,
    /// After a connected heavy or special: no cooldown for Light, and the next `quick` lights (for
    /// `quick_timer` more frames) start and recover faster. `fast` is set while such a light is thrown.
    pub quick: u8,
    pub quick_timer: u32,
    pub fast: bool,
    /// The raven is away from the shoulder (set by the game state each frame).
    pub raven_away: bool,
    /// Dash double-tap: direction of the last tap and frames since.
    pub tap_dir: i32,
    pub tap_timer: u32,
    /// What this frame's inputs ask for: a dash or a roll (-1 or 1).
    pub want_dash: i32,
    pub want_roll: i32,
    /// Direction of the dash or roll in progress.
    pub move_dir: i32,
    /// This jump's second jump and air roll have been used.
    pub double_jumped: bool,
    pub air_roll_used: bool,
    /// Frames before another ground roll is allowed.
    pub roll_cd: u32,
    /// Plunging to the floor (down held in the air). `slam_landed` is set for the frame it lands, when
    /// the game state deals the splash.
    pub slamming: bool,
    pub slam_landed: bool,
}

/// Input flag for a move's button.
fn button_flag(b: Button) -> u16 {
    match b {
        Button::Light => Input::LIGHT,
        Button::Heavy => Input::HEAVY,
        Button::Special => Input::SPECIAL,
    }
}

impl Fighter {
    pub const MAX_HEALTH: i32 = 1000;
    pub const METER_MAX: i32 = 100;
    /// Frames the moon stays in the sky.
    pub const MOON_LIFE: u32 = 200;
    /// Frames after the last connecting light before the combo runs out (a completed combo is shown for less).
    pub const COMBO_WINDOW: u32 = 75;
    pub const COMBO_DISPLAY: u32 = 30;
    /// A broken guard can't block again for 2.4 seconds.
    pub const BLOCK_DISABLED_FRAMES: u32 = 144;
    /// Tapping the same direction again within this many frames is a dash.
    pub const DASH_TAP_WINDOW: u32 = 15;
    pub const DASH_FRAMES: u32 = 14;
    /// In subpixels per frame.
    pub const DASH_SPEED: i32 = 520;
    /// A dash can be cancelled into an attack from this frame.
    pub const DASH_CANCEL: u32 = 6;
    pub const ROLL_FRAMES: u32 = 24;
    pub const ROLL_SPEED: i32 = 420;
    /// Frames of a roll in which the fighter can't be hit.
    pub const ROLL_SAFE: std::ops::RangeInclusive<u32> = 3..=17;
    pub const ROLL_COOLDOWN: u32 = 14;
    /// Ground pound: fall speed (subpixels per frame), minimum height, splash damage (half of one of
    /// Anaran's goblins' health) and reach either side, in pixels.
    pub const SLAM_SPEED: i32 = 1500;
    pub const SLAM_MIN_HEIGHT: i32 = 14 * PX;
    pub const SLAM_DAMAGE: i32 = 28;
    pub const SLAM_RADIUS: i32 = 44;
    /// The second jump is this percent of the first jump's speed.
    pub const DOUBLE_JUMP_PCT: i32 = 90;
    /// Frames after a connected heavy or special in which the next three lights are quick.
    pub const QUICK_WINDOW: u32 = 90;
    /// A blocked combo finisher stuns whoever threw it for 1.2 seconds.
    pub const FINISHER_STUN: u32 = 72;
    pub const GRAVITY: i32 = PX / 2;
    /// Half the body width, used to keep fighters apart and on the stage.
    pub const HALF_WIDTH: i32 = 17 * PX;
    pub const STAND_HEIGHT: i32 = 55;
    pub const CROUCH_HEIGHT: i32 = 40;
    pub const KO_HEIGHT: i32 = 14;
    /// How fast a pulled fighter is dragged, subpixels per frame.
    pub const PULL_SPEED: i32 = 14 * PX;

    pub fn new(x: i32, facing: i32) -> Self {
        Fighter {
            x,
            y: 0,
            vx: 0,
            vy: 0,
            facing,
            state: FighterState::Idle,
            state_frame: 0,
            health: Self::MAX_HEALTH,
            max_health: Self::MAX_HEALTH,
            prev_input: Input::default(),
            buffer: Input::default(),
            attack: 0,
            hit_done: false,
            launched: false,
            air_used: false,
            charge: 0,
            meter: 0,
            shots: 0,
            shield_hits: 0,
            shield_timer: 0,
            moon: None,
            lob_target: None,
            curse: None,
            punch_side: 0,
            since_punch: 1000,
            combo: 0,
            combo_timer: 0,
            finisher: false,
            block_disabled: 0,
            stun: 0,
            hitstop: 0,
            whip_boost: false,
            boosted: false,
            pull: 0,
            aim_low: false,
            air_count: 0,
            hovering: false,
            quick: 0,
            quick_timer: 0,
            fast: false,
            tap_dir: 0,
            tap_timer: 1000,
            want_dash: 0,
            want_roll: 0,
            move_dir: 1,
            double_jumped: false,
            air_roll_used: false,
            roll_cd: 0,
            slamming: false,
            slam_landed: false,
            raven_away: false,
        }
    }

    pub fn grounded(&self) -> bool {
        self.y == 0 && self.vy <= 0
    }

    /// Hurtbox height in pixels.
    pub fn height(&self) -> i32 {
        match self.state {
            FighterState::Crouch => Self::CROUCH_HEIGHT,
            FighterState::Ko => Self::KO_HEIGHT,
            _ => Self::STAND_HEIGHT,
        }
    }

    /// Can't be hit right now (mid-roll).
    pub fn invulnerable(&self) -> bool {
        self.state == FighterState::Roll && Self::ROLL_SAFE.contains(&self.state_frame)
    }

    fn begin_dash(&mut self, dir: i32) {
        self.move_dir = dir;
        self.vx = dir * Self::DASH_SPEED;
        self.set_state(FighterState::Dash);
        self.state_frame = 0;
    }

    fn begin_roll(&mut self, dir: i32) {
        self.move_dir = dir;
        self.vx = dir * Self::ROLL_SPEED;
        if !self.grounded() {
            self.air_roll_used = true;
            self.vy = 0;
        }
        self.set_state(FighterState::Roll);
        self.state_frame = 0;
    }

    /// Whether the fighter can act right now (walk, jump, attack).
    pub fn is_neutral(&self) -> bool {
        matches!(self.state, FighterState::Idle | FighterState::Walk | FighterState::Crouch | FighterState::Guard) && self.grounded()
    }

    pub fn set_state(&mut self, state: FighterState) {
        if self.state != state {
            self.state = state;
            self.state_frame = 0;
        }
    }

    /// Start a stun from a hit or block; restarts the state even if already in it.
    pub fn start_stun(&mut self, state: FighterState, frames: u32, push: i32) {
        // a ground pound isn't interrupted by hits (only a KO stops it), so the splash still lands
        if self.slamming && state != FighterState::Ko {
            return;
        }
        self.slamming = false;
        self.state = state;
        self.state_frame = 0;
        self.stun = frames;
        self.vx = push;
    }

    fn begin_attack(&mut self, index: usize, mv: &MoveData) {
        // the third light of a combo is the finisher
        if mv.button == Button::Light {
            if self.combo >= 3 {
                self.combo = 0;
            }
            self.finisher = self.combo == 2;
        } else {
            self.finisher = false;
        }
        // quick lights skip a third of their startup (and of their recovery, see `step`)
        self.fast = mv.button == Button::Light && self.quick > 0;
        if self.fast {
            self.quick -= 1;
        }
        // a move with extra reach spends the boost a finisher earned
        if mv.boost_reach > 0 {
            self.boosted = self.whip_boost;
            self.whip_boost = false;
        } else {
            self.boosted = false;
        }
        // spammed punches alternate arms; after a pause the right goes first again
        if mv.anim == AnimKind::Punch {
            self.punch_side = if self.since_punch > 45 { 0 } else { self.punch_side ^ 1 };
            self.since_punch = 0;
        }
        self.set_state(FighterState::Attack);
        self.state_frame = if self.fast { mv.startup / 3 } else { 0 };
        self.attack = index;
        self.hit_done = false;
        self.launched = false;
        self.charge = 0;
        self.shots = 0;
        self.lob_target = None;
        self.vx = 0;
    }

    /// Advance one frame.
    pub fn step(&mut self, input: Input, data: &CharacterData) {
        let pressed = Input((input.0 & !self.prev_input.0) | self.buffer.0);
        self.prev_input = input;

        if self.hitstop > 0 {
            self.hitstop -= 1;
            self.buffer = pressed;
            return;
        }
        self.buffer = Input::default();
        self.hovering = false;
        // being pulled (a whip) drags the fighter along the ground
        if self.pull != 0 {
            let step = self.pull.signum() * self.pull.abs().min(Self::PULL_SPEED);
            self.x += step;
            self.pull -= step;
        }
        self.state_frame += 1;
        // double-tapping left or right is a dash; Q and E are rolls
        self.tap_timer = self.tap_timer.saturating_add(1);
        self.roll_cd = self.roll_cd.saturating_sub(1);
        self.want_dash = 0;
        for (flag, dir) in [(Input::LEFT, -1), (Input::RIGHT, 1)] {
            if pressed.has(flag) {
                if self.tap_dir == dir && self.tap_timer <= Self::DASH_TAP_WINDOW {
                    self.want_dash = dir;
                    self.tap_dir = 0;
                } else {
                    self.tap_dir = dir;
                    self.tap_timer = 0;
                }
            }
        }
        self.want_roll = pressed.has(Input::ROLL_RIGHT) as i32 - pressed.has(Input::ROLL_LEFT) as i32;
        self.since_punch = self.since_punch.saturating_add(1);
        self.block_disabled = self.block_disabled.saturating_sub(1);
        if self.quick_timer > 0 {
            self.quick_timer -= 1;
            if self.quick_timer == 0 {
                self.quick = 0;
            }
        }
        if self.combo > 0 {
            self.combo_timer += 1;
            if self.combo_timer > if self.combo >= 3 { Self::COMBO_DISPLAY } else { Self::COMBO_WINDOW } {
                self.combo = 0;
            }
        }
        if self.shield_timer > 0 {
            self.shield_timer -= 1;
            if self.shield_timer == 0 {
                self.shield_hits = 0;
            }
        }
        if let Some((x, y, age)) = self.moon {
            self.moon = if age >= Self::MOON_LIFE { None } else { Some((x, y, age + 1)) };
        }

        match self.state {
            FighterState::Idle | FighterState::Walk | FighterState::Crouch | FighterState::Guard if self.grounded() => {
                self.vy = 0;
                self.neutral(input, pressed, data);
            }
            FighterState::Attack => {
                let mv = &data.moves[self.attack];
                self.hovering = mv.hover && !self.grounded() && self.state_frame <= mv.total_frames() - mv.recovery;
                // a leap jumps when its startup ends; landing ends it
                if mv.leap > 0 && self.state_frame == mv.startup && self.grounded() {
                    self.vy = mv.leap;
                    self.vx = self.facing * mv.leap_vx;
                    self.air_used = true;
                }
                // each further hit window of a multi-hit move can hit again
                if mv.hits > 1 && (1..mv.hits).any(|k| self.state_frame == mv.startup + k * mv.hit_gap + 1) {
                    self.hit_done = false;
                }
                // a lobbing special hangs its moon where the move started
                if self.state_frame == 1 {
                    if let Some(p) = mv.projectile.as_ref().filter(|p| p.lob) {
                        self.moon = Some((self.x + self.facing * p.x * PX, p.moon_height * PX, 0));
                    }
                }
                if self.grounded() && !self.air_used {
                    // some moves allow walking while attacking
                    self.vx = input.horizontal() * data.walk_speed * mv.walk_pct / 100;
                }
                // once a move has connected (hit or blocked) it can be cancelled into a heavier one: light -> heavy -> special
                if self.hit_done && self.grounded() {
                    let rank = |b: Button| match b {
                        Button::Light => 0,
                        Button::Heavy => 1,
                        Button::Special => 2,
                    };
                    let crouching = input.has(Input::DOWN);
                    for (flag, button) in [(Input::SPECIAL, Button::Special), (Input::HEAVY, Button::Heavy)] {
                        if pressed.has(flag) && rank(button) > rank(mv.button) && (button != Button::Special || self.meter >= Self::METER_MAX) {
                            if let Some(index) = data.find_move(button, crouching, false) {
                                self.begin_attack(index, &data.moves[index]);
                                return self.finish_step();
                            }
                        }
                    }
                }
                // a connected heavy or special can cancel into Light at once, and the next three lights are quick
                if data.light_after_heavy && self.hit_done && mv.button != Button::Light {
                    self.quick = 3;
                    self.quick_timer = Self::QUICK_WINDOW;
                    if pressed.has(Input::LIGHT) {
                        let air = !self.grounded();
                        if let Some(index) = data.find_move(Button::Light, input.has(Input::DOWN) && !air, air) {
                            let vx = self.vx;
                            self.begin_attack(index, &data.moves[index]);
                            if air {
                                self.vx = vx;
                                self.air_used = true;
                            }
                            return self.finish_step();
                        }
                    }
                }
                // air combos: a connected air light can chain into another one or the air heavy, which ends the chain
                let air_chain = data.air_chain.unwrap_or(1);
                if self.hit_done && !self.grounded() && mv.air && mv.button == Button::Light && (self.air_count as u32) < air_chain {
                    for (flag, button) in [(Input::HEAVY, Button::Heavy), (Input::LIGHT, Button::Light)] {
                        if pressed.has(flag) {
                            if let Some(index) = data.find_move(button, false, true) {
                                let vx = self.vx;
                                self.begin_attack(index, &data.moves[index]);
                                self.vx = vx;
                                self.air_used = true;
                                self.air_count += 1;
                                return self.finish_step();
                            }
                        }
                    }
                }
                // a chargeable move waits at the end of its startup while the button is held
                if mv.charge_frames > 0 && self.state_frame == mv.startup && self.charge < mv.charge_frames && input.has(button_flag(mv.button)) {
                    self.charge += 1;
                    self.state_frame -= 1;
                }
                if self.state_frame > mv.total_frames() - if self.fast { mv.recovery / 3 } else { 0 } {
                    self.set_state(if self.grounded() { FighterState::Idle } else { FighterState::Jump });
                }
            }
            FighterState::Dash => {
                self.vx = self.move_dir * Self::DASH_SPEED;
                // a dash can be cut short by an attack
                if self.state_frame >= Self::DASH_CANCEL && self.try_attack(input, pressed, data) {
                    return self.finish_step();
                }
                if self.state_frame > Self::DASH_FRAMES || !self.grounded() {
                    self.vx = 0;
                    self.set_state(if self.grounded() { FighterState::Idle } else { FighterState::Jump });
                }
            }
            FighterState::Roll => {
                self.vx = self.move_dir * Self::ROLL_SPEED;
                self.hovering = !self.grounded();
                if self.state_frame > Self::ROLL_FRAMES {
                    if self.grounded() {
                        self.vx = 0;
                        self.roll_cd = Self::ROLL_COOLDOWN;
                        self.set_state(FighterState::Idle);
                    } else {
                        self.vx = self.move_dir * Self::ROLL_SPEED / 2;
                        self.set_state(FighterState::Jump);
                    }
                }
            }
            FighterState::Hitstun | FighterState::Blockstun => {
                // pushback fades quickly
                if self.state_frame > self.stun {
                    self.vx = 0;
                    self.set_state(FighterState::Idle);
                } else {
                    self.vx = self.vx * 9 / 10;
                }
            }
            FighterState::Ko => self.vx = self.vx * 9 / 10,
            // jumping: air attacks keep the jump's momentum
            FighterState::Jump => {
                // down in the air plunges to the floor; the splash comes on landing
                if input.has(Input::DOWN) && self.y > Self::SLAM_MIN_HEIGHT {
                    self.slamming = true;
                    self.vx = 0;
                }
                if self.slamming {
                    self.vy = -Self::SLAM_SPEED;
                    return self.finish_step();
                }
                // air roll (once per jump), then the second jump (once)
                if self.want_roll != 0 && !self.air_roll_used {
                    self.begin_roll(self.want_roll);
                    return self.finish_step();
                }
                if pressed.has(Input::UP) && !self.double_jumped {
                    self.double_jumped = true;
                    self.vy = data.jump_speed * Self::DOUBLE_JUMP_PCT / 100;
                    if input.horizontal() != 0 {
                        self.vx = input.horizontal() * data.walk_speed;
                    }
                }
                // One air attack per jump, or more after a connected air attack (see `air_chain`).
                let more = self.hit_done && (self.air_count as u32) < data.air_chain.unwrap_or(1);
                if !self.air_used || more {
                    for (flag, button) in [(Input::SPECIAL, Button::Special), (Input::HEAVY, Button::Heavy), (Input::LIGHT, Button::Light)] {
                        if pressed.has(flag) && (button != Button::Special || self.meter >= Self::METER_MAX) {
                            if let Some(index) = data.find_move(button, false, true) {
                                let vx = self.vx;
                                self.begin_attack(index, &data.moves[index]);
                                self.vx = vx;
                                self.air_used = true;
                                self.air_count += 1;
                                break;
                            }
                        }
                    }
                }
            }
            // knocked into the air: velocity was set earlier
            _ => {}
        }

        self.finish_step();
    }

    /// Gravity, movement and landing at the end of a frame.
    fn finish_step(&mut self) {
        if self.hovering {
            self.vy = 0;
        } else if !self.grounded() {
            self.vy -= Self::GRAVITY;
        }
        self.x += self.vx;
        self.y += self.vy;
        if self.y <= 0 {
            self.y = 0;
            self.vy = 0;
            if self.state == FighterState::Jump || (self.state == FighterState::Attack && self.air_used) {
                self.vx = 0;
                self.set_state(FighterState::Idle);
            }
            self.air_used = false;
            self.air_count = 0;
            self.double_jumped = false;
            self.air_roll_used = false;
            if self.slamming {
                self.slamming = false;
                self.slam_landed = true;
            }
        }
    }

    /// Start an attack if a button was pressed for one this frame; returns whether one started.
    fn try_attack(&mut self, input: Input, pressed: Input, data: &CharacterData) -> bool {
        let crouching = input.has(Input::DOWN);
        for (flag, button) in [
            (Input::SPECIAL, Button::Special),
            (Input::HEAVY, Button::Heavy),
            (Input::LIGHT, Button::Light),
        ] {
            if pressed.has(flag) && (button != Button::Special || self.meter >= Self::METER_MAX) {
                // the light combo can have its own move for each hit (the third is the finisher, which needs the raven at home if it's the raven's claw)
                let stepped = if button == Button::Light && !crouching { data.find_combo_move(self.combo, !self.raven_away) } else { None };
                if let Some(index) = stepped.or_else(|| data.find_move(button, crouching, false)) {
                    // a raven that's away can't be sent again
                    if data.moves[index].raven_move.is_some() && self.raven_away {
                        continue;
                    }
                    self.begin_attack(index, &data.moves[index]);
                    return true;
                }
            }
        }

        false
    }

    fn neutral(&mut self, input: Input, pressed: Input, data: &CharacterData) {
        if self.try_attack(input, pressed, data) {
            return;
        }
        // a roll (not right after another) or a dash
        if self.want_roll != 0 && self.roll_cd == 0 {
            self.begin_roll(self.want_roll);
            return;
        }
        if self.want_dash != 0 && !input.has(Input::BLOCK) {
            self.begin_dash(self.want_dash);
            return;
        }
        let crouching = input.has(Input::DOWN);
        if input.has(Input::BLOCK) && self.block_disabled == 0 {
            self.vx = 0;
            self.set_state(FighterState::Guard);
            return;
        }
        if input.has(Input::UP) {
            self.vy = data.jump_speed;
            self.vx = input.horizontal() * data.walk_speed;
            self.set_state(FighterState::Jump);
        } else if crouching {
            self.vx = 0;
            self.set_state(FighterState::Crouch);
        } else {
            self.vx = input.horizontal() * data.walk_speed;
            self.set_state(if self.vx == 0 { FighterState::Idle } else { FighterState::Walk });
        }
    }
}
