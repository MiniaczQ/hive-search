use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::message::*;

/*
Communication with the clients.
Control LAN hosting flow.
*/
pub struct ServerConfig {
    pub server_addr: SocketAddr,
}

type TcpStreamVec = Arc<Mutex<Vec<TcpStream>>>;
type AddrMap = Arc<Mutex<HashMap<IpAddr, u16>>>;

/*
Main function of the module.
Handles all server communication.
*/
pub fn run(config: ServerConfig) {
    let writers: TcpStreamVec = Arc::new(Mutex::new(Vec::new()));
    let hosts: AddrMap = Arc::new(Mutex::new(HashMap::new()));
    await_clients(config.server_addr, writers, hosts);
}

/*
Awaits for new clients.
*/
fn await_clients(
    addr: SocketAddr,
    writers: TcpStreamVec,
    hosts: AddrMap,
) {
    let listener = TcpListener::bind(addr).expect("Failed to bind client socket.");
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
    writers: TcpStreamVec,
    hosts: AddrMap,
) {
    let reader = stream.try_clone().expect("Failed to clone TCP stream.");
    {
        let mut writers = writers.lock().expect("Failed to acquire mutex lock.");
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
    writers: TcpStreamVec,
    hosts: AddrMap,
) {
    loop {
        let result = bincode::deserialize_from::<&mut TcpStream, ClientMessage>(&mut reader);
        if let Ok(message) = result {
            {
                let mut hosts = hosts.lock().expect("Failed to acquire mutex lock.");
                let ip = addr.ip();
                match message.update {
                    ClientUpdate::StartedHosting => {
                        hosts.insert(ip,message.port);
                    }
                    ClientUpdate::StoppedHosting => {
                        hosts.remove(&ip);
                    }
                }
            }
            update_server(&writers, &hosts, message.update);
        } else {
            let ip = addr.ip();
            {
                let mut hosts = hosts.lock().expect("Failed to acquire mutex lock.");
                hosts.remove(&ip);
            }
            update_server(&writers, &hosts, ClientUpdate::StoppedHosting);
            break
        }
    }
}

/*
Writes information to all clients
*/
fn broadcast(message: ServerMessage, writers: &TcpStreamVec) {
    let mut writers = writers.lock().expect("Failed to acquire mutex lock.");
    let mut dropped: Vec<usize> = Vec::new();
    for (i, writer) in writers.iter_mut().enumerate() {
        let result = bincode::serialize_into(writer, &message);
        if let Err(_) = result {
            dropped.push(i);
        }
    }
    for i in dropped.iter().rev() {
        writers.remove(*i);
    }
}

/*
Returns whether server status was updated and to what.
*/
fn get_server_update(
    hosts: &AddrMap,
    update: ClientUpdate,
) -> Option<ServerMessage> {
    let hosts = hosts.lock().expect("Failed to acquire mutex lock.");
    let len = hosts.len();
    let prev_len = if let ClientUpdate::StartedHosting = update {
        len - 1
    } else {
        len + 1
    };
    if len.clamp(0, 2) != prev_len.clamp(0, 2) {
        let addr: SocketAddr = "0.0.0.0:0".parse().expect("Failed to parse constant expression to IP address.");
        match len {
            0 => {
                return Some(ServerMessage {
                    update: ServerUpdate::NoHosts,
                    addr,
                })
            }
            1 => {
                let result = hosts.iter().next().expect("Failed to extract the first element of a 1 long HashMap.");
                let addr = SocketAddr::new(result.0.clone(), result.1.clone());
                return Some(ServerMessage {
                    update: ServerUpdate::OneHost,
                    addr,
                })
            }
            _ => {
                return Some(ServerMessage {
                    update: ServerUpdate::ManyHosts,
                    addr,
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
    writers: &TcpStreamVec,
    hosts: &AddrMap,
    update: ClientUpdate,
) {
    if let Some(message) = get_server_update(hosts, update) {
        println!("SERVER: Updated address: {}", message.addr);
        broadcast(message, writers);
    }
}
