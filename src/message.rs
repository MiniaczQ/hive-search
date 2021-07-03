/*
Socket communication standard
*/

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientMessage {
    pub update: ClientUpdate,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum ClientUpdate {
    StartedHosting,
    StoppedHosting,
    Joined,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerMessage {
    pub update: ServerUpdate,
    pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum ServerUpdate {
    NoHosts,
    OneHost,
    ManyHosts,
}
