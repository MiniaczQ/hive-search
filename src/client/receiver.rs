/*
Receives messages from the server.
*/

use std::{net::TcpStream, sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::Sender}};

use druid::{ExtEventSink, Target};

use crate::{message::server::ServerMessage, nbt::servers::NbtCommand, ui::layouts::client::LAN_COUNT};

pub fn receiver(
    mut server_source: TcpStream,
    nbt_sink: Sender<NbtCommand>,
    ui_handle: ExtEventSink,
    breaker: Arc<AtomicBool>,
) {
    while breaker.load(Ordering::Relaxed) {
        let result = bincode::deserialize_from::<&mut TcpStream, ServerMessage>(&mut server_source);
        if let Ok(message) = result {
            let result = match message {
                ServerMessage::NoHost => {
                    ui_handle.submit_command(LAN_COUNT, 0, Target::Auto).expect("Failed to submit command.");
                    nbt_sink.send(NbtCommand::SetToNoHost)
                },
                ServerMessage::OneHost(addr) => {
                    ui_handle.submit_command(LAN_COUNT, 1, Target::Auto).expect("Failed to submit command.");
                    nbt_sink.send(NbtCommand::SetToOneHost(addr))
                },
                ServerMessage::ManyHosts => {
                    ui_handle.submit_command(LAN_COUNT, 2, Target::Auto).expect("Failed to submit command.");
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
