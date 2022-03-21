mod messages;
mod client;
mod server;
mod ui;
mod sync;
mod assets;
mod log_reader;
mod nbt_editor;
mod codec;
mod resources;

use druid::*;
use ui::{data::*, delegate::Delegate, main::hive, widgets::timer::TimerData};

fn main() {
    let hive_window = WindowDesc::new(hive())
        .window_size_policy(WindowSizePolicy::Content)
        .title("HiveSearch")
        .resizable(false);
    let data: AppData = AppData {
        settings: load_settings(),
        timer: TimerData::load(),
        ..Default::default()
    };
    AppLauncher::with_window(hive_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(data)
        .ok();
}

