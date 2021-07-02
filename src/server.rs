use std::collections::HashSet;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::message::*;

/*
Communication with the clients.
Control LAN hosting flow.
*/
pub struct ServerConfig {
    pub addr: SocketAddr,
}

/*
Main function of the module.
Handles all server communication.
*/
pub fn run(config: ServerConfig) {
    let writers: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    let hosts: Arc<Mutex<HashSet<SocketAddr>>> = Arc::new(Mutex::new(HashSet::new()));
    await_clients(config.addr, writers, hosts);
}

/*
Awaits for new clients.
*/
fn await_clients(
    addr: SocketAddr,
    writers: Arc<Mutex<Vec<TcpStream>>>,
    hosts: Arc<Mutex<HashSet<SocketAddr>>>,
) {
    let listener = TcpListener::bind(addr).unwrap();
    loop {
        let incoming = listener.accept();
        if let Ok((stream, addr)) = incoming {
            handle_client(stream, addr, writers.clone(), hosts.clone());
        }
    }
}

/*
Starts read and write handlers for the client.
*/
fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    writers: Arc<Mutex<Vec<TcpStream>>>,
    hosts: Arc<Mutex<HashSet<SocketAddr>>>,
) {
    let reader = stream.try_clone().unwrap();
    {
        let mut writers = writers.lock().unwrap();
        writers.push(stream);
    }
    thread::spawn(move || from_client(reader, addr, writers, hosts));
}

/*
Reads from the client.
Updates the server based on that information.
*/
fn from_client(
    mut reader: TcpStream,
    addr: SocketAddr,
    writers: Arc<Mutex<Vec<TcpStream>>>,
    hosts: Arc<Mutex<HashSet<SocketAddr>>>,
) {
    loop {
        let message: ClientMessage = bincode::deserialize_from(&mut reader).unwrap();
        {
            let mut hosts = hosts.lock().unwrap();
            match message.update {
                ClientUpdate::StartedHosting => {
                    hosts.insert(to_addr(&message.addr));
                }
                ClientUpdate::StoppedHosting => {
                    hosts.remove(&to_addr(&message.addr));
                }
            }
        }
        update_server(&writers, &hosts, message.update);
    }
}

/*
Writes information to all clients
*/
fn broadcast(message: ServerMessage, writers: &Arc<Mutex<Vec<TcpStream>>>) {
    let mut writers = writers.lock().unwrap();
    for writer in writers.iter_mut() {
        bincode::serialize_into(writer, &message);
    }
}

/*
Returns whether server status was updated and to what.
*/
fn get_server_update(
    hosts: &Arc<Mutex<HashSet<SocketAddr>>>,
    update: ClientUpdate,
) -> Option<ServerMessage> {
    let hosts = hosts.lock().unwrap();
    let len = hosts.len();
    let prev_len = if let ClientUpdate::StartedHosting = update {
        len - 1
    } else {
        len + 1
    };
    if len.clamp(0, 2) != prev_len.clamp(0, 2) {
        match len {
            0 => {
                return Some(ServerMessage {
                    update: ServerUpdate::NoHosts,
                    addr: ([0u8; 4], 0u16),
                })
            }
            1 => {
                return Some(ServerMessage {
                    update: ServerUpdate::OneHost,
                    addr: from_addr(hosts.iter().next().unwrap()),
                })
            }
            _ => {
                return Some(ServerMessage {
                    update: ServerUpdate::ManyHosts,
                    addr: ([0u8; 4], 0u16),
                })
            }
        }
    }
    None
}

/*
Performs a server update.
Broadcasts it to all clients.
*/
fn update_server(
    writers: &Arc<Mutex<Vec<TcpStream>>>,
    hosts: &Arc<Mutex<HashSet<SocketAddr>>>,
    update: ClientUpdate,
) {
    if let Some(message) = get_server_update(hosts, update) {
        broadcast(message, writers);
    }
}
