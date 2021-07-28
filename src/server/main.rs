use std::{collections::HashMap, net::{IpAddr, SocketAddr, TcpListener, TcpStream}, sync::{Arc, Mutex, atomic::AtomicBool, mpsc::{channel, Sender}}, thread::{self, JoinHandle, sleep}, time::Duration};

use druid::ExtEventSink;

use crate::message::{client::ClientMessage, server::ServerMessage};

use super::state::server_state;

const CLIENT_ACCEPT_INTERVAL: f32 = 5.;

fn run(
    addr: SocketAddr,
    ui_handle: ExtEventSink,
    breaker: Arc<AtomicBool>,
) {
    let (clients_sink, clients_source) = channel::<(ClientMessage, SocketAddr)>();
    let client_sinks = Arc::new(Mutex::new(HashMap::<IpAddr, Sender<ServerMessage>>::new()));
    let client_sinks_cpy = client_sinks.clone();
    let breaker_cpy = breaker.clone();

    let client_handler = thread::Builder::new()
        .name("Client Handler".to_string())
        .spawn(move || handle_clients(clients_sink, client_sinks, addr, breaker_cpy))
        .expect("Failed to start the Client Handler thread.");

    let server_state = thread::Builder::new()
        .name("Server State".to_string())
        .spawn(move || server_state(client_sinks_cpy, clients_source, ui_handle, breaker))
        .expect("Failed to start the Server State thread.");

    client_handler.join().expect("Failed to join thread Client Handler.");
    server_state.join().expect("Failed to join thread Server State.");
}

pub fn spawn(
    addr: SocketAddr,
    ui_handle: ExtEventSink,
    breaker: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Server".to_string())
        .spawn(move || run(addr, ui_handle, breaker))
        .expect("Failed to start the Server thread.")
}

fn handle_clients(
    clients_sink: Sender<(ClientMessage, SocketAddr)>,
    client_sinks: Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    addr: SocketAddr,
    breaker: Arc<AtomicBool>,
) {
    let listener = TcpListener::bind(addr).expect("Failed to bind client socket.");
    listener.set_nonblocking(true).expect("Failed to turn the socket to non-blocking mode.");
    while breaker.load(std::sync::atomic::Ordering::Relaxed) {
        let incoming = listener.accept();
        if let Ok((connection, addr)) = incoming {
            connection.set_nonblocking(false).expect("Failed to turn the socket to blocking mode.");
            add_client(clients_sink.clone(), client_sinks.clone(), connection, addr, breaker.clone());
        }
        sleep(Duration::from_secs_f32(CLIENT_ACCEPT_INTERVAL));
    }
}

fn add_client(
    clients_sink: Sender<(ClientMessage, SocketAddr)>,
    client_sinks: Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    connection: TcpStream,
    addr: SocketAddr,
    breaker: Arc<AtomicBool>,
) {
    let (client_sink, client_source) = channel::<ServerMessage>();

    {
        let mut server_sink = client_sinks.lock().expect("Failed to acquire mutex lock.");
        server_sink.insert(addr.ip(), client_sink);
    }

    let reader = connection;
    let writer = reader.try_clone().expect("Failed to clone TCP connection.");
    let breaker_cpy = breaker.clone();

    let _sender = thread::Builder::new()
        .name("Server Socket Sender".to_string())
        .spawn(move || super::sender::sender(client_source, writer, breaker_cpy))
        .expect("Failed to start the Server Socket Sender thread.");

    let _receiver = thread::Builder::new()
        .name("Server Socket Receiver".to_string())
        .spawn(move || super::receiver::receiver(clients_sink, reader, addr, breaker))
        .expect("Failed to start the Server Socket Receiver thread.");
}


