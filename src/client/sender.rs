use std::{net::TcpStream, sync::mpsc::Receiver};

use crate::message::client::ClientMessage;

/*
Sends messages to the server.
*/

pub fn sender(
    client_source: Receiver<ClientMessage>,
    mut server_sink: TcpStream,
) {
    loop {
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