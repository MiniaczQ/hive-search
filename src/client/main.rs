/*
Client stuff.
*/

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use std::{net::TcpStream, sync::mpsc::channel};
use std::thread::{self, JoinHandle};

use druid::ExtEventSink;

use crate::assets::icons::ServerIcons;
use crate::{message::client::ClientMessage, nbt::servers::NbtCommand};

const CONNECTION_TIMEOUT: f32 = 10.;

fn run(
    icons: ServerIcons,
    server_data_path: String,
    log_path: String,
    server_addr: SocketAddr,
    ui_handle: ExtEventSink,
    breaker: Arc<AtomicBool>,
) {
    let server_source = TcpStream::connect_timeout(&server_addr, Duration::from_secs_f32(CONNECTION_TIMEOUT))
        .expect("Failed to connect to TCP socket.");
    let server_sink = server_source.try_clone()
        .expect("Failed to clone TCP stream.");

    let (client_sink, client_source) = channel::<ClientMessage>();
    client_sink.send(ClientMessage::Joined).expect("Failed to send a client message.");
    let (nbt_sink, nbt_source) = channel::<NbtCommand>();

    let breaker_cpy = breaker.clone();
    let sender = thread::Builder::new()
        .name("Client Sender".to_string())
        .spawn(move ||
            super::sender::sender(client_source, server_sink, breaker_cpy)
        ).expect("Failed to start the Client Sender thread.");
    
    let breaker_cpy = breaker.clone();
    let receiver = thread::Builder::new()
        .name("Client Receiver".to_string())
        .spawn(move ||
            super::receiver::receiver(server_source, nbt_sink, ui_handle, breaker_cpy)
        ).expect("Failed to start the Client Receiver thread.");

    let nbt = crate::nbt::servers::spawn(nbt_source, icons, server_data_path, breaker.clone());
    let log = crate::logs::reader::spawn(log_path, client_sink, breaker);

    sender.join().expect("Failed to join thread Client Sender.");
    receiver.join().expect("Failed to join thread Client Receiver.");
    nbt.join().expect("Failed to join thread Server Data Editor.");
    log.join().expect("Failed to join thread Client Log Reader.");
}

pub fn spawn(
    icons: ServerIcons,
    server_data_path: String,
    log_path: String,
    server_addr: SocketAddr,
    ui_handle: ExtEventSink,
    breaker: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Client".to_string())
        .spawn(move ||
            run(icons, server_data_path, log_path, server_addr, ui_handle, breaker)
        ).expect("Failed to start the Client thread.")
}