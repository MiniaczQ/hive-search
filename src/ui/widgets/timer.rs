use std::{fs::{File, OpenOptions}, time::{Duration, Instant}};

use druid::*;
use druid::{
    piet::{D2DText, Text},
    widget::*,
};

use serde::{Deserialize, Serialize};

use super::color_picker::RGBA;

pub const TIMER_START: Selector<Instant> = Selector::new("timer-start");
pub const TIMER_RESET: Selector<()> = Selector::new("timer-reset");

const TIMER_UPDATE_DURATION: f32 = 0.01;

const TIMER_DEFAULT: &str = "--:--.---";

const TIMER_CONFIG: &str = "timer.config";

#[derive(Clone, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct TimerData {
    pub enabled: bool,
    pub locked: bool,
    pub position: [f64; 2],
    pub font_family: String,
    pub font_size: f64,
    pub font_bold: bool,
    pub font_italic: bool,
    pub color: RGBA,
    pub bg_color: RGBA,
    pub min_width: f64,
}

impl TimerData {
    pub fn load() -> Self {
        if let Ok(file) = File::open(TIMER_CONFIG) {
            return bincode::deserialize_from(file).unwrap()
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(file) = OpenOptions::new().write(true).truncate(true).create(true).open(TIMER_CONFIG) {
            bincode::serialize_into(file, self).ok();
        }
    }

    pub fn get_font(&self, text: &mut D2DText) -> FontDescriptor {
        FontDescriptor::new(match text.font_family(&self.font_family) {
            Some(font_family) => font_family,
            None => FontFamily::default(),
        })
        .with_size(self.font_size)
        .with_weight(
            match self.font_bold {
                false => FontWeight::NORMAL,
                true => FontWeight::BOLD,
            }
        )
        .with_style(if self.font_italic {
            FontStyle::Italic
        } else {
            FontStyle::Regular
        })
    }

    pub fn get_color(&self) -> Color {
        self.color.into()
    }

    pub fn get_bg_color(&self) -> Color {
        self.bg_color.into()
    }
}

impl Default for TimerData {
    fn default() -> Self {
        let mut font = FontDescriptor::default();
        font.size = 80.;
        Self {
            enabled: true,
            locked: false,
            position: [0., 0.],
            font_family: "Minecraft".to_owned(),
            font_size: 40.,
            font_bold: false,
            font_italic: false,
            color: RGBA {r: 209, g: 160, b: 68, a: 220},
            bg_color: RGBA {r: 0, g: 0, b: 0, a: 0},
            min_width: 0.,
        }
    }
}

pub struct Timer {
    label: Label<TimerData>,
    token: TimerToken,
    drag: Option<Vec2>,
    start: Option<Instant>,
}

impl Timer {
    pub fn new(font: FontDescriptor, color: Color) -> Self {
        Self {
            label: Label::new(TIMER_DEFAULT).with_font(font).with_text_color(color),
            token: TimerToken::INVALID,
            drag: None,
            start: None,
        }
    }
}

impl Widget<TimerData> for Timer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut TimerData, env: &Env) {
        self.label.event(ctx, event, data, env);
        match event {
            Event::Command(cmd) => {
                if let Some(start_point) = cmd.get(TIMER_START) {
                    self.start = Some(*start_point);
                    self.token = ctx.request_timer(Duration::from_secs_f32(TIMER_UPDATE_DURATION));
                }
                if let Some(_) = cmd.get(TIMER_RESET) {
                    self.start = None;
                    self.token = TimerToken::INVALID;
                    self.label.set_text(TIMER_DEFAULT);
                    ctx.request_update();
                }
            }
            Event::Timer(timer_token) => {
                if self.token == *timer_token {
                    if let Some(reference) = self.start {
                        self.token =
                            ctx.request_timer(Duration::from_secs_f32(TIMER_UPDATE_DURATION));
                        let delta = Instant::now().duration_since(reference);
                        let milliseconds = delta.as_millis();
                        let minutes = milliseconds / 60000;
                        let seconds = (milliseconds / 1000) - (minutes * 60);
                        let milliseconds = milliseconds % 1000;
                        self.label
                            .set_text(format!("{:01}:{:02}.{:02}", minutes, seconds, milliseconds / 10));
                        ctx.request_update();
                    }
                }
            }
            Event::MouseDown(m) => {
                if !data.locked && ctx.is_hot() {
                    ctx.set_active(true);
                    self.drag = Some(ctx.to_screen(m.pos).to_vec2());
                }
            }
            Event::MouseMove(m) => {
                if ctx.is_active() {
                    let curr_pos = ctx.to_screen(m.pos).to_vec2();
                    if let Some(prev_pos) = self.drag {
                        let delta = curr_pos - prev_pos;
                        let window = ctx.window();
                        let new_pos = window.get_position().to_vec2() + delta;
                        window.set_position(new_pos.to_point());
                    }
                    self.drag = Some(curr_pos);
                }
            }
            Event::MouseUp(_m) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    let v = ctx.window().get_position().to_vec2();
                    data.position = [v.x, v.y];
                }
            }
            Event::WindowDisconnected => {
                data.enabled = false;
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &TimerData,
        env: &Env,
    ) {
        self.label.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &TimerData, data: &TimerData, env: &Env) {
        if !old_data.eq(data)  {
            self.label.set_font(data.get_font(ctx.text()));
            self.label.set_text_color(data.get_color());
        }
        self.label.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &TimerData,
        env: &Env,
    ) -> Size {
        let mut min = bc.min();
        min.width = data.min_width;
        let bc = BoxConstraints::new(min, bc.max());
        self.label.layout(ctx, &bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &TimerData, env: &Env) {
        let rect = ctx.size().to_rect();
        ctx.fill(rect, &data.get_bg_color());
        self.label.paint(ctx, data, env);
    }
}
