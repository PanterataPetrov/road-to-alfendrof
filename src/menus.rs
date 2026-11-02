//! Title, character select and stage select, all driven by one cursor.
//! Both players' keys work: WASD + F/G or arrows + J/K, Enter and Esc.

use bevy::prelude::*;

use crate::{AppState, Mode, Roster, Selection, Stages};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .init_resource::<Picking>()
            .add_systems(OnEnter(AppState::Title), spawn_title)
            .add_systems(OnEnter(AppState::CharSelect), spawn_char_select)
            .add_systems(OnEnter(AppState::StageSelect), spawn_stage_select)
            .add_systems(
                Update,
                (
                    (title_confirm.run_if(in_state(AppState::Title)),
                     char_confirm.run_if(in_state(AppState::CharSelect)),
                     stage_confirm.run_if(in_state(AppState::StageSelect)))
                        .before(menu_labels),
                    menu_nav,
                    menu_labels,
                    glitch_subtitles,
                    animate_title.run_if(in_state(AppState::Title)),
                    menu_camera.run_if(not(in_state(AppState::Fight))),
                ),
            );
    }
}

/// Cursor position and item count of the current menu.
#[derive(Resource, Default)]
struct Menu {
    cursor: usize,
    len: usize,
}

/// Who is picking: 0 = P1, 1 = P2.
#[derive(Resource, Default)]
struct Picking(usize);

#[derive(Component)]
struct MenuItem {
    index: usize,
    label: String,
    color: Color,
    enabled: bool,
}

#[derive(Component)]
struct Header;

struct Item {
    label: String,
    color: Color,
    enabled: bool,
    /// Square picture left of the name.
    portrait: Option<Handle<Image>>,
    /// Small line under the name.
    subtitle: Option<String>,
    /// Draw the subtitle as glitching text.
    corrupted: bool,
}

/// Subtitle that keeps its wording but flickers: letters turn to noise, it jitters and changes colour.
#[derive(Component)]
struct Glitch(String);

/// Portrait beside a menu item; grows while selected.
#[derive(Component)]
struct Portrait(usize);

impl Item {
    fn new(label: impl Into<String>, color: Color) -> Self {
        Item { label: label.into(), color, enabled: true, portrait: None, subtitle: None, corrupted: false }
    }
}

fn confirm_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    [KeyCode::Enter, KeyCode::KeyF, KeyCode::KeyJ, KeyCode::Space].iter().any(|&k| keys.just_pressed(k))
}

fn back_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    [KeyCode::Escape, KeyCode::KeyG, KeyCode::KeyK, KeyCode::Backspace].iter().any(|&k| keys.just_pressed(k))
}

fn spawn_text(commands: &mut Commands, state: AppState, text: String, size: f32, color: Color, y: f32) -> Entity {
    commands
        .spawn((
            DespawnOnExit(state),
            Text2d::new(text),
            crate::pixel_font(size),
            TextColor(color),
            Transform::from_xyz(0.0, y, 5.0),
        ))
        .id()
}

fn spawn_menu(commands: &mut Commands, menu: &mut Menu, state: AppState, title: &str, hint: &str, items: Vec<Item>, top_override: Option<f32>) {
    menu.cursor = 0;
    menu.len = items.len();
    let header = spawn_text(commands, state.clone(), title.into(), 24.0, Color::srgb(1.0, 0.9, 0.4), 82.0);
    commands.entity(header).insert(Header);
    spawn_text(commands, state.clone(), hint.into(), 9.0, Color::srgb(0.6, 0.6, 0.7), -92.0);

    // rows with pictures need more room
    let with_pictures = items.iter().any(|i| i.portrait.is_some());
    let spacing = if with_pictures { (140.0 / items.len().max(1) as f32).min(40.0) } else { 22.0 };
    let top = top_override.unwrap_or(if with_pictures { 56.0 } else { 40.0 });
    for (index, item) in items.into_iter().enumerate() {
        let y = top - index as f32 * spacing;
        let e = spawn_text(commands, state.clone(), item.label.clone(), 16.0, item.color, y);
        commands.entity(e).insert(MenuItem { index, label: item.label, color: item.color, enabled: item.enabled });
        if let Some(sub) = item.subtitle {
            let e = spawn_text(commands, state.clone(), sub.clone(), 8.0, item.color.with_alpha(0.7), y - 11.0);
            commands.entity(e).insert(Transform::from_xyz(0.0, y - 11.0, 5.0));
            if item.corrupted {
                commands.entity(e).insert(Glitch(sub));
            }
        }
        if let Some(image) = item.portrait {
            let size = (spacing - 4.0).min(40.0);
            // dark frame, then the picture, left of the name
            commands.spawn((
                DespawnOnExit(state.clone()),
                Sprite::from_color(Color::srgb(0.02, 0.02, 0.05), Vec2::splat(size + 4.0)),
                Transform::from_xyz(-142.0, y, 4.0),
            ));
            commands.spawn((
                DespawnOnExit(state.clone()),
                Portrait(index),
                Sprite { image, custom_size: Some(Vec2::splat(size)), ..default() },
                Transform::from_xyz(-142.0, y, 5.0),
            ));
        }
    }
}

/// The logo bobs.
#[derive(Component)]
struct TitleLogo;

/// Start screen: the map behind, the logo over it, the menu below.
fn spawn_title(mut commands: Commands, mut menu: ResMut<Menu>, asset_server: Res<AssetServer>) {
    let state = AppState::Title;
    commands.spawn((
        DespawnOnExit(state.clone()),
        Sprite { image: asset_server.load("backgrounds/galata/near.png"), custom_size: Some(Vec2::new(384.0, 216.0)), ..default() },
        Transform::from_xyz(0.0, 0.0, -8.0),
    ));
    // dark band behind the menu so it reads, vignette at the top
    commands.spawn((DespawnOnExit(state.clone()), Sprite::from_color(Color::srgba(0.03, 0.02, 0.08, 0.82), Vec2::new(384.0, 96.0)), Transform::from_xyz(0.0, -62.0, 1.0)));
    commands.spawn((DespawnOnExit(state.clone()), Sprite::from_color(Color::srgba(0.03, 0.02, 0.08, 0.35), Vec2::new(384.0, 22.0)), Transform::from_xyz(0.0, 105.0, 1.0)));
    commands.spawn((
        DespawnOnExit(state.clone()),
        TitleLogo,
        Sprite::from_image(asset_server.load("ui/logo.png")),
        Transform::from_xyz(39.0, 58.0, 3.0),
    ));
    // the Templar-cross shield beside the wordmark
    commands.spawn((
        DespawnOnExit(state.clone()),
        TitleLogo,
        Sprite::from_image(asset_server.load("ui/emblem.png")),
        Transform::from_xyz(-105.0, 58.0, 3.0),
    ));
    commands.spawn((DespawnOnExit(state.clone()), Sprite::from_color(Color::srgba(0.03, 0.02, 0.08, 0.7), Vec2::new(86.0, 12.0)), Transform::from_xyz(0.0, 8.0, 4.0)));
    let tagline = spawn_text(&mut commands, state.clone(), "A T2SSR Brawl".into(), 9.0, Color::srgb(0.98, 0.92, 0.72), 8.0);
    commands.entity(tagline).insert(Transform::from_xyz(0.0, 8.0, 5.0));

    let white = Color::WHITE;
    let online = Item::new("Online", white);
    spawn_menu(
        &mut commands,
        &mut menu,
        state,
        "",
        "W/S or Up/Down: move    F / J / Enter: select    Esc: quit",
        vec![Item::new("vs CPU", white), online],
        Some(-34.0),
    );
}

fn animate_title(time: Res<Time>, mut logo: Query<&mut Transform, With<TitleLogo>>) {
    let t = time.elapsed_secs();
    for mut tf in &mut logo {
        tf.translation.y = 58.0 + (t * 1.6).sin() * 2.0;
    }
}

fn spawn_char_select(
    mut commands: Commands,
    mut menu: ResMut<Menu>,
    roster: Res<Roster>,
    mut picking: ResMut<Picking>,
    mode: Res<Mode>,
    asset_server: Res<AssetServer>,
) {
    picking.0 = 0;
    let items = roster
        .0
        .iter()
        .map(|c| {
            let mut item = Item::new(c.name.clone(), Color::srgb(c.color.0, c.color.1, c.color.2));
            item.subtitle = c.subtitle.clone();
            item.corrupted = c.corrupted_subtitle;
            // portraits/<sprite folder>.png, if it exists
            let folder = c.sprites.clone().unwrap_or_else(|| c.name.to_lowercase());
            if crate::assets_dir().join(format!("portraits/{folder}.png")).exists() {
                item.portrait = Some(asset_server.load(format!("portraits/{folder}.png")));
            }
            item
        })
        .collect();
    spawn_menu(
        &mut commands,
        &mut menu,
        AppState::CharSelect,
        &pick_title(0, *mode),
        "W/S or Up/Down: move    F / J / Enter: pick    G / K / Esc: back",
        items,
        None,
    );
}

fn pick_title(player: usize, mode: Mode) -> String {
    match (player, mode) {
        (0, _) => "Choose your fighter".into(),
        (_, Mode::VsCpu) => "Choose the CPU's fighter".into(),
        (_, Mode::Online) => "Choose your fighter".into(),
    }
}

fn spawn_stage_select(mut commands: Commands, mut menu: ResMut<Menu>, stages: Res<Stages>) {
    let items = stages
        .0
        .iter()
        .map(|s| Item::new(s.name.clone(), Color::srgb(s.floor_a.0 + 0.3, s.floor_a.1 + 0.3, s.floor_a.2 + 0.3)))
        .collect();
    spawn_menu(
        &mut commands,
        &mut menu,
        AppState::StageSelect,
        "Choose a stage",
        "W/S or Up/Down: move    F / J / Enter: fight    G / K / Esc: back",
        items,
        None,
    );
}

fn menu_nav(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<Menu>) {
    if menu.len == 0 {
        return;
    }
    let up = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
    let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown);
    if up {
        menu.cursor = (menu.cursor + menu.len - 1) % menu.len;
    }
    if down {
        menu.cursor = (menu.cursor + 1) % menu.len;
    }
}

fn menu_labels(menu: Res<Menu>, mut items: Query<(&MenuItem, &mut Text2d, &mut TextColor)>, mut portraits: Query<(&Portrait, &mut Sprite)>) {
    for (Portrait(index), mut sprite) in &mut portraits {
        sprite.color = if *index == menu.cursor { Color::WHITE } else { Color::srgb(0.55, 0.55, 0.6) };
    }
    for (item, mut text, mut color) in &mut items {
        let selected = item.index == menu.cursor;
        text.0 = if selected { format!("> {} <", item.label) } else { item.label.clone() };
        color.0 = if selected || !item.enabled { item.color } else { item.color.with_alpha(0.55) };
    }
}

/// Glitch the corrupted subtitles: letters turn to noise, the line jitters and flickers red, violet and white.
fn glitch_subtitles(time: Res<Time>, mut lines: Query<(&Glitch, &mut Text2d, &mut TextColor, &mut Transform)>) {
    let tick = (time.elapsed_secs() * 14.0) as u32;
    let noise = |n: u32| {
        let mut h = n.wrapping_mul(0x9E37_79B1) ^ 0x85EB_CA6B;
        h ^= h >> 15;
        h = h.wrapping_mul(0x2C1B_3C6D);
        h ^ (h >> 12)
    };
    const JUNK: &[u8] = b"#%&@$?!/\\<>*^~|01";
    for (Glitch(original), mut text, mut color, mut tf) in &mut lines {
        let calm = noise(tick / 5) % 4 == 0;
        text.0 = original
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let n = noise(tick.wrapping_mul(31).wrapping_add(i as u32 * 977));
                if !calm && c != '.' && c != ' ' && n % 100 < 32 { JUNK[(n >> 8) as usize % JUNK.len()] as char } else { c }
            })
            .collect();
        tf.translation.x = if calm { 0.0 } else { (noise(tick) % 7) as f32 - 3.0 };
        color.0 = match noise(tick / 2) % 5 {
            0 => Color::srgb(1.0, 0.2, 0.25),
            1 => Color::srgb(0.7, 0.3, 1.0),
            2 => Color::srgb(0.95, 0.95, 0.95),
            _ => Color::srgb(0.75, 0.15, 0.3),
        };
    }
}

fn title_confirm(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<Menu>,
    mut mode: ResMut<Mode>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    if back_pressed(&keys) && keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    } else if confirm_pressed(&keys) {
        match menu.cursor {
            0 => {
                *mode = Mode::VsCpu;
                next.set(AppState::CharSelect);
            }
            _ => {
                *mode = Mode::Online;
                next.set(AppState::Online);
            }
        }
    }
}

fn char_confirm(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<Menu>,
    mut picking: ResMut<Picking>,
    mut sel: ResMut<Selection>,
    mode: Res<Mode>,
    stages: Res<Stages>,
    mut header: Single<&mut Text2d, With<Header>>,
    mut next: ResMut<NextState<AppState>>,
    mut lobby: ResMut<crate::online::OnlineLobby>,
    mut info: ResMut<crate::online::OnlineInfo>,
) {
    // online you only pick your own fighter; the match starts when both are in (see `online::drive_lobby`)
    if *mode == Mode::Online {
        if back_pressed(&keys) {
            if let Some(l) = lobby.0.as_mut() {
                l.leave();
            }
            info.locked = false;
            next.set(AppState::Online);
        } else if confirm_pressed(&keys) && !info.locked {
            if let Some(l) = lobby.0.as_mut() {
                l.pick(menu.cursor as u8, true);
                info.locked = true;
                header.0 = "Waiting for your friend to pick...".into();
            }
        }
        return;
    }
    if back_pressed(&keys) {
        if picking.0 == 1 {
            picking.0 = 0;
            header.0 = pick_title(0, *mode);
        } else {
            next.set(AppState::Title);
        }
    } else if confirm_pressed(&keys) {
        sel.chars[picking.0] = menu.cursor;
        if picking.0 == 0 {
            picking.0 = 1;
            header.0 = pick_title(1, *mode);
            menu.cursor = 0;
        } else if stages.0.len() == 1 {
            // only one stage, so go straight to the fight
            sel.stage = 0;
            next.set(AppState::Fight);
        } else {
            next.set(AppState::StageSelect);
        }
    }
}

fn stage_confirm(keys: Res<ButtonInput<KeyCode>>, menu: Res<Menu>, mut sel: ResMut<Selection>, mut next: ResMut<NextState<AppState>>) {
    if back_pressed(&keys) {
        next.set(AppState::CharSelect);
    } else if confirm_pressed(&keys) {
        sel.stage = menu.cursor;
        next.set(AppState::Fight);
    }
}

/// Menus are drawn around the origin at the normal zoom: the fight camera zooms and moves to follow the
/// fighters, and leaving a fight at any zoom would otherwise leave the menu drawn at that scale and in the
/// wrong place, no longer filling the window.
fn menu_camera(mut cam: Single<(&mut Transform, &mut Projection), With<Camera2d>>) {
    cam.0.translation.x = 0.0;
    cam.0.translation.y = 0.0;
    if let Projection::Orthographic(ortho) = &mut *cam.1 {
        ortho.scale = 1.0;
    }
}
