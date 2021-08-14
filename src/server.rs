//! HiveSearch server.
//! Manages internal state and communication with clients.
//!
//! - Start with the `start` function.
//! - Control (stop/pause) with tokens.
//! - Gather response through event sink.
//! - The server is setup on the provided address.

use std::{collections::HashMap, net::IpAddr};

use async_std::{channel::{unbounded, Receiver, Sender}, net::{SocketAddr, TcpListener, TcpStream}, sync::Arc, task::spawn};
use druid::{ExtEventSink, Target};
use futures::*;

use crate::{codec::BincodeCodec, messages::{ClientMessage, ServerMessage}, sync::PauseToken, ui::layouts::host::USER_COUNT};

/// Starts the server threads:
///
/// Receive client connections
/// - adds new clients to the collection
///
/// Server state manager
/// - receives client updates
/// - updates server state
/// - sends updates to clients
/// - removes disconnected clients
pub fn start(
    ui_event_sink: ExtEventSink,
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    server_address: SocketAddr,
) {
    let (new_client_ios_sender, new_client_ios_receiver) = unbounded();

    let _pause_token = pause_token.clone();
    let _stop_token = stop_token.clone();

    spawn(client_connections_receiver(
        _stop_token,
        _pause_token,
        new_client_ios_sender,
        server_address,
    ));

    spawn(server_state_manager(
        ui_event_sink,
        stop_token,
        pause_token,
        new_client_ios_receiver,
    ));
}

/// Client communication interface.
struct ClientIO {
    to: Sender<ServerMessage>,
    from: Receiver<ClientMessage>,
    ip: IpAddr,
}

/// Map of all connected clients.
type ClientIOs = HashMap<u64, ClientIO>;

/// Socket with bincode encoding and asymetric data.
type EncodedSocket =
    asynchronous_codec::Framed<TcpStream, BincodeCodec<ServerMessage, ClientMessage>>;

/// Awaits for incoming client connections
/// Setups further communication
async fn client_connections_receiver(
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    new_client_ios: Sender<ClientIO>,
    server_address: SocketAddr,
) {
    let listener = TcpListener::bind(server_address).await.unwrap();
    while stop_token.is_paused().await {
        let incoming_connection = listener.accept().fuse();
        let stop = stop_token.wait().fuse();
        pin_mut!(incoming_connection);
        pin_mut!(stop);
        select! {
            incoming_connection = incoming_connection => {
                if let Ok((stream, client_address)) = incoming_connection {
                    let stream: EncodedSocket = asynchronous_codec::Framed::new(stream, BincodeCodec::new());
                    let (to_server, from_client) = unbounded::<ClientMessage>();
                    let (to_client, from_server) = unbounded::<ServerMessage>();
                    let client_io = ClientIO{to: to_client, from: from_client, ip: client_address.ip()};
                    if let Err(_) = new_client_ios.send(client_io).await {
                        break
                    }
                    let stop_token = stop_token.clone();
                    let pause_token = pause_token.clone();
                    spawn(run_client_io(
                        stop_token,
                        pause_token,
                        to_server,
                        from_server,
                        stream
                    ));
                }
            },
            _ = stop => break,
        };
        pause_token.wait().await;
    }
}

/// Sends and receives messages from a client.
///
/// Stops if
/// - to-server channel is terminated
/// - from-server channel is terminated
/// - socket is terminated
///
/// Pauses if pause token gets paused.
async fn run_client_io(
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    to_server: Sender<ClientMessage>,
    from_server: Receiver<ServerMessage>,
    mut stream: EncodedSocket,
) {
    while stop_token.is_paused().await {
        let send = from_server.recv().fuse();
        let recv = stream.try_next().fuse();
        let stop = stop_token.wait().fuse();
        pin_mut!(send);
        pin_mut!(recv);
        pin_mut!(stop);

        select! {
            send = send => {
                match send {
                    Ok(message) => {
                        if let Err(_) = stream.send(message).await {
                            break
                        }
                    }
                    Err(_) => break,
                }
            },
            recv = recv => {
                match recv {
                    Ok(optional) => {
                        if let Some(message) = optional {
                            if let Err(_) = to_server.send(message).await {
                                break
                            }
                        }
                    }
                    Err(_) => break,
                }
            },
            _ = stop => break,
        };

        pause_token.wait().await;
    }
}

/// Creates incremental u64 ids starting from 1.
struct IdDistributor {
    current: u64,
}

impl IdDistributor {
    /// Creates a new distributor.
    fn new() -> Self {
        Self { current: 0 }
    }

    /// Returns the next id.
    fn next(&mut self) -> u64 {
        self.current += 1;
        self.current
    }
}

/// Internal server state.
struct ServerState {
    lan_games: HashMap<u64, SocketAddr>,
}

/// Possible update causes for the server.
enum ServerWakeupCause {
    NewMessage(Result<ClientMessage, ()>, u64),
    NewClient(ClientIO),
    Stop,
}

/// No client connected.
/// Await stop or new client.
async fn await_wakeup_no_clients(
    stop_token: &Arc<PauseToken>,
    new_client_ios: &Receiver<ClientIO>,
) -> ServerWakeupCause {
    let new = new_client_ios.recv().fuse();
    let stop = stop_token.wait().fuse();
    pin_mut!(new);
    pin_mut!(stop);
    select! {
        result = new => {
            if let Ok(client_io) = result {
                return ServerWakeupCause::NewClient(client_io)
            } else {
                return ServerWakeupCause::Stop
            }
        },
        _ = stop => {
            return ServerWakeupCause::Stop
        },
    };
}

async fn await_wakeup(
    stop_token: &Arc<PauseToken>,
    new_client_ios: &Receiver<ClientIO>,
    client_ios: &mut ClientIOs,
) -> ServerWakeupCause {
    let iterator = client_ios.iter().map(|(id, client_io)| {
        let result = async move {
            let result = client_io.from.recv().await;
            if let Ok(message) = result {
                (Ok(message), *id)
            } else {
                (Err(()), *id)
            }
        };
        Box::pin(result)
    });
    let mut recv = futures::future::select_all(iterator).fuse();
    let new = new_client_ios.recv().fuse();
    let stop = stop_token.wait().fuse();
    pin_mut!(new);
    pin_mut!(stop);
    select! {
        ((result, id), _, _) = recv => {
            return ServerWakeupCause::NewMessage(result, id)
        },
        result = new => {
            if let Ok(client_io) = result {
                return ServerWakeupCause::NewClient(client_io)
            } else {
                return ServerWakeupCause::Stop
            }
        },
        _ = stop => {
            return ServerWakeupCause::Stop
        },
    };
}

/// Receives updates from clients.
/// Updates internal state.
/// Spreads new state to clients.
async fn server_state_manager(
    ui_event_sink: ExtEventSink,
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    new_client_ios: Receiver<ClientIO>,
) {
    let mut state = ServerState {
        lan_games: HashMap::new(),
    };
    let mut client_ios: ClientIOs = HashMap::new();
    let mut id_distributor = IdDistributor::new();

    while stop_token.is_paused().await {
        let update: ServerWakeupCause;
        if client_ios.len() == 0 {
            update = await_wakeup_no_clients(&stop_token, &new_client_ios).await;
        } else {
            update = await_wakeup(&stop_token, &new_client_ios, &mut client_ios).await;
        }

        match update {
            ServerWakeupCause::Stop => break,
            ServerWakeupCause::NewClient(client_io) => {
                client_ios.insert(id_distributor.next(), client_io);
            }
            ServerWakeupCause::NewMessage(result, id) => {
                if let Ok(message) = result {
                    update_state(&ui_event_sink, &mut state, &mut client_ios, id, message).await;
                } else {
                    client_ios.remove(&id);
                    update_state(
                        &ui_event_sink,
                        &mut state,
                        &mut client_ios,
                        id,
                        ClientMessage::StoppedHosting,
                    )
                    .await;
                }
            }
        }

        pause_token.wait().await;
    }
}

/// Updates server state based on the message.
/// Sends the update to client/-s.
async fn update_state(
    ui_event_sink: &ExtEventSink,
    state: &mut ServerState,
    client_ios: &mut ClientIOs,
    id: u64,
    message: ClientMessage,
) {
    match message {
        ClientMessage::StoppedHosting => {
            stopped_hosting(ui_event_sink, state, client_ios, id).await;
        }
        ClientMessage::StartedHosting(port) => {
            started_hosting(ui_event_sink, state, client_ios, id, port).await;
        }
        ClientMessage::Joined => {
            joined(ui_event_sink, state, client_ios, id).await;
        }
    }
}

/// Client stopped a LAN game.
async fn stopped_hosting(
    ui_event_sink: &ExtEventSink,
    state: &mut ServerState,
    client_ios: &mut ClientIOs,
    id: u64,
) {
    let prev_len = state.lan_games.len();
    state.lan_games.remove(&id);
    let post_len = state.lan_games.len();
    if prev_len.clamp(0, 2) != post_len.clamp(0, 2) {
        let message = state_into_message(state);
        send_to_all(ui_event_sink, client_ios, message).await;
    }
}

/// Client started a LAN game.
async fn started_hosting(
    ui_event_sink: &ExtEventSink,
    state: &mut ServerState,
    client_ios: &mut ClientIOs,
    id: u64,
    port: u16,
) {
    let prev_len = state.lan_games.len();
    let ip;
    {
        let client_io = client_ios.get(&id).unwrap();
        ip = client_io.ip.clone();
    }
    let address = SocketAddr::new(ip, port);
    let result = state.lan_games.insert(id, address.clone());
    let post_len = state.lan_games.len();
    if (prev_len.clamp(0, 2) != post_len.clamp(0, 2))
        || (if let Some(prev_address) = result {
            prev_address != address
        } else {
            false
        })
    {
        let message = state_into_message(state);
        send_to_all(ui_event_sink, client_ios, message).await;
    }
}

/// Client joined.
/// Send him the current status.
async fn joined(
    ui_event_sink: &ExtEventSink,
    state: &ServerState,
    client_ios: &mut ClientIOs,
    id: u64,
) {
    let message = state_into_message(state);
    send_to_one(ui_event_sink, client_ios, message, id).await;
}

fn state_into_message(state: &ServerState) -> ServerMessage {
    match state.lan_games.len() {
        0 => ServerMessage::NoHost,
        1 => {
            let (_, address) = state.lan_games.iter().next().unwrap();
            ServerMessage::OneHost(address.clone())
        }
        _ => ServerMessage::ManyHosts,
    }
}

/// Send message to the targeted client.
async fn send_to_one(
    ui_event_sink: &ExtEventSink,
    client_ios: &mut ClientIOs,
    message: ServerMessage,
    target_id: u64,
) {
    let result = client_ios.get_mut(&target_id);
    if let Some(client_io) = result {
        let _ = client_io.to.send(message).await;
    }
    ui_event_sink
        .submit_command(USER_COUNT, client_ios.len(), Target::Auto)
        .unwrap();
}

/// Send message to all clients.
async fn send_to_all(
    _ui_event_sink: &ExtEventSink,
    client_ios: &mut ClientIOs,
    message: ServerMessage,
) {
    for (_id, client_io) in client_ios.iter_mut() {
        let _ = client_io.to.send(message.clone()).await;
    }
}
