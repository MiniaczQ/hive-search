use std::sync::atomic::Ordering;

use druid::widget::*;
use druid::*;

use super::super::data::*;
use super::arch::*;
use super::consts::*;

pub const LAN_COUNT: Selector<u8> = Selector::new("lan-count");

pub fn client_status() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(new_label("Status:").expand(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(
            DynLabel::new(lan_count_to_string(&0), LAN_COUNT, lan_count_to_string)
                .align_horizontal(UnitPoint::CENTER)
                .background(Color::rgb8(0x90, 0x90, 0xFF))
                .expand(),
            1.,
        )
}

pub fn client() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(
            new_button::<AppData>("Disconnect")
                .on_click(|_event, data, _env| {
                    data.state = State::Config;
                    if let Some(b) = &mut data.breaker {
                        b.store(false, Ordering::Relaxed);
                    }
                    data.breaker = None;
                })
                .expand(),
            1.,
        )
        .with_spacer(SPACER_SIZE)
        .with_flex_child(client_status(), 1.)
        .padding(SPACER_SIZE)
}

fn lan_count_to_string(count: &u8) -> String {
    match count {
        0 => "No games.".to_string(),
        1 => "One game.".to_string(),
        _ => "Many games.".to_string(),
    }
}
