use std::{
    net::{IpAddr, SocketAddr},
    sync::{
        mpsc::Sender,
        Arc, Mutex,
    },
    collections::HashMap,
};

use druid::{ExtEventSink, Target};

use crate::{message::server::ServerMessage, ui::layouts::host::USER_COUNT};

pub fn one(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    target: SocketAddr,
    message: ServerMessage,
    ui_handle: &ExtEventSink,
) {
    let mut client_sinks = client_sinks.lock().expect("Failed to acquire lock.");
    let result = client_sinks.get_mut(&target.ip());
    if let Some(sender) = result {
        let result = sender.send(message);
        if let Err(_) = result {
            client_sinks.remove(&target.ip());
        }
    }
    println!("Joined -> {}", client_sinks.len());
    ui_handle.submit_command(USER_COUNT, client_sinks.len(), Target::Auto).expect("Failed to submit command.");
}

pub fn all(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    message: ServerMessage,
    ui_handle: &ExtEventSink,
) {
    let mut for_removal: Vec<IpAddr> = Vec::new();
    let mut client_sinks = client_sinks.lock().expect("Failed to acquire lock.");
    for (addr, sender) in client_sinks.iter_mut() {
        let result = sender.send(message.clone());
        if let Err(_) = result {
            for_removal.push(addr.to_owned());
        }
    }
    if for_removal.len() != 0 {
        for addr in for_removal {
            client_sinks.remove(&addr);
        }
        println!("Disconnected -> {}", client_sinks.len());
        ui_handle.submit_command(USER_COUNT, client_sinks.len(), Target::Auto).expect("Failed to submit command.");
    }
}