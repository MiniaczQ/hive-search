use druid::*;
use druid::widget::*;

use super::data::{AppData, State};

use super::layouts::{config::config, host::host, client::client};

pub fn hive() -> impl Widget<AppData> {
    switcher()
}

/// Changes UI based on the application state.
fn switcher() -> impl Widget<AppData> {
    ViewSwitcher::new(
        |data: &AppData, _env| {data.state},
        |selector, _data, _env| {
            match selector {
                &State::Config => Box::new(config()),
                &State::Host => Box::new(host()),
                &State::Client => Box::new(client()),
            }
        },
    )
}
