//! Player codes: an address (IPv4 and port) written as a short code to send a friend, like `K7QH-2M9X-RT4C`,
//! instead of the bare address. The six bytes are scrambled with a fixed key and a checksum is added, so the
//! code doesn't read as an address and a mistyped one is caught. The letters skip I and O (and the digits 0
//! and 1) so nothing can be misread.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

const ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const KEY: [u8; 6] = [0x5a, 0xc3, 0x17, 0x9e, 0x64, 0xb1];
/// Twelve letters hold the 56 bits (six bytes and the check byte).
const LETTERS: usize = 12;

fn scramble(bytes: [u8; 6]) -> [u8; 6] {
    let mut out = [0u8; 6];
    let mut carry = 0x2du8;
    for k in 0..6 {
        // each byte also depends on the one before it, so neighbouring addresses give unrelated codes
        out[k] = (bytes[k] ^ KEY[k]).wrapping_add(carry);
        carry = out[k].rotate_left(3);
    }
    out
}

fn unscramble(out: [u8; 6]) -> [u8; 6] {
    let mut bytes = [0u8; 6];
    let mut carry = 0x2du8;
    for k in 0..6 {
        bytes[k] = out[k].wrapping_sub(carry) ^ KEY[k];
        carry = out[k].rotate_left(3);
    }
    bytes
}

fn check(bytes: &[u8; 6]) -> u8 {
    bytes.iter().fold(0x77u8, |acc, &b| acc.rotate_left(1) ^ b.wrapping_mul(31))
}

/// The code for an address, or `None` for one that isn't IPv4.
pub fn encode(addr: SocketAddr) -> Option<String> {
    let SocketAddr::V4(v4) = addr else { return None };
    let [a, b, c, d] = v4.ip().octets();
    let [p, q] = v4.port().to_be_bytes();
    let bytes = scramble([a, b, c, d, p, q]);
    let mut bits: u64 = 0;
    for byte in bytes.iter().chain(std::iter::once(&check(&bytes))) {
        bits = (bits << 8) | *byte as u64;
    }
    let mut letters = Vec::with_capacity(LETTERS);
    for k in (0..LETTERS).rev() {
        letters.push(ALPHABET[((bits >> (5 * k)) & 31) as usize] as char);
    }
    let text: String = letters.into_iter().collect();
    Some(format!("{}-{}-{}", &text[0..4], &text[4..8], &text[8..12]))
}

/// The address in a code (dashes, spaces and lower case are fine), or `None` if it isn't a valid code.
pub fn decode(code: &str) -> Option<SocketAddr> {
    let letters: Vec<u8> = code.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase() as u8).collect();
    if letters.len() != LETTERS {
        return None;
    }
    let mut bits: u64 = 0;
    for l in letters {
        bits = (bits << 5) | ALPHABET.iter().position(|&a| a == l)? as u64;
    }
    // 60 bits were written for 56: the top four must be empty
    if bits >> 56 != 0 {
        return None;
    }
    let all = bits.to_be_bytes();
    let bytes: [u8; 6] = all[1..7].try_into().ok()?;
    if check(&bytes) != all[7] {
        return None;
    }
    let [a, b, c, d, p, q] = unscramble(bytes);
    Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), u16::from_be_bytes([p, q]))))
}

/// The code for an address, or the address itself if it can't be written as one.
pub fn show(addr: SocketAddr) -> String {
    encode(addr).unwrap_or_else(|| addr.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_turns_back_into_its_address() {
        for addr in ["203.0.113.5:7777", "192.168.1.20:7778", "1.2.3.4:1", "255.255.255.255:65535", "0.0.0.0:0"] {
            let addr: SocketAddr = addr.parse().unwrap();
            let code = encode(addr).unwrap();
            assert_eq!(code.len(), 14, "{code}");
            assert!(!code.contains('.') && !code.contains(':'), "{code} doesn't look like an address");
            assert_eq!(decode(&code), Some(addr));
            assert_eq!(decode(&code.to_lowercase().replace('-', " ")), Some(addr), "typed loosely");
        }
    }

    #[test]
    fn neighbouring_addresses_give_unrelated_codes_and_typos_are_caught() {
        let a = encode("203.0.113.5:7777".parse().unwrap()).unwrap();
        let b = encode("203.0.113.6:7777".parse().unwrap()).unwrap();
        assert!(a.chars().zip(b.chars()).filter(|(x, y)| x != y).count() > 3, "{a} {b}");
        // change one letter: the check catches it
        let mut typo: Vec<char> = a.chars().collect();
        typo[5] = if typo[5] == 'X' { 'Y' } else { 'X' };
        assert_eq!(decode(&typo.into_iter().collect::<String>()), None);
        assert_eq!(decode("hello"), None);
    }
}
