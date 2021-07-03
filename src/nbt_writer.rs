use nbt::*;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};

use crate::icons::{WAITCHAMP, WEIRDCHAMP};
use crate::message::ServerMessage;
use crate::message::ServerUpdate;

/*
Edit server nbt.
*/

#[derive(Debug, Serialize, Deserialize)]
struct Server {
    name: Option<String>,
    ip: Option<String>,
    icon: Option<String>,
    #[serde(rename = "acceptTextures")]
    accept_textures: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerData {
    servers: Vec<Server>,
}

fn starts_with_opt(s: &mut Server) -> Option<&mut Server> {
    if let Some(name) = &s.name {
        if name.starts_with("§rHive§rSearch§r") {
            return Some(s);
        }
    }
    None
}

pub fn update_nbt(message: ServerMessage, nbt_path: &String) {
    let file = File::open(nbt_path).expect("Failed to open 'servers.dat'.");
    let mut server_data: ServerData = from_reader(file).expect("Failed to parse 'servers.dat'.");
    let servers = &mut server_data.servers;

    let hs = servers.into_iter().find_map(|s| starts_with_opt(s));

    let name = Some(name_from_message(&message.update));
    let ip = Some(message.addr.to_string());
    let icon = icon_from_message(&message.update);

    if let Some(server) = hs {
        server.name = name;
        server.ip = ip;
        server.icon = icon;
    } else {
        let server = Server {
            name,
            ip,
            icon,
            accept_textures: None,
        };
        servers.push(server);
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(nbt_path)
        .expect("Failed to open 'servers.dat'.");
    to_writer(&mut file, &server_data, None).expect("Failed to write to 'servers.dat'.");
}

fn name_from_message(message: &ServerUpdate) -> String {
    match message {
        ServerUpdate::NoHosts => "§rHive§rSearch§r: §7No Games Open".to_string(),
        ServerUpdate::OneHost => "§rHive§rSearch§r: §aGame Open".to_string(),
        ServerUpdate::ManyHosts => "§rHive§rSearch§r: §6Multiple Games Open".to_string(),
    }
}

fn icon_from_message(message: &ServerUpdate) -> Option<String> {
    match message {
        ServerUpdate::NoHosts => Some(WAITCHAMP.to_string()),
        ServerUpdate::OneHost => None,
        ServerUpdate::ManyHosts => Some(WEIRDCHAMP.to_string()),
    }
}