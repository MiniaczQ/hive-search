/*
Communication with the server.
Send current state.
Receive shared state.
*/

use std::{
    net::{SocketAddr, TcpStream},
    thread,
};

use bincode::ErrorKind;

use crate::{log_reader, message::{ClientMessage, ClientUpdate, ServerMessage}, nbt_writer::update_nbt};

pub enum ClientStates {
    NotHosting,
    Hosting(u16),
}

pub struct ClientConfig {
    pub server_addr: SocketAddr,
    pub log_path: String,
    pub nbt_path: String,
}

pub fn run(config: ClientConfig) {
    let reader = TcpStream::connect(config.server_addr).expect("Failed to connect to TCP socket.");
    let writer = reader.try_clone().expect("Failed to clone TCP stream.");
    
    let log_path = config.log_path;
    let nbt_path = config.nbt_path;

    let log_reader = thread::Builder::new()
        .name("log reader".to_string())
        .spawn(move || log_reader::run(log_path, writer))
        .expect("Failed to start a thread.");

    let nbt_writer = thread::Builder::new()
        .name("nbt writer".to_string())
        .spawn(move || from_server(reader, nbt_path))
        .expect("Failed to start a thread.");

    log_reader.join().expect("Failed to join threads.");
    nbt_writer.join().expect("Failed to join threads.");
}

pub fn update_client_state(stream: &mut TcpStream, new: ClientStates) -> Result<(), Box<ErrorKind>> {
    to_server(stream, new)
}

fn to_server(writer: &mut TcpStream, state: ClientStates) -> Result<(), Box<ErrorKind>> {
    let message: ClientMessage;
    {
        if let ClientStates::Hosting(p) = state {
            message = ClientMessage {
                update: ClientUpdate::StartedHosting,
                port: p,
            }
        } else {
            message = ClientMessage {
                update: ClientUpdate::StoppedHosting,
                port: 0,
            }
        }
    }
    bincode::serialize_into(writer, &message)
}

fn from_server(mut reader: TcpStream, nbt_path: String) {
    loop {
        let result = bincode::deserialize_from::<&mut TcpStream, ServerMessage>(&mut reader);
        if let Ok(message) = result {
            update_nbt(message, & nbt_path);
        } else {
            break
        }
    }
}
