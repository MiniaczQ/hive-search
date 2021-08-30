use std::time::{Duration, Instant};

use druid::commands::CLOSE_WINDOW;
use druid::lens::Unit;
use druid::widget::*;
use druid::*;

use crate::ui::data::AppData;

const WAIT_DURATION: Duration = Duration::from_millis(700);
const RESCHEDULE_DURATION: Duration = Duration::from_millis(50);

enum TooltipState {
    Showing(WindowId),
    Waiting {
        last_move: Instant,
        timer_expire: Instant,
        token: TimerToken,
        window_pos: Point,
    },
    Fresh,
}

pub struct TooltipController {
    tip: String,
    state: TooltipState,
}

impl TooltipController {
    pub fn new(tip: impl Into<String>) -> Self {
        TooltipController {
            tip: tip.into(),
            state: TooltipState::Fresh,
        }
    }
}

impl<T, W: Widget<T>> Controller<T, W> for TooltipController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let cursor_size = Size::new(15., 15.);
        let now = Instant::now();
        let new_state = match &self.state {
            TooltipState::Fresh => match event {
                Event::MouseMove(me) if ctx.is_hot() => Some(TooltipState::Waiting {
                    last_move: now,
                    timer_expire: now + WAIT_DURATION,
                    token: ctx.request_timer(WAIT_DURATION),
                    window_pos: me.window_pos,
                }),
                _ => None,
            },
            TooltipState::Waiting {
                last_move,
                timer_expire,
                token,
                window_pos,
            } => match event {
                Event::MouseMove(me) if ctx.is_hot() => {
                    let (cur_token, cur_expire) = if *timer_expire - now < RESCHEDULE_DURATION {
                        (ctx.request_timer(WAIT_DURATION), now + WAIT_DURATION)
                    } else {
                        (*token, *timer_expire)
                    };
                    Some(TooltipState::Waiting {
                        last_move: now,
                        timer_expire: cur_expire,
                        token: cur_token,
                        window_pos: me.window_pos,
                    })
                }
                Event::Timer(tok) if tok == token => {
                    let deadline = *last_move + WAIT_DURATION;
                    ctx.set_handled();
                    if deadline > now {
                        let wait_for = deadline - now;
                        Some(TooltipState::Waiting {
                            last_move: *last_move,
                            timer_expire: deadline,
                            token: ctx.request_timer(wait_for),
                            window_pos: *window_pos,
                        })
                    } else {
                        let win_id = ctx.new_sub_window(
                            WindowConfig::default()
                                .show_titlebar(false)
                                .window_size_policy(WindowSizePolicy::Content)
                                .set_level(WindowLevel::Tooltip)
                                .set_position(
                                    ctx.window().get_position()
                                        + window_pos.to_vec2()
                                        + cursor_size.to_vec2(),
                                ),
                            Label::<()>::new(self.tip.clone()),
                            (),
                            env.clone(),
                        );
                        Some(TooltipState::Showing(win_id))
                    }
                }
                _ => None,
            },
            TooltipState::Showing(win_id) => {
                match event {
                    Event::MouseMove(me) if !ctx.is_hot() => {
                        ctx.submit_command(CLOSE_WINDOW.to(*win_id));
                        Some(TooltipState::Waiting {
                            last_move: now,
                            timer_expire: now + WAIT_DURATION,
                            token: ctx.request_timer(WAIT_DURATION),
                            window_pos: me.window_pos,
                        })
                    }
                    _ => None,
                }
            }
        };

        if let Some(state) = new_state {
            self.state = state;
        }

        if !ctx.is_handled() {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(false) = event {
            if let TooltipState::Showing(win_id) = self.state {
                ctx.submit_command(CLOSE_WINDOW.to(win_id));
            }
            self.state = TooltipState::Fresh;
        }
        child.lifecycle(ctx, event, data, env)
    }
}