use core::pin::Pin;

use skia_safe::Surface;

pub trait Element {
    fn draw(self: Pin<&Self>, surface: &Surface);
}


