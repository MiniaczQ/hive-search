/*
mod client;
mod log_reader;
mod message;
mod server;
mod config;
mod nbt_writer;
mod icons;
mod ui;
*/

mod assets;
mod nbt;

use assets::icons::ServerIcons;

fn main() {
    let icons = ServerIcons::get_icons();
    println!("{}", icons.no_hosts.unwrap());
}

/*
use druid::*;

use ui::*;

fn main() {
    let hive_window = WindowDesc::new(hive())
        .title("HiveSearch");
    let data: HiveSearchData = HiveSearchData::default();
    AppLauncher::with_window(hive_window)
        .delegate(HiveSearchDelegate)
        .log_to_console()
        .launch(data)
        .expect("Failed to open HiveSearch window.");
}
*/
/*
use std::thread;

use client::*;
use config::load_config;
use server::*;

fn main() {
    let config = load_config();

    let client_config = ClientConfig {
        server_addr: config.server_addr.clone(),
        log_path: config.log_path.clone(),
        nbt_path: config.nbt_path.clone(),
    };
    let client = thread::Builder::new()
        .name("client".to_string())
        .spawn(move || client::run(client_config))
        .expect("Failed to start a thread.");

    if config.server_host {
        let server_config = ServerConfig {
            server_addr: config.server_addr.clone(),
        };
        let server = thread::Builder::new()
            .name("server".to_string())
            .spawn(move || server::run(server_config))
            .expect("Failed to start a thread.");
        
        server.join().expect("Failed to join threads.");
    }

    client.join().expect("Failed to join threads.");
}
*/