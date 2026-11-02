//! Online play: rollback netcode over UDP with `ggrs`.
//!
//! Only inputs cross the network. When a remote input differs from the prediction, ggrs rewinds to the
//! saved state and the simulation is stepped forward again; that works because `GameState` is
//! `Clone + Hash` and deterministic. Both players need the same build and character files.
//! Player 0 is the left fighter.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::time::Duration;

use ggrs::{
    Config, DesyncDetection, Frame, GameStateCell, GgrsError, GgrsRequest, InputStatus, P2PSession, PlayerType,
    PredictRepeatLast, SessionBuilder, SyncTestSession,
};
use serde::{Deserialize, Serialize};

pub mod lobby;
pub mod stun;
pub mod transport;
pub mod upnp;

use crate::sim::{CharacterData, GameState, Input};

/// One player's controller state for one frame, as sent over the network.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NetInput(pub u16);

impl From<Input> for NetInput {
    fn from(i: Input) -> Self {
        NetInput(i.0)
    }
}

impl From<NetInput> for Input {
    fn from(i: NetInput) -> Self {
        Input(i.0)
    }
}

pub struct NetConfig;

impl Config for NetConfig {
    type Input = NetInput;
    // a held button is the best guess for what the other player is still doing
    type InputPredictor = PredictRepeatLast;
    type State = GameState;
    type Address = SocketAddr;
}

/// Game speed and netcode tuning: two frames of input delay on your own buttons, up to 8 frames
/// predicted ahead of the other player.
pub const FPS: usize = 60;
pub const INPUT_DELAY: usize = 2;
pub const MAX_PREDICTION: usize = 8;
/// The port both players use unless told otherwise.
pub const DEFAULT_PORT: u16 = 7777;

fn builder() -> Result<SessionBuilder<NetConfig>, GgrsError> {
    Ok(SessionBuilder::<NetConfig>::new()
        .with_num_players(2)?
        .with_fps(FPS)?
        .with_max_prediction_window(MAX_PREDICTION)
        .with_input_delay(INPUT_DELAY)
        // both sides compare a checksum of the state every 60 frames to catch a desync
        .with_desync_detection_mode(DesyncDetection::On { interval: 60 })
        .with_disconnect_timeout(Duration::from_secs(6))
        .with_disconnect_notify_delay(Duration::from_secs(2)))
}

/// Start a session as player `local` (0 = left, 1 = right) with the other player at `remote`, over the
/// socket the lobby used (so the router mapping stays open).
pub fn start_p2p(socket: transport::GameSocket, local: usize, remote: SocketAddr) -> Result<P2PSession<NetConfig>, GgrsError> {
    builder()?
        .add_player(PlayerType::Local, local)?
        .add_player(PlayerType::Remote(remote), 1 - local)?
        .start_p2p_session(socket)
}

/// An offline session that rolls back every frame and compares checksums, to find anything
/// non-deterministic.
pub fn start_synctest(check_distance: usize) -> Result<SyncTestSession<NetConfig>, GgrsError> {
    SessionBuilder::<NetConfig>::new()
        .with_num_players(2)?
        .with_check_distance(check_distance)
        .with_input_delay(INPUT_DELAY)
        .start_synctest_session()
}

/// A fingerprint of the whole state, compared between the players.
pub fn checksum(state: &GameState) -> u128 {
    let mut h = DefaultHasher::new();
    state.hash(&mut h);
    h.finish() as u128
}

/// Do what ggrs asks: save the state, load a saved one, or step the simulation.
pub fn handle_requests(state: &mut GameState, requests: Vec<GgrsRequest<NetConfig>>, chars: [&CharacterData; 2]) {
    for request in requests {
        match request {
            GgrsRequest::SaveGameState { cell, frame } => save(state, cell, frame),
            GgrsRequest::LoadGameState { cell, .. } => *state = cell.load().expect("ggrs asked to load a state that was never saved"),
            GgrsRequest::AdvanceFrame { inputs } => {
                // a disconnected player's input is the default (nothing pressed)
                let pad = |i: usize| match inputs.get(i) {
                    Some((input, status)) if *status != InputStatus::Disconnected => Input::from(*input),
                    _ => Input::default(),
                };
                state.step([pad(0), pad(1)], chars);
            }
        }
    }
}

fn save(state: &GameState, cell: GameStateCell<GameState>, frame: Frame) {
    debug_assert_eq!(state.frame as i32, frame, "the simulation and ggrs disagree about the frame");
    cell.save(frame, Some(state.clone()), Some(checksum(state)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggrs::{GgrsEvent, SessionState};

    fn load(name: &str) -> CharacterData {
        let text = std::fs::read_to_string(format!("assets/characters/{name}.ron")).unwrap();
        CharacterData::from_ron(&text).unwrap()
    }

    /// A deterministic stream of button presses, each held for a while like a player would.
    fn pad(seed: u32, frame: u32) -> NetInput {
        let window = frame / 7;
        let mut x = window.wrapping_mul(2_654_435_761) ^ seed.wrapping_mul(40_503);
        x ^= x >> 15;
        x = x.wrapping_mul(2_246_822_519);
        x ^= x >> 13;
        let mut bits = 0u16;
        for (k, flag) in [Input::LEFT, Input::RIGHT, Input::UP, Input::DOWN, Input::LIGHT, Input::HEAVY, Input::SPECIAL, Input::BLOCK].into_iter().enumerate() {
            if (x >> (k * 3)) & 7 == 0 {
                bits |= flag;
            }
        }
        NetInput(bits)
    }

    /// Every character pairing is rolled back every frame with specials ready; the checksums must match
    /// each time (ggrs errors if not).
    #[test]
    fn the_simulation_survives_rollback_every_frame() {
        let names = ["ayz", "anaran", "junkhead", "scribe", "cowboy", "evilayz"];
        for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)] {
            let (ca, cb) = (load(names[a]), load(names[b]));
            let mut session = start_synctest(7).unwrap();
            let mut state = GameState::new();
            state.fighters[0].meter = 100;
            state.fighters[1].meter = 100;
            for frame in 0..900u32 {
                session.add_local_input(0, pad(1, frame)).unwrap();
                session.add_local_input(1, pad(2, frame)).unwrap();
                let requests = session.advance_frame().unwrap_or_else(|e| panic!("{} vs {} desynced at frame {frame}: {e}", names[a], names[b]));
                handle_requests(&mut state, requests, [&ca, &cb]);
            }
        }
    }

    /// Two sessions over localhost UDP, each seeing the other's inputs late; checksums are compared every
    /// 60 frames and a desync would show up as an event.
    #[test]
    fn two_players_over_localhost_stay_in_sync() {
        let (ca, cb) = (load("junkhead"), load("anaran"));
        let (addr_a, addr_b): (SocketAddr, SocketAddr) = ("127.0.0.1:47231".parse().unwrap(), "127.0.0.1:47232".parse().unwrap());
        let mut a = start_p2p(transport::Transport::bind(addr_a.port()).unwrap().into_game_socket(), 0, addr_b).unwrap();
        let mut b = start_p2p(transport::Transport::bind(addr_b.port()).unwrap().into_game_socket(), 1, addr_a).unwrap();
        let (mut sa, mut sb) = (GameState::new(), GameState::new());
        sa.fighters[0].meter = 100;
        sb.fighters[0].meter = 100;

        let mut advanced = 0;
        let start = std::time::Instant::now();
        let mut desyncs = 0;
        while advanced < 600 && start.elapsed() < Duration::from_secs(30) {
            a.poll_remote_clients();
            b.poll_remote_clients();
            for ev in a.events().chain(b.events()) {
                if let GgrsEvent::DesyncDetected { .. } = ev {
                    desyncs += 1;
                }
            }
            if a.current_state() == SessionState::Running && b.current_state() == SessionState::Running {
                let f = advanced;
                a.add_local_input(0, pad(1, f)).unwrap();
                b.add_local_input(1, pad(2, f)).unwrap();
                // a frame may be skipped when one side is too far ahead; that's fine
                if let Ok(r) = a.advance_frame() {
                    handle_requests(&mut sa, r, [&ca, &cb]);
                }
                if let Ok(r) = b.advance_frame() {
                    handle_requests(&mut sb, r, [&ca, &cb]);
                }
                advanced += 1;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(advanced >= 600, "the sessions never got going ({advanced} frames)");
        assert_eq!(desyncs, 0, "the two simulations drifted apart");
    }
}
