use core::{
    future::{pending, Future},
    pin::Pin,
};

use skia_safe::Canvas;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

pub mod ui;

pub trait Element {
    fn on_event(self: Pin<&Self>, _el: &ActiveEventLoop, _event: &mut WindowEvent) {}

    fn draw(self: Pin<&Self>, _canvas: &Canvas) {}
}

pub trait SetupFn<'a> {
    type Output;

    fn setup(self: Pin<&'a Self>) -> impl Future<Output = Self::Output> + 'a {
        pending()
    }
}
