//! Network communication standards.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Messages generated by server for clients.
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    NoHost,
    OneHost(SocketAddr),
    ManyHosts,
}

/// Messages generated by clients for server.
#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    StartedHosting(u16),
    StoppedHosting,
    Joined,
}