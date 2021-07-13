mod assets;
mod logs;
mod nbt;
mod message;
mod client;
mod server;
mod ui;

use druid::*;

use ui::main::{hive, HiveSearchData, HiveSearchDelegate};

fn main() {
    let hive_window = WindowDesc::new(hive())
        .title("HiveSearch");
    let data: HiveSearchData = HiveSearchData::default();
    AppLauncher::with_window(hive_window)
        .delegate(HiveSearchDelegate)
        .log_to_console()
        .launch(data)
        .expect("Failed to start App.");
}

/*
let icons = icons::ServerIcons::get_icons();
let server_data_path = "servers.dat".to_string();
let log_path = "latest.log".to_string();
let server_addr = SocketAddr::from_str("127.0.0.1:2137").unwrap();
let client = client::main::spawn(icons, server_data_path, log_path, server_addr.clone());
let server = server::main::spawn(server_addr);
client.join().expect("Failed to join thread Client.");
server.join().expect("Failed to join thread Server.");
*/