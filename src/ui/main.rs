/*
UI stuff.
*/

use std::error::Error;
use std::fmt::Display;
use std::net::SocketAddr;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

use druid::*;
use druid::text::{Formatter, Validation};
use druid::text::ValidationError;
use druid::widget::*;

const SPACER_SIZE: f64 = 2.;

#[derive(Clone, Data, PartialEq, Copy, Serialize, Deserialize)]
pub enum HiveSearchState {
    Config,
    Host,
    User,
}

#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct HiveSearchData {
    pub state: HiveSearchState,
    pub minecraft_path: String,
    pub server_addr: String,
}

impl Default for HiveSearchData {
    fn default() -> Self {
        Self {
            state: HiveSearchState::Config,
            minecraft_path: String::new(),
            server_addr: String::new(),
        }
    }
}

pub struct HiveSearchDelegate;

impl AppDelegate<HiveSearchData> for HiveSearchDelegate {
    fn command (
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut HiveSearchData,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            data.minecraft_path = file_info.path().to_str().unwrap().to_string();
            return Handled::Yes;
        }
        Handled::No
    }
}

pub fn hive() -> impl Widget<HiveSearchData> {
    switcher()
}

fn configuration() -> impl Widget<HiveSearchData> {
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

fn address_input() -> impl Widget<HiveSearchData> {
    Flex::row()
        .with_child(Label::new("Host address")
            .padding(Insets::uniform_xy(5., 0.))
            .align_horizontal(UnitPoint::CENTER)
            .background(Color::rgb8(0x90, 0x90, 0xFF))
            .expand_height())
        .with_spacer(SPACER_SIZE)
        .with_flex_child(TextBox::new()
            .with_formatter(AddressFormater)
            .validate_while_editing(false)
            .lens(HiveSearchData::server_addr)
            .expand(), 1.)
}

fn minecraft_browser_top() -> impl Widget<HiveSearchData> {
    Flex::row()
        .with_flex_child(new_label("Minecraft folder").expand(), 3.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(new_button("Browse").lens(HiveSearchData::state).on_click(
            move |ctx, _, _|
            {ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(FileDialogOptions::new().select_directories()))}
            ).expand(), 1.)
        
}

fn minecraft_browser_bottom() -> impl Widget<HiveSearchData> {
    Flex::row()
        .with_flex_child(Label::new(
            |data: &String, _env: &Env| data.clone()
        ).lens(HiveSearchData::minecraft_path).align_horizontal(UnitPoint::CENTER)
        .background(Color::rgb8(0x90, 0x90, 0xFF)).expand(), 1.)
}

fn networking_select() -> impl Widget<HiveSearchData> {
    Flex::row()
        .with_flex_child(new_button("Host").on_click(
            |_event, data, _env|
            { *data = HiveSearchState::Host }
        ).lens(HiveSearchData::state).expand(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(new_button("Connect").on_click(
            |_event, data, _env|
            { *data = HiveSearchState::User }
        ).lens(HiveSearchData::state).expand(), 1.)
}

fn switcher() -> impl Widget<HiveSearchData> {
    ViewSwitcher::new(
        networking_picker,
        networking_builder,
    )
}

fn networking_picker(
    data: &HiveSearchData,
    _env: &Env
) -> HiveSearchState {
    data.state
}

fn networking_builder(
    selector: &HiveSearchState,
    _data: &HiveSearchData,
    _env: &Env
) -> Box<dyn Widget<HiveSearchData>> {
    match selector {
        &HiveSearchState::Config => Box::new(configuration()),
        &HiveSearchState::Host => Box::new(networking_host()),
        &HiveSearchState::User => Box::new(networking_connect()),
    }
}

fn networking_host() -> impl Widget<HiveSearchData> {
    Flex::column()
        .with_flex_child(new_button("Stop hosting").on_click(
            |_event, data, _env|
            { *data = HiveSearchState::Config }
        ).lens(HiveSearchData::state).expand(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(new_label("Status:").expand(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(new_label("0 users").expand(), 1.)
        .padding(SPACER_SIZE)
}

fn networking_connect() -> impl Widget<HiveSearchData> {
    Flex::column()
        .with_flex_child(new_button("Disconnect").on_click(
            |_event, data, _env|
            { *data = HiveSearchState::Config }
        ).lens(HiveSearchData::state).expand(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(new_label("Status:").expand(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(new_label("No game").expand(), 1.)
        .padding(SPACER_SIZE)
}

fn button_highlight(
    ctx: &mut PaintCtx,
    _: &HiveSearchState,
    _env: &Env,
) {
    let bounds = ctx.size().to_rect();

    ctx.fill(bounds, &Color::rgb8(0x90, 0x90, 0x90));

    if ctx.is_hot() {
        ctx.stroke(bounds.inset(-0.5), &Color::WHITE, 1.0);
    }

    if ctx.is_active() {
        ctx.fill(bounds, &Color::rgb8(0x71, 0x71, 0x71));
    }
}

fn new_button(
    text: impl Into<LabelText<HiveSearchState>>
) -> impl Widget<HiveSearchState> {
    Label::new(text)
        .align_horizontal(UnitPoint::CENTER)
        .background(Painter::new(button_highlight))
}

fn new_label(
    text: impl Into<LabelText<HiveSearchData>>
) -> impl Widget<HiveSearchData> {
    Label::new(text)
        .align_horizontal(UnitPoint::CENTER)
        .background(Color::rgb8(0x90, 0x90, 0xFF))
}

#[derive(Debug)]
struct SocketAddressError;

impl Display for SocketAddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid address.")
    }
}

impl Error for SocketAddressError {}

struct AddressFormater;

impl Formatter<String> for AddressFormater {
    fn format(&self, value: &String) -> String {
        value.to_string()
    }

    fn value(&self, input: &str) -> Result<String, ValidationError> {
        let result = SocketAddr::from_str(input);
        if let Ok(_) = result {
            return Ok(input.to_string())
        }
        Err(ValidationError::new(SocketAddressError))
    }

    fn validate_partial_input(&self, _input: &str, _sel: &text::Selection) -> Validation {
        Validation::success()
    }
}