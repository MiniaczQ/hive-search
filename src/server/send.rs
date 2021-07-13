use std::{
    net::{IpAddr, SocketAddr},
    sync::{
        mpsc::Sender,
        Arc, Mutex,
    },
    collections::HashMap,
};

use crate::message::server::ServerMessage;

pub fn one(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    target: SocketAddr,
    message: ServerMessage,
) {
    let mut client_sinks = client_sinks.lock().expect("Failed to acquire lock.");
    let result = client_sinks.get_mut(&target.ip());
    if let Some(sender) = result {
        let result = sender.send(message);
        if let Err(_) = result {
            client_sinks.remove(&target.ip());
        }
    }
}

pub fn all(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    message: ServerMessage,
) {
    let mut for_removal: Vec<IpAddr> = Vec::new();
    let mut client_sinks = client_sinks.lock().expect("Failed to acquire lock.");
    for (addr, sender) in client_sinks.iter_mut() {
        let result = sender.send(message.clone());
        if let Err(_) = result {
            for_removal.push(addr.to_owned());
        }
    }
    for addr in for_removal {
        client_sinks.remove(&addr);
    }
}