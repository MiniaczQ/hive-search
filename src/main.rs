#![feature(async_closure)]

mod messages;
mod client;
mod server;
mod ui;
mod sync;

use std::{net::SocketAddr, str::FromStr};

use druid::*;

use async_std::sync::Arc;

use sync::PauseToken;
use ui::{main::hive, data::*, delegate::Delegate};

fn main() {
    let hive_window = WindowDesc::new(hive())
        .title("HiveSearch")
        .resizable(false)
        .window_size((600., 160.));
    let data: AppData = AppData {
        settings: load_settings(),
        ..Default::default()
    };
    let ui_event_sink = AppLauncher::with_window(hive_window)
        .delegate(Delegate)
        .log_to_console().get_external_handle();
        //.launch(data)
        //.expect("Failed to start App.");

    let stop_token = Arc::new(PauseToken::new(true));
    let pause_token = Arc::new(PauseToken::new(false));
    let server_address = SocketAddr::from_str("127.0.0.1:2137").unwrap();
    server::start(ui_event_sink, stop_token, pause_token, server_address);
    std::thread::sleep(std::time::Duration::from_secs_f32(180.));
}
