//! The screen before a match: both fighters' skills and the game's controls, until both players press Enter.
//! The match waits in `Phase::Briefing` (see the sim); a player is ready when their input carries `Input::READY`,
//! so online both sides learn it the same way as every other button.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use t2f::sim::{Input, Phase};
use t2f::skills::skill_lines;

use crate::menus::{confirm_pressed, panel_parts};
use crate::online::NetSession;
use crate::settings::{key_name, Settings};
use crate::{rgb, HudRoot, Mode, Roster, Selection, Sim};

/// The whole screen; despawned when the match starts.
#[derive(Component)]
pub struct BriefingUi;
/// "READY" / "Press Enter" beside a fighter's name.
#[derive(Component)]
pub struct ReadyText(usize);
#[derive(Component)]
pub struct HintText;

/// A confirm key was pressed during the briefing: this player is ready.
#[derive(Resource, Default)]
pub struct ReadyPress(pub bool);

const GOLD: Color = Color::srgb(1.0, 0.9, 0.4);
const GREY: Color = Color::srgb(0.6, 0.6, 0.7);
const BODY: Color = Color::srgb(0.82, 0.82, 0.9);
const COLUMN_X: [f32; 2] = [-176.0, 6.0];

/// Draws the briefing over the fight scene, in the HUD (which stays put on screen).
pub fn spawn(
    mut commands: Commands,
    hud: Single<Entity, With<HudRoot>>,
    sim: Res<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut press: ResMut<ReadyPress>,
) {
    press.0 = false;
    if !matches!(sim.0.phase, Phase::Briefing(_)) {
        return;
    }
    let key = |flag: u16| key_name(settings.key(flag));
    let controls = [
        format!("Move {}/{}, tap twice to dash.  Jump {}.  Crouch {}.", key(Input::LEFT), key(Input::RIGHT), key(Input::UP), key(Input::DOWN)),
        format!("Light {}, Heavy {}, Special {} once the meter is full.", key(Input::LIGHT), key(Input::HEAVY), key(Input::SPECIAL)),
        format!("Block {}: hold to guard, hits only chip you.", key(Input::BLOCK)),
        format!("Roll {}/{}: slip through hits.  Jump twice to double jump.", key(Input::ROLL_LEFT), key(Input::ROLL_RIGHT)),
        format!("Ground slam: hold {} in the air to crash down, hurting", key(Input::DOWN)),
        "anyone near where you land (they can block it).".to_string(),
    ];
    let root = commands
        .spawn((BriefingUi, Transform::from_xyz(0.0, 0.0, 30.0), Visibility::default()))
        .with_children(|ui| {
            // covers the fight behind it, however wide the window
            ui.spawn((Sprite::from_color(Color::srgb(0.04, 0.03, 0.09), Vec2::new(3000.0, 1600.0)), Transform::from_xyz(0.0, 0.0, 0.0)));
            ui.spawn((
                Sprite { image: asset_server.load("backgrounds/select.png"), custom_size: Some(Vec2::new(384.0, 216.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
            for (sprite, at) in panel_parts() {
                ui.spawn((sprite, Transform::from_translation(at + Vec3::Z * 0.2)));
            }
            let mut text = |value: String, size: f32, color: Color, x: f32, y: f32| {
                ui.spawn((Text2d::new(value), crate::pixel_font(size), TextColor(color), Anchor::CENTER_LEFT, Transform::from_xyz(x, y, 1.0)));
            };
            for side in 0..2 {
                let c = &roster.0[sel.chars[side]];
                let x = COLUMN_X[side];
                text(c.name.clone(), 16.0, rgb(c.color), x, 80.0);
                for (n, line) in skill_lines(c).into_iter().enumerate() {
                    text(line.text, 8.0, if line.first { Color::WHITE } else { BODY }, x, 62.0 - n as f32 * 9.0);
                }
            }
            text("Controls".into(), 8.0, GOLD, -176.0, -12.0);
            for (n, line) in controls.into_iter().enumerate() {
                text(line, 8.0, BODY, -176.0, -23.0 - n as f32 * 9.0);
            }
            for side in 0..2 {
                ui.spawn((
                    ReadyText(side),
                    Text2d::new(""),
                    crate::pixel_font(8.0),
                    TextColor(GREY),
                    Anchor::CENTER_RIGHT,
                    Transform::from_xyz(COLUMN_X[side] + 172.0, 88.0, 1.0),
                ));
            }
            ui.spawn((HintText, Text2d::new(""), crate::pixel_font(8.0), TextColor(GREY), Transform::from_xyz(0.0, -92.0, 1.0)));
        })
        .id();
    commands.entity(*hud).add_child(root);
}

/// Enter (or F, J, Space) says "ready"; the sim hears it on its next step.
pub fn press_ready(keys: Res<ButtonInput<KeyCode>>, sim: Res<Sim>, mut press: ResMut<ReadyPress>) {
    if matches!(sim.0.phase, Phase::Briefing(_)) && confirm_pressed(&keys) {
        press.0 = true;
    }
}

/// Who is ready, and what to press; removes the screen once the match has begun.
pub fn update(
    mut commands: Commands,
    sim: Res<Sim>,
    mode: Res<Mode>,
    net: Option<Res<NetSession>>,
    screen: Query<Entity, With<BriefingUi>>,
    mut ready_texts: Query<(&ReadyText, &mut Text2d, &mut TextColor), Without<HintText>>,
    mut hint: Query<&mut Text2d, (With<HintText>, Without<ReadyText>)>,
) {
    let Phase::Briefing(ready) = sim.0.phase else {
        for e in &screen {
            commands.entity(e).despawn();
        }
        return;
    };
    let local = match (*mode, net) {
        (Mode::Online, Some(net)) => net.local,
        _ => 0,
    };
    for (ReadyText(side), mut text, mut color) in &mut ready_texts {
        let (label, tone) = if ready & (1 << side) != 0 {
            ("READY", Color::srgb(0.5, 1.0, 0.6))
        } else if *mode == Mode::VsCpu && *side == 1 {
            ("CPU", GREY)
        } else if *side == local {
            ("Press Enter", GOLD)
        } else {
            ("Not ready", GREY)
        };
        if text.0 != label {
            text.0 = label.into();
        }
        if color.0 != tone {
            color.0 = tone;
        }
    }
    let line = if ready & (1 << local) != 0 && ready != 0b11 {
        "Waiting for your friend...    Esc: leave"
    } else if *mode == Mode::Online {
        "Enter / F / J / Space: ready    Esc: leave"
    } else {
        "Enter / F / J / Space: fight    Esc: back to the menu"
    };
    for mut text in &mut hint {
        if text.0 != line {
            text.0 = line.into();
        }
    }
}
