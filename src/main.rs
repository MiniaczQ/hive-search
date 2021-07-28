mod assets;
mod logs;
mod nbt;
mod message;
mod client;
mod server;
mod ui;

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
        .log_to_console()
        .launch(data)
        .expect("Failed to start App.");
}
