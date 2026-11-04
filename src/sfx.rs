//! Fight sound effects from `assets/sfx`. Works like the particles: it watches both fighters, the projectiles
//! and the minions between sim ticks, so the sim stays free of anything audio.
//!
//! Sounds are found by name: `assets/sfx/<name>.wav`, or several `<name>_0.wav`, `<name>_1.wav`... to pick from.
//! Every move can name its own in the character's .ron (`sfx: (cast, launch, hit, block)`, see `MoveSfx`); what it
//! leaves out falls back to a generic sound for its button, or for how hard the hit was.
//!
//! Generic names: `whoosh_light`, `whoosh_heavy`, `hit_small`, `hit_large`, `hit_heavy`, `block_small`,
//! `block_large`, `ko`, `smash`. Movement: `jump`, `jump_double`, `land`, `dash`, `roll`, `step`, `guard`,
//! `guard_break`. Match: `round_start`, `round_end`, `match_win`. Menus: `ui_move`, `ui_confirm`, `ui_back`.

use std::collections::HashMap;

use bevy::audio::Volume;
use bevy::prelude::*;
use t2f::sim::data::{Button, MoveSfx};
use t2f::sim::{FighterState, Phase};

use crate::settings::Settings;
use crate::{AppState, Roster, Selection, Sim};

pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Memory>()
            .add_message::<UiSound>()
            .add_systems(Startup, load_sounds)
            .add_systems(OnEnter(AppState::Fight), |mut memory: ResMut<Memory>| *memory = Memory::default())
            .add_systems(FixedUpdate, play_fight_sounds.run_if(in_state(AppState::Fight)))
            .add_systems(PostUpdate, play_ui_sounds);
    }
}

/// A menu sound, sent by the menus.
#[derive(Message, Clone, Copy)]
pub enum UiSound {
    Move,
    Confirm,
    Back,
    /// A sample hit, to hear the effects volume while setting it.
    Preview,
}

/// Damage at which a hit counts as large and as heavy (a light attack does about 25).
const LARGE_HIT: i32 = 35;
const HEAVY_HIT: i32 = 80;
/// Blocked damage at which the clack is the big one.
const LARGE_BLOCK: i32 = 15;
/// A launch sound isn't started again within this many frames (a volley of 20 knives would be a wall of noise).
const SAME_SOUND_GAP: u32 = 4;
/// Frames between footsteps while walking.
const STEP_EVERY: u32 = 14;

/// Every sound in `assets/sfx` by group name.
#[derive(Resource, Default)]
struct Sounds {
    groups: HashMap<String, Vec<Handle<AudioSource>>>,
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut files: Vec<String> = std::fs::read_dir(crate::assets_dir().join("sfx"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".wav"))
        .collect();
    files.sort();
    let mut groups: HashMap<String, Vec<Handle<AudioSource>>> = HashMap::new();
    for file in files {
        let stem = file.trim_end_matches(".wav");
        // "hit_small_1" belongs to the group "hit_small"
        let group = match stem.rsplit_once('_') {
            Some((group, number)) if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) => group,
            _ => stem,
        };
        groups.entry(group.to_string()).or_default().push(asset_server.load(format!("sfx/{file}")));
    }
    commands.insert_resource(Sounds { groups });
}

/// Starts sounds: picks a variant, varies the pitch a little and keeps the same sound from stacking up.
#[derive(Default)]
struct Voice {
    count: u32,
    last: HashMap<String, u32>,
}

impl Voice {
    /// `throttle`: not again within `SAME_SOUND_GAP` frames of the last time it was started.
    fn play(&mut self, commands: &mut Commands, sounds: &Sounds, volume: f32, frame: u32, name: &str, throttle: bool) {
        let Some(variants) = sounds.groups.get(name).filter(|v| !v.is_empty()) else { return };
        if volume <= 0.0 || (throttle && self.last.get(name).is_some_and(|&at| frame.wrapping_sub(at) < SAME_SOUND_GAP)) {
            return;
        }
        self.last.insert(name.to_string(), frame);
        self.count = self.count.wrapping_add(1);
        let hash = self.count.wrapping_mul(2654435761);
        let sound = &variants[(hash >> 8) as usize % variants.len()];
        // a little pitch variation so repeated hits don't sound identical (not for the jingles and menus)
        let jingle = ["round_", "match_", "ui_"].iter().any(|p| name.starts_with(p));
        let speed = if jingle { 1.0 } else { 0.94 + (hash >> 24) as f32 / 255.0 * 0.12 };
        commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)).with_speed(speed)));
    }
}

/// What a fighter looked like last tick.
#[derive(Clone, Copy)]
struct Snap {
    health: i32,
    state: FighterState,
    state_frame: u32,
    slamming: bool,
    on_floor: bool,
    hit_done: bool,
    double_jumped: bool,
    block_disabled: bool,
    /// Shots fired by the current move so far.
    shots: u32,
}

/// A projectile, told apart from the others by who threw it, with which move and on which frame.
type ShotId = (usize, usize, u32);

#[derive(Resource, Default)]
struct Memory {
    voice: Voice,
    fighters: [Option<Snap>; 2],
    shots: Vec<ShotId>,
    /// (owner, slot, summoned on frame) of every minion and whether it had bitten.
    minions: Vec<(usize, u32, u32, bool)>,
    started: bool,
    round: u8,
    round_over: bool,
    match_over: bool,
}

fn move_sfx<'a>(roster: &'a Roster, sel: &Selection, fighter: usize, index: usize) -> &'a MoveSfx {
    let data = &roster.0[sel.chars[fighter]];
    &data.moves[index.min(data.moves.len() - 1)].sfx
}

fn hit_name(lost: i32) -> &'static str {
    match lost {
        l if l >= HEAVY_HIT => "hit_heavy",
        l if l >= LARGE_HIT => "hit_large",
        _ => "hit_small",
    }
}

fn play_fight_sounds(
    mut commands: Commands,
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    settings: Res<Settings>,
    sounds: Res<Sounds>,
    mut memory: ResMut<Memory>,
) {
    let volume = settings.sfx_gain();
    let s = &sim.0;
    let frame = s.frame;
    let Memory { voice, fighters, shots, minions, started, round, round_over, match_over } = &mut *memory;
    let mut play = |name: &str, throttle: bool| voice.play(&mut commands, &sounds, volume, frame, name, throttle);

    // the match and its rounds
    if !*started || s.round != *round {
        play("round_start", false);
    }
    *started = true;
    *round = s.round;
    let over = matches!(s.phase, Phase::RoundEnd(_) | Phase::MatchOver);
    if matches!(s.phase, Phase::MatchOver) && !*match_over {
        play("match_win", false);
    } else if over && !*round_over && !matches!(s.phase, Phase::MatchOver) {
        play("round_end", false);
    }
    *round_over = over;
    *match_over = matches!(s.phase, Phase::MatchOver);

    // the shots that ended this tick: one of them may be what hit (launch sounds come from the thrower, below)
    let now_shots: Vec<ShotId> = s.projectiles.iter().map(|p| (p.owner, p.move_index, frame.wrapping_sub(p.age))).collect();
    let ended_shots: Vec<ShotId> = shots.iter().copied().filter(|id| !now_shots.contains(id)).collect();
    let now_minions: Vec<(usize, u32, u32, bool)> = s.minions.iter().map(|m| (m.owner, m.slot, frame.wrapping_sub(m.age), m.bit)).collect();
    // a minion that has just bitten: the move's hit sound
    let biter = s.minions.iter().find(|m| {
        let id = (m.owner, m.slot, frame.wrapping_sub(m.age));
        m.bit && minions.iter().any(|&(o, sl, at, bit)| (o, sl, at) == id && !bit)
    });

    let before = *fighters;
    for i in 0..2 {
        let f = &s.fighters[i];
        let now = Snap {
            health: f.health,
            state: f.state,
            state_frame: f.state_frame,
            slamming: f.slamming,
            on_floor: f.y == 0,
            hit_done: f.hit_done,
            double_jumped: f.double_jumped,
            block_disabled: f.block_disabled > 0,
            shots: f.shots,
        };
        if let Some(prev) = before[i] {
            let entered = |state: FighterState| now.state == state && (prev.state != state || now.state_frame < prev.state_frame);
            let lost = prev.health - now.health;
            let foe = 1 - i;

            // being hit or blocking: which move did it? a melee swing that connected, a shot that ended, a bite
            let hurt = entered(FighterState::Hitstun) || entered(FighterState::Blockstun) || lost > 0;
            if entered(FighterState::Ko) {
                play("ko", false);
            } else if hurt {
                let attacker = &s.fighters[foe];
                let melee = before[foe].is_some_and(|p| attacker.state == FighterState::Attack && attacker.hit_done && !p.hit_done);
                let source = if melee {
                    Some(attacker.attack)
                } else if let Some(&(_, index, _)) = ended_shots.iter().find(|id| id.0 == foe) {
                    Some(index)
                } else if let Some(m) = biter.filter(|m| m.owner == foe) {
                    Some(m.move_index)
                } else if attacker.state == FighterState::Attack {
                    // a shot that hit the moment it was fired was never seen in flight: it was this move's
                    Some(attacker.attack)
                } else {
                    None
                };
                let sfx = source.map(|index| move_sfx(&roster, &sel, foe, index));
                if now.state == FighterState::Blockstun {
                    let generic = if lost >= LARGE_BLOCK { "block_large" } else { "block_small" };
                    play(sfx.and_then(|s| s.block.as_deref()).unwrap_or(generic), false);
                } else {
                    play(sfx.and_then(|s| s.hit.as_deref()).unwrap_or(hit_name(lost)), false);
                }
            }

            // a plunge that has just hit the floor: the ground pound and Junkhead's whip smash
            if prev.slamming && !f.slamming && now.on_floor && now.state != FighterState::Ko {
                play("smash", false);
            }

            // a move starting: its own cast sound, or a whoosh for lights and heavies
            let data = &roster.0[sel.chars[i]];
            let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
            if entered(FighterState::Attack) {
                match (&mv.sfx.cast, mv.button) {
                    (Some(name), _) => play(name, false),
                    (None, Button::Light) => play("whoosh_light", false),
                    (None, Button::Heavy) => play("whoosh_heavy", false),
                    (None, Button::Special) => {}
                }
            }
            // a shot leaving or minions arriving: the move's launch sound. This follows the thrower's own
            // counter, not the shot, so a shot that hits or vanishes at once is still heard.
            let summoning = mv.minions.is_some() && now.state == FighterState::Attack && f.state_frame == mv.startup + 1;
            if (f.shots > prev.shots && now.state == FighterState::Attack) || summoning {
                if let Some(name) = &mv.sfx.launch {
                    play(name, true);
                }
            }

            // movement
            if prev.on_floor && !now.on_floor && now.state == FighterState::Jump {
                play("jump", false);
            }
            if !prev.double_jumped && now.double_jumped {
                play("jump_double", false);
            }
            if !prev.on_floor && now.on_floor && now.state != FighterState::Ko && !prev.slamming {
                play("land", false);
            }
            if entered(FighterState::Dash) {
                play("dash", false);
            }
            if entered(FighterState::Roll) {
                play("roll", false);
            }
            if entered(FighterState::Guard) {
                play("guard", false);
            }
            if !prev.block_disabled && now.block_disabled {
                play("guard_break", false);
            }
            if now.state == FighterState::Walk && now.on_floor && now.state_frame % STEP_EVERY == 0 {
                play("step", false);
            }
        }
        fighters[i] = Some(now);
    }
    *shots = now_shots;
    *minions = now_minions;
}

fn play_ui_sounds(mut commands: Commands, mut messages: MessageReader<UiSound>, sounds: Res<Sounds>, settings: Res<Settings>, mut voice: Local<Voice>, time: Res<Time>) {
    // menus have no sim frame: a running count of rendered frames keeps the same-sound gap working
    let frame = (time.elapsed_secs() * 60.0) as u32;
    for message in messages.read() {
        let name = match message {
            UiSound::Move => "ui_move",
            UiSound::Confirm => "ui_confirm",
            UiSound::Back => "ui_back",
            UiSound::Preview => "hit_small",
        };
        voice.play(&mut commands, &sounds, settings.sfx_gain(), frame, name, false);
    }
}
