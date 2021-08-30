use druid::piet::Text;
use druid::theme::*;
use druid::widget::*;
use druid::*;

use crate::resources::MC_FONT;

use super::data::{AppData, State};

use super::layouts::{client::client, config::config, host::host};
use super::widgets::consts::*;

pub fn hive() -> impl Widget<AppData> {
    ResourceLoader::new().background(Painter::new(|ctx, _data, _env| {
        let rect = ctx.size().to_rect();
        ctx.fill(rect, &BG_COLOR);
    }))
}

/// Changes UI based on the application state.
fn switcher() -> ViewSwitcher<AppData, State> {
    ViewSwitcher::new(
        |data: &AppData, _env| data.state,
        |selector, _data, _env| match selector {
            &State::Config => Box::new(config()),
            &State::Host => Box::new(host()),
            &State::Client => Box::new(client()),
        },
    )
}

struct ResourceLoader {
    inner: SizedBox<AppData>,
}

impl ResourceLoader {
    pub fn new() -> Self {
        Self {
            inner: SizedBox::empty().height(0.).width(0.),
        }
    }
}

pub const MC_FONT_KEY: Key<FontDescriptor> = Key::new("mc-font");

impl Widget<AppData> for ResourceLoader {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppData, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let mc_font = ctx.text().load_font(MC_FONT).unwrap();
            self.inner = SizedBox::new(switcher().env_scope(move |env, _data| {
                env.set(
                    MC_FONT_KEY,
                    FontDescriptor::new(mc_font.clone()).with_weight(FontWeight::NORMAL),
                );
                // Default font
                env.set(
                    UI_FONT,
                    FontDescriptor::new(mc_font.clone())
                        .with_weight(FontWeight::NORMAL)
                        .with_size(15.),
                );
                // Textbox stuff
                env.set(TEXTBOX_INSETS, Insets::uniform_xy(4., 4.));
                env.set(TEXTBOX_BORDER_RADIUS, 5.);
                // ???
                env.set(WINDOW_BACKGROUND_COLOR, BG_COLOR);
                // Text box and checkbox background
                env.set(BACKGROUND_LIGHT, INPUT_BG_COLOR);
                env.set(BACKGROUND_DARK, INPUT_BG_COLOR);
                // Border
                env.set(BORDER_DARK, SUB_SECTION_SECONDARY_COLOR);
                env.set(BORDER_LIGHT, SUB_SECTION_SECONDARY_COLOR);
            }));
        }
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, env: &Env) {
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &AppData,
        env: &Env,
    ) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}
