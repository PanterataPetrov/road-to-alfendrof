use super::data::{AnimKind, Button, CharacterData, MoveData};
use super::input::Input;

/// Positions and velocities are integers in subpixel units: 1 pixel = `PX` units.
/// The simulation never uses floats, so it stays deterministic for rollback.
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
    Ko,
}

/// A curse on a fighter (see `CurseData`): which halves are still there, and how it is going.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Curse {
    /// Who cast it.
    pub owner: usize,
    pub left: bool,
    pub right: bool,
    pub age: u32,
    /// Frames since both halves were hit (0 = not yet); the curse squeezes shut meanwhile.
    pub squeeze: u32,
    pub squeeze_frames: u32,
    pub duration: u32,
    pub damage: i32,
}

#[derive(Clone, Hash, Debug)]
pub struct Fighter {
    /// Horizontal centre. Larger is further right.
    pub x: i32,
    /// Height above the ground. 0 is standing on the floor.
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    /// 1 faces right, -1 faces left.
    pub facing: i32,
    pub state: FighterState,
    /// Frames spent in the current state, starting at 1.
    pub state_frame: u32,
    pub health: i32,
    /// The health this fighter starts every round with (the health bar's full length).
    pub max_health: i32,
    pub prev_input: Input,
    /// Buttons pressed during hitstop, remembered so cancels can still use them.
    pub buffer: Input,
    /// Index into the character's `moves` while in `Attack`.
    pub attack: usize,
    /// True once the current attack has connected (a move hits at most once).
    pub hit_done: bool,
    /// True once the current attack has launched its projectile (a move launches at most one).
    pub launched: bool,
    /// True from the first air attack of a jump until landing (one air attack per jump).
    pub air_used: bool,
    /// Frames a chargeable move has been charged for.
    pub charge: u32,
    /// The special meter, filled by landing hits (0..=`METER_MAX`). The special needs it full.
    pub meter: i32,
    /// Shots of the current attack's volley fired so far.
    pub shots: u32,
    /// Blocked hits taken lately (a volley breaks the guard only when all its shots land).
    pub shield_hits: u32,
    pub shield_timer: u32,
    /// The moon summoned by a lobbing special: where it hangs (subpixels) and its age in frames.
    pub moon: Option<(i32, i32, u32)>,
    /// Where the lobbed projectile will land (subpixels), once locked in.
    pub lob_target: Option<i32>,
    /// A curse put on this fighter by the opponent.
    pub curse: Option<Curse>,
    /// Which arm throws the next punch (0 right, 1 left) and frames since the last punch started.
    pub punch_side: u8,
    pub since_punch: u32,
    /// Light-attack combo: how many light attacks in a row have connected (0..=3; the third is the
    /// finisher) and the frames since the last one. `finisher` is set while a finisher is being thrown.
    pub combo: u8,
    pub combo_timer: u32,
    pub finisher: bool,
    /// Frames left in which this fighter cannot block (after their guard was broken).
    pub block_disabled: u32,
    /// Frames of hitstun or blockstun to serve.
    pub stun: u32,
    /// Freeze frames after contact. The fighter does nothing while this is above zero.
    pub hitstop: u32,
    /// The next move with `boost_reach` gets its extra reach (set when a combo finisher lands), and
    /// whether the current move is using it.
    pub whip_boost: bool,
    pub boosted: bool,
    /// Subpixels this fighter is still being pulled along the ground (negative = left).
    pub pull: i32,
    /// The current move (with a `low_hitbox`) goes low instead of high.
    pub aim_low: bool,
}

/// The input flag of a move's button.
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
    /// Frames after the last connecting light attack before the combo runs out (the display of a
    /// completed combo goes quicker).
    pub const COMBO_WINDOW: u32 = 75;
    pub const COMBO_DISPLAY: u32 = 30;
    /// A broken guard cannot block again for 2.4 seconds.
    pub const BLOCK_DISABLED_FRAMES: u32 = 144;
    /// A blocked combo finisher stuns whoever threw it for 1.2 seconds.
    pub const FINISHER_STUN: u32 = 72;
    pub const GRAVITY: i32 = PX / 2;
    /// Half the width of the body used to keep fighters apart and inside the stage.
    pub const HALF_WIDTH: i32 = 17 * PX;
    pub const STAND_HEIGHT: i32 = 55;
    pub const CROUCH_HEIGHT: i32 = 40;
    pub const KO_HEIGHT: i32 = 14;
    /// How fast a pulled fighter is dragged, in subpixels per frame.
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

    /// Whether the fighter can take a new action (walk, jump, attack) right now.
    pub fn is_neutral(&self) -> bool {
        matches!(self.state, FighterState::Idle | FighterState::Walk | FighterState::Crouch | FighterState::Guard) && self.grounded()
    }

    pub fn set_state(&mut self, state: FighterState) {
        if self.state != state {
            self.state = state;
            self.state_frame = 0;
        }
    }

    /// Enter a stun state from a hit or block. Always restarts the state, even from itself.
    pub fn start_stun(&mut self, state: FighterState, frames: u32, push: i32) {
        self.state = state;
        self.state_frame = 0;
        self.stun = frames;
        self.vx = push;
    }

    fn begin_attack(&mut self, index: usize, mv: &MoveData) {
        // The third light attack of a combo is the finisher.
        if mv.button == Button::Light {
            if self.combo >= 3 {
                self.combo = 0;
            }
            self.finisher = self.combo == 2;
        } else {
            self.finisher = false;
        }
        // A move with extra reach spends the boost a finisher earned.
        if mv.boost_reach > 0 {
            self.boosted = self.whip_boost;
            self.whip_boost = false;
        } else {
            self.boosted = false;
        }
        // Spammed punches alternate arms; after a pause the right arm goes first again.
        if mv.anim == AnimKind::Punch {
            self.punch_side = if self.since_punch > 45 { 0 } else { self.punch_side ^ 1 };
            self.since_punch = 0;
        }
        self.set_state(FighterState::Attack);
        self.state_frame = 0;
        self.attack = index;
        self.hit_done = false;
        self.launched = false;
        self.charge = 0;
        self.shots = 0;
        self.lob_target = None;
        self.vx = 0;
    }

    /// Advance this fighter by one frame given its input.
    pub fn step(&mut self, input: Input, data: &CharacterData) {
        let pressed = Input((input.0 & !self.prev_input.0) | self.buffer.0);
        self.prev_input = input;

        if self.hitstop > 0 {
            self.hitstop -= 1;
            self.buffer = pressed;
            return;
        }
        self.buffer = Input::default();
        // Being pulled (a whip) drags the fighter along the ground.
        if self.pull != 0 {
            let step = self.pull.signum() * self.pull.abs().min(Self::PULL_SPEED);
            self.x += step;
            self.pull -= step;
        }
        self.state_frame += 1;
        self.since_punch = self.since_punch.saturating_add(1);
        self.block_disabled = self.block_disabled.saturating_sub(1);
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
                // A leaping move jumps when its startup ends; landing ends it.
                if mv.leap > 0 && self.state_frame == mv.startup && self.grounded() {
                    self.vy = mv.leap;
                    self.vx = self.facing * mv.leap_vx;
                    self.air_used = true;
                }
                // Each further hit window of a multi-hit move can hit again.
                if mv.hits > 1 && (1..mv.hits).any(|k| self.state_frame == mv.startup + k * mv.hit_gap + 1) {
                    self.hit_done = false;
                }
                // A lobbing special hangs its moon in the sky where the move was started.
                if self.state_frame == 1 {
                    if let Some(p) = mv.projectile.as_ref().filter(|p| p.lob) {
                        self.moon = Some((self.x + self.facing * p.x * PX, p.moon_height * PX, 0));
                    }
                }
                if self.grounded() && !self.air_used {
                    // Some moves let the fighter keep walking while attacking.
                    self.vx = input.horizontal() * data.walk_speed * mv.walk_pct / 100;
                }
                // Once a move has connected (hit or blocked), it can be cancelled into a heavier
                // button: light -> heavy -> special. This is what makes combos and block strings.
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
                // A chargeable move waits at the end of its startup while the button is held.
                if mv.charge_frames > 0 && self.state_frame == mv.startup && self.charge < mv.charge_frames && input.has(button_flag(mv.button)) {
                    self.charge += 1;
                    self.state_frame -= 1;
                }
                if self.state_frame > mv.total_frames() {
                    self.set_state(if self.grounded() { FighterState::Idle } else { FighterState::Jump });
                }
            }
            FighterState::Hitstun | FighterState::Blockstun => {
                // Pushback fades out quickly.
                if self.state_frame > self.stun {
                    self.vx = 0;
                    self.set_state(FighterState::Idle);
                } else {
                    self.vx = self.vx * 9 / 10;
                }
            }
            FighterState::Ko => self.vx = self.vx * 9 / 10,
            // Jumping: one air attack per jump, keeping the jump's momentum.
            FighterState::Jump => {
                if !self.air_used {
                    for (flag, button) in [(Input::HEAVY, Button::Heavy), (Input::LIGHT, Button::Light)] {
                        if pressed.has(flag) {
                            if let Some(index) = data.find_move(button, false, true) {
                                let vx = self.vx;
                                self.begin_attack(index, &data.moves[index]);
                                self.vx = vx;
                                self.air_used = true;
                                break;
                            }
                        }
                    }
                }
            }
            // Knocked into the air: velocity was set earlier.
            _ => {}
        }

        self.finish_step();
    }

    /// Gravity, movement and landing, at the end of a frame.
    fn finish_step(&mut self) {
        if !self.grounded() {
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
        }
    }

    fn neutral(&mut self, input: Input, pressed: Input, data: &CharacterData) {
        let crouching = input.has(Input::DOWN);
        for (flag, button) in [
            (Input::SPECIAL, Button::Special),
            (Input::HEAVY, Button::Heavy),
            (Input::LIGHT, Button::Light),
        ] {
            if pressed.has(flag) && (button != Button::Special || self.meter >= Self::METER_MAX) {
                // The third light attack of a combo may be a move of its own.
                let finisher = if button == Button::Light && !crouching && self.combo == 2 { data.find_finisher() } else { None };
                if let Some(index) = finisher.or_else(|| data.find_move(button, crouching, false)) {
                    self.begin_attack(index, &data.moves[index]);
                    return;
                }
            }
        }

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
