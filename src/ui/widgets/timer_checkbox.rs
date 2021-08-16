use druid::commands::{CLOSE_WINDOW};
use druid::widget::*;
use druid::*;

pub struct TimerCheckbox {
    checkbox: Checkbox,
    window: Option<WindowId>,
}

impl TimerCheckbox {
    pub fn new(text: impl Into<LabelText<bool>>) -> Self {
        Self {
            checkbox: Checkbox::new(text),
            window: None,
        }
    }
}

impl Widget<bool> for TimerCheckbox {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut bool, env: &Env) {
        self.checkbox.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &bool, env: &Env) {
        self.checkbox.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &bool, data: &bool, env: &Env) {
        self.checkbox.update(ctx, old_data, data, env);
        match (data, self.window) {
            (true, None) => {
                self.window = Some(
                    ctx.new_sub_window(
                        WindowConfig::default()
                            .window_size_policy(WindowSizePolicy::Content)
                            .set_level(WindowLevel::Modal)
                            .set_topmost(true),
                        Label::new("00:00.000"),
                        (),
                        env.clone(),
                    ),
                );
            }
            (false, Some(window)) => {
                ctx.submit_command(CLOSE_WINDOW.to(window));
                self.window = None;
            }
            _ => {}
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &bool, env: &Env) -> Size {
        self.checkbox.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &bool, env: &Env) {
        self.checkbox.paint(ctx, data, env);
    }
}
