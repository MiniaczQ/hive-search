/*
Client (to server) message.
*/

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    StartedHosting(u16),
    StoppedHosting,
}