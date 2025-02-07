use core::pin::Pin;

use skia_safe::Surface;

pub trait Drawable {
    fn draw(self: Pin<&Self>, surface: &Surface);
}
