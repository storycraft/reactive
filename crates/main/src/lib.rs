pub mod children;
pub mod element;
pub mod event;
mod event_loop;
pub mod resource;
pub mod state;
pub mod window;

use children::NoChild;
use core::{
    future::{pending, Future},
    pin::Pin,
};
use never_say_never::Never;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

pub use event_loop::{render, run};

pub trait Component<'a> {
    fn setup(self: Pin<&'a Self>) -> impl Future<Output = Never> + 'a
    where
        Self: Sized,
    {
        NoChild
    }

    fn resumed(self: Pin<&Self>, _el: &ActiveEventLoop) {}
    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {}

    fn on_window_event(
        self: Pin<&Self>,
        _el: &ActiveEventLoop,
        _window_id: WindowId,
        _event: &mut WindowEvent,
    ) {
    }
}

impl<'a, F: Fn() -> Fut, Fut: Future + 'a> Component<'a> for F {
    async fn setup(self: Pin<&'a Self>) -> Never {
        self().await;
        pending().await
    }
}

impl<'a> Component<'a> for () {
    fn setup(self: Pin<&'a Self>) -> impl Future<Output = Never> + 'a {
        pending()
    }
}
