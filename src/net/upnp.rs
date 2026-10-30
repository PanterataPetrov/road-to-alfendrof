//! Asking the router to forward the game's UDP port (UPnP), so a friend can invite you even if you
//! haven't invited them. If it doesn't work, the invite still goes through when both players enter each
//! other's address (see `lobby`).

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use igd_next::{search_gateway, PortMappingProtocol, SearchOptions};

/// Result of the attempt, which runs on its own thread (finding a router takes a moment).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mapping {
    Working,
    /// The router said no, or there isn't one; the text says why.
    Unavailable(String),
    /// The router's own idea of the public address, if it gave one.
    Opened(SocketAddr),
}

/// This computer's address on the local network: the interface the OS would use to reach the
/// internet (nothing is sent).
pub fn lan_ip() -> Option<Ipv4Addr> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("8.8.8.8:80").ok()?;
    match s.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(v4) => Some(v4),
        _ => None,
    }
}

/// Try to open `port` (UDP) on the router in the background.
pub fn open_port(port: u16) -> Receiver<Mapping> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let result = (|| -> Result<SocketAddr, String> {
            let local = lan_ip().ok_or("no local network address")?;
            let mut options = SearchOptions::default();
            options.timeout = Some(Duration::from_secs(3));
            let gateway = search_gateway(options).map_err(|e| format!("no UPnP router found ({e})"))?;
            gateway
                .add_port(PortMappingProtocol::UDP, port, SocketAddr::new(local.into(), port), 4 * 3600, "Road to Alfendrof")
                .map_err(|e| format!("the router refused ({e})"))?;
            let ip = gateway.get_external_ip().map_err(|e| format!("the router did not say its address ({e})"))?;
            Ok(SocketAddr::new(ip, port))
        })();
        let _ = tx.send(match result {
            Ok(addr) => Mapping::Opened(addr),
            Err(why) => Mapping::Unavailable(why),
        });
    });
    rx
}
