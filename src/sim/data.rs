use serde::Deserialize;

/// Loaded from `assets/characters/*.ron`. Times are in frames, distances in pixels, except
/// `walk_speed` and `jump_speed` (subpixels per frame, see `PX`).
#[derive(Clone, Debug, Deserialize)]
pub struct CharacterData {
    pub name: String,
    /// Second line, shown only in the character select.
    #[serde(default)]
    pub subtitle: Option<String>,
    /// Draw the subtitle as glitching text.
    #[serde(default)]
    pub corrupted_subtitle: bool,
    /// A connected heavy or special can be cancelled into Light at once, and the next three lights are quick.
    #[serde(default)]
    pub light_after_heavy: bool,
    /// Folder under `assets/sprites/` with this character's drawings (default: the lower-cased name).
    #[serde(default)]
    pub sprites: Option<String>,
    /// Placeholder body colour (r, g, b), 0..1.
    pub color: (f32, f32, f32),
    pub walk_speed: i32,
    pub jump_speed: i32,
    /// How close (pixels, centre to centre) the CPU gets before it attacks with this character
    /// (default 50). Long-reach fighters set it higher.
    #[serde(default)]
    pub cpu_range: Option<i32>,
    /// Air attacks allowed per jump (default 1). With more, a connected air light can chain into
    /// another one or the air heavy.
    #[serde(default)]
    pub air_chain: Option<u32>,
    /// Whip layers are drawn over the body (`anim::whip_layer`).
    #[serde(default)]
    pub whip: bool,
    /// A raven on the shoulder that `raven_move` moves can send out.
    #[serde(default)]
    pub raven: Option<RavenData>,
    /// The CPU plays this one as a zoner: it backs away to keep its distance and shoots.
    #[serde(default)]
    pub keeps_distance: bool,
    /// Goes after everyone else in the character select.
    #[serde(default)]
    pub menu_last: bool,
    /// Starting health (default 1000).
    #[serde(default)]
    pub health: Option<i32>,
    /// How fast the special meter fills, in percent of normal (default 100).
    #[serde(default)]
    pub meter_gain_pct: Option<i32>,
    pub moves: Vec<MoveData>,
    /// How the special being ready is shown.
    #[serde(default)]
    pub special_mark: Option<MarkKind>,
}

/// Overlay showing how full the special meter is (`<frame>_mark.png` over each frame).
#[derive(Clone, Debug, Deserialize)]
pub enum MarkKind {
    /// A mark on the skin that fades in as the meter fills and glows when ready.
    Scar { color: (f32, f32, f32), glow: (f32, f32, f32) },
    /// A hat that is only on the head when the special is ready.
    Hat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Button {
    Light,
    Heavy,
    Special,
}

/// A box in pixels. `x` is the distance in front of the fighter (mirrored by facing), `y` the height above the feet.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Which set of upper-body drawings a move uses. Visual only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum AnimKind {
    /// Soft attack: 2 drawings.
    #[default]
    Soft,
    /// Hard attack: 4 drawings.
    Hard,
    /// Special summon: 6 drawings.
    Summon,
    /// Light punch that alternates arms while spammed.
    Punch,
    /// Hard attack as a leap with a series of kicks.
    Kicks,
    /// Hard attack as a lecture: speech bubble, raised finger, blast of words.
    Lecture,
    /// Hard attack as a volley of shots from alternating hands.
    Volley,
    /// Speaking at the opponent: 2 drawings, breath then the word.
    Speak,
    /// Commanding the raven: raised finger, then the arm out (3 drawings).
    Point,
    /// A grab: beckon, pull in, hold by the collar while talking, shove away (6 drawings).
    Grab,
}

/// A projectile a move launches. It flies in a straight line and hits once for the move's damage.
#[derive(Clone, Debug, Deserialize)]
pub struct ProjectileData {
    /// Where it appears: pixels in front of the body centre and above the feet.
    pub x: i32,
    pub y: i32,
    /// Height above the opponent's feet it aims for, in pixels.
    pub aim_y: i32,
    /// Frames to reach where the opponent was when it was launched.
    pub travel_frames: u32,
    /// Frames before it fizzles out.
    pub lifetime: u32,
    /// Hitbox width and height in pixels.
    pub size: (i32, i32),
    /// Drawing in flight (`<name>_0.png`, `<name>_1.png` in the sprite folder); default "star".
    #[serde(default)]
    pub image: Option<String>,
    /// Item shown over the head during the wind-up; default "crescent".
    #[serde(default)]
    pub summon_image: Option<String>,
    /// Speed in pixels per frame, straight ahead (default 5). With `lifetime` it sets the range.
    #[serde(default)]
    pub speed: i32,
    /// Same, in tenths of a pixel (32 = 3.2 px); wins over `speed`.
    #[serde(default)]
    pub speed_tenths: i32,
    /// Homes in on where the opponent is at launch. Everything else flies straight ahead in the
    /// direction the attacker faces.
    #[serde(default)]
    pub aimed: bool,
    /// Spins in flight (a thrown hat).
    #[serde(default)]
    pub spin: bool,
    /// Never turned to face where it flies (words stay readable).
    #[serde(default)]
    pub upright: bool,
    /// A lob: flies from a spot in the sky (the moon) down to where the opponent stood at launch
    /// and splashes for `splash_radius` pixels, hurting anyone near the ground.
    #[serde(default)]
    pub lob: bool,
    #[serde(default)]
    pub splash_radius: i32,
    /// How high the moon hangs, in pixels.
    #[serde(default)]
    pub moon_height: i32,
    /// Frame of the move where the landing spot is locked in and shown on the ground; the
    /// projectile launches later so there is time to move.
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

/// Minions a move summons: `count` stay beside the summoner for `lifetime` frames, rush the
/// opponent at `speed` pixels a frame and bite (the move's damage) within `reach` pixels, one bite
/// every `interval` frames.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MinionData {
    pub count: u32,
    pub lifetime: u32,
    pub interval: u32,
    pub speed: i32,
    pub reach: i32,
    /// Damage a minion can take before it dies.
    pub health: i32,
}

/// A raven on a shoulder: where (pixels in front of the body centre, above the feet) and how
/// fast it flies (pixels a frame).
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct RavenData {
    pub home: (i32, i32),
    pub speed: i32,
}

/// What a move sends the raven to do. It is away until it is back.
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum RavenMove {
    /// Fly at the opponent, claw them once (the move's damage and stun), fly back.
    Claw,
    /// Fly to the opponent and peck them `pecks` times (the move's damage each), switching sides
    /// between pecks, then drop a rose over its owner that heals `heal_pct` percent of their health,
    /// then go home.
    Storm { pecks: u32, heal_pct: i32 },
}

/// What one hit window of a multi-hit move does instead of the move's own numbers.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct HitOverride {
    pub damage: i32,
    pub hitstun: u32,
    pub knockback: i32,
}

/// A curse has a left and a right half. Hitting the cursed fighter from the left removes the
/// left half, from the right the right half. With both gone it squeezes shut for `squeeze` frames
/// and deals the damage. It wears off after `duration` frames.
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
    /// Percent of walk speed kept while the move plays (0 = rooted).
    #[serde(default)]
    pub walk_pct: i32,
    /// Needs Down held when the button is pressed.
    pub crouching: bool,
    /// Used in the air (once per jump). Ground and air moves are separate.
    #[serde(default)]
    pub air: bool,
    /// Chargeable move: frames the button can be held at the end of the startup, and the extra
    /// damage (percent) of a full charge.
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
    /// Initial pushback of the defender, pixels per frame (decays quickly).
    pub knockback: i32,
    /// Frames both fighters freeze on contact.
    pub hitstop: u32,
    /// Blocked hits it takes to break a guard (0 = never, the default for lights; 1 for the rest).
    /// A volley's shots add up.
    #[serde(default)]
    pub guard_hits: Option<u32>,
    /// A leap: upward speed (subpixels per frame) and forward speed, applied when the startup ends.
    /// Landing ends the move.
    #[serde(default)]
    pub leap: i32,
    #[serde(default)]
    pub leap_vx: i32,
    /// How many times the hitbox goes active (1 if 0), `hit_gap` frames apart; each can hit again.
    #[serde(default)]
    pub hits: u32,
    #[serde(default)]
    pub hit_gap: u32,
    /// Melee aim assist, in pixels: reaches this much further, grows this much taller each way, and
    /// slides up to twice this far towards the opponent's height.
    #[serde(default)]
    pub assist: i32,
    /// A curse put on the opponent instead of a hit (the move's damage is dealt when it completes).
    #[serde(default)]
    pub curse: Option<CurseData>,
    /// Minions the move summons (see `MinionData`).
    #[serde(default)]
    pub minions: Option<MinionData>,
    /// The move's own hitbox. Width 0 means no melee hit (a summon).
    pub hitbox: Rect,
    /// Shots fired (1 if 0), `volley_gap` frames apart. Each is a projectile hitting for the move's
    /// damage, so keep that small.
    #[serde(default)]
    pub volley: u32,
    #[serde(default)]
    pub volley_gap: u32,
    /// Projectile launched on the first active frame, using this move's damage and stun.
    #[serde(default)]
    pub projectile: Option<ProjectileData>,
    /// Light move thrown instead of the normal light as the third hit of a combo (the finisher).
    /// Must come after the normal light in the move list.
    #[serde(default)]
    pub finisher_move: bool,
    /// Extra reach in pixels, once, after a combo finisher has landed.
    #[serde(default)]
    pub boost_reach: i32,
    /// On a hit the attacker is pulled towards the opponent until `pull_gap` pixels apart (half way
    /// if blocked).
    #[serde(default)]
    pub pull_self: bool,
    /// On a hit the opponent is dragged towards the attacker until `pull_gap` pixels apart.
    #[serde(default)]
    pub pull_foe: bool,
    #[serde(default)]
    pub pull_gap: i32,
    /// A second hitbox: each time the move starts it picks `hitbox` or this at random (a lash that
    /// goes high or low).
    #[serde(default)]
    pub low_hitbox: Option<Rect>,
    /// Air move that hangs (no gravity) from its start to its last active frame.
    #[serde(default)]
    pub hover: bool,
    /// Hitbox heights are measured from the ground instead of the feet (an air move that lashes at
    /// the floor).
    #[serde(default)]
    pub world_y: bool,
    /// Standing light used at this step of the light combo (0 = first, 1 = second; the third is
    /// `finisher_move`).
    #[serde(default)]
    pub combo_step: Option<u8>,
    /// Sends the raven out (see `RavenMove`) when the startup is over.
    #[serde(default)]
    pub raven_move: Option<RavenMove>,
    /// What each hit window of a multi-hit move does instead of `damage`, `hitstun` and
    /// `knockback` (a grab, then a shove).
    #[serde(default)]
    pub hit_overrides: Vec<HitOverride>,
}

impl MoveData {
    pub fn guard_need(&self) -> u32 {
        self.guard_hits.unwrap_or(if self.button == Button::Light { 0 } else { 1 })
    }

    pub fn total_frames(&self) -> u32 {
        self.startup + (self.hits.max(1) - 1) * self.hit_gap + self.active + self.recovery
    }

    /// The move as it hits in window `k`, with that window's own numbers if it has them.
    pub fn for_window(&self, k: Option<u32>) -> std::borrow::Cow<'_, MoveData> {
        match k.and_then(|k| self.hit_overrides.get(k as usize)) {
            Some(o) => {
                let mut m = self.clone();
                m.damage = o.damage;
                m.hitstun = o.hitstun;
                m.knockback = o.knockback;
                std::borrow::Cow::Owned(m)
            }
            None => std::borrow::Cow::Borrowed(self),
        }
    }

    /// `frame` counts from 1 on the first frame of the move.
    pub fn is_active(&self, frame: u32) -> bool {
        self.window(frame).is_some()
    }

    /// Which hit window (0, 1, ...) `frame` is in, if any.
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

    /// The light move thrown as the third hit of a combo, if there is one.
    pub fn find_finisher(&self) -> Option<usize> {
        self.moves.iter().position(|m| m.finisher_move)
    }

    /// The standing light for a combo with `combo` hits so far: the finisher on the third (when it
    /// can be thrown), otherwise the `combo_step` move for that hit, or `None` for the ordinary light.
    pub fn find_combo_move(&self, combo: u8, finisher_ok: bool) -> Option<usize> {
        if combo == 2 && finisher_ok {
            if let Some(i) = self.find_finisher() {
                return Some(i);
            }
        }
        let step = if combo == 1 { 1 } else { 0 };
        self.moves.iter().position(|m| m.button == Button::Light && !m.air && !m.crouching && m.combo_step == Some(step))
    }

    /// The move for `button`, preferring a crouching version when Down is held. Air moves are only
    /// found with `air` set.
    pub fn find_move(&self, button: Button, crouching: bool, air: bool) -> Option<usize> {
        let exact = self.moves.iter().position(|m| m.button == button && m.air == air && m.crouching == crouching);
        exact.or_else(|| self.moves.iter().position(|m| m.button == button && m.air == air && !m.crouching))
    }
}
