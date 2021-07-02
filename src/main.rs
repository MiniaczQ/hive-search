mod log_reader;
mod client_state;
mod client;
mod server;
mod message;

use std::net::SocketAddr;
use std::thread;

use log_reader::track_logs;
use client_state::ClientState;
use server::*;

fn main() {
    let mut state = ClientState::new();
    let a = thread::Builder::new()
        .name("log reader".to_string())
        .spawn(move || track_logs(&"latest.log".to_string(), &mut state))
        .unwrap();

   let addr: SocketAddr = "127.0.0.1:2137".parse().unwrap();
   let server_config = ServerConfig {addr};
   let b = thread::Builder::new()
       .name("server".to_string())
       .spawn(move || server::run(server_config))
       .unwrap();

   a.join();
   b.join();
}