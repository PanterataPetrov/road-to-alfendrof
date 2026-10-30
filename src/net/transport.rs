//! One UDP socket for everything online: the lobby's invitations, the router-address lookup (STUN)
//! and the game's own packets. Using a single socket matters: a router only lets replies through
//! to the exact local port that sent to the other side first, so the same port that was used to
//! find out the public address and to invite has to carry the game.
//!
//! Every packet starts with a tag byte: lobby or game. STUN replies are recognised by their
//! header (they start with 0x01).

use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use ggrs::{Message, NonBlockingSocket};

use super::lobby::LobbyMsg;

pub const TAG_LOBBY: u8 = 0xA0;
pub const TAG_GAME: u8 = 0xA1;
const BUFFER: usize = 4096;

/// What arrived on the socket.
pub enum Packet {
    Lobby(LobbyMsg),
    /// The reply of a STUN server (the raw message).
    Stun(Vec<u8>),
    /// A game packet; only the game socket looks at these.
    Game(Vec<u8>),
}

pub struct Transport {
    socket: UdpSocket,
}

impl Transport {
    /// Bind 0.0.0.0:`port` (0 for any free port) without blocking.
    pub fn bind(port: u16) -> std::io::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))?;
        socket.set_nonblocking(true)?;
        Ok(Transport { socket })
    }

    pub fn port(&self) -> u16 {
        self.socket.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    pub fn send_lobby(&self, msg: &LobbyMsg, to: SocketAddr) {
        let mut bytes = vec![TAG_LOBBY];
        bytes.extend(bincode::serialize(msg).expect("lobby messages serialize"));
        let _ = self.socket.send_to(&bytes, to);
    }

    pub fn send_raw(&self, bytes: &[u8], to: SocketAddr) {
        let _ = self.socket.send_to(bytes, to);
    }

    /// Everything that has arrived since the last call.
    pub fn receive(&self) -> Vec<(SocketAddr, Packet)> {
        let mut out = Vec::new();
        let mut buf = [0u8; BUFFER];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((n, from)) if n > 0 => {
                    let data = &buf[..n];
                    match data[0] {
                        TAG_LOBBY => {
                            if let Ok(msg) = bincode::deserialize::<LobbyMsg>(&data[1..]) {
                                out.push((from, Packet::Lobby(msg)));
                            }
                        }
                        TAG_GAME => out.push((from, Packet::Game(data[1..].to_vec()))),
                        0x01 => out.push((from, Packet::Stun(data.to_vec()))),
                        _ => {}
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => return out,
                // Windows reports an ICMP "port unreachable" from an earlier send as a reset.
                Err(e) if e.kind() == ErrorKind::ConnectionReset => {}
                Err(_) => return out,
            }
        }
    }

    /// Hand the socket to the game. `repeat` is a lobby message that keeps being resent (every 250 ms,
    /// for up to 3 s, or until a game packet arrives) in case the other side has not seen it yet.
    pub fn into_game_socket_repeating(self, repeat: Option<(LobbyMsg, SocketAddr)>) -> GameSocket {
        GameSocket { socket: self.socket, repeat, last: Instant::now(), deadline: Instant::now() + Duration::from_secs(3), heard_game: false }
    }

    pub fn into_game_socket(self) -> GameSocket {
        self.into_game_socket_repeating(None)
    }
}

/// The socket as ggrs sees it: only game packets.
pub struct GameSocket {
    socket: UdpSocket,
    repeat: Option<(LobbyMsg, SocketAddr)>,
    last: Instant,
    deadline: Instant,
    heard_game: bool,
}

impl NonBlockingSocket<SocketAddr> for GameSocket {
    fn send_to(&mut self, msg: &Message, addr: &SocketAddr) {
        let mut bytes = vec![TAG_GAME];
        bytes.extend(bincode::serialize(msg).expect("game messages serialize"));
        let _ = self.socket.send_to(&bytes, addr);
    }

    fn receive_all_messages(&mut self) -> Vec<(SocketAddr, Message)> {
        // Until the other side's game starts talking, keep repeating the last lobby message.
        if let Some((msg, to)) = &self.repeat {
            if !self.heard_game && Instant::now() < self.deadline && self.last.elapsed() >= Duration::from_millis(250) {
                let mut bytes = vec![TAG_LOBBY];
                bytes.extend(bincode::serialize(msg).expect("lobby messages serialize"));
                let _ = self.socket.send_to(&bytes, to);
                self.last = Instant::now();
            }
        }
        let mut out = Vec::new();
        let mut buf = [0u8; BUFFER];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((n, from)) if n > 1 && buf[0] == TAG_GAME => {
                    if let Ok(msg) = bincode::deserialize::<Message>(&buf[1..n]) {
                        self.heard_game = true;
                        out.push((from, msg));
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => return out,
                Err(e) if e.kind() == ErrorKind::ConnectionReset => {}
                Err(_) => return out,
            }
        }
    }
}
