use druid::*;

use super::data::AppData;

pub struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command (
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
        Handled::No
    }
}