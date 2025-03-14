use skia_safe::Canvas;

use crate::screen::ScreenRect;

#[derive(Debug)]
pub struct DrawContext {
    pub canvas: Canvas,
    pub screen: ScreenRect,
}

impl DrawContext {
    #[inline]
    pub const fn new(screen: ScreenRect, canvas: Canvas) -> Self {
        Self { canvas, screen }
    }
}
