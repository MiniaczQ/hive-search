use druid::kurbo::RoundedRect;
use druid::piet::d2d::Bitmap;
use druid::piet::InterpolationMode;
use druid::text::{Formatter, Selection, Validation, ValidationError};
use druid::widget::*;
use druid::*;

use serde::{Deserialize, Serialize};

use super::consts::*;
use super::section::titled_section;

#[derive(Serialize, Deserialize, Clone, Copy, Data, Lens, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<RGBA> for Color {
    fn from(c: RGBA) -> Self {
        Color::rgba8(c.r, c.g, c.b, c.a)
    }
}

impl From<Color> for RGBA {
    fn from(c: Color) -> Self {
        let c = c.as_rgba8();
        RGBA {
            r: c.0,
            g: c.1,
            b: c.2,
            a: c.3,
        }
    }
}

fn named_u8_textbox(text: impl Into<LabelText<u8>>) -> Flex<u8> {
    Flex::row()
        .with_child(
            Label::new(text)
                .align_horizontal(UnitPoint::CENTER)
                .fix_width(20.),
        )
        .with_child(
            TextBox::with_formatter(TextBox::new(), U8Formatter)
                .update_data_while_editing(true)
                .fix_width(60.),
        )
}

fn rgba_input_column() -> Flex<RGBA> {
    Flex::column()
        .with_child(named_u8_textbox("R").lens(RGBA::r))
        .with_spacer(2.)
        .with_child(named_u8_textbox("G").lens(RGBA::g))
        .with_spacer(2.)
        .with_child(named_u8_textbox("B").lens(RGBA::b))
        .with_spacer(2.)
        .with_child(named_u8_textbox("A").lens(RGBA::a))
}

fn color_preview_box() -> Painter<RGBA> {
    Painter::new(|ctx, data: &RGBA, _env| {
        let color: Color = (*data).into();
        let size = ctx.size();
        let rect = size.to_rect();
        let rounded_rect = size.to_rounded_rect(SECTION_RADIUS);

        let bitmap = generate_checkerboard(ctx, 16);
        let max = rect.width().max(rect.height());
        let src_rect = Rect::new(0., 0., rect.width() * 16. / max, rect.height() * 16. / max);
        ctx.draw_image_area(&bitmap, src_rect, rect, InterpolationMode::NearestNeighbor);

        ctx.fill(rounded_rect, &color);
        let rounded_rect2 = RoundedRect::new(
            rect.x0 - BORDER_WIDTH,
            rect.y0 - BORDER_WIDTH,
            rect.x1 + BORDER_WIDTH,
            rect.y1 + BORDER_WIDTH,
            SECTION_RADIUS,
        );
        ctx.stroke(rounded_rect2, &SUB_SECTION_PRIMARY_COLOR, BORDER_WIDTH * 2.);
        ctx.stroke(rounded_rect, &BORDER_COLOR, BORDER_WIDTH);
    })
}

fn generate_checkerboard(ctx: &mut PaintCtx, size: usize) -> Bitmap {
    let mut s = false;
    let mut v: Vec<u8> = Vec::new();
    for _ in 0..size {
        for _ in 0..size {
            match s {
                true => {
                    for _ in 0..3 {
                        v.push(255)
                    }
                }
                false => {
                    for _ in 0..3 {
                        v.push(110)
                    }
                }
            }
            s = !s;
        }
        s = !s;
    }
    ctx.make_image(size, size, v.as_slice(), piet::ImageFormat::Rgb)
        .unwrap()
}

pub fn color_picker(text: impl Into<LabelText<RGBA>>) -> Flex<RGBA> {
    titled_section(
        text,
        SUB_SECTION_DECAL,
        Flex::row()
            .with_child(rgba_input_column())
            .with_spacer(5.)
            .with_flex_child(color_preview_box(), 1.)
            .fix_height(120.),
    )
}

struct U8Formatter;

impl Formatter<u8> for U8Formatter {
    fn format(&self, value: &u8) -> String {
        format!("{}", value)
    }

    fn validate_partial_input(&self, input: &str, _sel: &text::Selection) -> text::Validation {
        if let Ok(n) = input.parse::<i128>() {
            let n = n.clamp(0, 255);
            return Validation::success().change_text(n.to_string());
        }

        Validation::success()
            .change_text("0".to_owned())
            .change_selection(Selection::caret(1))
    }

    fn value(&self, input: &str) -> Result<u8, text::ValidationError> {
        input.parse::<u8>().map_err(|e| ValidationError::new(e))
    }
}
