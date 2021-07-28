use std::{collections::HashMap, net::{IpAddr, SocketAddr}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}}};

use druid::ExtEventSink;

use crate::message::{client::ClientMessage, server::ServerMessage};

use super::send;

pub struct ServerState {
    pub hosts: HashMap<IpAddr, u16>,
}

pub fn server_state(
    mut client_sinks: Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    clients_source: Receiver<(ClientMessage, SocketAddr)>,
    ui_handle: ExtEventSink,
    breaker: Arc<AtomicBool>,
) {
    let mut state = ServerState {
        hosts: HashMap::new(),
    };
    while breaker.load(Ordering::Relaxed) {
        let result = clients_source.recv();
        if let Ok((message, addr)) = result {
            update_state(&mut client_sinks, &mut state,  message, addr, &ui_handle);
        } else {
            break
        }
    }
}

fn update_state(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    state: &mut ServerState,
    message: ClientMessage,
    addr: SocketAddr,
    ui_handle: &ExtEventSink,
) {
    match message {
        ClientMessage::StoppedHosting => {
            stopped_hosting(client_sinks, state, addr, ui_handle);
        },
        ClientMessage::StartedHosting(port) => {
            started_hosting(client_sinks, state, addr, ui_handle, port);
        },
        ClientMessage::Joined => {
            joined(client_sinks, state, addr, ui_handle);
        },
    }
}

fn stopped_hosting(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    state: &mut ServerState,
    addr: SocketAddr,
    ui_handle: &ExtEventSink,
) {
    let prev_len = state.hosts.len();
    state.hosts.remove(&addr.ip());
    let post_len = state.hosts.len();
    if prev_len.clamp(0, 2) != post_len.clamp(0, 2) {
        let out_message = state2message(state);
        send::all(client_sinks, out_message, ui_handle);
    }
}

fn started_hosting(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    state: &mut ServerState,
    addr: SocketAddr,
    ui_handle: &ExtEventSink,
    port: u16,
) {
    let prev_len = state.hosts.len();
    let result = state.hosts.insert(addr.ip(), port);
    let post_len = state.hosts.len();
    if (prev_len.clamp(0, 2) != post_len.clamp(0, 2)) ||
       (if let Some(prev_port) = result {prev_port != port} else {false}) {
        let out_message = state2message(state);
        send::all(client_sinks, out_message, ui_handle);
    }
}

fn joined(
    client_sinks: &mut Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    state: &ServerState,
    addr: SocketAddr,
    ui_handle: &ExtEventSink,
) {
    let out_message = state2message(state);
    send::one(client_sinks, addr, out_message, ui_handle);
}

fn state2message(
    state: &ServerState,
) -> ServerMessage {
    match state.hosts.len() {
        0 => {
            ServerMessage::NoHost
        },
        1 => {
            let (ip, port) = state.hosts.iter().next().expect("Failed to extract first member of 1 long HashSet.");
            ServerMessage::OneHost(SocketAddr::new(ip.to_owned(), port.to_owned()))
        },
        _ => {
            ServerMessage::ManyHosts
        },
    }
}