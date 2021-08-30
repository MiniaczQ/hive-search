use std::time::Instant;

use druid::commands::{CLOSE_ALL_WINDOWS, CLOSE_WINDOW};
use druid::widget::*;
use druid::*;

use crate::ui::data::AppData;

use super::my_widget_ext::MyWidgetExt;
use super::timer::{TIMER_START, Timer, TimerData};

pub struct TimerToggle {
    inner: Flex<TimerData>,
    window: Option<WindowId>,
}

impl TimerToggle {
    pub fn new() -> Self {
        Self {
            inner: Flex::row()
                .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                .must_fill_main_axis(true)
                .with_child(Checkbox::new("Enabled").lens(TimerData::enabled).with_tooltip("Whether the timer is enabled or not."))
                .with_child(Checkbox::new("Locked").lens(TimerData::locked).with_tooltip("Whether the timer can be moved around or not.")),
            window: None,
        }
    }
}

impl Widget<TimerData> for TimerToggle {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut TimerData, env: &Env) {
        self.inner.event(ctx, event, data, env);
        match (data.enabled, self.window) {
            (true, None) => {
                let window = WindowDesc::new(
                    Timer::new(data.get_font(ctx.text()), data.get_color())
                    .lens(AppData::timer),
                )
                .window_size_policy(WindowSizePolicy::Content)
                .transparent(true)
                .show_titlebar(false)
                .resizable(false)
                .title("Timer")
                .set_topmost(true)
                .set_position(Point::new(data.position[0], data.position[1]));
                
                let id = window.id;
                self.window = Some(id);
                ctx.new_window(window);
                ctx.submit_command(TIMER_START.with(Instant::now()).to(id));
            },
            (false, Some(window)) => {
                ctx.submit_command(CLOSE_WINDOW.to(window));
                self.window = None;
            },
            _ => {}
        }
        match event {
            Event::WindowDisconnected => {
                ctx.submit_command(CLOSE_ALL_WINDOWS);
            },
            _ => {},
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &TimerData, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &TimerData, data: &TimerData, env: &Env) {
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &TimerData, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &TimerData, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}
