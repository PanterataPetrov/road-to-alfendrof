//! The step before a match: finding your public address, inviting a friend (or being invited),
//! and both choosing a fighter. Everything is peer to peer over one UDP socket; there is no host.
//!
//! Packets can get lost, so each message is simply repeated every few hundred milliseconds until
//! the other side's answer shows it arrived.
//!
//! Connecting through two routers: if you have opened your port (UPnP does that automatically on
//! many routers) the friend's invitation reaches you. If not, both of you enter the other's address
//! and invite at the same moment: each side's packets open the way for the other's. Two invitations
//! that cross are treated as accepted by both.

use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::stun;
use super::transport::{GameSocket, Packet, Transport};
use super::upnp::{self, Mapping};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LobbyMsg {
    /// "Play with me." `nonce` settles who is player 0 if two invitations cross.
    Invite { build: u32, nonce: u64 },
    /// My fighter (an index into the roster) and whether I have locked it in. Also the answer to an invitation.
    Pick { build: u32, index: u8, ready: bool, player: u8 },
    Decline,
    Leave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Idle,
    /// I have invited this address and wait for the answer.
    Inviting(SocketAddr),
    /// This address has invited me.
    Invited(SocketAddr),
    /// Both sides are choosing fighters.
    Picking(SocketAddr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Someone invites you: accept or decline.
    Invited(SocketAddr),
    Declined,
    /// The other side runs a different version or different character files.
    BuildMismatch,
    /// Both sides agreed; pick a fighter. `player` is 0 (left) or 1 (right).
    Picking { player: usize },
    PeerPicked { index: u8, ready: bool },
    PeerLeft,
    /// Both fighters are locked in: start the match.
    Ready { player: usize, mine: u8, theirs: u8, peer: SocketAddr },
}

pub struct Lobby {
    transport: Transport,
    build: u32,
    nonce: u64,
    txid: [u8; 12],
    stun_servers: Vec<SocketAddr>,
    last_stun: Option<Instant>,
    upnp: Option<Receiver<Mapping>>,
    /// The address the internet sees for this socket (what a friend types), once known.
    pub public: Option<SocketAddr>,
    pub mapping: Mapping,
    pub stage: Stage,
    player: usize,
    mine: (u8, bool),
    theirs: Option<(u8, bool)>,
    last_send: Instant,
    started: bool,
}

fn random_bytes<const N: usize>() -> [u8; N] {
    // Not secret, just different on each run: the clock and the process id, stirred.
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(1);
    let mut x = (t as u64) ^ ((std::process::id() as u64) << 32) ^ 0x9E37_79B9_7F4A_7C15;
    let mut out = [0u8; N];
    for b in out.iter_mut() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *b = (x >> 24) as u8;
    }
    out
}

impl Lobby {
    /// Open the socket on `port` (0 = any) and start looking up the public address and asking the router
    /// to forward the port. `build` identifies the version and character files; both sides must match.
    pub fn new(port: u16, build: u32, with_router: bool) -> std::io::Result<Self> {
        let transport = Transport::bind(port)?;
        let port = transport.port();
        Ok(Lobby {
            transport,
            build,
            nonce: u64::from_le_bytes(random_bytes::<8>()),
            txid: random_bytes::<12>(),
            stun_servers: if with_router { stun::resolve_servers() } else { Vec::new() },
            last_stun: None,
            upnp: with_router.then(|| upnp::open_port(port)),
            public: None,
            mapping: if with_router { Mapping::Working } else { Mapping::Unavailable("off".into()) },
            stage: Stage::Idle,
            player: 0,
            mine: (0, false),
            theirs: None,
            last_send: Instant::now() - Duration::from_secs(10),
            started: false,
        })
    }

    pub fn port(&self) -> u16 {
        self.transport.port()
    }

    /// Invite a friend by the address they gave you.
    pub fn invite(&mut self, to: SocketAddr) {
        self.stage = Stage::Inviting(to);
        self.theirs = None;
        self.mine.1 = false;
        self.started = false;
        self.last_send = Instant::now() - Duration::from_secs(10);
    }

    /// Accept the invitation that is pending.
    pub fn accept(&mut self) -> Option<Event> {
        if let Stage::Invited(from) = self.stage {
            self.stage = Stage::Picking(from);
            self.player = 1;
            self.theirs = None;
            self.mine.1 = false;
            self.started = false;
            self.last_send = Instant::now() - Duration::from_secs(10);
            return Some(Event::Picking { player: 1 });
        }
        None
    }

    pub fn decline(&mut self) {
        if let Stage::Invited(from) = self.stage {
            self.transport.send_lobby(&LobbyMsg::Decline, from);
        }
        self.stage = Stage::Idle;
    }

    /// Choose a fighter (`ready` locks it in).
    pub fn pick(&mut self, index: u8, ready: bool) {
        self.mine = (index, ready);
        self.last_send = Instant::now() - Duration::from_secs(10);
    }

    /// Give the invitation up, or leave the pick screen.
    pub fn leave(&mut self) {
        match self.stage {
            Stage::Inviting(to) | Stage::Picking(to) => self.transport.send_lobby(&LobbyMsg::Leave, to),
            _ => {}
        }
        self.stage = Stage::Idle;
        self.theirs = None;
        self.mine.1 = false;
    }

    /// Receive and resend; call about every frame.
    pub fn poll(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        self.poll_router();
        for (from, packet) in self.transport.receive() {
            match packet {
                Packet::Stun(msg) => {
                    if let Some(addr) = stun::parse_response(&msg, self.txid) {
                        self.public = Some(addr);
                    }
                }
                Packet::Lobby(msg) => self.on_message(from, msg, &mut events),
                Packet::Game(_) => {}
            }
        }
        // Resend whatever the stage needs.
        if self.last_send.elapsed() >= Duration::from_millis(300) {
            match self.stage {
                Stage::Inviting(to) => {
                    self.transport.send_lobby(&LobbyMsg::Invite { build: self.build, nonce: self.nonce }, to);
                    self.last_send = Instant::now();
                }
                Stage::Picking(peer) => {
                    self.transport.send_lobby(&LobbyMsg::Pick { build: self.build, index: self.mine.0, ready: self.mine.1, player: self.player as u8 }, peer);
                    self.last_send = Instant::now();
                }
                _ => {}
            }
        }
        if let Stage::Picking(peer) = self.stage {
            if let (true, Some((theirs, true)), false) = (self.mine.1, self.theirs, self.started) {
                self.started = true;
                events.push(Event::Ready { player: self.player, mine: self.mine.0, theirs, peer });
            }
        }
        events
    }

    fn poll_router(&mut self) {
        // Ask the STUN servers until one answers (a packet may be lost).
        if self.public.is_none() && self.last_stun.map_or(true, |t| t.elapsed() >= Duration::from_millis(1500)) {
            for server in &self.stun_servers {
                self.transport.send_raw(&stun::request(self.txid), *server);
            }
            self.last_stun = Some(Instant::now());
        }
        if let Some(rx) = &self.upnp {
            if let Ok(m) = rx.try_recv() {
                if let Mapping::Opened(addr) = &m {
                    self.public.get_or_insert(*addr);
                }
                self.mapping = m;
                self.upnp = None;
            }
        }
    }

    fn on_message(&mut self, from: SocketAddr, msg: LobbyMsg, events: &mut Vec<Event>) {
        match msg {
            LobbyMsg::Invite { build, nonce } => {
                if build != self.build {
                    self.transport.send_lobby(&LobbyMsg::Decline, from);
                    events.push(Event::BuildMismatch);
                    return;
                }
                match self.stage {
                    Stage::Idle => {
                        self.stage = Stage::Invited(from);
                        events.push(Event::Invited(from));
                    }
                    // Both invited each other: that counts as accepted. The higher number is player 0.
                    Stage::Inviting(to) if to == from => {
                        self.player = if self.nonce > nonce { 0 } else { 1 };
                        self.stage = Stage::Picking(from);
                        self.theirs = None;
                        self.mine.1 = false;
                        self.started = false;
                        self.last_send = Instant::now() - Duration::from_secs(10);
                        events.push(Event::Picking { player: self.player });
                    }
                    Stage::Invited(a) | Stage::Picking(a) if a == from => {}
                    _ => self.transport.send_lobby(&LobbyMsg::Decline, from),
                }
            }
            LobbyMsg::Pick { build, index, ready, player } => {
                if build != self.build {
                    return;
                }
                match self.stage {
                    // The answer to my invitation.
                    // (their pick says which player they are, so crossed invitations cannot disagree)
                    Stage::Inviting(to) if to == from => {
                        self.player = 1 - (player as usize).min(1);
                        self.stage = Stage::Picking(from);
                        self.theirs = Some((index, ready));
                        self.mine.1 = false;
                        self.started = false;
                        events.push(Event::Picking { player: self.player });
                        events.push(Event::PeerPicked { index, ready });
                    }
                    Stage::Picking(peer) if peer == from => {
                        if self.theirs != Some((index, ready)) {
                            events.push(Event::PeerPicked { index, ready });
                        }
                        self.theirs = Some((index, ready));
                    }
                    _ => {}
                }
            }
            LobbyMsg::Decline => {
                if self.stage == Stage::Inviting(from) {
                    self.stage = Stage::Idle;
                    events.push(Event::Declined);
                }
            }
            LobbyMsg::Leave => match self.stage {
                Stage::Invited(a) | Stage::Picking(a) | Stage::Inviting(a) if a == from => {
                    self.stage = Stage::Idle;
                    self.theirs = None;
                    self.mine.1 = false;
                    events.push(Event::PeerLeft);
                }
                _ => {}
            },
        }
    }

    /// The match starts: hand the socket to the game. Until the other side's game starts talking, the
    /// last pick keeps being resent so that they cannot be left waiting for it.
    pub fn into_game(self) -> (GameSocket, SocketAddr, usize) {
        let peer = match self.stage {
            Stage::Picking(p) => p,
            _ => panic!("into_game outside the pick stage"),
        };
        let last = LobbyMsg::Pick { build: self.build, index: self.mine.0, ready: true, player: self.player as u8 };
        (self.transport.into_game_socket_repeating(Some((last, peer))), peer, self.player)
    }
}

/// A fingerprint of everything both sides must share (version and character files).
pub fn build_id(parts: &[&str]) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for part in parts {
        for b in part.bytes().chain(std::iter::once(0)) {
            h ^= b as u32;
            h = h.wrapping_mul(16_777_619);
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pump(a: &mut Lobby, b: &mut Lobby, ea: &mut Vec<Event>, eb: &mut Vec<Event>, until: impl Fn(&[Event], &[Event]) -> bool) {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) && !until(ea, eb) {
            ea.extend(a.poll());
            eb.extend(b.poll());
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn two() -> (Lobby, Lobby, SocketAddr, SocketAddr) {
        let a = Lobby::new(0, 7, false).unwrap();
        let b = Lobby::new(0, 7, false).unwrap();
        let (pa, pb) = (a.port(), b.port());
        (a, b, format!("127.0.0.1:{pa}").parse().unwrap(), format!("127.0.0.1:{pb}").parse().unwrap())
    }

    #[test]
    fn an_invitation_is_accepted_and_both_pick_fighters() {
        let (mut a, mut b, _addr_a, addr_b) = two();
        let (mut ea, mut eb) = (Vec::new(), Vec::new());
        a.invite(addr_b);
        pump(&mut a, &mut b, &mut ea, &mut eb, |_, eb| eb.iter().any(|e| matches!(e, Event::Invited(_))));
        assert!(eb.iter().any(|e| matches!(e, Event::Invited(_))), "B should be invited");
        b.accept();
        pump(&mut a, &mut b, &mut ea, &mut eb, |ea, _| ea.iter().any(|e| matches!(e, Event::Picking { .. })));
        assert!(ea.contains(&Event::Picking { player: 0 }), "the inviter is player 0");
        a.pick(2, true);
        b.pick(4, true);
        pump(&mut a, &mut b, &mut ea, &mut eb, |ea, eb| ea.iter().any(|e| matches!(e, Event::Ready { .. })) && eb.iter().any(|e| matches!(e, Event::Ready { .. })));
        let ready = |es: &[Event]| es.iter().find_map(|e| if let Event::Ready { player, mine, theirs, .. } = e { Some((*player, *mine, *theirs)) } else { None });
        assert_eq!(ready(&ea), Some((0, 2, 4)));
        assert_eq!(ready(&eb), Some((1, 4, 2)));
    }

    #[test]
    fn two_invitations_that_cross_are_accepted_by_both() {
        let (mut a, mut b, addr_a, addr_b) = two();
        let (mut ea, mut eb) = (Vec::new(), Vec::new());
        a.invite(addr_b);
        b.invite(addr_a);
        pump(&mut a, &mut b, &mut ea, &mut eb, |ea, eb| ea.iter().any(|e| matches!(e, Event::Picking { .. })) && eb.iter().any(|e| matches!(e, Event::Picking { .. })));
        let player = |es: &[Event]| es.iter().find_map(|e| if let Event::Picking { player } = e { Some(*player) } else { None });
        let (pa, pb) = (player(&ea).expect("A picks"), player(&eb).expect("B picks"));
        assert_ne!(pa, pb, "one is player 0 and the other player 1");
    }

    #[test]
    fn a_different_build_is_turned_away() {
        let mut a = Lobby::new(0, 1, false).unwrap();
        let mut b = Lobby::new(0, 2, false).unwrap();
        let addr_b: SocketAddr = format!("127.0.0.1:{}", b.port()).parse().unwrap();
        let (mut ea, mut eb) = (Vec::new(), Vec::new());
        a.invite(addr_b);
        pump(&mut a, &mut b, &mut ea, &mut eb, |ea, eb| ea.contains(&Event::Declined) && eb.contains(&Event::BuildMismatch));
        assert!(eb.contains(&Event::BuildMismatch) && ea.contains(&Event::Declined));
    }

    #[test]
    fn the_match_runs_over_the_socket_the_lobby_used() {
        use crate::net::{handle_requests, start_p2p};
        use crate::sim::{CharacterData, GameState, Input};
        use ggrs::SessionState;
        let (mut a, mut b, _addr_a, addr_b) = two();
        let (mut ea, mut eb) = (Vec::new(), Vec::new());
        a.invite(addr_b);
        pump(&mut a, &mut b, &mut ea, &mut eb, |_, eb| eb.iter().any(|e| matches!(e, Event::Invited(_))));
        b.accept();
        a.pick(0, true);
        b.pick(1, true);
        pump(&mut a, &mut b, &mut ea, &mut eb, |ea, eb| ea.iter().any(|e| matches!(e, Event::Ready { .. })) && eb.iter().any(|e| matches!(e, Event::Ready { .. })));
        let (sock_a, peer_a, pl_a) = a.into_game();
        let (sock_b, peer_b, pl_b) = b.into_game();
        let mut sa = start_p2p(sock_a, pl_a, peer_a).unwrap();
        let mut sb = start_p2p(sock_b, pl_b, peer_b).unwrap();
        let c = CharacterData::from_ron(&std::fs::read_to_string("assets/characters/ayz.ron").unwrap()).unwrap();
        let (mut ga, mut gb) = (GameState::new(), GameState::new());
        let start = Instant::now();
        let mut frames = 0;
        while frames < 120 && start.elapsed() < Duration::from_secs(10) {
            sa.poll_remote_clients();
            sb.poll_remote_clients();
            if sa.current_state() == SessionState::Running && sb.current_state() == SessionState::Running {
                for (s, g, pl) in [(&mut sa, &mut ga, pl_a), (&mut sb, &mut gb, pl_b)] {
                    s.add_local_input(pl, Input(Input::RIGHT).into()).unwrap();
                    if let Ok(r) = s.advance_frame() {
                        handle_requests(g, r, [&c, &c]);
                    }
                }
                frames += 1;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(frames >= 120, "the game never started over the lobby's socket");
    }
}
