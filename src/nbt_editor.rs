use async_std::channel::Receiver;
use futures::{FutureExt, pin_mut, select};
use nbt::*;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;

use crate::assets::ServerIcons;
use crate::sync::PauseToken;

/// Hive Search main address
const MARKER: &str = "§5§2§7§d§8§2§a§e§r"; // 0x527D82AE

/// Possible NBT operations
pub enum NbtInstruction {
    SetToNoHost,
    SetToOneHost(SocketAddr),
    SetToManyHosts,
}

/// Minecraft server representation
#[derive(Serialize, Deserialize)]
struct Server {
    name: Option<String>,
    ip: Option<String>,
    icon: Option<String>,
    #[serde(rename = "acceptTextures")]
    accept_textures: Option<bool>,
}

impl Server {
    fn new(name: Option<String>, ip: Option<String>, icon: Option<String>) -> Self {
        Self {
            name,
            ip,
            icon,
            accept_textures: None,
        }
    }

    /// Change notable properties
    fn update(&mut self, name: Option<String>, ip: Option<String>, icon: Option<String>) {
        self.name = name;
        self.ip = ip;
        self.icon = icon;
    }
}

/// Minecraft server list representation
#[derive(Serialize, Deserialize)]
struct ServerData {
    servers: Vec<Server>,
}

/// Checks if a server has a marker
fn has_marker<'a>(server: &'a mut Server, marker: &str) -> Option<&'a mut Server> {
    if let Some(name) = &server.name {
        if name.starts_with(marker) {
            return Some(server);
        }
    }
    None
}

/// Returns the marked server if it exists
fn get_marked_server<'a>(servers: &'a mut Vec<Server>, marker: &str) -> Option<&'a mut Server> {
    servers
        .into_iter()
        .find_map(|server| has_marker(server, marker))
}

/// Loads server list from a file
fn load_data(server_data_path: &String) -> ServerData {
    let file = File::open(server_data_path).expect("Failed to open 'servers.dat'.");
    from_reader(file).expect("Failed to parse 'servers.dat'.")
}

/// Saves server list to a file
fn save_data(server_data_path: &String, data: &ServerData) {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(server_data_path)
        .expect("Failed to open 'servers.dat'.");
    to_writer(&mut file, data, None).expect("Failed to write to 'servers.dat'.");
}

/// Reloads the server list if it was updated
fn reload(
    mut data: ServerData,
    server_data_path: &String,
    last_modification: &SystemTime,
) -> ServerData {
    let current_modification = File::open(&server_data_path)
        .expect("Failed to open file 'servers.dat'.")
        .metadata()
        .expect("Failed to extract metadata of file 'servers.dat'.")
        .modified()
        .expect("Failed to extract last modification time of file 'servers.dat'");
    if !current_modification.eq(last_modification) {
        data = load_data(server_data_path);
    }
    data
}

/// Applies command to the server list
fn update_server_data(data: &mut ServerData, instruction: NbtInstruction, icons: &ServerIcons) {
    let hive_search_server = get_marked_server(&mut data.servers, MARKER);
    let (name, opt_ip, opt_icon) = data_from_instruction(instruction, icons);
    if let Some(server) = hive_search_server {
        server.update(Some(name), opt_ip, opt_icon);
    } else {
        data.servers.push(Server::new(Some(name), opt_ip, opt_icon));
    }
}

/// Applies instruction to a server
fn data_from_instruction(
    instruction: NbtInstruction,
    icons: &ServerIcons,
) -> (String, Option<String>, Option<String>) {
    match instruction {
        NbtInstruction::SetToNoHost => (
            format!("{}HiveSearch: §7No Games Open", MARKER),
            None,
            icons.no_hosts.clone(),
        ),
        NbtInstruction::SetToOneHost(addr) => (
            format!("{}HiveSearch: §aGame Open", MARKER),
            Some(addr.to_string()),
            None,
        ),
        NbtInstruction::SetToManyHosts => (
            format!("{}HiveSearch: §6Multiple Games Open", MARKER),
            None,
            icons.many_hosts.clone(),
        ),
    }
}

/// Edits NBT based on incoming commands.
pub async fn nbt_editor(
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    nbt_instruction_recv: Receiver<NbtInstruction>,
    icons: ServerIcons,
    server_data_path: String,
) {
    let mut last_modification: SystemTime = SystemTime::now();
    let mut data: ServerData = load_data(&server_data_path);
    while stop_token.is_paused().await {
        let command = nbt_instruction_recv.recv().fuse();
        let stop = stop_token.wait().fuse();
        pin_mut!(command);
        pin_mut!(stop);

        select! {
            command = command => {
                if let Ok(command) = command {
                    data = reload(data, &server_data_path, &last_modification);
                    update_server_data(&mut data, command, &icons);
                    save_data(&server_data_path, &data);
                    last_modification = File::open(&server_data_path)
                        .unwrap()
                        .metadata()
                        .unwrap()
                        .modified()
                        .unwrap()
                } else {
                    break
                }
            }
            _ = stop => break,
        }
        pause_token.wait().await;
    }
}
