//! Online screens and match loop: the Online screen (your address, a friend's, invitations), the fighter pick,
//! then the match on the rollback session. The screens drive `t2f::net::lobby::Lobby`.

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use std::net::{IpAddr, SocketAddr};
use t2f::net::code;
use t2f::net::lobby::{build_id, name_char, Event, Lobby, Stage, MAX_NAME};
use t2f::net::upnp::{lan_ip, Mapping};
use t2f::net::{self, NetConfig, DEFAULT_PORT};

use crate::settings::Settings;
use crate::{AppState, Mode, Roster, Selection, Sim};

pub struct OnlinePlugin;

impl Plugin for OnlinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OnlineLobby>()
            .init_resource::<OnlineInfo>()
            .init_resource::<PlayerNames>()
            .add_systems(OnEnter(AppState::Online), spawn_online_screen)
            .add_systems(OnEnter(AppState::Title), drop_online)
            .add_systems(
                Update,
                (
                    drive_lobby.run_if(|state: Res<State<AppState>>| matches!(state.get(), AppState::Online | AppState::CharSelect)),
                    (online_screen, name_field).chain().run_if(in_state(AppState::Online)),
                    dev_auto.run_if(|state: Res<State<AppState>>| matches!(state.get(), AppState::Online | AppState::CharSelect)),
                ),
            );
    }
}

/// The lobby socket, from the Online screen until the match starts.
#[derive(Resource, Default)]
pub struct OnlineLobby(pub Option<Lobby>);

/// What the screens show.
#[derive(Resource, Default)]
pub struct OnlineInfo {
    /// 0 = left fighter, 1 = right; known once an invite is accepted.
    pub local: usize,
    /// Friend's address as typed.
    pub typed: String,
    pub message: String,
    pub invite_from: Option<SocketAddr>,
    /// Locked in, waiting for the other player.
    pub locked: bool,
    /// Typing goes to your username and not to the friend's code (Tab switches).
    pub editing_name: bool,
}

/// The two players' usernames for the HUD, by side (empty: none).
#[derive(Resource, Default)]
pub struct PlayerNames(pub [String; 2]);

/// The running match.
#[derive(Resource)]
pub struct NetSession {
    pub session: ggrs::P2PSession<NetConfig>,
    pub local: usize,
}

#[derive(Component)]
struct AddrText;
#[derive(Component)]
struct LanText;
#[derive(Component)]
struct InputText;
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct PopupText;
#[derive(Component)]
struct NameText;

/// Both players need the same version and character files.
fn build_of(roster: &Roster) -> u32 {
    let mut parts: Vec<String> = vec![env!("CARGO_PKG_VERSION").to_string()];
    for c in &roster.0 {
        parts.push(format!("{}:{:?}:{}:{}", c.name, c.moves.iter().map(|m| (m.damage, m.startup, m.active, m.recovery)).collect::<Vec<_>>(), c.walk_speed, c.health.unwrap_or(0)));
    }
    let refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
    build_id(&refs)
}

fn drop_online(mut lobby: ResMut<OnlineLobby>, mut info: ResMut<OnlineInfo>, mut commands: Commands) {
    if let Some(l) = lobby.0.as_mut() {
        l.leave();
    }
    lobby.0 = None;
    *info = OnlineInfo::default();
    commands.remove_resource::<NetSession>();
}

fn text(commands: &mut Commands, marker: impl Component, value: &str, size: f32, color: Color, y: f32) {
    commands.spawn((
        DespawnOnExit(AppState::Online),
        marker,
        Text2d::new(crate::menus::fit(value, crate::menus::LINE_CHARS)),
        bevy::text::TextLayout::new(bevy::text::Justify::Center, bevy::text::LineBreak::NoWrap),
        crate::pixel_font(size),
        TextColor(color),
        Transform::from_xyz(0.0, y, 5.0),
    ));
}

#[derive(Component)]
struct Label;

fn spawn_online_screen(mut commands: Commands, mut lobby: ResMut<OnlineLobby>, roster: Res<Roster>, mut info: ResMut<OnlineInfo>, settings: Res<Settings>) {
    if lobby.0.is_none() {
        // the usual port first, then any free one (a second copy on this PC)
        lobby.0 = Lobby::new(DEFAULT_PORT, build_of(&roster), true).or_else(|_| Lobby::new(0, build_of(&roster), true)).ok();
        if lobby.0.is_none() {
            info.message = "Could not open a network port.".into();
        }
    }
    info.locked = false;
    info.editing_name = false;
    if let Some(l) = lobby.0.as_mut() {
        l.set_name(&settings.username);
    }
    crate::menus::spawn_panel(&mut commands, AppState::Online);
    let gold = Color::srgb(1.0, 0.9, 0.4);
    let grey = Color::srgb(0.6, 0.6, 0.7);
    text(&mut commands, Label, "Online", 24.0, gold, 82.0);
    text(&mut commands, Label, "Your code (send it to your friend)", 9.0, grey, 56.0);
    text(&mut commands, AddrText, "looking it up...", 16.0, Color::WHITE, 42.0);
    text(&mut commands, LanText, "", 8.0, grey, 28.0);
    text(&mut commands, Label, "Your friend's code", 9.0, grey, 6.0);
    text(&mut commands, InputText, "_", 16.0, Color::srgb(0.7, 0.95, 1.0), -8.0);
    text(&mut commands, StatusText, "", 9.0, Color::srgb(0.95, 0.8, 0.6), -28.0);
    text(&mut commands, PopupText, "", 12.0, Color::srgb(0.5, 1.0, 0.6), -46.0);
    text(&mut commands, Label, "Your name (Tab: edit)", 8.0, grey, -66.0);
    text(&mut commands, NameText, "", 12.0, Color::srgb(0.7, 0.95, 1.0), -78.0);
    text(
        &mut commands,
        Label,
        "Enter: invite   Tab: your name   Ctrl+C: copy yours   Ctrl+V: paste   Esc: back",
        8.0,
        grey,
        -93.0,
    );
}

/// Poll the lobby every frame on either online screen.
fn drive_lobby(
    mut lobby: ResMut<OnlineLobby>,
    mut info: ResMut<OnlineInfo>,
    mut sel: ResMut<Selection>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
    mut commands: Commands,
    settings: Res<Settings>,
    mut names: ResMut<PlayerNames>,
) {
    let Some(l) = lobby.0.as_mut() else { return };
    let mut start = None;
    for event in l.poll() {
        match event {
            Event::Invited(from) => {
                info.invite_from = Some(from);
                let who = if l.peer_name.is_empty() { code::show(from) } else { l.peer_name.clone() };
                info.message = format!("{who} wants to play!");
            }
            Event::Declined => info.message = "They declined.".into(),
            Event::BuildMismatch => info.message = "That player has a different version of the game.".into(),
            Event::Picking { player } => {
                info.local = player;
                info.invite_from = None;
                info.locked = false;
                if *state.get() == AppState::Online {
                    next.set(AppState::CharSelect);
                }
            }
            Event::PeerPicked { .. } => {}
            Event::PeerLeft => {
                info.message = "They left.".into();
                info.invite_from = None;
                info.locked = false;
                if *state.get() == AppState::CharSelect {
                    next.set(AppState::Online);
                }
            }
            Event::Ready { player, mine, theirs, .. } => start = Some((player, mine as usize, theirs as usize)),
        }
    }
    if let Some((player, mine, theirs)) = start {
        let peer_name = lobby.0.as_ref().map(|l| l.peer_name.clone()).unwrap_or_default();
        let lobby_owned = lobby.0.take().expect("the lobby is here");
        let (socket, peer, _) = lobby_owned.into_game();
        match net::start_p2p(socket, player, peer) {
            Ok(session) => {
                sel.chars[player] = mine;
                sel.chars[1 - player] = theirs;
                sel.stage = 0;
                names.0[player] = settings.username.clone();
                names.0[1 - player] = peer_name;
                commands.insert_resource(NetSession { session, local: player });
                next.set(AppState::Fight);
            }
            Err(e) => {
                info.message = format!("Could not start the match: {e}");
                next.set(AppState::Online);
            }
        }
    }
}

/// The Online screen: typing an address, inviting, answering invites.
fn online_screen(
    keys: Res<ButtonInput<KeyCode>>,
    mut typing: MessageReader<KeyboardInput>,
    mut lobby: ResMut<OnlineLobby>,
    mut info: ResMut<OnlineInfo>,
    mut next: ResMut<NextState<AppState>>,
    mut addr_text: Single<&mut Text2d, (With<AddrText>, Without<LanText>, Without<InputText>, Without<StatusText>, Without<PopupText>)>,
    mut lan_text: Single<&mut Text2d, (With<LanText>, Without<AddrText>, Without<InputText>, Without<StatusText>, Without<PopupText>)>,
    mut input_text: Single<&mut Text2d, (With<InputText>, Without<AddrText>, Without<LanText>, Without<StatusText>, Without<PopupText>)>,
    mut status_text: Single<&mut Text2d, (With<StatusText>, Without<AddrText>, Without<LanText>, Without<InputText>, Without<PopupText>)>,
    mut popup_text: Single<&mut Text2d, (With<PopupText>, Without<AddrText>, Without<LanText>, Without<InputText>, Without<StatusText>)>,
    time: Res<Time>,
    settings: Res<Settings>,
    // the machine's address on the local network and when it was last looked up (it opens a socket, so not every frame)
    mut lan_cache: Local<Option<(f32, Option<std::net::Ipv4Addr>)>>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let invited = matches!(lobby.0.as_ref().map(|l| l.stage), Some(Stage::Invited(_)));

    // typing: a code (letters, digits, dashes), or an address (digits, '.' and ':')
    for ev in typing.read() {
        if ev.state != ButtonState::Pressed || ctrl || invited || info.editing_name {
            continue;
        }
        match (&ev.logical_key, &ev.text) {
            (bevy::input::keyboard::Key::Backspace, _) => {
                info.typed.pop();
            }
            (_, Some(t)) => {
                for c in t.chars().filter(|c| typeable(*c)) {
                    if info.typed.len() < 21 {
                        info.typed.push(c.to_ascii_uppercase());
                    }
                }
            }
            _ => {}
        }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyV) {
        if let Ok(mut clip) = arboard::Clipboard::new() {
            if let Ok(t) = clip.get_text() {
                info.typed = t.trim().chars().filter(|c| typeable(*c)).map(|c| c.to_ascii_uppercase()).take(21).collect();
            }
        }
    }

    let Some(l) = lobby.0.as_mut() else {
        input_text.0 = "-".into();
        status_text.0 = info.message.clone();
        return;
    };
    l.set_name(&settings.username);
    let mine = l.public;
    if ctrl && keys.just_pressed(KeyCode::KeyC) {
        if let (Some(addr), Ok(mut clip)) = (mine, arboard::Clipboard::new()) {
            let _ = clip.set_text(code::show(addr));
            info.message = "Copied your code.".into();
        }
    }

    if invited {
        if keys.just_pressed(KeyCode::KeyF) || keys.just_pressed(KeyCode::Enter) {
            // accepting types their code into the field and presses Enter: the invitation goes back to them (two
            // invitations that cross count as accepted by both)
            if let Stage::Invited(from) = l.stage {
                info.typed = code::show(from).chars().filter(|c| typeable(*c)).map(|c| c.to_ascii_uppercase()).take(21).collect();
                l.invite(from);
                info.invite_from = None;
                info.message = "Accepted. Connecting...".into();
            }
        } else if keys.just_pressed(KeyCode::KeyG) {
            l.decline();
            info.invite_from = None;
            info.message = "Declined.".into();
        }
    } else if keys.just_pressed(KeyCode::Enter) && !info.typed.is_empty() && !info.editing_name {
        let parsed = code::decode(&info.typed)
            .or_else(|| info.typed.parse::<SocketAddr>().ok())
            .or_else(|| info.typed.parse::<IpAddr>().ok().map(|ip| SocketAddr::new(ip, DEFAULT_PORT)));
        match parsed {
            Some(addr) if l.is_own_address(addr) => {
                info.message = "That is your own code. Send it to your friend and enter theirs here.".into();
            }
            Some(addr) => {
                l.invite(addr);
                info.message = format!("Invited {}. Waiting... your friend should enter YOUR code and invite you too.", code::show(addr));
            }
            None => info.message = "That code doesn't work. Check it letter by letter.".into(),
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        if matches!(l.stage, Stage::Inviting(_)) {
            l.leave();
            info.message = "Cancelled.".into();
        } else {
            next.set(AppState::Title);
        }
    }

    let new_addr = match mine {
        Some(a) => code::show(a),
        None => "looking it up...".into(),
    };
    if addr_text.0 != new_addr {
        addr_text.0 = new_addr;
    }
    let now = time.elapsed_secs();
    if lan_cache.is_none_or(|(at, _)| now - at > 3.0) {
        *lan_cache = Some((now, lan_ip()));
    }
    let lan = lan_cache.and_then(|(_, ip)| ip).map(|ip| code::show(SocketAddr::new(IpAddr::V4(ip), l.port())));
    let router = match &l.mapping {
        Mapping::Working => "asking router...".to_string(),
        Mapping::Opened(_) => "router port opened".to_string(),
        Mapping::Unavailable(why) if why == "off" => String::new(),
        Mapping::Unavailable(_) => "router: port not opened".to_string(),
    };
    let new_lan = match lan {
        Some(a) => format!("on the same network: {a}   {router}"),
        None => router,
    };
    if lan_text.0 != new_lan {
        lan_text.0 = new_lan;
    }
    let blink = if (time.elapsed_secs() * 2.0) as u32 % 2 == 0 && !info.editing_name { "_" } else { " " };
    let new_input = format!("{}{}", info.typed, blink);
    if input_text.0 != new_input {
        input_text.0 = new_input;
    }
    let message = crate::menus::fit(&info.message, crate::menus::LINE_CHARS);
    if status_text.0 != message {
        status_text.0 = message;
    }
    let new_popup = match (l.stage, info.invite_from) {
        (Stage::Invited(from), _) | (_, Some(from)) if invited => {
            let who = if l.peer_name.is_empty() { format!("Someone ({})", code::show(from)) } else { l.peer_name.clone() };
            format!("{who} has invited you!   F: accept    G: decline")
        }
        _ => String::new(),
    };
    let new_popup = crate::menus::fit(&new_popup, crate::menus::LINE_CHARS);
    if popup_text.0 != new_popup {
        popup_text.0 = new_popup;
    }
}

/// The username field: Tab to edit it, Enter or Tab to stop. It is saved with the other settings as it is typed.
fn name_field(
    keys: Res<ButtonInput<KeyCode>>,
    mut typing: MessageReader<KeyboardInput>,
    mut info: ResMut<OnlineInfo>,
    mut settings: ResMut<Settings>,
    mut field: Single<&mut Text2d, With<NameText>>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::Tab) || (info.editing_name && keys.just_pressed(KeyCode::Enter)) {
        info.editing_name = !info.editing_name;
    }
    let mut name = settings.username.clone();
    for ev in typing.read() {
        if ev.state != ButtonState::Pressed || !info.editing_name {
            continue;
        }
        match (&ev.logical_key, &ev.text) {
            (bevy::input::keyboard::Key::Backspace, _) => {
                name.pop();
            }
            (_, Some(t)) => {
                for c in t.chars().filter(|c| name_char(*c)) {
                    if name.chars().count() < MAX_NAME {
                        name.push(c);
                    }
                }
            }
            _ => {}
        }
    }
    if name != settings.username {
        settings.username = name;
    }
    let blink = if info.editing_name && (time.elapsed_secs() * 2.0) as u32 % 2 == 0 { "_" } else { " " };
    let shown = if settings.username.is_empty() && !info.editing_name { "(none)".to_string() } else { format!("{}{}", settings.username, blink) };
    if field.0 != shown {
        field.0 = shown;
    }
}

/// What can be typed into the friend's code: a code's letters and dashes, or an address's digits, dots and colon.
fn typeable(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == ':'
}

/// Test helper (see `DevOptions`): invite, accept and pick with no one at the keyboard.
fn dev_auto(dev: Res<crate::DevOptions>, mut lobby: ResMut<OnlineLobby>, mut info: ResMut<OnlineInfo>, mut done: Local<(bool, bool)>) {
    let Some(l) = lobby.0.as_mut() else { return };
    if let (Some(addr), false, Stage::Idle) = (&dev.invite, done.0, l.stage) {
        if let Ok(addr) = addr.parse::<SocketAddr>() {
            l.invite(addr);
            done.0 = true;
        }
    }
    if dev.autoaccept && matches!(l.stage, Stage::Invited(_)) {
        l.accept();
    }
    if let (Some(index), Stage::Picking(_), false) = (dev.online_pick, l.stage, done.1) {
        l.pick(index as u8, true);
        info.locked = true;
        done.1 = true;
    }
}

/// One frame of the online match.
pub fn step_online(
    net: &mut NetSession,
    sim: &mut Sim,
    roster: &Roster,
    sel: &Selection,
    local_input: t2f::sim::Input,
    fixed: &mut Time<Fixed>,
) -> Option<String> {
    use ggrs::{GgrsEvent, SessionState};
    net.session.poll_remote_clients();
    let mut problem = None;
    for event in net.session.events() {
        match event {
            GgrsEvent::Disconnected { .. } => problem = Some("The other player disconnected.".to_string()),
            GgrsEvent::DesyncDetected { frame, .. } => problem = Some(format!("The two games went out of sync (frame {frame}).")),
            _ => {}
        }
    }
    // the faster side slows down a little so the other can catch up
    fixed.set_timestep_hz(if net.session.frames_ahead() > 0 { 54.0 } else { 60.0 });
    if net.session.current_state() == SessionState::Running {
        let local = net.local;
        if net.session.add_local_input(local, local_input.into()).is_ok() {
            if let Ok(requests) = net.session.advance_frame() {
                net::handle_requests(&mut sim.0, requests, [&roster.0[sel.chars[0]], &roster.0[sel.chars[1]]]);
            }
        }
    }
    problem
}

pub fn local_online_input(mode: Mode) -> bool {
    mode == Mode::Online
}
