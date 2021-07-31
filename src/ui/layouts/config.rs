use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use druid::widget::*;
use druid::*;

use crate::assets::icons::ServerIcons;
use crate::client;
use crate::server;

use super::super::data::*;
use super::arch::*;
use super::consts::*;

/*
Configuration menu.
*/
pub fn config() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(address_input().expand_height(), 1.)
        .with_spacer(SPACER_SIZE * 2.)
        .with_flex_child(minecraft_browser_top().expand_height(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(minecraft_browser_bottom().expand_height(), 1.)
        .with_spacer(SPACER_SIZE * 2.)
        .with_flex_child(networking_select().expand_height(), 1.)
        .padding(SPACER_SIZE)
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
fn on_click_host(
    event: &mut EventCtx,
    data: &mut AppData,
    _env: &Env,
) {
    let settings = &data.settings;
    let result = validate_settings(settings);
    if let Ok(_) = result {
        save_settings(settings);
        data.state = State::Host;
        let breaker = Arc::new(AtomicBool::new(true));
        data.breaker = Some(breaker.clone());
        let (icons, server_data_path, log_path, server_addr) = startup_data(settings);
        //let _server = server::main::start(server_addr.clone(), event.get_external_handle(), breaker.clone());
        let _client = client::main::spawn(icons, server_data_path, log_path, server_addr, event.get_external_handle(), breaker);
    }
}

/// Called when the 'Connect' button is clicked.
///
/// Validates settings and starts server and client threads.
fn on_click_connect(
    event: &mut EventCtx,
    data: &mut AppData,
    _env: &Env,
) {
    let settings = &data.settings;
    let result = validate_settings(settings);
    if let Ok(_) = result {
        save_settings(settings);
        data.state = State::Client;
        let breaker = Arc::new(AtomicBool::new(true));
        data.breaker = Some(breaker.clone());
        let (icons, server_data_path, log_path, server_addr) = startup_data(settings);
        let _client = client::main::spawn(icons, server_data_path, log_path, server_addr, event.get_external_handle(), breaker);
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
