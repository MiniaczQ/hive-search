use async_std::task::block_on;
use druid::*;

use super::data::{AppData, State};

pub const RUNTIME_ERROR: Selector<()> = Selector::new("runtime-error");

pub struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppData,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            data.settings.minecraft_path = file_info.path().to_str().unwrap().to_string();
            return Handled::Yes;
        }
        if let Some(()) = cmd.get(RUNTIME_ERROR) {
            data.state = State::Config;
            if let Some(stop_token) = &mut data.stop_token {
                block_on(stop_token.resume());
                data.stop_token = None;
                data.pause_token = None;
            }
            return Handled::Yes;
        }
        Handled::No
    }
}
