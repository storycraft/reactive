use core::pin::Pin;

use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

pub trait EventHandler {
    fn window_id(self: Pin<&Self>) -> Option<WindowId>;
    fn request_redraw(self: Pin<&Self>);

    // TODO::
    fn resumed(self: Pin<&Self>, _el: &ActiveEventLoop) {}
    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {}

    fn on_window_event(self: Pin<&Self>, _el: &ActiveEventLoop, _event: &mut WindowEvent) {}
}
