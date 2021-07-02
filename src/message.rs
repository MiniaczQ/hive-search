use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use serde::{Serialize, Deserialize};

/*
Socket communication standard
*/

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientMessage {
    pub update: ClientUpdate,
    pub addr: ([u8; 4], u16),
}

#[derive(Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum ClientUpdate {
    StartedHosting,
    StoppedHosting,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerMessage {
    pub update: ServerUpdate,
    pub addr: ([u8; 4], u16),
}

#[derive(Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum ServerUpdate {
    NoHosts,
    OneHost,
    ManyHosts,
}

pub fn to_addr(addr: & ([u8; 4], u16)) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(addr.0[0], addr.0[1], addr.0[2], addr.0[3])), addr.1)
}

pub fn from_addr(addr: & SocketAddr) -> ([u8; 4], u16) {
    if let IpAddr::V4(ip) = addr.ip() {
        return (ip.octets(), addr.port())
    }
    ([0u8; 4], 0u16)
}