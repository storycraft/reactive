use nalgebra::Affine3;
use skia_safe::Canvas;

use crate::screen::ScreenRect;

#[derive(Debug)]
pub struct DrawContext<'a> {
    pub canvas: &'a mut Canvas,
    pub screen: ScreenRect,
    pub matrix: Affine3<f32>,
}

impl<'a> DrawContext<'a> {
    pub fn new(screen: ScreenRect, canvas: &'a mut Canvas) -> Self {
        Self {
            canvas,
            screen,
            matrix: Affine3::identity(),
        }
    }

    #[inline]
    pub fn sub<'b>(&'b mut self, matrix: &Affine3<f32>) -> DrawContext<'b> {
        DrawContext {
            canvas: self.canvas,
            screen: self.screen.clone(),
            matrix: self.matrix * matrix
        }
    }
}
