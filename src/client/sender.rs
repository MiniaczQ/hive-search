use std::{net::TcpStream, sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::Receiver}};

use crate::messages::ClientMessage;

/*
Sends messages to the server.
*/

pub fn sender(
    client_source: Receiver<ClientMessage>,
    mut server_sink: TcpStream,
    breaker: Arc<AtomicBool>,
) {
    while breaker.load(Ordering::Relaxed) {
        let result = client_source.recv();
        if let Ok(message) = result {
            let result = bincode::serialize_into(&mut server_sink, &message);
            if let Err(_) = result {
                break
            }
        } else {
            break
        }
    }
}