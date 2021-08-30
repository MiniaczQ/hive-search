use druid::text::{Formatter, Selection, Validation, ValidationError};
use druid::widget::*;
use druid::*;

use super::color_picker::color_picker;
use super::consts::*;
use super::my_widget_ext::MyWidgetExt;
use super::section::titled_section;
use super::timer::TimerData;
use super::timer_toggle::TimerToggle;

pub struct TimerConfig {
    inner: SizedBox<TimerData>,
}

impl TimerConfig {
    pub fn new() -> Self {
        Self {
            inner: titled_section(
                "Timer Configuration",
                MAIN_SECTION_DECAL,
                Flex::column()
                    .must_fill_main_axis(true)
                    .with_child(titled_section(
                        "General",
                        SUB_SECTION_DECAL,
                        TimerToggle::new(),
                    ))
                    .with_spacer(SECTION_PADDING)
                    .with_child(font_config())
                    .with_spacer(SECTION_PADDING)
                    .with_child(color_picker("Text color").lens(TimerData::color).with_tooltip("Color of the timer text."))
                    .with_spacer(SECTION_PADDING)
                    .with_child(color_picker("Background color").lens(TimerData::bg_color).with_tooltip("Color of the timer background.")),
            )
            .padding(SECTION_PADDING)
            .fix_width(500.),
        }
    }
}

fn font_config() -> impl Widget<TimerData> {
    titled_section(
        "Font",
        SUB_SECTION_DECAL,
        Flex::column()
            .with_child(
                Flex::row()
                    .with_child(Label::new("Font name").align_vertical(UnitPoint::CENTER).fix_width(150.))
                    .with_flex_child(
                        TextBox::new().lens(TimerData::font_family).expand_width(),
                        1.,
                    ).with_tooltip("Name of the font family."),
            )
            .with_spacer(SECTION_PADDING)
            .with_child(
                Flex::row()
                    .with_child(Label::new("Text size").align_vertical(UnitPoint::CENTER).fix_width(150.))
                    .with_flex_child(
                        TextBox::new()
                            .with_formatter(F64fromU64Formatter)
                            .update_data_while_editing(true)
                            .lens(TimerData::font_size)
                            .expand_width(),
                        1.,
                    ).with_tooltip("Size of the text in pixels."),
            )
            .with_spacer(SECTION_PADDING)
            .with_child(
                Flex::row()
                    .with_child(Label::new("Minimum width").align_vertical(UnitPoint::CENTER).fix_width(150.))
                    .with_flex_child(
                        TextBox::new()
                            .with_formatter(F64fromU64Formatter)
                            .update_data_while_editing(true)
                            .lens(TimerData::min_width)
                            .expand_width(),
                        1.,
                    ).with_tooltip("Minimum width of the timer.\nUseful when the font doesn't have constant width."),
            )
            .with_spacer(SECTION_PADDING)
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(Checkbox::new("Bold").lens(TimerData::font_bold).with_tooltip("Whether the text should be bold."))
                    .with_child(Checkbox::new("Italic").lens(TimerData::font_italic).with_tooltip("Whether the text should be italic.")),
            ),
    )
}

impl Widget<TimerData> for TimerConfig {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut TimerData, env: &Env) {
        self.inner.event(ctx, event, data, env);
        match event {
            Event::WindowConnected => {
                *data = TimerData::load();
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
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &TimerData, data: &TimerData, env: &Env) {
        self.inner.update(ctx, old_data, data, env);
        if !old_data.eq(data) {
            data.save();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &TimerData,
        env: &Env,
    ) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &TimerData, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}

struct F64fromU64Formatter;

impl Formatter<f64> for F64fromU64Formatter {
    fn format(&self, value: &f64) -> String {
        format!("{}", value.trunc())
    }

    fn validate_partial_input(&self, input: &str, _sel: &text::Selection) -> text::Validation {
        if let Ok(n) = input.parse::<u64>() {
            return Validation::success().change_text(format!("{}", n.min(400)));
        }

        Validation::success()
            .change_text("0".to_owned())
            .change_selection(Selection::caret(1))
    }

    fn value(&self, input: &str) -> Result<f64, text::ValidationError> {
        input
            .parse::<u64>()
            .map_err(|e| ValidationError::new(e))
            .map(|v| v as f64)
    }
}
