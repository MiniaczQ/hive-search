use std::sync::atomic::Ordering;

use druid::widget::*;
use druid::*;

use super::super::data::*;
use super::arch::*;
use super::client::client_status;
use super::consts::*;

pub const USER_COUNT: Selector<usize> = Selector::new("user-count");

fn host_status() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(new_label("Status:").expand(), 1.)
        .with_spacer(SPACER_SIZE)
        .with_flex_child(
            DynLabel::new(user_count_to_string(&0), USER_COUNT, user_count_to_string)
                .align_horizontal(UnitPoint::CENTER)
                .background(Color::rgb8(0x90, 0x90, 0xFF))
                .expand(),
            1.,
        )
}

pub fn host() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(
            new_button::<AppData>("Stop hosting")
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
        .with_flex_child(
            Flex::row()
                .with_flex_child(host_status(), 1.)
                .with_spacer(SPACER_SIZE)
                .with_flex_child(client_status(), 1.),
            1.,
        )
        .padding(SPACER_SIZE)
}

fn user_count_to_string(count: &usize) -> String {
    match count {
        0 => "0 users.".to_string(),
        1 => "1 user.".to_string(),
        _ => count.to_string() + " users.",
    }
}
