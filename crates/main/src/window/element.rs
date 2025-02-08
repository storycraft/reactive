use core::{
    future::{pending, Future},
    pin::Pin,
};

use never_say_never::Never;
use skia_safe::Canvas;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

pub trait Element<'a> {
    fn setup(self: Pin<&'a Self>) -> impl Future<Output = Never> + 'a
    where
        Self: Sized,
    {
        pending()
    }

    fn on_event(
        self: Pin<&Self>,
        _el: &ActiveEventLoop,
        _window_id: WindowId,
        _event: &mut WindowEvent,
    ) {
    }

    fn draw(self: Pin<&Self>, _canvas: &Canvas) {}
}

impl Element<'_> for () {}
