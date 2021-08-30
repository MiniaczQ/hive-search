use druid::piet::*;
use druid::widget::*;
use druid::*;

use super::consts::*;

#[derive(Clone, Copy)]
pub struct SectionDecal {
    pub font_size: f64,
    pub primary_color: &'static Color,
    pub secondary_color: &'static Color,
}

/// Encase widget in a section
pub fn section<T: Data>(
    decal: SectionDecal,
    inner: impl Widget<T> + 'static
) -> Container<T> {
    inner.expand_width().padding(SECTION_PADDING).background(Painter::new(move |ctx, _, _env| {
        let rect = ctx.size().to_rounded_rect(SECTION_RADIUS);
        ctx.fill(rect, decal.primary_color);
    }))
}

/// Encase widget in a section with a title
pub fn titled_section<T: Data>(
    title: impl Into<LabelText<T>>,
    decal: SectionDecal,
    inner: impl Widget<T> + 'static,
) -> Flex<T> {
    Flex::column()
        .with_child(Label::new(title).with_text_size(decal.font_size)
            .align_horizontal(UnitPoint::CENTER)
            .padding(SECTION_PADDING)
            .background(Painter::new(move |ctx, _, _env| {
                let rect = ctx.size().to_rounded_rect(SECTION_RADIUS);
                ctx.fill(rect, decal.secondary_color);
            })).expand_width())
        .with_spacer(SECTION_PADDING)
        .with_child(section(decal, inner))
}
