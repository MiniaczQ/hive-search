#![feature(async_closure)]

mod messages;
mod client;
mod server;
mod ui;
mod sync;
mod assets;
mod log_reader;
mod nbt_editor;
mod codec;

use druid::*;
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
    AppLauncher::with_window(hive_window)
        .delegate(Delegate)
        .launch(data)
        .ok();
}

