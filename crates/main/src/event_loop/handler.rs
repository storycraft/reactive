use core::{
    future::{pending, Future},
    pin::{pin, Pin},
};

use reactivity::list::Node;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use super::context::AppCx;

pub trait EventHandler {
    fn request_redraw(self: Pin<&Self>);

    fn resumed(self: Pin<&Self>, _el: &ActiveEventLoop) {}
    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {}

    fn on_window_event(
        self: Pin<&Self>,
        _el: &ActiveEventLoop,
        _id: WindowId,
        _event: &mut WindowEvent,
    ) {
    }
}

pub struct HandlerKey {
    ptr: *const dyn EventHandler,
}

impl HandlerKey {
    pub fn with<R>(&self, f: impl FnOnce(Pin<&(dyn EventHandler)>) -> R) -> R {
        // SAFETY: Component is pinned and guaranteed won't drop before the Node drops
        f(unsafe { Pin::new_unchecked(&*self.ptr) })
    }

    pub async fn register<T, Fut>(handler: Pin<&T>, fut: Fut)
    where
        T: EventHandler,
        Fut: Future<Output = ()>,
    {
        if AppCx::is_set() {
            let node = pin!(Node::new(Self {
                ptr: &*handler as *const _ as *const _,
            }));

            AppCx::with(|cx| {
                cx.as_ref().handlers().push_front(node.into_ref().entry());
            });

            fut.await;
            pending::<()>().await;
        }
    }
}
