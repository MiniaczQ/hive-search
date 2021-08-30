use druid::*;
use druid::widget::*;

use super::tooltip::TooltipController;

pub trait MyWidgetExt<T: Data>: Widget<T> + Sized + 'static {
    fn with_tooltip(self, tip: impl Into<String>) -> ControllerHost<Self, TooltipController> {
        self.controller(TooltipController::new(tip))
    }
}

impl<T: Data, W: Widget<T> + 'static> MyWidgetExt<T> for W {}