use std::{
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    collections::HashMap,
};

use crate::message::{client::ClientMessage, server::ServerMessage};

use super::state::server_state;

fn run(addr: SocketAddr) {
    let (clients_sink, clients_source) = channel::<(ClientMessage, SocketAddr)>();
    let client_sinks = Arc::new(Mutex::new(HashMap::<IpAddr, Sender<ServerMessage>>::new()));
    let client_sinks_cpy = client_sinks.clone();

    let client_handler = thread::Builder::new()
        .name("Client Handler".to_string())
        .spawn(move || handle_clients(clients_sink, client_sinks, addr))
        .expect("Failed to start the Client Handler thread.");

    let server_state = thread::Builder::new()
        .name("Server State".to_string())
        .spawn(move || server_state(client_sinks_cpy, clients_source))
        .expect("Failed to start the Server State thread.");

    client_handler.join().expect("Failed to join thread Client Handler.");
    server_state.join().expect("Failed to join thread Server State.");
}

pub fn spawn(addr: SocketAddr) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Server".to_string())
        .spawn(move || run(addr))
        .expect("Failed to start the Server thread.")
}

fn handle_clients(
    clients_sink: Sender<(ClientMessage, SocketAddr)>,
    client_sinks: Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    addr: SocketAddr,
) -> JoinHandle<()> {
    let listener = TcpListener::bind(addr).expect("Failed to bind client socket.");
    loop {
        let incoming = listener.accept();
        if let Ok((connection, addr)) = incoming {
            add_client(clients_sink.clone(), client_sinks.clone(), connection, addr);
        }
    }
}

fn add_client(
    clients_sink: Sender<(ClientMessage, SocketAddr)>,
    client_sinks: Arc<Mutex<HashMap<IpAddr, Sender<ServerMessage>>>>,
    connection: TcpStream,
    addr: SocketAddr,
) {
    let (client_sink, client_source) = channel::<ServerMessage>();

    {
        let mut server_sink = client_sinks.lock().expect("Failed to acquire mutex lock.");
        server_sink.insert(addr.ip(), client_sink);
    }

    let reader = connection;
    let writer = reader.try_clone().expect("Failed to clone TCP connection.");

    let _sender = thread::Builder::new()
        .name("Server Socket Sender".to_string())
        .spawn(move || super::sender::sender(client_source, writer))
        .expect("Failed to start the Server Socket Sender thread.");

    let _receiver = thread::Builder::new()
        .name("Server Socket Receiver".to_string())
        .spawn(move || super::receiver::receiver(clients_sink, reader, addr))
        .expect("Failed to start the Server Socket Receiver thread.");
}


