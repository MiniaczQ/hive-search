/*
Server data manipulation.
*/

use nbt::*;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

use crate::assets::icons::ServerIcons;

const MARKER: &str = "§5§2§7§d§8§2§a§e§r"; // 0x527D82AE

pub enum Instructions {
    SetToNoHost,
    SetToOneHost(SocketAddr),
    SetToManyHosts,
}

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

    fn update(&mut self, name: Option<String>, ip: Option<String>, icon: Option<String>) {
        self.name = name;
        self.ip = ip;
        self.icon = icon;
    }
}

#[derive(Serialize, Deserialize)]
struct ServerData {
    servers: Vec<Server>,
}

/*
Return optional server if it starts with the marker.
*/
fn has_marker<'a>(server: &'a mut Server, marker: &str) -> Option<&'a mut Server> {
    if let Some(name) = &server.name {
        if name.starts_with(marker) {
            return Some(server);
        }
    }
    None
}

/*
Return optional server if one starts with the marker.
*/
fn get_marked_server<'a>(servers: &'a mut Vec<Server>, marker: &str) -> Option<&'a mut Server> {
    servers
        .into_iter()
        .find_map(|server| has_marker(server, marker))
}

/*
Load 'servers.dat'.
Either for the first time or after an update from client.
*/
fn load_data(server_data_path: &String) -> ServerData {
    let file = File::open(server_data_path).expect("Failed to open 'servers.dat'.");
    from_reader(file).expect("Failed to parse 'servers.dat'.")
}

/*
Stores current server data into
*/
fn save_data(server_data_path: &String, data: &ServerData) {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(server_data_path)
        .expect("Failed to open 'servers.dat'.");
    to_writer(&mut file, data, None).expect("Failed to write to 'servers.dat'.");
}

/*
Reloads server data if it got externally modified.
*/
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

/*
Applies instruction to server data.
If server didn't exist before, it gets created.
*/
fn update_server_data(data: &mut ServerData, instruction: Instructions, icons: &ServerIcons) {
    let hive_search_server = get_marked_server(&mut data.servers, MARKER);
    let (name, opt_ip, opt_icon) = data_from_instruction(instruction, icons);
    if let Some(server) = hive_search_server {
        server.update(Some(name), opt_ip, opt_icon);
    } else {
        data.servers.push(Server::new(Some(name), opt_ip, opt_icon));
    }
}

/*
Turns instruction into usable data.
*/
fn data_from_instruction(
    instruction: Instructions,
    icons: &ServerIcons,
) -> (String, Option<String>, Option<String>) {
    match instruction {
        Instructions::SetToNoHost => (
            format!("{}HiveSearch: §7No Games Open", MARKER),
            None,
            icons.no_hosts.clone(),
        ),
        Instructions::SetToOneHost(addr) => (
            format!("{}HiveSearch: §aGame Open", MARKER),
            Some(addr.to_string()),
            None,
        ),
        Instructions::SetToManyHosts => (
            format!("{}HiveSearch: §6Multiple Games Open", MARKER),
            None,
            icons.many_hosts.clone(),
        ),
    }
}

/*
Runs the functionality.
*/
fn run(instruction_source: Receiver<Instructions>, icons: ServerIcons, server_data_path: String) {
    let mut last_modification: SystemTime = SystemTime::now();
    let mut data: ServerData = load_data(&server_data_path);
    loop {
        let result = instruction_source.recv();
        if let Ok(instruction) = result {
            data = reload(data, &server_data_path, &last_modification);
            update_server_data(&mut data, instruction, &icons);
            save_data(&server_data_path, &data);
            last_modification = File::open(&server_data_path)
                .expect("Failed to open file 'servers.dat'.")
                .metadata()
                .expect("Failed to extract metadata of file 'servers.dat'.")
                .modified()
                .expect("Failed to extract last modification time of file 'servers.dat'");
        } else {
            break;
        }
    }
}

/*
Start the functionality in another thread.
Returns handle.
*/
pub fn spawn(
    instruction_source: Receiver<Instructions>,
    icons: ServerIcons,
    server_data_path: String,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Server Data Editor".to_string())
        .spawn(move || run(instruction_source, icons, server_data_path))
        .expect("Couldn't start the Server Data Eeditor thread.")
}
