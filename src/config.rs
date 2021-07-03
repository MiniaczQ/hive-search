/*
Loads the config.
*/

use std::{fs::File, io::{BufRead, BufReader}, net::SocketAddr};

pub struct Config {
    pub server_addr: SocketAddr,
    pub server_host: bool,
    pub log_path: String,
    pub nbt_path: String,
}

pub fn load_config() -> Config {
    let config = BufReader::new(File::open("config.txt").unwrap());

    let mut lines = config.lines();

    let server_addr: SocketAddr = lines.next().unwrap().unwrap().parse().unwrap();
    let server_host: bool = lines.next().unwrap().unwrap().parse().unwrap();
    let log_path: String = lines.next().unwrap().unwrap().parse().unwrap();
    let nbt_path: String = lines.next().unwrap().unwrap().parse().unwrap();

    Config {
        server_addr,
        server_host,
        log_path,
        nbt_path,
    }
}