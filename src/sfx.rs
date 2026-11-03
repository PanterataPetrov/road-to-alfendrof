//! Fight sound effects from `assets/sfx`: a whoosh when an attack starts, a thud when it lands (by how hard),
//! a clack when it is blocked and a heavy crunch on the KO. Works like the particles: it watches both fighters
//! between sim ticks, so the sim stays free of anything audio.

use bevy::prelude::*;
use t2f::sim::data::Button;
use t2f::sim::FighterState;

use crate::settings::Settings;
use crate::{AppState, Roster, Selection, Sim};

pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sounds)
            .add_systems(FixedUpdate, play_fight_sounds.run_if(in_state(AppState::Fight)));
    }
}

/// Damage at which a hit counts as large and as heavy (a light attack does about 25).
const LARGE_HIT: i32 = 35;
const HEAVY_HIT: i32 = 80;

#[derive(Resource)]
struct Sounds {
    hit_small: [Handle<AudioSource>; 2],
    hit_large: [Handle<AudioSource>; 2],
    hit_heavy: [Handle<AudioSource>; 2],
    block_small: Handle<AudioSource>,
    block_large: Handle<AudioSource>,
    whoosh_light: Handle<AudioSource>,
    whoosh_heavy: Handle<AudioSource>,
    ko: Handle<AudioSource>,
    smash: Handle<AudioSource>,
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let load = |name: &str| asset_server.load::<AudioSource>(format!("sfx/{name}.wav"));
    let pair = |name: &str| [load(&format!("{name}_0")), load(&format!("{name}_1"))];
    commands.insert_resource(Sounds {
        hit_small: pair("hit_small"),
        hit_large: pair("hit_large"),
        hit_heavy: pair("hit_heavy"),
        block_small: load("block_small"),
        block_large: load("block_large"),
        whoosh_light: load("whoosh_light"),
        whoosh_heavy: load("whoosh_heavy"),
        ko: load("ko"),
        smash: load("smash"),
    });
}

/// What a fighter looked like last tick.
#[derive(Clone, Copy)]
struct Snap {
    health: i32,
    state: FighterState,
    state_frame: u32,
    slamming: bool,
    on_floor: bool,
}

fn play_fight_sounds(
    mut commands: Commands,
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    settings: Res<Settings>,
    sounds: Option<Res<Sounds>>,
    mut last: Local<[Option<Snap>; 2]>,
    mut count: Local<u32>,
) {
    let Some(sounds) = sounds else { return };
    let volume = settings.sfx_gain();
    let play = |commands: &mut Commands, sound: &Handle<AudioSource>, count: &mut u32| {
        *count = count.wrapping_add(1);
        if volume > 0.0 {
            // a little pitch variation so repeated hits don't sound identical
            let speed = 0.93 + (count.wrapping_mul(2654435761) >> 24) as f32 / 255.0 * 0.14;
            commands.spawn((
                AudioPlayer(sound.clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume)).with_speed(speed),
            ));
        }
    };

    for i in 0..2 {
        let f = &sim.0.fighters[i];
        let now = Snap { health: f.health, state: f.state, state_frame: f.state_frame, slamming: f.slamming, on_floor: f.y == 0 };
        if let Some(prev) = last[i] {
            let entered = |state: FighterState| now.state == state && (prev.state != state || now.state_frame < prev.state_frame);
            let lost = prev.health - now.health;
            let variant = *count as usize % 2;
            if entered(FighterState::Ko) {
                play(&mut commands, &sounds.ko, &mut count);
            } else if entered(FighterState::Blockstun) {
                let block = if lost >= 15 { &sounds.block_large } else { &sounds.block_small };
                play(&mut commands, block, &mut count);
            } else if lost > 0 && now.state != FighterState::Blockstun {
                let hit = match lost {
                    l if l >= HEAVY_HIT => &sounds.hit_heavy[variant],
                    l if l >= LARGE_HIT => &sounds.hit_large[variant],
                    _ => &sounds.hit_small[variant],
                };
                play(&mut commands, hit, &mut count);
            }
            // a plunge that has just hit the floor: the ground pound and Junkhead's whip smash
            if prev.slamming && !f.slamming && now.on_floor && now.state != FighterState::Ko {
                play(&mut commands, &sounds.smash, &mut count);
            }
            if entered(FighterState::Attack) {
                let data = &roster.0[sel.chars[i]];
                match data.moves[f.attack.min(data.moves.len() - 1)].button {
                    Button::Light => play(&mut commands, &sounds.whoosh_light, &mut count),
                    Button::Heavy => play(&mut commands, &sounds.whoosh_heavy, &mut count),
                    Button::Special => {}
                }
            }
        }
        last[i] = Some(now);
    }
}
