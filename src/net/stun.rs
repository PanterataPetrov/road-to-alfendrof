//! Finding the address the internet sees: a STUN request to a public server comes back with the IP and
//! port the router gave this socket, which is what a friend types to invite you.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

const COOKIE: u32 = 0x2112_A442;

/// Public STUN servers (they only tell you your own address).
pub const SERVERS: &[&str] = &["stun.l.google.com:19302", "stun1.l.google.com:19302", "stun.cloudflare.com:3478"];

/// The server addresses that resolve.
pub fn resolve_servers() -> Vec<SocketAddr> {
    SERVERS.iter().filter_map(|s| s.to_socket_addrs().ok()?.find(|a| a.is_ipv4())).collect()
}

/// A binding request with the given transaction id.
pub fn request(txid: [u8; 12]) -> Vec<u8> {
    let mut m = Vec::with_capacity(20);
    m.extend_from_slice(&0x0001u16.to_be_bytes()); // binding request
    m.extend_from_slice(&0u16.to_be_bytes()); // no attributes
    m.extend_from_slice(&COOKIE.to_be_bytes());
    m.extend_from_slice(&txid);
    m
}

/// The address a binding response reports, if it's a reply for `txid`.
pub fn parse_response(msg: &[u8], txid: [u8; 12]) -> Option<SocketAddr> {
    if msg.len() < 20 || msg[0..2] != 0x0101u16.to_be_bytes() || msg[4..8] != COOKIE.to_be_bytes() || msg[8..20] != txid {
        return None;
    }
    let len = u16::from_be_bytes([msg[2], msg[3]]) as usize;
    let end = (20 + len).min(msg.len());
    let mut at = 20;
    let mut plain: Option<SocketAddr> = None;
    while at + 4 <= end {
        let (kind, size) = (u16::from_be_bytes([msg[at], msg[at + 1]]), u16::from_be_bytes([msg[at + 2], msg[at + 3]]) as usize);
        let body = msg.get(at + 4..at + 4 + size)?;
        if body.len() >= 8 && body[1] == 0x01 {
            let port = u16::from_be_bytes([body[2], body[3]]);
            let ip = [body[4], body[5], body[6], body[7]];
            match kind {
                // XOR-MAPPED-ADDRESS: port and address are xor-ed with the cookie
                0x0020 => {
                    let xp = port ^ (COOKIE >> 16) as u16;
                    let c = COOKIE.to_be_bytes();
                    let xi = [ip[0] ^ c[0], ip[1] ^ c[1], ip[2] ^ c[2], ip[3] ^ c[3]];
                    return Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(xi)), xp));
                }
                // MAPPED-ADDRESS, from old servers
                0x0001 => plain = Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)),
                _ => {}
            }
        }
        at += 4 + ((size + 3) & !3);
    }
    plain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_an_xor_mapped_address() {
        let txid = [7u8; 12];
        // 203.0.113.5:54321, xor-ed the way a server sends it
        let c = COOKIE.to_be_bytes();
        let ip = [203u8, 0, 113, 5];
        let xp = 54321u16 ^ (COOKIE >> 16) as u16;
        let mut attr = vec![0, 0x20, 0, 8, 0, 1];
        attr.extend(xp.to_be_bytes());
        attr.extend([ip[0] ^ c[0], ip[1] ^ c[1], ip[2] ^ c[2], ip[3] ^ c[3]]);
        let mut msg = vec![0x01, 0x01];
        msg.extend((attr.len() as u16).to_be_bytes());
        msg.extend(COOKIE.to_be_bytes());
        msg.extend(txid);
        msg.extend(attr);
        assert_eq!(parse_response(&msg, txid), Some("203.0.113.5:54321".parse().unwrap()));
        assert_eq!(parse_response(&msg, [8u8; 12]), None, "a reply to another request is ignored");
    }
}
