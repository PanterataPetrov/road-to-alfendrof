//! Title, options, controls, character select and stage select, all driven by one cursor.
//! Both players' keys work: WASD + F/G or arrows + J/K, Enter and Esc.

use bevy::prelude::*;

use crate::settings::{self, Display, Settings, ACTIONS};
use crate::sfx::UiSound;
use t2f::skills::move_names;
use bevy::sprite::Anchor;
use t2f::sim::Button;
use crate::{AppState, Mode, Roster, Selection, Sim, Stages};
use t2f::sim::Phase;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .init_resource::<Picking>()
            .init_resource::<Rebinding>()
            .init_resource::<Pause>()
            .init_resource::<Surrendering>()
            .add_systems(OnEnter(AppState::Title), spawn_title)
            .add_systems(OnEnter(AppState::Options), spawn_options)
            .add_systems(OnEnter(AppState::Controls), spawn_controls)
            .add_systems(OnEnter(AppState::CharSelect), spawn_char_select)
            .add_systems(OnEnter(AppState::StageSelect), spawn_stage_select)
            .add_systems(
                Update,
                (
                    (title_confirm.run_if(in_state(AppState::Title)),
                     char_confirm.run_if(in_state(AppState::CharSelect)),
                     stage_confirm.run_if(in_state(AppState::StageSelect)),
                     options_confirm.run_if(options_open),
                     controls_confirm.run_if(controls_open))
                        .before(menu_labels),
                    pause_main.run_if(in_state(AppState::Fight)).before(options_confirm).before(controls_confirm),
                    sync_pause.run_if(in_state(AppState::Fight)).after(pause_main).after(options_confirm).after(controls_confirm),
                    adopt_overlay.after(sync_pause),
                    // (menus only: during a fight there is nothing for them to do)
                    menu_nav.run_if(menu_live).run_if(not(in_state(AppState::CharSelect))),
                    carousel_nav.run_if(in_state(AppState::CharSelect)),
                    menu_click.run_if(menu_live).run_if(not(in_state(AppState::Online))),
                    menu_labels.run_if(menu_live),
                    menu_pages.run_if(menu_live),
                    (carousel_slide, carousel_info).run_if(in_state(AppState::CharSelect)),
                    glitch_subtitles.run_if(menu_live),
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
    /// Items shown at a time; the rest are on other pages (A/D or Left/Right turn the page).
    per_page: usize,
}

impl Menu {
    fn page(&self) -> usize {
        self.cursor / self.per_page.max(1)
    }

    fn pages(&self) -> usize {
        self.len.div_ceil(self.per_page.max(1)).max(1)
    }
}

/// Who is picking: 0 = P1, 1 = P2.
#[derive(Resource, Default)]
struct Picking(usize);

/// The control waiting for its new key in the Controls menu.
#[derive(Resource, Default)]
struct Rebinding(Option<usize>);

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

pub fn confirm_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    [KeyCode::Enter, KeyCode::KeyF, KeyCode::KeyJ, KeyCode::Space].iter().any(|&k| keys.just_pressed(k))
}

fn back_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    [KeyCode::Escape, KeyCode::KeyG, KeyCode::KeyK, KeyCode::Backspace].iter().any(|&k| keys.just_pressed(k))
}

/// The box every menu's text sits in, so all screens use the same part of the window: size, and the sprites
/// (a dark fill, a violet frame) as `(sprite, position)` for the caller to spawn.
pub const PANEL: Vec2 = Vec2::new(372.0, 204.0);

pub fn panel_parts() -> Vec<(Sprite, Vec3)> {
    let frame = Color::srgb(0.55, 0.42, 0.85);
    let inner = Color::srgb(0.22, 0.16, 0.4);
    let (w, h) = (PANEL.x, PANEL.y);
    let mut parts = vec![(Sprite::from_color(Color::srgba(0.03, 0.02, 0.08, 0.84), PANEL), Vec3::new(0.0, 0.0, 1.0))];
    for (color, grow, z) in [(frame, 0.0, 1.1), (inner, -4.0, 1.1)] {
        let (w, h) = (w + grow, h + grow);
        parts.push((Sprite::from_color(color, Vec2::new(w, 2.0)), Vec3::new(0.0, h / 2.0 - 1.0, z)));
        parts.push((Sprite::from_color(color, Vec2::new(w, 2.0)), Vec3::new(0.0, -h / 2.0 + 1.0, z)));
        parts.push((Sprite::from_color(color, Vec2::new(2.0, h)), Vec3::new(-w / 2.0 + 1.0, 0.0, z)));
        parts.push((Sprite::from_color(color, Vec2::new(2.0, h)), Vec3::new(w / 2.0 - 1.0, 0.0, z)));
    }
    parts
}

pub fn spawn_panel(commands: &mut Commands, state: AppState) {
    for (sprite, at) in panel_parts() {
        commands.spawn((DespawnOnExit(state.clone()), sprite, Transform::from_translation(at)));
    }
}

/// The pixel-art backdrop of the character and stage select.
fn spawn_backdrop(commands: &mut Commands, state: AppState, asset_server: &AssetServer) {
    commands.spawn((
        DespawnOnExit(state),
        Sprite { image: asset_server.load("backgrounds/select.png"), custom_size: Some(Vec2::new(384.0, 216.0)), ..default() },
        Transform::from_xyz(0.0, 0.0, -8.0),
    ));
}

/// Most characters of small text on one line: the pixel font is about 5 px a character and the view 384 px wide.
pub const LINE_CHARS: usize = 66;

/// Fits long text on the screen: a line made of several hints (set apart by three spaces or more) is split between
/// them, and what is still too long is wrapped at spaces.
pub fn fit(text: &str, max: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    for part in text.split("   ").map(str::trim).filter(|p| !p.is_empty()) {
        match lines.last_mut() {
            Some(last) if last.chars().count() + 4 + part.chars().count() <= max => {
                last.push_str("    ");
                last.push_str(part);
            }
            _ => lines.push(part.to_string()),
        }
    }
    lines.iter().map(|line| if line.chars().count() > max { wrap(line, max) } else { line.clone() }).collect::<Vec<_>>().join("
")
}

fn spawn_text(commands: &mut Commands, state: AppState, text: String, size: f32, color: Color, y: f32) -> Entity {
    commands
        .spawn((
            DespawnOnExit(state),
            Text2d::new(text),
            // (several lines are centred too)
            bevy::text::TextLayout::new(bevy::text::Justify::Center, bevy::text::LineBreak::NoWrap),
            crate::pixel_font(size),
            TextColor(color),
            Transform::from_xyz(0.0, y, 5.0),
        ))
        .id()
}

/// Which page of a long menu a piece of it is on.
#[derive(Component)]
struct Page(usize);

/// "Page 1/2" under a paged menu.
#[derive(Component)]
struct PageText;

fn spawn_menu(commands: &mut Commands, menu: &mut Menu, state: AppState, title: &str, hint: &str, items: Vec<Item>, top_override: Option<f32>) {
    menu.cursor = 0;
    menu.len = items.len();
    if state != AppState::Title {
        spawn_panel(commands, state.clone());
    }
    let header = spawn_text(commands, state.clone(), title.into(), 24.0, Color::srgb(1.0, 0.9, 0.4), 82.0);
    commands.entity(header).insert(Header);
    // (long hints take two lines; the text grows upwards from just above the bottom of the panel)
    let hint = spawn_text(commands, state.clone(), fit(hint, LINE_CHARS), 9.0, Color::srgb(0.6, 0.6, 0.7), -99.0);
    commands.entity(hint).insert(Anchor::BOTTOM_CENTER);

    // rows with pictures need more room: four to a page, the rest on the next; long lists of text get smaller
    let with_pictures = items.iter().any(|i| i.portrait.is_some());
    let compact = !with_pictures && items.len() > 6;
    menu.per_page = if with_pictures { 4 } else { items.len().max(1) };
    let spacing = if with_pictures { 35.0 } else if compact { 12.0 } else { 22.0 };
    let top = top_override.unwrap_or(if with_pictures { 56.0 } else if compact { 62.0 } else { 40.0 });
    let size = if compact { 8.0 } else { 16.0 };
    for (index, item) in items.into_iter().enumerate() {
        let page = Page(index / menu.per_page);
        let y = top - (index % menu.per_page) as f32 * spacing;
        let e = spawn_text(commands, state.clone(), item.label.clone(), size, item.color, y);
        commands.entity(e).insert((MenuItem { index, label: item.label, color: item.color, enabled: item.enabled }, Page(page.0)));
        if let Some(sub) = item.subtitle {
            let e = spawn_text(commands, state.clone(), sub.clone(), 8.0, item.color.with_alpha(0.7), y - 11.0);
            commands.entity(e).insert((Transform::from_xyz(0.0, y - 11.0, 5.0), Page(page.0)));
            if item.corrupted {
                commands.entity(e).insert(Glitch(sub));
            }
        }
        if let Some(image) = item.portrait {
            let size = (spacing - 4.0).min(40.0);
            // dark frame, then the picture, left of the name
            commands.spawn((
                DespawnOnExit(state.clone()),
                Page(page.0),
                Sprite::from_color(Color::srgb(0.02, 0.02, 0.05), Vec2::splat(size + 4.0)),
                Transform::from_xyz(-142.0, y, 4.0),
            ));
            commands.spawn((
                DespawnOnExit(state.clone()),
                Portrait(index),
                Page(page.0),
                Sprite { image, custom_size: Some(Vec2::splat(size)), ..default() },
                Transform::from_xyz(-142.0, y, 5.0),
            ));
        }
    }
    if menu.pages() > 1 {
        let e = spawn_text(commands, state, String::new(), 8.0, Color::srgb(0.75, 0.7, 0.95), -78.0);
        commands.entity(e).insert(PageText);
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
    // sound credits, as the licences ask (the full list is in assets/sfx/CREDITS.txt)
    let credit = spawn_text(&mut commands, state.clone(), "Sound: Deadly Kombat, Soundrack (CC BY), Owlish Media + more".into(), 6.0, Color::srgb(0.5, 0.5, 0.6), -104.0);
    commands.entity(credit).insert(Transform::from_xyz(0.0, -104.0, 5.0));
    let tagline = spawn_text(&mut commands, state.clone(), "A T2SSR Brawl".into(), 9.0, Color::srgb(0.98, 0.92, 0.72), 8.0);
    commands.entity(tagline).insert(Transform::from_xyz(0.0, 8.0, 5.0));
    // the version, small in the bottom right corner
    let version = spawn_text(&mut commands, state.clone(), format!("v{}", env!("CARGO_PKG_VERSION")), 8.0, Color::srgba(0.85, 0.85, 0.95, 0.75), -102.0);
    commands.entity(version).insert((Transform::from_xyz(172.0, -102.0, 5.0), bevy::sprite::Anchor::CENTER_RIGHT));

    let white = Color::WHITE;
    let online = Item::new("Online", white);
    spawn_menu(
        &mut commands,
        &mut menu,
        state,
        "",
        "W/S or Up/Down: move    F / J / Enter: select    Esc: quit",
        vec![Item::new("vs CPU", white), online, Item::new("Options", white)],
        Some(-30.0),
    );
}

fn animate_title(time: Res<Time>, mut logo: Query<&mut Transform, With<TitleLogo>>) {
    let t = time.elapsed_secs();
    for mut tf in &mut logo {
        tf.translation.y = 58.0 + (t * 1.6).sin() * 2.0;
    }
}

/// A fighter's picture in the character select carousel; `0` is its place in the roster.
#[derive(Component)]
struct Slot(usize);

/// The name, subtitle and skills of the fighter in the middle of the carousel (rebuilt when it turns).
#[derive(Component)]
struct CarouselInfo;

/// Where the pictures sit left and right of the middle one, and how big they are.
const SLOT_X: [f32; 3] = [0.0, 62.0, 104.0];
const SLOT_SCALE: [f32; 3] = [1.15, 0.72, 0.5];

fn spawn_char_select(
    mut commands: Commands,
    mut menu: ResMut<Menu>,
    roster: Res<Roster>,
    mut picking: ResMut<Picking>,
    mode: Res<Mode>,
    asset_server: Res<AssetServer>,
) {
    picking.0 = 0;
    let state = AppState::CharSelect;
    spawn_backdrop(&mut commands, state.clone(), &asset_server);
    spawn_panel(&mut commands, state.clone());
    // one fighter at a time: the cursor is the page
    menu.cursor = 0;
    menu.len = roster.0.len();
    menu.per_page = 1;
    let header = spawn_text(&mut commands, state.clone(), pick_title(0, *mode), 16.0, Color::srgb(1.0, 0.9, 0.4), 90.0);
    commands.entity(header).insert(Header);
    spawn_text(&mut commands, state.clone(), "A/D: turn    F / J / Enter: pick    G / K / Esc: back".into(), 9.0, Color::srgb(0.6, 0.6, 0.7), -92.0);
    for (index, c) in roster.0.iter().enumerate() {
        // portraits/<sprite folder>.png, if it exists
        let folder = c.sprites.clone().unwrap_or_else(|| c.name.to_lowercase());
        let has_picture = crate::assets_dir().join(format!("portraits/{folder}.png")).exists();
        let picture = if has_picture {
            Sprite { image: asset_server.load(format!("portraits/{folder}.png")), custom_size: Some(Vec2::splat(40.0)), ..default() }
        } else {
            Sprite::from_color(Color::srgb(c.color.0, c.color.1, c.color.2), Vec2::splat(40.0))
        };
        commands
            .spawn((
                DespawnOnExit(state.clone()),
                Slot(index),
                Portrait(index),
                picture,
                Transform::from_xyz(0.0, 60.0, 5.0),
                Visibility::Hidden,
            ))
            .with_children(|p| {
                // a dark frame behind the picture
                p.spawn((Sprite::from_color(Color::srgb(0.02, 0.02, 0.05), Vec2::splat(44.0)), Transform::from_xyz(0.0, 0.0, -0.1)));
            });
    }
    commands.spawn((DespawnOnExit(state), CarouselInfo, Transform::default(), Visibility::default()));
}

/// The fighter whose place `i` takes in the carousel: a secret shares the place of the one it hides behind.
fn host_of(roster: &[t2f::sim::CharacterData], i: usize) -> usize {
    match &roster[i].secret_behind {
        Some(name) => roster.iter().position(|c| &c.name == name).unwrap_or(i),
        None => i,
    }
}

/// A/D (or the arrows) turn the carousel. Up on a fighter with a secret behind them turns up the secret, Down goes back.
fn carousel_nav(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<Menu>, roster: Res<Roster>, mut ui: MessageWriter<UiSound>) {
    let roster = &roster.0;
    if roster.is_empty() || menu.cursor >= roster.len() {
        return;
    }
    let host = host_of(roster, menu.cursor);
    let up = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
    let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown);
    let turn = left_right(&keys);
    if up && menu.cursor == host {
        if let Some(secret) = roster.iter().position(|c| c.secret_behind.as_ref() == Some(&roster[host].name)) {
            menu.cursor = secret;
            ui.write(UiSound::Move);
        }
    } else if down && menu.cursor != host {
        menu.cursor = host;
        ui.write(UiSound::Move);
    } else if turn != 0 {
        let shown: Vec<usize> = (0..roster.len()).filter(|&i| roster[i].secret_behind.is_none()).collect();
        if let Some(at) = shown.iter().position(|&i| i == host) {
            menu.cursor = shown[(at as i32 + turn).rem_euclid(shown.len() as i32) as usize];
            ui.write(UiSound::Move);
        }
    }
}

/// The distance in places from the middle one, going round (the roster is a ring).
fn ring_offset(index: usize, cursor: usize, len: usize) -> i32 {
    let len = len as i32;
    (index as i32 - cursor as i32 + len / 2).rem_euclid(len) - len / 2
}

/// The pictures slide to their places round the middle one; the ones further out than the sides are hidden.
fn carousel_slide(time: Res<Time>, menu: Res<Menu>, roster: Res<Roster>, mut slots: Query<(&Slot, &mut Transform, &mut Visibility)>) {
    let ease = 1.0 - (-18.0 * time.delta_secs()).exp();
    let roster = &roster.0;
    // the secrets aren't in the ring: they take the place of the fighter they hide behind while picked
    let ring: Vec<usize> = (0..roster.len()).filter(|&i| roster[i].secret_behind.is_none()).collect();
    let place = |i: usize| ring.iter().position(|&r| r == host_of(roster, i)).unwrap_or(0);
    for (Slot(index), mut tf, mut visibility) in &mut slots {
        let secret = roster[*index].secret_behind.is_some();
        let picked_secret = menu.cursor < roster.len() && roster[menu.cursor].secret_behind.is_some();
        // hidden: a secret that isn't picked, and the one it takes the place of while it is
        let out = if secret { menu.cursor != *index } else { picked_secret && host_of(roster, menu.cursor) == *index };
        if out {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            if secret {
                // waits in the middle, so it appears there
                tf.translation.x = 0.0;
                tf.scale = Vec3::new(SLOT_SCALE[0], SLOT_SCALE[0], 1.0);
            }
            continue;
        }
        let d = ring_offset(place(*index), place(menu.cursor.min(roster.len() - 1)), ring.len().max(1));
        let far = d.unsigned_abs() as usize;
        let shown = far < SLOT_X.len();
        let want = if shown { Visibility::Inherited } else { Visibility::Hidden };
        if *visibility != want {
            *visibility = want;
        }
        let side = if d < 0 { -1.0 } else { 1.0 };
        if shown {
            tf.translation.x += (side * SLOT_X[far] - tf.translation.x) * ease;
            let s = tf.scale.x + (SLOT_SCALE[far] - tf.scale.x) * ease;
            tf.scale = Vec3::new(s, s, 1.0);
        } else {
            // out of sight: wait at the edge on its own side, so it slides in from there
            tf.translation.x = side * 150.0;
            tf.scale = Vec3::new(0.4, 0.4, 1.0);
        }
        tf.translation.z = 5.0 - far as f32 * 0.1;
    }
}

/// Breaks `text` into lines of at most `width` characters.
fn wrap(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut line = 0;
    for word in text.split_whitespace() {
        if line > 0 && line + 1 + word.len() > width {
            out.push('\n');
            line = 0;
        } else if line > 0 {
            out.push(' ');
            line += 1;
        }
        out.push_str(word);
        line += word.len();
    }
    out
}

/// The name, subtitle, and each button's icon, moves and what it does, for the fighter in the middle.
fn carousel_info(
    mut commands: Commands,
    menu: Res<Menu>,
    roster: Res<Roster>,
    asset_server: Res<AssetServer>,
    info: Single<Entity, With<CarouselInfo>>,
    mut shown: Local<Option<usize>>,
) {
    if *shown == Some(menu.cursor) || menu.cursor >= roster.0.len() {
        return;
    }
    *shown = Some(menu.cursor);
    let c = &roster.0[menu.cursor];
    let folder = c.sprites.clone().unwrap_or_else(|| c.name.to_lowercase());
    let color = Color::srgb(c.color.0, c.color.1, c.color.2);
    let body = Color::srgb(0.82, 0.82, 0.9);
    let gold = Color::srgb(1.0, 0.9, 0.4);
    commands.entity(*info).despawn_related::<Children>().with_children(|ui| {
        fn text(ui: &mut bevy::ecs::relationship::RelatedSpawnerCommands<ChildOf>, value: String, size: f32, color: Color, anchor: Anchor, x: f32, y: f32) -> Entity {
            ui.spawn((Text2d::new(value), crate::pixel_font(size), TextColor(color), anchor, Transform::from_xyz(x, y, 5.0))).id()
        }
        text(ui, c.name.clone(), 16.0, color, Anchor::CENTER, 0.0, 28.0);
        let mut glitched = None;
        if let Some(sub) = c.subtitle.clone() {
            let e = text(ui, sub.clone(), 8.0, color.with_alpha(0.7), Anchor::CENTER, 0.0, 16.0);
            if c.corrupted_subtitle {
                glitched = Some((e, sub));
            }
        }
        for (row, (button, icon)) in [(Button::Light, "light"), (Button::Heavy, "heavy"), (Button::Special, "special")].into_iter().enumerate() {
            let top = 2.0 - row as f32 * 30.0;
            ui.spawn((
                Sprite { image: asset_server.load(format!("icons/{folder}/{icon}.png")), custom_size: Some(Vec2::splat(24.0)), ..default() },
                Transform::from_xyz(-166.0, top - 13.0, 5.0),
            ));
            let mut names = move_names(c, button);
            if names.len() > 56 {
                names.truncate(53);
                names.push_str("...");
            }
            text(ui, names, 8.0, gold, Anchor::TOP_LEFT, -148.0, top);
            if let Some(blurb) = c.blurbs.get(row) {
                text(ui, wrap(blurb, 56), 8.0, body, Anchor::TOP_LEFT, -148.0, top - 10.0);
            }
        }
        if let Some((e, sub)) = glitched {
            ui.commands().entity(e).insert(Glitch(sub));
        }
    });
}

fn pick_title(player: usize, mode: Mode) -> String {
    match (player, mode) {
        (0, _) => "Choose your fighter".into(),
        (_, Mode::VsCpu) => "Choose the CPU's fighter".into(),
        (_, Mode::Online) => "Choose your fighter".into(),
    }
}

fn spawn_stage_select(mut commands: Commands, mut menu: ResMut<Menu>, stages: Res<Stages>, asset_server: Res<AssetServer>) {
    spawn_backdrop(&mut commands, AppState::StageSelect, &asset_server);
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

fn menu_nav(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<Menu>, rebinding: Res<Rebinding>, mut ui: MessageWriter<UiSound>) {
    // W and S could be the key being bound
    if menu.len == 0 || rebinding.0.is_some() {
        return;
    }
    let up = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
    let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown);
    if up || down {
        ui.write(UiSound::Move);
    }
    if up {
        menu.cursor = (menu.cursor + menu.len - 1) % menu.len;
    }
    if down {
        menu.cursor = (menu.cursor + 1) % menu.len;
    }
    // a paged menu turns the page sideways, on the same row where there is one
    let turn = left_right(&keys);
    if turn != 0 && menu.pages() > 1 {
        let (per_page, pages) = (menu.per_page, menu.pages() as i32);
        let page = (menu.page() as i32 + turn).rem_euclid(pages) as usize;
        menu.cursor = (page * per_page + menu.cursor % per_page).min(menu.len - 1);
    }
}

/// The click of confirming or going back in the menus (not while a key is being bound).
fn menu_click(keys: Res<ButtonInput<KeyCode>>, rebinding: Res<Rebinding>, mut ui: MessageWriter<UiSound>) {
    if rebinding.0.is_some() {
        return;
    }
    if confirm_pressed(&keys) {
        ui.write(UiSound::Confirm);
    } else if back_pressed(&keys) {
        ui.write(UiSound::Back);
    }
}

/// Shows the page the cursor is on and says which it is.
fn menu_pages(menu: Res<Menu>, mut parts: Query<(&Page, &mut Visibility)>, mut label: Query<&mut Text2d, With<PageText>>) {
    let page = menu.page();
    for (Page(p), mut visibility) in &mut parts {
        let want = if *p == page { Visibility::Inherited } else { Visibility::Hidden };
        if *visibility != want {
            *visibility = want;
        }
    }
    for mut text in &mut label {
        let new = format!("< Page {}/{} >", page + 1, menu.pages());
        if text.0 != new {
            text.0 = new;
        }
    }
}

fn menu_labels(menu: Res<Menu>, mut items: Query<(&MenuItem, &mut Text2d, &mut TextColor)>, mut portraits: Query<(&Portrait, &mut Sprite)>) {
    for (Portrait(index), mut sprite) in &mut portraits {
        let tone = if *index == menu.cursor { Color::WHITE } else { Color::srgb(0.55, 0.55, 0.6) };
        if sprite.color != tone {
            sprite.color = tone;
        }
    }
    for (item, mut text, mut color) in &mut items {
        let selected = item.index == menu.cursor;
        let label = if selected { format!("> {} <", item.label) } else { item.label.clone() };
        if text.0 != label {
            text.0 = label;
        }
        let tone = if selected || !item.enabled { item.color } else { item.color.with_alpha(0.55) };
        if color.0 != tone {
            color.0 = tone;
        }
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
        let new_text: String = original
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let n = noise(tick.wrapping_mul(31).wrapping_add(i as u32 * 977));
                if !calm && c != '.' && c != ' ' && n % 100 < 32 { JUNK[(n >> 8) as usize % JUNK.len()] as char } else { c }
            })
            .collect();
        if text.0 != new_text {
            text.0 = new_text;
        }
        let x = if calm { 0.0 } else { (noise(tick) % 7) as f32 - 3.0 };
        if tf.translation.x != x {
            tf.translation.x = x;
        }
        let tone = match noise(tick / 2) % 5 {
            0 => Color::srgb(1.0, 0.2, 0.25),
            1 => Color::srgb(0.7, 0.3, 1.0),
            2 => Color::srgb(0.95, 0.95, 0.95),
            _ => Color::srgb(0.75, 0.15, 0.3),
        };
        if color.0 != tone {
            color.0 = tone;
        }
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
            1 => {
                *mode = Mode::Online;
                next.set(AppState::Online);
            }
            _ => next.set(AppState::Options),
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

fn left_right(keys: &ButtonInput<KeyCode>) -> i32 {
    let left = keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft);
    let right = keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight);
    right as i32 - left as i32
}

fn options_labels(settings: &Settings) -> Vec<String> {
    vec![
        format!("Music volume  < {} >", settings.music_volume),
        format!("Effects volume  < {} >", settings.sfx_volume),
        format!("Display  < {} >", settings.display.label()),
        "Controls".into(),
        "Back".into(),
    ]
}

fn spawn_options(mut commands: Commands, mut menu: ResMut<Menu>, settings: Res<Settings>) {
    build_options(&mut commands, &mut menu, AppState::Options, &settings);
}

/// The options screen, in the menus (`AppState::Options`) or over a fight (`AppState::Overlay`).
fn build_options(commands: &mut Commands, menu: &mut Menu, state: AppState, settings: &Settings) {
    let items = options_labels(settings).into_iter().map(|label| Item::new(label, Color::WHITE)).collect();
    spawn_menu(
        commands,
        menu,
        state,
        "Options",
        "W/S: move    A/D or Left/Right: change    F / J / Enter: select    Esc: back",
        items,
        None,
    );
}

/// Left/right change the volume and display mode (Enter also cycles the display mode).
fn options_confirm(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<Menu>,
    mut settings: ResMut<Settings>,
    mut items: Query<&mut MenuItem>,
    mut next: ResMut<NextState<AppState>>,
    mut ui: MessageWriter<UiSound>,
    mut pause: ResMut<Pause>,
) {
    let step = left_right(&keys);
    let confirm = confirm_pressed(&keys);
    // over a fight, Back goes to the pause menu instead of the title screen
    let in_fight = pause.open();
    if back_pressed(&keys) {
        if in_fight {
            *pause = Pause::Main;
        } else {
            next.set(AppState::Title);
        }
        return;
    }
    match menu.cursor {
        0 if step != 0 => settings.music_volume = (settings.music_volume as i32 + step).clamp(0, Settings::MAX_VOLUME as i32) as u8,
        1 if step != 0 => {
            settings.sfx_volume = (settings.sfx_volume as i32 + step).clamp(0, Settings::MAX_VOLUME as i32) as u8;
            ui.write(UiSound::Preview);
        }
        2 if step != 0 || confirm => {
            let n = Display::ALL.len() as i32;
            let i = Display::ALL.iter().position(|&d| d == settings.display).unwrap_or(0) as i32;
            settings.display = Display::ALL[(i + if step == 0 { 1 } else { step }).rem_euclid(n) as usize];
        }
        3 if confirm => {
            if in_fight {
                *pause = Pause::Controls;
            } else {
                next.set(AppState::Controls);
            }
        }
        4 if confirm => {
            if in_fight {
                *pause = Pause::Main;
            } else {
                next.set(AppState::Title);
            }
        }
        _ => {}
    }
    // F11 changes the display mode too
    if settings.is_changed() {
        let labels = options_labels(&settings);
        for mut item in &mut items {
            item.label = labels[item.index].clone();
        }
    }
}

fn controls_labels(settings: &Settings, rebinding: Option<usize>) -> Vec<String> {
    let mut labels: Vec<String> = ACTIONS
        .iter()
        .enumerate()
        .map(|(i, (name, _, _))| {
            if rebinding == Some(i) { format!("{name}: press a key") } else { format!("{name}: {}", settings::key_name(settings.keys[i])) }
        })
        .collect();
    labels.push("Reset to defaults".into());
    labels.push("Back".into());
    labels
}

fn spawn_controls(mut commands: Commands, mut menu: ResMut<Menu>, settings: Res<Settings>, mut rebinding: ResMut<Rebinding>) {
    rebinding.0 = None;
    build_controls(&mut commands, &mut menu, AppState::Controls, &settings);
}

fn build_controls(commands: &mut Commands, menu: &mut Menu, state: AppState, settings: &Settings) {
    let items = controls_labels(settings, None).into_iter().map(|label| Item::new(label, Color::WHITE)).collect();
    spawn_menu(
        commands,
        menu,
        state,
        "Controls",
        "F / J / Enter: change, then press the new key    Esc: back",
        items,
        None,
    );
}

/// Pick a control, then the next key pressed is its new one (a key another control had is swapped).
fn controls_confirm(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<Menu>,
    mut settings: ResMut<Settings>,
    mut rebinding: ResMut<Rebinding>,
    mut items: Query<&mut MenuItem>,
    mut next: ResMut<NextState<AppState>>,
    mut pause: ResMut<Pause>,
) {
    let in_fight = pause.open();
    let mut back = |next: &mut NextState<AppState>| {
        if in_fight {
            *pause = Pause::Options;
        } else {
            next.set(AppState::Options);
        }
    };
    if let Some(i) = rebinding.0 {
        if keys.just_pressed(KeyCode::Escape) {
            rebinding.0 = None;
        } else if let Some(&key) = keys.get_just_pressed().find(|&&k| settings::bindable(k)) {
            settings.bind(i, key);
            rebinding.0 = None;
        }
    } else if back_pressed(&keys) {
        back(&mut next);
    } else if confirm_pressed(&keys) {
        match menu.cursor {
            i if i < ACTIONS.len() => rebinding.0 = Some(i),
            i if i == ACTIONS.len() => settings.keys = Settings::default().keys,
            _ => back(&mut next),
        }
    }
    if settings.is_changed() || rebinding.is_changed() {
        let labels = controls_labels(&settings, rebinding.0);
        for mut item in &mut items {
            item.label = labels[item.index].clone();
        }
    }
}

// ---- the pause menu over a fight ----

/// Which screen of the pause menu is open over the fight (Esc opens it). Its pieces are drawn in the fight's HUD, and
/// the options and controls screens are the same ones as in the menus.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pause {
    #[default]
    Closed,
    Main,
    Options,
    Controls,
    /// "Really give up?"
    Surrender,
}

impl Pause {
    pub fn open(self) -> bool {
        self != Pause::Closed
    }
}

/// The player chose to surrender: the fight sends it until the match is over.
#[derive(Resource, Default)]
pub struct Surrendering(pub bool);

/// Where the pause menu's pieces go: in the HUD, on top of everything.
#[derive(Component)]
pub struct PauseRoot;

/// A piece of the pause menu (cleared when its screen changes).
#[derive(Component)]
struct PauseUi;

/// Menu systems work in the menus, and over a fight while the pause menu is open.
fn menu_live(state: Res<State<AppState>>, pause: Res<Pause>) -> bool {
    *state.get() != AppState::Fight || pause.open()
}

fn options_open(state: Res<State<AppState>>, pause: Res<Pause>) -> bool {
    *state.get() == AppState::Options || *pause == Pause::Options
}

fn controls_open(state: Res<State<AppState>>, pause: Res<Pause>) -> bool {
    *state.get() == AppState::Controls || *pause == Pause::Controls
}

/// Esc opens the pause menu in a fight that is on (not before it starts or after it ends); in it, the main screen
/// and the surrender question are answered here (the other screens by the options and controls systems).
fn pause_main(keys: Res<ButtonInput<KeyCode>>, mut pause: ResMut<Pause>, menu: Res<Menu>, sim: Res<Sim>, mut surrender: ResMut<Surrendering>) {
    let live = matches!(sim.0.phase, Phase::Countdown(_) | Phase::Fighting | Phase::RoundEnd(_));
    if pause.open() && !live {
        // the match ended behind the menu (the other player gave up)
        *pause = Pause::Closed;
        return;
    }
    match *pause {
        Pause::Closed => {
            if live && keys.just_pressed(KeyCode::Escape) {
                *pause = Pause::Main;
            }
        }
        Pause::Main => {
            if back_pressed(&keys) {
                *pause = Pause::Closed;
            } else if confirm_pressed(&keys) {
                *pause = match menu.cursor {
                    0 => Pause::Closed,
                    1 => Pause::Options,
                    _ => Pause::Surrender,
                };
            }
        }
        Pause::Surrender => {
            if back_pressed(&keys) {
                *pause = Pause::Main;
            } else if confirm_pressed(&keys) {
                if menu.cursor == 1 {
                    surrender.0 = true;
                    *pause = Pause::Closed;
                } else {
                    *pause = Pause::Main;
                }
            }
        }
        Pause::Options | Pause::Controls => {}
    }
}

/// Draw the screen of the pause menu that is open, replacing the last one.
fn sync_pause(
    mut commands: Commands,
    pause: Res<Pause>,
    mut shown: Local<Pause>,
    mut menu: ResMut<Menu>,
    settings: Res<Settings>,
    mode: Res<Mode>,
    mut rebinding: ResMut<Rebinding>,
    old: Query<Entity, With<PauseUi>>,
) {
    if *shown == *pause {
        return;
    }
    *shown = *pause;
    for piece in &old {
        commands.entity(piece).despawn();
    }
    rebinding.0 = None;
    let red = Color::srgb(1.0, 0.55, 0.55);
    match *pause {
        Pause::Closed => {}
        Pause::Main => {
            let title = if *mode == Mode::Online { "Paused (the match goes on)" } else { "Paused" };
            let items = vec![Item::new("Resume", Color::WHITE), Item::new("Options", Color::WHITE), Item::new("Surrender", red)];
            spawn_menu(&mut commands, &mut menu, AppState::Overlay, title, "W/S or Up/Down: move    F / J / Enter: select    Esc: resume", items, None);
        }
        Pause::Options => build_options(&mut commands, &mut menu, AppState::Overlay, &settings),
        Pause::Controls => build_controls(&mut commands, &mut menu, AppState::Overlay, &settings),
        Pause::Surrender => {
            let items = vec![Item::new("No, keep fighting", Color::WHITE), Item::new("Yes, surrender", red)];
            spawn_menu(&mut commands, &mut menu, AppState::Overlay, "Surrender?", "You lose the match.    Esc: back", items, None);
        }
    }
}

/// The pieces of the pause menu are spawned like any menu's; this puts them in the fight's HUD.
fn adopt_overlay(
    mut commands: Commands,
    new: Query<(Entity, &DespawnOnExit<AppState>), Added<DespawnOnExit<AppState>>>,
    root: Option<Single<Entity, With<PauseRoot>>>,
) {
    let Some(root) = root else { return };
    for (piece, scope) in &new {
        if scope.0 == AppState::Overlay {
            commands.entity(piece).insert((ChildOf(*root), PauseUi));
        }
    }
}

/// Menus are drawn around the origin at the normal zoom: the fight camera zooms and moves to follow the
/// fighters, and leaving a fight at any zoom would otherwise leave the menu drawn at that scale and in the
/// wrong place, no longer filling the window.
fn menu_camera(mut cam: Single<(&mut Transform, &mut Projection), With<Camera2d>>) {
    // only written when it is off (every write makes Bevy treat the camera as changed)
    if cam.0.translation.x != 0.0 || cam.0.translation.y != 0.0 {
        cam.0.translation.x = 0.0;
        cam.0.translation.y = 0.0;
    }
    if let Projection::Orthographic(ortho) = &*cam.1 {
        if ortho.scale != 1.0 {
            if let Projection::Orthographic(ortho) = &mut *cam.1 {
                ortho.scale = 1.0;
            }
        }
    }
}
