use core::{
    future::Future,
    pin::{pin, Pin},
};

use hkt_pin_list::Node;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use super::context::{self, EventLoopStatus};

pub trait WinitWindow {
    fn window_id(self: Pin<&Self>) -> Option<WindowId>;

    // TODO::
    fn resumed(self: Pin<&Self>, _el: &ActiveEventLoop) {}
    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {}

    fn on_window_event(self: Pin<&Self>, _el: &ActiveEventLoop, _event: &mut WindowEvent) {}
}

pub async fn add<Fut: Future>(this: Pin<&impl WinitWindow>, fut: Fut) -> Fut::Output {
    let node = pin!(Node::new(this as Pin<&dyn WinitWindow>));
    context::with(|cx| {
        cx.app.handlers().push_front(node.into_ref().entry());

        if cx.app.status() == EventLoopStatus::Resumed {
            this.resumed(cx.el);
        }
    });

    fut.await
}
