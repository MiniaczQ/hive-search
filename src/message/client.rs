/*
Client (to server) message.
*/

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    StartedHosting(u16),
    StoppedHosting,
    Joined,
}