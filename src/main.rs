mod menus;
mod online;
mod particles;
mod render;

use std::path::PathBuf;

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::WindowResolution;
use t2f::cpu::cpu_input_vs;
use t2f::sim::{CharacterData, Fighter, GameState, Input, Phase, PX};
use t2f::stage::StageData;

/// Pixel-art resolution; the window scales it up.
const VIEW_W: f32 = 384.0;
const VIEW_H: f32 = 216.0;
/// Camera zoom: the narrowest view (fighters close together), how much wider than the gap between the
/// fighters it is otherwise, how fast it follows, and how far up from the bottom of the view the floor sits.
const MIN_VIEW_W: f32 = 346.0;
const ZOOM_MARGIN: f32 = 220.0;
const ZOOM_SPEED: f32 = 5.0;
const FLOOR_FRACTION: f32 = 0.12;
/// Floor line, in world pixels.
const GROUND_Y: f32 = -80.0;

const BAR_W: f32 = 160.0;
const BAR_H: f32 = 10.0;
const BAR_Y: f32 = 94.0;
const BAR_EDGE: f32 = 180.0;

const MENU_BACKGROUND: Color = Color::srgb(0.07, 0.07, 0.12);

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    #[default]
    Title,
    CharSelect,
    StageSelect,
    Online,
    Fight,
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    #[default]
    VsCpu,
    Online,
}

/// Characters from `assets/characters`.
#[derive(Resource)]
pub struct Roster(pub Vec<CharacterData>);

/// Stages from `assets/stages`.
#[derive(Resource)]
pub struct Stages(pub Vec<StageData>);

/// What was picked in the menus.
#[derive(Resource, Default)]
pub struct Selection {
    pub chars: [usize; 2],
    pub stage: usize,
}

#[derive(Resource, Default)]
struct Sim(GameState);

#[derive(Resource)]
struct DebugView(bool);

/// Background layer; trails the camera by `1 - parallax`.
#[derive(Component)]
struct ParallaxLayer(f32);
/// Pulses slowly (the portal).
#[derive(Component)]
struct Glow;
#[derive(Component)]
struct HudRoot;
#[derive(Component)]
struct SpecialBar(usize);
#[derive(Component)]
struct HealthBar(usize);
#[derive(Component)]
struct RoundPip(usize, u8);
#[derive(Component)]
struct TimerText;
#[derive(Component)]
struct BannerText;
#[derive(Component)]
struct SubBannerText;

/// The assets folder: next to the working directory if there is one, otherwise the project's own.
fn assets_dir() -> PathBuf {
    let here = std::env::current_dir().unwrap_or_default().join("assets");
    if here.is_dir() { here } else { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets") }
}

/// Parse every `.ron` file in `assets/<sub>`, sorted by file name.
fn load_dir<T>(sub: &str, parse: impl Fn(&str) -> Result<T, ron::error::SpannedError>) -> Vec<T> {
    let dir = assets_dir().join(sub);
    assert!(dir.is_dir(), "assets/{sub} folder not found");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read assets/{sub}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "ron"))
        .collect();
    files.sort();
    let items: Vec<_> = files
        .iter()
        .map(|p| {
            let text = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            parse(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
        })
        .collect();
    assert!(!items.is_empty(), "nothing found in assets/{sub}");
    items
}

/// Command-line options for testing: `--fight` skips the menus, `--cpu` makes player 2 the CPU,
/// `--debug` shows hitboxes, `--screenshot <file.png> [--delay <s>]` saves a frame and quits.
#[derive(Resource, Default)]
struct DevOptions {
    fight: bool,
    /// `--charselect`: start on the character select.
    charselect: bool,
    /// `--online`: start on the online screen.
    online: bool,
    /// For two copies on one PC: `--invite <addr>`, `--autoaccept`, `--pick <n>`.
    invite: Option<String>,
    autoaccept: bool,
    online_pick: Option<usize>,
    cpu: bool,
    debug: bool,
    screenshot: Option<String>,
    delay: f32,
    /// `--p1 0 --p2 1`: roster indices.
    picks: [usize; 2],
    /// `--hold right,light`: inputs player 1 holds all the time.
    hold: u16,
    /// `--meter 0..100`: starting special meter.
    meter: i32,
    /// `--spread`: start the fighters at opposite ends of the map (to look at the zoomed-out camera).
    spread: bool,
}

impl DevOptions {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
        DevOptions {
            charselect: args.iter().any(|a| a == "--charselect"),
            online: args.iter().any(|a| a == "--online"),
            invite: value("--invite"),
            autoaccept: args.iter().any(|a| a == "--autoaccept"),
            online_pick: value("--pick").and_then(|v| v.parse().ok()),
            fight: (args.iter().any(|a| a == "--fight") || args.iter().any(|a| a == "--screenshot")) && !args.iter().any(|a| a == "--charselect") && !args.iter().any(|a| a == "--title") && !args.iter().any(|a| a == "--online"),
            cpu: args.iter().any(|a| a == "--cpu"),
            debug: args.iter().any(|a| a == "--debug"),
            screenshot: value("--screenshot"),
            delay: value("--delay").and_then(|d| d.parse().ok()).unwrap_or(3.0),
            picks: [
                value("--p1").and_then(|v| v.parse().ok()).unwrap_or(0),
                value("--p2").and_then(|v| v.parse().ok()).unwrap_or(1),
            ],
            spread: args.iter().any(|a| a == "--spread"),
            meter: value("--meter").and_then(|v| v.parse().ok()).unwrap_or(0),
            hold: value("--hold")
                .map(|names| {
                    names.split(',').fold(0, |bits, name| {
                        bits | match name.trim() {
                            "left" => Input::LEFT,
                            "right" => Input::RIGHT,
                            "up" => Input::UP,
                            "down" => Input::DOWN,
                            "light" => Input::LIGHT,
                            "heavy" => Input::HEAVY,
                            "special" => Input::SPECIAL,
                            "block" => Input::BLOCK,
                            "rollleft" => Input::ROLL_LEFT,
                            "rollright" => Input::ROLL_RIGHT,
                            _ => 0,
                        }
                    })
                })
                .unwrap_or(0),
        }
    }
}

/// With `--screenshot`: save a frame once things have loaded, then quit. It reads the GPU frame, so it works with the screen locked.
fn dev_screenshot(
    mut commands: Commands,
    time: Res<Time<Real>>,
    dev: Res<DevOptions>,
    mut taken_at: Local<Option<f32>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(path) = &dev.screenshot else { return };
    let now = time.elapsed_secs();
    match *taken_at {
        None if now >= dev.delay => {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
            *taken_at = Some(now);
        }
        Some(at) if now - at > 1.5 => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

fn main() {
    let dev = DevOptions::from_args();
    let mut app = App::new();
    app
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin { file_path: assets_dir().to_string_lossy().into_owned(), ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Road to Alfendrof".into(),
                        resolution: WindowResolution::new(1152, 648),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(MENU_BACKGROUND))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Roster({
            // characters marked `menu_last` go to the end
            let mut roster = load_dir("characters", CharacterData::from_ron);
            roster.sort_by_key(|c| c.menu_last);
            roster
        }))
        .insert_resource(Stages(load_dir("stages", StageData::from_ron)))
        .insert_resource(DebugView(dev.debug))
        .init_resource::<Mode>()
        .init_resource::<Selection>()
        .init_resource::<Sim>()
        .init_resource::<render::FrameHandles>()
        .init_resource::<particles::Shake>()
        .add_plugins(menus::MenuPlugin)
        .add_plugins(online::OnlinePlugin)
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(AppState::Fight), (spawn_fight, render::spawn_skill_bar).chain())
        .add_systems(OnExit(AppState::Fight), |mut clear: ResMut<ClearColor>, mut fixed: ResMut<Time<Fixed>>| {
            clear.0 = MENU_BACKGROUND;
            fixed.set_timestep_hz(60.0);
        })
        .add_systems(FixedUpdate, (step_sim, particles::emit).chain().run_if(in_state(AppState::Fight)))
        .add_systems(
            Update,
            (
                render::pose_bodies,
                render::animate_body,
                render::animate_crescent,
                render::animate_stars,
                render::animate_marks,
                render::animate_whips,
                render::animate_ravens,
                render::animate_skill_bar,
                render::animate_moons,
                render::animate_bubbles,
                render::animate_curses,
                render::animate_combo_ui,
                render::animate_minions,
                particles::update_particles,
                follow_camera,
                pulse_glow,
                sync_hud,
                draw_debug,
                restart_match,
                leave_fight,
            )
                .run_if(in_state(AppState::Fight)),
        )
        .add_systems(Update, (toggle_debug, dev_screenshot));

    if dev.fight {
        // keep a bad index from crashing
        let last = app.world().resource::<Roster>().0.len() - 1;
        app.insert_resource(Selection { chars: dev.picks.map(|p| p.min(last)), stage: 0 });
        app.insert_state(AppState::Fight);
    } else if dev.online {
        app.insert_resource(Mode::Online);
        app.insert_state(AppState::Online);
    } else if dev.charselect {
        app.insert_state(AppState::CharSelect);
    } else {
        app.init_state::<AppState>();
    }
    if dev.cpu {
        app.insert_resource(Mode::VsCpu);
    }
    app.insert_resource(dev).run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin { min_width: VIEW_W, min_height: VIEW_H },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn rgb((r, g, b): (f32, f32, f32)) -> Color {
    Color::srgb(r, g, b)
}

fn spawn_fight(
    mut commands: Commands,
    mut sim: ResMut<Sim>,
    mut clear: ResMut<ClearColor>,
    roster: Res<Roster>,
    stages: Res<Stages>,
    selection: Res<Selection>,
    asset_server: Res<AssetServer>,
    dev: Res<DevOptions>,
) {
    *sim = fresh_sim(&roster, &selection);
    if dev.spread {
        sim.0.fighters[0].x = -380 * PX;
        sim.0.fighters[1].x = 380 * PX;
    }
    for f in &mut sim.0.fighters {
        f.meter = dev.meter;
    }
    let stage = &stages.0[selection.stage];
    clear.0 = rgb(stage.sky);
    let scoped = DespawnOnExit(AppState::Fight);

    // HUD root follows the camera so it stays put on screen
    commands
        .spawn((HudRoot, scoped.clone(), Transform::default(), Visibility::default()))
        .with_children(|hud| {
            for side in 0..2usize {
                let dir = if side == 0 { -1.0 } else { 1.0 };
                let mid = dir * (BAR_EDGE - BAR_W / 2.0);
                hud.spawn((
                    Sprite::from_color(Color::srgb(0.15, 0.05, 0.05), Vec2::new(BAR_W + 2.0, BAR_H + 2.0)),
                    Transform::from_xyz(mid, BAR_Y, 10.0),
                ));
                hud.spawn((
                    HealthBar(side),
                    Sprite::from_color(Color::srgb(0.95, 0.8, 0.2), Vec2::new(BAR_W, BAR_H)),
                    Transform::from_xyz(mid, BAR_Y, 10.1),
                ));
                // special meter: a thin bar under the health bar
                let special_y = BAR_Y - BAR_H / 2.0 - 4.0;
                hud.spawn((
                    Sprite::from_color(Color::srgb(0.1, 0.1, 0.16), Vec2::new(BAR_W + 2.0, 5.0)),
                    Transform::from_xyz(mid, special_y, 10.0),
                ));
                hud.spawn((
                    SpecialBar(side),
                    Sprite::from_color(Color::srgb(0.55, 0.35, 0.9), Vec2::new(0.0, 3.0)),
                    Transform::from_xyz(mid, special_y, 10.1),
                ));
                for n in 0..GameState::ROUNDS_TO_WIN {
                    // pips under the inner end of the bar, names under the outer end
                    let x = dir * (BAR_EDGE - BAR_W + 4.0 + n as f32 * 10.0);
                    hud.spawn((
                        RoundPip(side, n),
                        Sprite::from_color(Color::srgb(0.25, 0.25, 0.3), Vec2::new(6.0, 6.0)),
                        Transform::from_xyz(x, BAR_Y - 17.0, 10.1),
                    ));
                }
                hud.spawn((
                    Text2d::new(roster.0[selection.chars[side]].name.clone()),
                    TextFont { font_size: FontSize::Px(10.0), ..default() },
                    TextColor(rgb(roster.0[selection.chars[side]].color)),
                    // long names sit further in so they stay on screen
                    Transform::from_xyz(dir * (BAR_EDGE - 6.0 - roster.0[selection.chars[side]].name.len() as f32 * 2.8), BAR_Y - 18.0, 10.2),
                ));
            }
            hud.spawn((
                TimerText,
                Text2d::new("99"),
                TextFont { font_size: FontSize::Px(18.0), ..default() },
                Transform::from_xyz(0.0, BAR_Y, 10.2),
            ));
            hud.spawn((
                BannerText,
                Text2d::new(""),
                TextFont { font_size: FontSize::Px(28.0), ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.4)),
                Transform::from_xyz(0.0, 20.0, 10.2),
            ));
            hud.spawn((
                SubBannerText,
                Text2d::new(""),
                TextFont { font_size: FontSize::Px(10.0), ..default() },
                TextColor(Color::srgb(0.95, 0.95, 1.0)),
                Transform::from_xyz(0.0, -2.0, 10.2),
            ));
        });

    // background layers, or plain pillars if the stage has none
    for layer in &stage.layers {
        let mut e = commands.spawn((
            scoped.clone(),
            ParallaxLayer(layer.parallax),
            Sprite::from_image(asset_server.load(layer.image.clone())),
            Transform::from_xyz(0.0, layer.y, layer.z),
        ));
        if layer.glow {
            e.insert(Glow);
        }
    }

    // floor: a repeated tile or two alternating colours, two rows deep so it still reaches the bottom when zoomed out
    let stage_w = (GameState::STAGE_HALF_WIDTH * 2 / PX) as f32;
    let half = stage_w / 2.0;
    for row in 0..2 {
        let mut x = -half + 16.0;
        let mut n = row;
        while x < half {
            let mut floor = match &stage.floor_tile {
                Some(path) => Sprite::from_image(asset_server.load(path.clone())),
                None => Sprite::from_color(rgb(if n % 2 == 0 { stage.floor_a } else { stage.floor_b }), Vec2::new(32.0, 40.0)),
            };
            floor.flip_x = n % 2 == 1; // break up the repetition
            commands.spawn((scoped.clone(), floor, Transform::from_xyz(x, GROUND_Y - 20.0 - 40.0 * row as f32, 0.1)));
            if row == 0 && stage.layers.is_empty() && n % 4 == 0 {
                commands.spawn((
                    scoped.clone(),
                    Sprite::from_color(rgb(stage.pillar), Vec2::new(20.0, 90.0)),
                    Transform::from_xyz(x, GROUND_Y + 45.0, -1.0),
                ));
            }
            x += 32.0;
            n += 1;
        }
    }

    let mut handles = render::FrameHandles::default();
    for i in 0..2 {
        let c = &roster.0[selection.chars[i]];
        // the crescent appears where the summon's projectile starts
        let summon = c.moves.iter().find(|m| m.anim == t2f::sim::AnimKind::Summon).and_then(|m| m.projectile.as_ref());
        // effect drawings the moves use
        let mut effects: Vec<String> = Vec::new();
        for p in c.moves.iter().filter_map(|m| m.projectile.as_ref()) {
            for name in [p.image_name(), p.summon_name()] {
                if !effects.iter().any(|e| e == name) {
                    effects.push(name.to_string());
                }
            }
        }
        if c.moves.iter().filter_map(|m| m.projectile.as_ref()).any(|p| p.lob) && !effects.iter().any(|e| e == "moon_star") {
            effects.push("moon_star".to_string());
        }
        if c.moves.iter().any(|m| m.minions.is_some()) && !effects.iter().any(|e| e == "goblin") {
            effects.push("goblin".to_string());
        }
        let frames = t2f::anim::frames_for(c);
        render::spawn_fighter(&mut commands, &asset_server, &assets_dir(), &mut handles, i, c.sprites.as_deref().unwrap_or(&c.name), rgb(c.color), summon, &effects, &frames, c.special_mark.is_some(), c.whip, c.raven.is_some());
    }
    render::spawn_curses(&mut commands, &asset_server);
    render::spawn_combo_ui(&mut commands, &asset_server);
    render::spawn_minion_sprites(&mut commands);
    commands.insert_resource(handles);
}

/// WASD to move, F / G / H for light / heavy / special, Space to block, Q / E to roll. Player 2 is the CPU.
fn read_input(keys: &ButtonInput<KeyCode>) -> Input {
    let mut bits = 0;
    for (key, flag) in [
        (KeyCode::KeyA, Input::LEFT),
        (KeyCode::KeyD, Input::RIGHT),
        (KeyCode::KeyW, Input::UP),
        (KeyCode::KeyS, Input::DOWN),
        (KeyCode::KeyF, Input::LIGHT),
        (KeyCode::KeyG, Input::HEAVY),
        (KeyCode::KeyH, Input::SPECIAL),
        (KeyCode::Space, Input::BLOCK),
        (KeyCode::KeyQ, Input::ROLL_LEFT),
        (KeyCode::KeyE, Input::ROLL_RIGHT),
    ] {
        if keys.pressed(key) {
            bits |= flag;
        }
    }
    Input(bits)
}

fn step_sim(
    mut sim: ResMut<Sim>,
    keys: Res<ButtonInput<KeyCode>>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    dev: Res<DevOptions>,
    mode: Res<Mode>,
    net: Option<ResMut<online::NetSession>>,
    mut fixed: ResMut<Time<Fixed>>,
    mut next: ResMut<NextState<AppState>>,
    mut info: ResMut<online::OnlineInfo>,
    mut commands: Commands,
) {
    let mut player = read_input(&keys);
    player.0 |= dev.hold;
    // online: the other player's buttons come from the network
    if *mode == Mode::Online {
        if let Some(mut net) = net {
            if let Some(problem) = online::step_online(&mut net, &mut sim, &roster, &sel, player, &mut fixed) {
                info.message = problem;
                commands.remove_resource::<online::NetSession>();
                next.set(AppState::Online);
            }
        }
        return;
    }
    let inputs = [player, cpu_input_vs(&sim.0, 1, &roster.0[sel.chars[1]], &roster.0[sel.chars[0]])];
    sim.0.step(inputs, [&roster.0[sel.chars[0]], &roster.0[sel.chars[1]]]);
}

/// Pulse between 70% and 100% opacity.
fn pulse_glow(time: Res<Time>, mut layers: Query<&mut Sprite, With<Glow>>) {
    let alpha = 0.85 + 0.15 * (time.elapsed_secs() * 2.6).sin();
    for mut sprite in &mut layers {
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

/// The camera follows the fighters and zooms: close together it zooms in, further apart it zooms
/// out, up to showing the whole map. The floor stays a fixed fraction up from the bottom of the view.
fn follow_camera(
    sim: Res<Sim>,
    time: Res<Time>,
    shake: Res<particles::Shake>,
    mut width: Local<f32>,
    mut cam: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut hud: Single<&mut Transform, (With<HudRoot>, Without<Camera2d>)>,
    mut layers: Query<(&ParallaxLayer, &mut Transform), (Without<Camera2d>, Without<HudRoot>)>,
) {
    let (a, b) = (sim.0.fighters[0].x as f32 / PX as f32, sim.0.fighters[1].x as f32 / PX as f32);
    let map_w = (GameState::STAGE_HALF_WIDTH * 2 / PX) as f32;
    let target = ((a - b).abs() + ZOOM_MARGIN).clamp(MIN_VIEW_W, map_w);
    if *width == 0.0 {
        *width = target;
    }
    *width += (target - *width) * (1.0 - (-ZOOM_SPEED * time.delta_secs()).exp());
    let zoom = *width / VIEW_W;
    let view_h = *width * VIEW_H / VIEW_W;
    let limit = (map_w - *width) / 2.0;
    let base = ((a + b) / 2.0).clamp(-limit, limit);
    // screen shake after a heavy hit; the HUD moves and scales with the camera
    let t = time.elapsed_secs();
    let (shake_x, shake_y) = ((t * 97.0).sin() * shake.0, (t * 131.0).cos() * shake.0);
    cam.0.translation.x = base + shake_x.round();
    cam.0.translation.y = GROUND_Y + view_h * (0.5 - FLOOR_FRACTION) + shake_y.round();
    if let Projection::Orthographic(ortho) = &mut *cam.1 {
        ortho.scale = zoom;
    }
    hud.translation.x = cam.0.translation.x;
    hud.translation.y = cam.0.translation.y;
    hud.scale = Vec3::new(zoom, zoom, 1.0);
    for (ParallaxLayer(parallax), mut tf) in &mut layers {
        // whole pixels only, so the background doesn't shimmer
        tf.translation.x = (base * parallax).round();
    }
}

fn sync_hud(
    sim: Res<Sim>,
    time: Res<Time>,
    mut specials: Query<(&SpecialBar, &mut Sprite, &mut Transform), (Without<HealthBar>, Without<RoundPip>, Without<TimerText>)>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    mut bars: Query<(&HealthBar, &mut Sprite, &mut Transform), (Without<RoundPip>, Without<TimerText>)>,
    mut pips: Query<(&RoundPip, &mut Sprite), Without<HealthBar>>,
    mut timer: Single<&mut Text2d, (With<TimerText>, Without<BannerText>, Without<SubBannerText>)>,
    mut banner: Single<&mut Text2d, (With<BannerText>, Without<TimerText>, Without<SubBannerText>)>,
    mut sub_banner: Single<&mut Text2d, (With<SubBannerText>, Without<TimerText>, Without<BannerText>)>,
) {
    let s = &sim.0;
    for (SpecialBar(side), mut sprite, mut tf) in &mut specials {
        let frac = s.fighters[*side].meter as f32 / Fighter::METER_MAX as f32;
        let w = BAR_W * frac.clamp(0.0, 1.0);
        let dir = if *side == 0 { -1.0 } else { 1.0 };
        sprite.custom_size = Some(Vec2::new(w, 3.0));
        tf.translation.x = dir * (BAR_EDGE - BAR_W + w / 2.0);
        // purple while filling, pulsing gold when ready
        sprite.color = if frac >= 1.0 {
            let pulse = 0.75 + 0.25 * (time.elapsed_secs() * 8.0).sin();
            Color::srgb(1.0 * pulse, 0.85 * pulse, 0.2 * pulse)
        } else {
            Color::srgb(0.55, 0.35, 0.9)
        };
    }
    for (HealthBar(side), mut sprite, mut tf) in &mut bars {
        let frac = s.fighters[*side].health.max(0) as f32 / s.fighters[*side].max_health.max(1) as f32;
        let w = BAR_W * frac;
        let dir = if *side == 0 { -1.0 } else { 1.0 };
        sprite.custom_size = Some(Vec2::new(w, BAR_H));
        // bars drain from the centre outwards
        tf.translation.x = dir * (BAR_EDGE - BAR_W + w / 2.0);
    }
    for (RoundPip(side, n), mut sprite) in &mut pips {
        let won = s.wins[*side] > *n;
        sprite.color = if won { Color::srgb(1.0, 0.85, 0.2) } else { Color::srgb(0.25, 0.25, 0.3) };
    }
    timer.0 = format!("{}", s.timer.div_ceil(60));

    let name = |i: usize| roster.0[sel.chars[i]].name.clone();
    banner.0 = match s.phase {
        Phase::Fighting if s.frame < 90 || (s.round > 1 && s.timer > GameState::ROUND_FRAMES - 60) => {
            format!("Round {}", s.round)
        }
        Phase::Fighting => String::new(),
        Phase::RoundEnd(_) => {
            if s.fighters.iter().any(|f| f.health <= 0) { "K.O.".into() } else { "Time!".into() }
        }
        Phase::MatchOver => match s.match_winner() {
            Some(w) => format!("{} wins!", name(w)),
            None => "Draw".into(),
        },
    };
    sub_banner.0 = if s.phase == Phase::MatchOver { "Enter: rematch    Esc: menu".into() } else { String::new() };
}

fn draw_debug(sim: Res<Sim>, roster: Res<Roster>, sel: Res<Selection>, debug: Res<DebugView>, mut gizmos: Gizmos) {
    if !debug.0 {
        return;
    }
    let to_rect = |b: [i32; 4]| {
        let (x0, y0, x1, y1) = (b[0] as f32 / PX as f32, b[1] as f32 / PX as f32, b[2] as f32 / PX as f32, b[3] as f32 / PX as f32);
        (Vec2::new((x0 + x1) / 2.0, GROUND_Y + (y0 + y1) / 2.0), Vec2::new(x1 - x0, y1 - y0))
    };
    for i in 0..2 {
        let (c, size) = to_rect(sim.0.hurtbox(i));
        gizmos.rect_2d(Isometry2d::from_translation(c), size, Color::srgb(0.2, 1.0, 0.3));
        if let Some(h) = sim.0.hitbox(i, &roster.0[sel.chars[i]]) {
            let (c, size) = to_rect(h);
            gizmos.rect_2d(Isometry2d::from_translation(c), size, Color::srgb(1.0, 0.2, 0.2));
        }
    }
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut debug: ResMut<DebugView>) {
    if keys.just_pressed(KeyCode::F1) {
        debug.0 = !debug.0;
    }
}

fn restart_match(
    keys: Res<ButtonInput<KeyCode>>,
    mut sim: ResMut<Sim>,
    roster: Res<Roster>,
    sel: Res<Selection>,
    mode: Res<Mode>,
    mut next: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Enter) && sim.0.phase == Phase::MatchOver {
        if *mode == Mode::Online {
            // online, a rematch means inviting again
            commands.remove_resource::<online::NetSession>();
            next.set(AppState::Online);
        } else {
            *sim = fresh_sim(&roster, &sel);
        }
    }
}

/// New match; each fighter starts with their character's health.
fn fresh_sim(roster: &Roster, sel: &Selection) -> Sim {
    let mut sim = Sim::default();
    sim.0.set_health([0, 1].map(|i| roster.0[sel.chars[i]].health.unwrap_or(Fighter::MAX_HEALTH)));
    sim
}

fn leave_fight(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<online::NetSession>();
        next.set(AppState::Title);
    }
}
