//! Online play in the game: the Online screen (your address, a box for a friend's, invitations), the
//! fighter pick that follows, and the match itself on the rollback session.
//!
//! The screens only drive `t2f::net::lobby::Lobby`; the match is `t2f::net`'s ggrs session.

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use std::net::{IpAddr, SocketAddr};
use t2f::net::lobby::{build_id, Event, Lobby, Stage};
use t2f::net::upnp::{lan_ip, Mapping};
use t2f::net::{self, NetConfig, DEFAULT_PORT};

use crate::{AppState, Mode, Roster, Selection, Sim};

pub struct OnlinePlugin;

impl Plugin for OnlinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OnlineLobby>()
            .init_resource::<OnlineInfo>()
            .add_systems(OnEnter(AppState::Online), spawn_online_screen)
            .add_systems(OnEnter(AppState::Title), drop_online)
            .add_systems(
                Update,
                (
                    drive_lobby.run_if(|state: Res<State<AppState>>| matches!(state.get(), AppState::Online | AppState::CharSelect)),
                    online_screen.run_if(in_state(AppState::Online)),
                    dev_auto.run_if(|state: Res<State<AppState>>| matches!(state.get(), AppState::Online | AppState::CharSelect)),
                ),
            );
    }
}

/// The lobby socket, alive from the Online screen until the match starts.
#[derive(Resource, Default)]
pub struct OnlineLobby(pub Option<Lobby>);

/// What the screens show and remember.
#[derive(Resource, Default)]
pub struct OnlineInfo {
    /// 0 = left fighter, 1 = right; known once an invitation is accepted.
    pub local: usize,
    /// The friend's address as typed.
    pub typed: String,
    pub message: String,
    pub invite_from: Option<SocketAddr>,
    /// My fighter is locked in and I am waiting for theirs.
    pub locked: bool,
}

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

/// Both players must have the same version and character files to be able to play.
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
        Text2d::new(value),
        TextFont { font_size: FontSize::Px(size), ..default() },
        TextColor(color),
        Transform::from_xyz(0.0, y, 5.0),
    ));
}

#[derive(Component)]
struct Label;

fn spawn_online_screen(mut commands: Commands, mut lobby: ResMut<OnlineLobby>, roster: Res<Roster>, mut info: ResMut<OnlineInfo>) {
    if lobby.0.is_none() {
        // The usual port first, then any free one (a second copy on the same computer).
        lobby.0 = Lobby::new(DEFAULT_PORT, build_of(&roster), true).or_else(|_| Lobby::new(0, build_of(&roster), true)).ok();
        if lobby.0.is_none() {
            info.message = "Could not open a network port.".into();
        }
    }
    info.locked = false;
    let gold = Color::srgb(1.0, 0.9, 0.4);
    let grey = Color::srgb(0.6, 0.6, 0.7);
    text(&mut commands, Label, "Online", 24.0, gold, 82.0);
    text(&mut commands, Label, "Your address (send it to your friend)", 9.0, grey, 56.0);
    text(&mut commands, AddrText, "looking it up...", 16.0, Color::WHITE, 42.0);
    text(&mut commands, LanText, "", 8.0, grey, 28.0);
    text(&mut commands, Label, "Your friend's address", 9.0, grey, 6.0);
    text(&mut commands, InputText, "_", 16.0, Color::srgb(0.7, 0.95, 1.0), -8.0);
    text(&mut commands, StatusText, "", 9.0, Color::srgb(0.95, 0.8, 0.6), -34.0);
    text(&mut commands, PopupText, "", 12.0, Color::srgb(0.5, 1.0, 0.6), -54.0);
    text(
        &mut commands,
        Label,
        "Enter: invite    Ctrl+C: copy yours    Ctrl+V: paste    Esc: back",
        8.0,
        grey,
        -92.0,
    );
}

/// Receive and send lobby messages every frame, whichever online screen is up.
fn drive_lobby(
    mut lobby: ResMut<OnlineLobby>,
    mut info: ResMut<OnlineInfo>,
    mut sel: ResMut<Selection>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    let Some(l) = lobby.0.as_mut() else { return };
    let mut start = None;
    for event in l.poll() {
        match event {
            Event::Invited(from) => {
                info.invite_from = Some(from);
                info.message = format!("{from} wants to play!");
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
        let lobby_owned = lobby.0.take().expect("the lobby is here");
        let (socket, peer, _) = lobby_owned.into_game();
        match net::start_p2p(socket, player, peer) {
            Ok(session) => {
                sel.chars[player] = mine;
                sel.chars[1 - player] = theirs;
                sel.stage = 0;
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

/// The Online screen: typing the friend's address, inviting, and answering an invitation.
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
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let invited = matches!(lobby.0.as_ref().map(|l| l.stage), Some(Stage::Invited(_)));

    // Typing the address: digits, '.', ':' (and letters for a host name are not needed).
    for ev in typing.read() {
        if ev.state != ButtonState::Pressed || ctrl || invited {
            continue;
        }
        match (&ev.logical_key, &ev.text) {
            (bevy::input::keyboard::Key::Backspace, _) => {
                info.typed.pop();
            }
            (_, Some(t)) => {
                for c in t.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == ':') {
                    if info.typed.len() < 21 {
                        info.typed.push(c);
                    }
                }
            }
            _ => {}
        }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyV) {
        if let Ok(mut clip) = arboard::Clipboard::new() {
            if let Ok(t) = clip.get_text() {
                info.typed = t.trim().chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == ':').take(21).collect();
            }
        }
    }

    let Some(l) = lobby.0.as_mut() else {
        input_text.0 = "-".into();
        status_text.0 = info.message.clone();
        return;
    };
    let mine = l.public;
    if ctrl && keys.just_pressed(KeyCode::KeyC) {
        if let (Some(addr), Ok(mut clip)) = (mine, arboard::Clipboard::new()) {
            let _ = clip.set_text(addr.to_string());
            info.message = "Copied your address.".into();
        }
    }

    if invited {
        if keys.just_pressed(KeyCode::KeyF) || keys.just_pressed(KeyCode::Enter) {
            l.accept();
        } else if keys.just_pressed(KeyCode::KeyG) {
            l.decline();
            info.invite_from = None;
            info.message = "Declined.".into();
        }
    } else if keys.just_pressed(KeyCode::Enter) && !info.typed.is_empty() {
        let parsed = info.typed.parse::<SocketAddr>().ok().or_else(|| info.typed.parse::<IpAddr>().ok().map(|ip| SocketAddr::new(ip, DEFAULT_PORT)));
        match parsed {
            Some(addr) => {
                l.invite(addr);
                info.message = format!("Invited {addr}. Waiting for them... (they can invite you back at the same time)");
            }
            None => info.message = "That is not an address (like 203.0.113.5:7777).".into(),
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

    // The texts.
    addr_text.0 = match mine {
        Some(a) => a.to_string(),
        None => "looking it up...".into(),
    };
    let lan = lan_ip().map(|ip| format!("{ip}:{}", l.port()));
    let router = match &l.mapping {
        Mapping::Working => "asking router...".to_string(),
        Mapping::Opened(_) => "router port opened".to_string(),
        Mapping::Unavailable(why) if why == "off" => String::new(),
        Mapping::Unavailable(_) => "router: port not opened".to_string(),
    };
    lan_text.0 = match lan {
        Some(a) => format!("same network: {a}   {router}"),
        None => router,
    };
    let blink = if (time.elapsed_secs() * 2.0) as u32 % 2 == 0 { "_" } else { " " };
    input_text.0 = format!("{}{}", info.typed, blink);
    status_text.0 = info.message.clone();
    popup_text.0 = match (l.stage, info.invite_from) {
        (Stage::Invited(from), _) | (_, Some(from)) if invited => format!("{from} invites you!   F: accept    G: decline"),
        _ => String::new(),
    };
}

/// Test helpers (see `DevOptions`): invite, accept and pick without anyone at the keyboard.
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

/// One frame of the online match: take your keys (and, for the other side, the network's input).
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
    // The faster side slows down a little so the other can keep up.
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
