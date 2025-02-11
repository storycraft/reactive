use core::{
    future::Future,
    pin::{pin, Pin},
};

use reactivity::list::Node;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use super::context::AppCx;

pub trait WinitWindow {
    fn window_id(self: Pin<&Self>) -> Option<WindowId>;
    fn request_redraw(self: Pin<&Self>);

    // TODO::
    fn resumed(self: Pin<&Self>, _el: &ActiveEventLoop) {}
    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {}

    fn on_window_event(self: Pin<&Self>, _el: &ActiveEventLoop, _event: &mut WindowEvent) {}
}

pub async fn add<Fut: Future>(this: Pin<&impl WinitWindow>, fut: Fut) -> Fut::Output {
    let node = pin!(Node::new(this as Pin<&dyn WinitWindow>));
    AppCx::with(|cx| {
        cx.as_ref().handlers().push_front(node.into_ref().entry());
    });

    fut.await
}
