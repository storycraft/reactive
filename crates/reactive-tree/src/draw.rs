use skia_safe::Canvas;

use crate::screen::ScreenRect;

#[derive(Debug)]
pub struct DrawContext {
    screen: ScreenRect,
    canvas: Canvas,
}

impl DrawContext {
    pub const fn new(screen: ScreenRect, canvas: Canvas) -> Self {
        Self { screen, canvas }
    }
}
