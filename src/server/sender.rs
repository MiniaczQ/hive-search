use std::{net::TcpStream, sync::mpsc::Receiver};

use crate::message::server::ServerMessage;

pub fn sender(
    client_source: Receiver<ServerMessage>,
    mut writer: TcpStream,
) {
    loop {
        let result = client_source.recv();
        if let Ok(message) = result {
            let result = bincode::serialize_into(&mut writer, &message);
            if let Err(_) = result {
                break
            }
        } else {
            break
        }
    }
}