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

use bevy::audio::{AudioSinkPlayback, Volume};
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
            .add_systems(FixedUpdate, (play_fight_sounds, fade_sustained).chain().run_if(in_state(AppState::Fight)))
            .add_systems(PostUpdate, play_ui_sounds);
    }
}

/// A sound that loops while a held move is held (`MoveSfx::sustain`): `on` until the move stops, then it fades out.
#[derive(Component)]
struct Sustained {
    owner: usize,
    on: bool,
    level: f32,
}

/// Fade each sustained sound in or out, and remove it once it is silent.
fn fade_sustained(mut commands: Commands, settings: Res<Settings>, mut sounds: Query<(Entity, &mut Sustained, &mut AudioSink)>) {
    for (entity, mut s, mut sink) in &mut sounds {
        s.level = if s.on { (s.level + 1.0 / FADE_IN).min(1.0) } else { (s.level - 1.0 / FADE_OUT).max(0.0) };
        sink.set_volume(Volume::Linear(settings.sfx_gain() * s.level));
        if !s.on && s.level <= 0.0 {
            commands.entity(entity).despawn();
        }
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

/// Which of `len` variants to play: picked from `hash`, but never the one played last time (when there is a choice).
fn pick_variant(last: Option<usize>, len: usize, hash: u32) -> usize {
    let pick = (hash >> 8) as usize % len;
    match last {
        Some(l) if len > 1 && l == pick => (pick + 1 + (hash >> 20) as usize % (len - 1)) % len,
        _ => pick,
    }
}

/// Groups named `voice_...` are a character's own sounds: played on top of the generic ones, at their natural
/// pitch, never the same variant twice in a row, and not again within `VOICE_GAP` frames (a flurry of hits is
/// one cry, not ten).
const VOICE_PREFIX: &str = "voice_";
const VOICE_GAP: u32 = 24;
/// Frames a sustained sound takes to fade in, and to fade out when its move stops.
const FADE_IN: f32 = 24.0;
const FADE_OUT: f32 = 60.0;

/// Starts sounds: picks a variant, varies the pitch a little and keeps the same sound from stacking up.
#[derive(Default)]
struct Voice {
    count: u32,
    last: HashMap<String, u32>,
    /// The variant each `voice_` group played last.
    last_variant: HashMap<String, usize>,
    /// Who the next `voice_` sound belongs to, and until which frame each character's voice lines are protected
    /// (a long cry started with `MoveSfx::voice_lock`): other lines from them stay quiet until then.
    speaker: usize,
    locks: [u32; 2],
}

impl Voice {
    /// True `pct` percent of the time (only for sound choices: nothing in the sim depends on it).
    fn chance(&mut self, pct: u32) -> bool {
        self.count = self.count.wrapping_add(1);
        (self.count.wrapping_mul(2654435761) >> 12) % 100 < pct
    }

    /// `throttle`: not again within `SAME_SOUND_GAP` frames of the last time it was started.
    fn play(&mut self, commands: &mut Commands, sounds: &Sounds, volume: f32, frame: u32, name: &str, throttle: bool) {
        let Some(variants) = sounds.groups.get(name).filter(|v| !v.is_empty()) else { return };
        let own = name.starts_with(VOICE_PREFIX);
        let gap = if own { VOICE_GAP } else { SAME_SOUND_GAP };
        if own && self.locks[self.speaker] > frame {
            return;
        }
        if volume <= 0.0 || ((throttle || own) && self.last.get(name).is_some_and(|&at| frame.wrapping_sub(at) < gap)) {
            return;
        }
        self.last.insert(name.to_string(), frame);
        self.count = self.count.wrapping_add(1);
        let hash = self.count.wrapping_mul(2654435761);
        let pick = if own { pick_variant(self.last_variant.get(name).copied(), variants.len(), hash) } else { (hash >> 8) as usize % variants.len() };
        if own {
            self.last_variant.insert(name.to_string(), pick);
        }
        let sound = &variants[pick];
        // a little pitch variation so repeated hits don't sound identical (not for the jingles, menus and voices)
        let jingle = own || ["round_", "match_", "ui_"].iter().any(|p| name.starts_with(p));
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
    mut sustained: Query<(Entity, &mut Sustained)>,
) {
    let volume = settings.sfx_gain();
    let mut start_sustain: Vec<(usize, Handle<AudioSource>)> = Vec::new();
    let s = &sim.0;
    let frame = s.frame;
    let Memory { voice, fighters, shots, minions, started, round, round_over, match_over } = &mut *memory;
    macro_rules! play {
        ($name:expr, $throttle:expr) => {
            voice.play(&mut commands, &sounds, volume, frame, $name, $throttle)
        };
    }
    let winner = s.match_winner();

    // the match and its rounds
    // the countdown ticks on each number, and the round-start sound says FIGHT! (`started`: it was counting down last tick)
    let counting = matches!(s.phase, Phase::Countdown(_));
    if let Phase::Countdown(n) = s.phase {
        if n % 60 == 0 {
            play!("ui_move", false);
        }
    }
    if *started && s.phase == Phase::Fighting {
        play!("round_start", false);
    }
    *started = counting;
    *round = s.round;
    let over = matches!(s.phase, Phase::RoundEnd(_) | Phase::MatchOver);
    let match_just_ended = matches!(s.phase, Phase::MatchOver) && !*match_over;
    // a character with a voice line of their own for winning gets that alone, without the generic jingle
    let own_win = winner.is_some_and(|w| roster.0[sel.chars[w]].voice.win.is_some());
    if match_just_ended {
        if !own_win {
            play!("match_win", false);
        }
    } else if over && !*round_over && !matches!(s.phase, Phase::MatchOver) {
        play!("round_end", false);
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
        let own = &roster.0[sel.chars[i]].voice;
        voice.speaker = i;
        if match_just_ended && winner == Some(i) {
            if let Some(name) = &own.win {
                play!(name, false);
            }
        }
        // a held move's looping sound (the scroll being read) runs from the start of the move until it stops
        {
            let data = &roster.0[sel.chars[i]];
            let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
            let active = f.state == FighterState::Attack && f.state_frame + 1 <= mv.startup;
            let live = sustained.iter_mut().find(|(_, s)| s.owner == i && s.on);
            match (mv.sfx.sustain.as_deref().filter(|_| active), live) {
                (Some(name), None) => {
                    if let Some(handle) = sounds.groups.get(name).and_then(|v| v.first()) {
                        start_sustain.push((i, handle.clone()));
                    }
                }
                (None, Some((_, mut s))) => s.on = false,
                _ => {}
            }
        }
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
                play!("ko", false);
                if let Some(name) = &own.ko {
                    play!(name, false);
                }
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
                    play!(sfx.and_then(|s| s.block.as_deref()).unwrap_or(generic), false);
                    if let Some(name) = &own.block {
                        play!(name, false);
                    }
                } else {
                    play!(sfx.and_then(|s| s.hit.as_deref()).unwrap_or(hit_name(lost)), false);
                    if let Some(name) = &own.hurt {
                        if voice.chance(own.hurt_chance.unwrap_or(100)) {
                            play!(name, false);
                        }
                    }
                    // the attacker's own cry for landing this move
                    if let Some((name, lock)) = sfx.and_then(|s| s.hit_voice.as_deref().map(|n| (n, s.voice_lock))) {
                        voice.speaker = foe;
                        if lock > 0 {
                            voice.locks[foe] = 0;
                        }
                        play!(name, false);
                        if lock > 0 {
                            voice.locks[foe] = frame + lock;
                        }
                        voice.speaker = i;
                    }
                }
            }

            // a plunge that has just hit the floor: the ground pound and Junkhead's whip smash
            if prev.slamming && !f.slamming && now.on_floor && now.state != FighterState::Ko {
                play!("smash", false);
                if let Some(name) = &own.smash {
                    play!(name, false);
                }
            }

            // a move starting: its own cast sound, or a whoosh for lights and heavies
            let data = &roster.0[sel.chars[i]];
            let mv = &data.moves[f.attack.min(data.moves.len() - 1)];
            if entered(FighterState::Attack) {
                match (&mv.sfx.cast, mv.button) {
                    (Some(name), _) => play!(name, false),
                    (None, Button::Light) => play!("whoosh_light", false),
                    (None, Button::Heavy) => play!("whoosh_heavy", false),
                    (None, Button::Special) => {}
                }
                if let Some(name) = &mv.sfx.voice {
                    if mv.sfx.voice_lock > 0 {
                        voice.locks[i] = 0;
                    }
                    play!(name, false);
                    if mv.sfx.voice_lock > 0 {
                        voice.locks[i] = frame + mv.sfx.voice_lock;
                    }
                }
            }
            // a shot leaving or minions arriving: the move's launch sound. This follows the thrower's own
            // counter, not the shot, so a shot that hits or vanishes at once is still heard.
            let summoning = mv.minions.is_some() && now.state == FighterState::Attack && f.state_frame == mv.startup + 1;
            if (f.shots > prev.shots && now.state == FighterState::Attack) || summoning {
                if let Some(name) = &mv.sfx.launch {
                    play!(name, true);
                }
            }

            // movement
            if prev.on_floor && !now.on_floor && now.state == FighterState::Jump {
                play!("jump", false);
            }
            if !prev.double_jumped && now.double_jumped {
                play!("jump_double", false);
            }
            if !prev.on_floor && now.on_floor && now.state != FighterState::Ko && !prev.slamming {
                play!("land", false);
            }
            if entered(FighterState::Dash) {
                play!("dash", false);
                if let Some(name) = &own.dash {
                    play!(name, false);
                }
            }
            if entered(FighterState::Roll) {
                play!("roll", false);
            }
            if entered(FighterState::Guard) {
                play!("guard", false);
            }
            if !prev.block_disabled && now.block_disabled {
                play!("guard_break", false);
                if let Some(name) = &own.guard_break {
                    play!(name, false);
                }
            }
            if now.state == FighterState::Walk && now.on_floor && now.state_frame % STEP_EVERY == 0 {
                play!("step", false);
            }
        }
        fighters[i] = Some(now);
    }
    *shots = now_shots;
    *minions = now_minions;
    if volume > 0.0 {
        for (owner, handle) in start_sustain {
            commands.spawn((
                AudioPlayer(handle),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
                Sustained { owner, on: true, level: 0.0 },
                DespawnOnExit(AppState::Fight),
            ));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_character_voice_is_never_the_same_variant_twice_in_a_row() {
        for len in [2usize, 3, 4] {
            let mut last = None;
            let mut seen = vec![false; len];
            for count in 1u32..2000 {
                let pick = pick_variant(last, len, count.wrapping_mul(2654435761));
                assert!(pick < len);
                assert_ne!(Some(pick), last, "{len} variants: repeated {pick} at {count}");
                seen[pick] = true;
                last = Some(pick);
            }
            assert!(seen.iter().all(|&s| s), "{len} variants: every one gets picked sometimes");
        }
        // with a single sound there is nothing else to pick
        assert_eq!(pick_variant(Some(0), 1, 12345), 0);
    }
}
