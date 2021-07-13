/*
Server (to client) message.
*/

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    NoHost,
    OneHost(SocketAddr),
    ManyHosts,
}