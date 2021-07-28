use std::{net::{SocketAddr, TcpStream}, sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::Sender}};

use crate::message::client::ClientMessage;

pub fn receiver(
    clients_sink: Sender<(ClientMessage, SocketAddr)>,
    mut reader: TcpStream,
    addr: SocketAddr,
    breaker: Arc<AtomicBool>,
) {
    while breaker.load(Ordering::Relaxed) {
        let result = bincode::deserialize_from::<&mut TcpStream, ClientMessage>(&mut reader);
        if let Ok(message) = result {
            let result = clients_sink.send((message, addr));
            if let Err(_) = result {
                break
            }
        } else {
            break
        }
    }
}