/*
Synchronous access to client state.
*/

use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub enum ClientStates {
    NotHosting,     // Minecraft client is running
    Hosting(u16),   // User is hosting on his own world
}

pub struct ClientState(Arc<RwLock<ClientStates>>);

impl ClientState {
    pub fn new() -> Self {
        ClientState(Arc::new(RwLock::new(ClientStates::NotHosting)))
    }

    pub fn set(&mut self, new: ClientStates) {
        let mut state = self.0.write().unwrap();
        *state = new;
    }

    pub fn get(self) -> ClientStates {
        let state = self.0.read().unwrap();
        state.clone()
    }
}