use std::any::Any;

use druid::*;
use druid::widget::*;

pub struct DynLabel<T: Any, F: Fn(&T) -> String, D: Data> {
    inner: Label<D>,
    selector: Selector<T>,
    formatter: F,
}

impl<T: Any, F: Fn(&T) -> String, D: Data> DynLabel<T, F, D> {
    pub fn new(
        text: String,
        selector: Selector<T>,
        formatter: F,
    ) -> Self {
        DynLabel {
            inner: Label::new(text),
            selector,
            formatter,
        }
    }
}

impl<T: Any, F: Fn(&T) -> String, D: Data> Widget<D> for DynLabel<T, F, D> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _: &mut D, _env: &Env) {
        if let Event::Command(cmd) = event {
            let result = cmd.get(self.selector);
            if let Some(payload) = result {
                let new_text = (self.formatter)(payload);
                self.inner.set_text(new_text);
            }
        }
        ctx.request_update();
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &D, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &D, data: &D, env: &Env) {
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &D, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &D, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}