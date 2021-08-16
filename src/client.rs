/*
Client stuff.
*/

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_std::channel::{unbounded, Receiver, Sender};
use async_std::io::timeout;
use async_std::net::TcpStream;
use async_std::task::{block_on, spawn};
use druid::{ExtEventSink, Target};
use futures::{FutureExt, SinkExt, TryStreamExt, pin_mut, select};

use crate::assets::ServerIcons;
use crate::codec::BincodeCodec;
use crate::log_reader::{log_reader, ClientChange};
use crate::messages::{ClientMessage, ServerMessage};
use crate::nbt_editor::{nbt_editor, NbtInstruction};
use crate::sync::PauseToken;
use crate::ui::delegate::RUNTIME_ERROR;
use crate::ui::layouts::client::LAN_COUNT;

const CONNECTION_TIMEOUT: f32 = 5.;

/// Socket with bincode encoding and asymetric data.
type EncodedSocket =
    asynchronous_codec::Framed<TcpStream, BincodeCodec<ClientMessage, ServerMessage>>;

pub fn start(
    ui_event_sink: ExtEventSink,
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    icons: ServerIcons,
    server_data_path: String,
    log_path: String,
    server_addr: SocketAddr,
) {
    let stream = block_on(timeout(
        Duration::from_secs_f32(CONNECTION_TIMEOUT),
        TcpStream::connect(server_addr),
    ));
    if let Ok(stream) = stream {
        let stream: EncodedSocket = asynchronous_codec::Framed::new(stream, BincodeCodec::new());

        let (log_sink, log_source) = unbounded::<ClientChange>();
        let (nbt_instruction_send, nbt_instruction_recv) = unbounded::<NbtInstruction>();
        let _stop_token = stop_token.clone();
        let _pause_token = pause_token.clone();
        spawn(communicate(
            ui_event_sink,
            _stop_token,
            _pause_token,
            log_source,
            nbt_instruction_send,
            stream,
        ));

        let init_duration = Duration::from_secs(5);
        let (_durations_send, durations_recv) = unbounded::<Duration>();
        let _stop_token = stop_token.clone();
        let _pause_token = pause_token.clone();
        spawn(log_reader(
            _stop_token,
            _pause_token,
            init_duration,
            durations_recv,
            log_path,
            log_sink,
        ));

        spawn(nbt_editor(
            stop_token,
            pause_token,
            nbt_instruction_recv,
            icons,
            server_data_path,
        ));
        Box::leak(Box::new(_durations_send));
    } else {
        ui_event_sink
            .submit_command(RUNTIME_ERROR, (), Target::Auto).ok();
    }
}

async fn communicate(
    ui_event_sink: ExtEventSink,
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    log_source: Receiver<ClientChange>,
    nbt_instruction_send: Sender<NbtInstruction>,
    mut stream: EncodedSocket,
) {
    println!("[client] started");
    stream.send(ClientMessage::Joined).await.ok();
    while stop_token.is_paused().await {
        let server_message = stream.try_next().fuse();
        let client_change = log_source.recv().fuse();
        let stop = stop_token.wait().fuse();
        pin_mut!(server_message);
        pin_mut!(client_change);
        pin_mut!(stop);

        select! {
            server_message = server_message => {
                match server_message {
                    Ok(opt_message) => {
                        if let Some(message) = opt_message {
                            from_server(&ui_event_sink, &nbt_instruction_send, message).await;
                        }
                    },
                    Err(_) => {
                        println!("[client] socket disconnected");
                        break
                    }
                }
            },
            client_change = client_change => {
                if let Ok(client_change) = client_change {
                    to_server(&mut stream, client_change).await;
                } else {
                    println!("[client] log reader disconnected");
                    ui_event_sink
                        .submit_command(RUNTIME_ERROR, (), Target::Auto).ok();
                    break
                }
            },
            _ = stop => {
                println!("[client] stop requested");
                break
            },
        }

        pause_token.wait().await;
    }
    println!("[client] stopped");
}

async fn from_server(
    ui_event_sink: &ExtEventSink,
    nbt_instruction_send: &Sender<NbtInstruction>,
    server_message: ServerMessage,
) {
    match server_message {
        ServerMessage::NoHost => {
            ui_event_sink
                .submit_command(LAN_COUNT, 0, Target::Auto).ok();
            nbt_instruction_send
                .send(NbtInstruction::SetToNoHost)
                .await
                .ok()
        }
        ServerMessage::OneHost(addr) => {
            ui_event_sink
                .submit_command(LAN_COUNT, 1, Target::Auto).ok();
            nbt_instruction_send
                .send(NbtInstruction::SetToOneHost(addr))
                .await
                .ok()
        }
        ServerMessage::ManyHosts => {
            ui_event_sink
                .submit_command(LAN_COUNT, 2, Target::Auto).ok();
            nbt_instruction_send
                .send(NbtInstruction::SetToManyHosts)
                .await
                .ok()
        }
    };
}

async fn to_server(stream: &mut EncodedSocket, client_change: ClientChange) {
    let message = match client_change {
        ClientChange::StartedHosting(port) => ClientMessage::StartedHosting(port),
        ClientChange::StoppedHosting => ClientMessage::StoppedHosting,
    };
    stream.send(message).await.ok();
}
