use serde::Deserialize;

/// Loaded from `assets/characters/*.ron`. Lengths are in frames and distances in pixels, except
/// `walk_speed` and `jump_speed` which are in subpixel units per frame (see `PX`).
#[derive(Clone, Debug, Deserialize)]
pub struct CharacterData {
    pub name: String,
    /// A second line shown only in the character select ("Arch-Historian").
    #[serde(default)]
    pub subtitle: Option<String>,
    /// The subtitle is drawn as glitching, corrupted text (the Negaprofessor).
    #[serde(default)]
    pub corrupted_subtitle: bool,
    /// Folder under `assets/sprites/` holding this character's drawings (the lower-cased name if left out).
    #[serde(default)]
    pub sprites: Option<String>,
    /// Placeholder body colour (r, g, b) in 0..1 until real sprites exist.
    pub color: (f32, f32, f32),
    pub walk_speed: i32,
    pub jump_speed: i32,
    /// How close (centre to centre, in pixels) the CPU gets before it starts attacking when it plays
    /// this character (50 if left out). A long-reach fighter sets it higher.
    #[serde(default)]
    pub ai_range: Option<i32>,
    /// The CPU plays this character as a zoner: it backs away to keep its distance and shoots.
    #[serde(default)]
    pub keeps_distance: bool,
    /// Listed after everyone else in the character select (the Negaprofessor).
    #[serde(default)]
    pub menu_last: bool,
    /// Starting health (1000 if left out).
    #[serde(default)]
    pub health: Option<i32>,
    /// How fast the special meter fills, in percent of the normal rate (100 if left out).
    #[serde(default)]
    pub meter_gain_pct: Option<i32>,
    pub moves: Vec<MoveData>,
    /// How the character shows that the special is ready.
    #[serde(default)]
    pub special_mark: Option<MarkKind>,
}

/// A drawing that shows how full the special meter is (`<frame>_mark.png` over each frame).
#[derive(Clone, Debug, Deserialize)]
pub enum MarkKind {
    /// A mark on the skin that fades in as the meter fills, then glows when the special is ready.
    Scar { color: (f32, f32, f32), glow: (f32, f32, f32) },
    /// A hat that is on the head only when the special is ready.
    Hat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Button {
    Light,
    Heavy,
    Special,
}

/// A box in pixels. `x` is the distance in front of the fighter's centre (mirrored by facing),
/// `y` is the height above the feet.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Which set of upper-body drawings a move plays. Purely visual.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum AnimKind {
    /// The soft attack: 2 drawings.
    #[default]
    Soft,
    /// The hard attack: 4 drawings.
    Hard,
    /// The special, a summon: 6 drawings.
    Summon,
    /// The light attack as a punch that alternates between the right and the left arm while spammed.
    Punch,
    /// The hard attack as a leap with a series of kicks downwards.
    Kicks,
    /// The hard attack as a lecture: a speech bubble, a raised finger, and a blast of words.
    Lecture,
    /// The hard attack as a volley of shots, alternating hands (the Gunslinger's finger guns).
    Volley,
}

/// A projectile a move launches (a shooting star). It is aimed at the opponent when it
/// appears and flies in a straight line, hitting once for the move's damage.
#[derive(Clone, Debug, Deserialize)]
pub struct ProjectileData {
    /// Where it appears: pixels in front of the body centre, and pixels above the feet.
    pub x: i32,
    pub y: i32,
    /// Height above the opponent's feet that it aims for, in pixels.
    pub aim_y: i32,
    /// Frames it takes to reach where the opponent stood when it was launched.
    pub travel_frames: u32,
    /// Frames before it fizzles out if it hits nothing.
    pub lifetime: u32,
    /// Width and height of its hitbox in pixels.
    pub size: (i32, i32),
    /// Drawing of the projectile in flight (`<name>_0.png`, `<name>_1.png` in the character's
    /// sprite folder); "star" if left out.
    #[serde(default)]
    pub image: Option<String>,
    /// Drawing of the item summoned over the head while the move is wound up; "crescent" if left out.
    #[serde(default)]
    pub summon_image: Option<String>,
    /// Fixed speed in pixels per frame, straight ahead, instead of aiming at the opponent (0 = aimed).
    /// With `lifetime` this makes a short-range shot.
    #[serde(default)]
    pub speed: i32,
    /// Homes in on where the opponent is when it is launched. Everything else flies straight ahead
    /// in the direction the attacker faces, at `speed` (5 px a frame if left out), wherever the opponent is.
    #[serde(default)]
    pub aimed: bool,
    /// The projectile spins as it flies (a thrown hat).
    #[serde(default)]
    pub spin: bool,
    /// A lobbed projectile: it flies from a spot in the sky that stays where the move was started
    /// (the moon) down to where the opponent stood when it was launched, and splashes there for
    /// `splash_radius` pixels around, hurting anyone close to the ground.
    #[serde(default)]
    pub lob: bool,
    #[serde(default)]
    pub splash_radius: i32,
    /// How high above the ground the moon hangs, in pixels.
    #[serde(default)]
    pub moon_height: i32,
    /// The frame of the move at which the landing spot is locked in and shown on the ground; the
    /// projectile is only launched later, so there is time to get out of the way.
    #[serde(default)]
    pub lock_frame: u32,
}

impl ProjectileData {
    pub fn image_name(&self) -> &str {
        self.image.as_deref().unwrap_or("star")
    }
    pub fn summon_name(&self) -> &str {
        self.summon_image.as_deref().unwrap_or("crescent")
    }
}

/// Minions a move summons: `count` of them stay beside the summoner for `lifetime` frames and, one
/// after another (`interval` frames apart), rush the opponent at `speed` pixels a frame and bite
/// (the move's own damage and stun) if they get within `reach` pixels, then run home again.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MinionData {
    pub count: u32,
    pub lifetime: u32,
    pub interval: u32,
    pub speed: i32,
    pub reach: i32,
    /// Damage a minion can take before it dies; the opponent's attacks hit it like they would a fighter.
    pub health: i32,
}

/// A curse: it has a left half and a right half. Hitting the cursed fighter from the left removes
/// the left half, from the right the right half. With both gone it squeezes shut for
/// `squeeze` frames and deals the damage. It wears off after `duration` frames.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct CurseData {
    pub duration: u32,
    pub squeeze: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MoveData {
    pub name: String,
    pub button: Button,
    #[serde(default)]
    pub anim: AnimKind,
    /// Percent of walk speed the fighter keeps while this move plays (0 = rooted in place).
    #[serde(default)]
    pub walk_pct: i32,
    /// True if the move needs Down held when the button is pressed.
    pub crouching: bool,
    /// True if the move is used in the air (once per jump). Ground and air moves are separate.
    #[serde(default)]
    pub air: bool,
    /// For a chargeable move: how many frames the button can be held at the end of the startup
    /// to charge it, and the extra damage (percent) a full charge adds.
    #[serde(default)]
    pub charge_frames: u32,
    #[serde(default)]
    pub charge_bonus_pct: i32,
    pub startup: u32,
    pub active: u32,
    pub recovery: u32,
    pub damage: i32,
    pub hitstun: u32,
    pub blockstun: u32,
    /// Initial pushback of the defender in pixels per frame (decays quickly).
    pub knockback: i32,
    /// Frames both fighters freeze on contact.
    pub hitstop: u32,
    /// How many blocked hits it takes this move to break a guard (0 = it can never break one,
    /// which is the default for light attacks; 1 for everything else). A volley's shots add up.
    #[serde(default)]
    pub guard_hits: Option<u32>,
    /// A move that leaps: the upward speed (subpixels per frame) and the forward speed, applied
    /// when the startup ends. Landing ends the move.
    #[serde(default)]
    pub leap: i32,
    #[serde(default)]
    pub leap_vx: i32,
    /// How many times the hitbox becomes active (1 if 0), `hit_gap` frames apart; each time it can hit again.
    #[serde(default)]
    pub hits: u32,
    #[serde(default)]
    pub hit_gap: u32,
    /// A curse the move puts on the opponent instead of hitting them (the move's damage is dealt
    /// when the curse is completed).
    #[serde(default)]
    pub curse: Option<CurseData>,
    /// Minions the move summons (see `MinionData`).
    #[serde(default)]
    pub minions: Option<MinionData>,
    /// The move's own hitbox. A width of 0 means the move has no melee hit (a summon).
    pub hitbox: Rect,
    /// How many shots the move fires (1 if 0), `volley_gap` frames apart. Each shot is a projectile
    /// that hits for the move's damage, so make the damage small.
    #[serde(default)]
    pub volley: u32,
    #[serde(default)]
    pub volley_gap: u32,
    /// A projectile launched on the first active frame. It uses this move's damage and stun.
    #[serde(default)]
    pub projectile: Option<ProjectileData>,
    /// A light move that is thrown instead of the normal light attack as the third hit of a combo
    /// (the finisher). It must come after the normal light attack in the character's move list.
    #[serde(default)]
    pub finisher_move: bool,
    /// Extra reach in pixels the move's hitbox gets once after a combo finisher has landed.
    #[serde(default)]
    pub boost_reach: i32,
    /// On a hit the attacker is pulled towards the opponent until they are `pull_gap` pixels apart
    /// (only half the way if the hit was blocked).
    #[serde(default)]
    pub pull_self: bool,
    /// On a hit the opponent is dragged towards the attacker until they are `pull_gap` pixels apart.
    #[serde(default)]
    pub pull_foe: bool,
    #[serde(default)]
    pub pull_gap: i32,
    /// A second hitbox: each time the move starts it randomly uses `hitbox` or this one (a lash that
    /// goes either high or low).
    #[serde(default)]
    pub low_hitbox: Option<Rect>,
}

impl MoveData {
    pub fn guard_need(&self) -> u32 {
        self.guard_hits.unwrap_or(if self.button == Button::Light { 0 } else { 1 })
    }

    pub fn total_frames(&self) -> u32 {
        self.startup + (self.hits.max(1) - 1) * self.hit_gap + self.active + self.recovery
    }

    /// `frame` counts from 1 on the first frame of the move.
    pub fn is_active(&self, frame: u32) -> bool {
        self.window(frame).is_some()
    }

    /// Which of the move's hit windows (0, 1, ...) `frame` is in, if any.
    pub fn window(&self, frame: u32) -> Option<u32> {
        (0..self.hits.max(1)).find(|&k| {
            let start = self.startup + k * self.hit_gap;
            frame > start && frame <= start + self.active
        })
    }
}

impl CharacterData {
    pub fn from_ron(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(text)
    }

    /// The light move thrown as the third hit of a combo, if the character has one.
    pub fn find_finisher(&self) -> Option<usize> {
        self.moves.iter().position(|m| m.finisher_move)
    }

    /// The move triggered by `button`, preferring a crouching version when Down is held. Air
    /// moves are only found with `air` set.
    pub fn find_move(&self, button: Button, crouching: bool, air: bool) -> Option<usize> {
        let exact = self.moves.iter().position(|m| m.button == button && m.air == air && m.crouching == crouching);
        exact.or_else(|| self.moves.iter().position(|m| m.button == button && m.air == air && !m.crouching))
    }
}
