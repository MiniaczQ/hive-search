/*
Receives messages from the server.
*/

use std::{net::TcpStream, sync::mpsc::Sender};

use crate::{message::server::ServerMessage, nbt::servers::NbtCommand};

pub fn receiver(
    mut server_source: TcpStream,
    nbt_sink: Sender<NbtCommand>,
) {
    loop {
        let result = bincode::deserialize_from::<&mut TcpStream, ServerMessage>(&mut server_source);
        if let Ok(message) = result {
            let result = match message {
                ServerMessage::NoHost => {
                    nbt_sink.send(NbtCommand::SetToNoHost)
                },
                ServerMessage::OneHost(addr) => {
                    nbt_sink.send(NbtCommand::SetToOneHost(addr))
                },
                ServerMessage::ManyHosts => {
                    nbt_sink.send(NbtCommand::SetToManyHosts)
                },
            };
            if let Err(_) = result {
                break;
            }
        } else {
            break;
        }
    }
}
