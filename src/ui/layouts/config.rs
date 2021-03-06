use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use druid::widget::*;
use druid::*;

use crate::assets::ServerIcons;
use crate::client;
use crate::server;
use crate::sync::PauseToken;
use crate::ui::widgets::timer_config::TimerConfig;
use crate::ui::widgets::wrappers::{new_button, new_label};

use super::super::data::*;
use super::consts::*;

/*
Configuration menu.
*/
pub fn config() -> impl Widget<AppData> {
    //Flex::column()
    //    .with_flex_child(address_input().expand_height(), 1.)
    //    .with_spacer(SPACER_SIZE * 2.)
    //    .with_flex_child(minecraft_browser_top().expand_height(), 1.)
    //    .with_spacer(SPACER_SIZE)
    //    .with_flex_child(minecraft_browser_bottom().expand_height(), 1.)
    //    .with_spacer(SPACER_SIZE * 2.)
    //    .with_flex_child(networking_select().expand_height(), 1.)
    //    .with_spacer(SPACER_SIZE * 2.)
    //    .with_flex_child(TimerConfig::new().lens(AppData::timer).expand_height(), 6.)
    //    .padding(SPACER_SIZE);
    TimerConfig::new().lens(AppData::timer)
}

/*
Host address input selection.
Title and input field.
*/
fn address_input() -> impl Widget<AppData> {
    Flex::row()
        .with_child(
            Label::new("Host address")
                .padding(Insets::uniform_xy(5., 0.))
                .align_horizontal(UnitPoint::CENTER)
                .background(Color::rgb8(0x90, 0x90, 0xFF))
                .expand_height(),
        )
        .with_spacer(SPACER_SIZE)
        .with_flex_child(
            TextBox::new()
                .lens(Settings::server_addr)
                .lens(AppData::settings)
                .expand(),
            1.,
        )
}

/*
Top half of the minecraft path selection.
Displays the field title and browse button.
*/
fn minecraft_browser_top() -> impl Widget<AppData> {
    Flex::row()
        .with_flex_child(new_label("Minecraft folder").expand(), 3.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(
            new_button::<AppData>("Browse")
                .on_click(move |ctx, _, _| {
                    ctx.submit_command(
                        druid::commands::SHOW_OPEN_PANEL
                            .with(FileDialogOptions::new().select_directories()),
                    )
                })
                .expand(),
            1.,
        )
}

/*
Bottom half of the minecraft path selection.
Displays the path.
*/
fn minecraft_browser_bottom() -> impl Widget<AppData> {
    Flex::row().with_flex_child(
        Label::new(|data: &String, _env: &Env| data.clone())
            .lens(Settings::minecraft_path)
            .lens(AppData::settings)
            .align_horizontal(UnitPoint::CENTER)
            .background(Color::rgb8(0x90, 0x90, 0xFF))
            .expand(),
        1.,
    )
}

/*
Generates client and server data from settings.
*/
fn startup_data(settings: &Settings) -> (ServerIcons, String, String, SocketAddr) {
    (
        ServerIcons::get_icons(),
        settings.minecraft_path.clone() + SERVERS,
        settings.minecraft_path.clone() + LATEST_LOG,
        SocketAddr::from_str(&settings.server_addr).unwrap(),
    )
}

/// Called when the 'Host' button is clicked.
///
/// Validates settings and starts server and client threads.
fn on_click_host(event: &mut EventCtx, data: &mut AppData, _env: &Env) {
    let settings = &data.settings;
    let result = validate_settings(settings);
    if let Ok(_) = result {
        save_settings(settings);
        data.state = State::Host;
        let stop_token = Arc::new(PauseToken::new(true));
        let pause_token = Arc::new(PauseToken::new(false));
        data.stop_token = Some(stop_token.clone());
        data.pause_token = Some(pause_token.clone());
        let (icons, server_data_path, log_path, server_addr) = startup_data(settings);
        let _server = server::start(
            event.get_external_handle(),
            stop_token.clone(),
            pause_token.clone(),
            server_addr.clone(),
        );
        let _client = client::start(
            event.get_external_handle(),
            stop_token.clone(),
            pause_token.clone(),
            icons,
            server_data_path,
            log_path,
            server_addr,
        );
    }
}

/// Called when the 'Connect' button is clicked.
///
/// Validates settings and starts server and client threads.
fn on_click_connect(event: &mut EventCtx, data: &mut AppData, _env: &Env) {
    let settings = &data.settings;
    let result = validate_settings(settings);
    if let Ok(_) = result {
        save_settings(settings);
        data.state = State::Client;
        let stop_token = Arc::new(PauseToken::new(true));
        let pause_token = Arc::new(PauseToken::new(false));
        data.stop_token = Some(stop_token.clone());
        data.pause_token = Some(pause_token.clone());
        let (icons, server_data_path, log_path, server_addr) = startup_data(settings);
        let _client = client::start(
            event.get_external_handle(),
            stop_token.clone(),
            pause_token.clone(),
            icons,
            server_data_path,
            log_path,
            server_addr,
        );
    }
}

fn networking_select() -> impl Widget<AppData> {
    Flex::row()
        .with_flex_child(
            new_button::<AppData>("Host")
                .on_click(on_click_host)
                .expand(),
            1.,
        )
        .with_spacer(SPACER_SIZE)
        .with_flex_child(
            new_button::<AppData>("Connect")
                .on_click(on_click_connect)
                .expand(),
            1.,
        )
}