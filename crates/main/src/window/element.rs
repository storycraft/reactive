pub mod ui;

use core::pin::Pin;

use skia_safe::Canvas;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

pub trait Element {
    fn on_event(self: Pin<&Self>, _el: &ActiveEventLoop, _event: &mut WindowEvent) {}

    fn draw(self: Pin<&Self>, _canvas: &Canvas) {}
}
