use druid::*;
use druid::widget::*;

fn bg_highlight<T>(
    ctx: &mut PaintCtx,
    _: &T,
    _env: &Env,
) {
    let bounds = ctx.size().to_rect();

    ctx.fill(bounds, &Color::rgb8(0x90, 0x90, 0x90));

    if ctx.is_hot() {
        ctx.stroke(bounds.inset(-0.5), &Color::WHITE, 1.0);
    }

    if ctx.is_active() {
        ctx.fill(bounds, &Color::rgb8(0x71, 0x71, 0x71));
    }
}

pub fn new_button<T: Data>(
    text: &str
) -> Container<T> {
    Label::new(text)
        .align_horizontal(UnitPoint::CENTER)
        .background(Painter::new(bg_highlight::<T>))
}

pub fn new_label<T: Data>(
    text: &str
) -> Container<T> {
    Label::new(text)
        .align_horizontal(UnitPoint::CENTER)
        .background(Color::rgb8(0x90, 0x90, 0xFF))
}

