pub(crate) mod context;
pub mod handler;

use core::{
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll, Waker},
};
use std::rc::Rc;

use context::{AppShared, EventLoopStatus};
use waker_fn::waker_fn;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopBuilder},
    window::WindowId,
};

struct App<'a, Fut> {
    waker: Waker,
    cx: Pin<Rc<AppShared>>,
    fut: Pin<&'a mut Fut>,
}

impl<Fut> App<'_, Fut>
where
    Fut: Future<Output = ()>,
{
    fn poll(&mut self, el: &ActiveEventLoop) -> Poll<Fut::Output> {
        let cx = self.cx.as_ref();
        context::set(cx, el, || {
            if self
                .fut
                .as_mut()
                .poll(&mut Context::from_waker(&self.waker))
                .is_ready()
            {
                return Poll::Ready(());
            }

            cx.queue().run(&self.waker);
            Poll::Pending
        })
    }
}

impl<Fut> ApplicationHandler for App<'_, Fut>
where
    Fut: Future<Output = ()>,
{
    fn resumed(&mut self, el: &ActiveEventLoop) {
        let cx = self.cx.as_ref();
        if cx.status() == EventLoopStatus::Suspended {
            cx.set_status(EventLoopStatus::Resumed);
        }

        context::set(cx, el, || {
            cx.handlers().iter(|mut iter| {
                while let Some(entry) = iter.next() {
                    entry.value().resumed(el);
                }
            });
        });
    }

    fn window_event(&mut self, el: &ActiveEventLoop, window_id: WindowId, mut event: WindowEvent) {
        let cx = self.cx.as_ref();

        context::set(cx, el, || {
            cx.handlers().iter(|mut iter| {
                while let Some(entry) = iter.next() {
                    let handler = entry.value();
                    if handler
                        .window_id()
                        .map(|id| id == window_id)
                        .unwrap_or(false)
                    {
                        handler.on_window_event(el, &mut event);
                    }
                }
            });
        });
    }

    fn suspended(&mut self, el: &ActiveEventLoop) {
        let cx = self.cx.as_ref();

        if cx.status() == EventLoopStatus::Resumed {
            cx.set_status(EventLoopStatus::Suspended);
        }

        context::set(cx, el, || {
            cx.handlers().iter(|mut iter| {
                while let Some(entry) = iter.next() {
                    entry.value().suspended(el);
                }
            });
        });
    }

    fn user_event(&mut self, el: &ActiveEventLoop, _: ()) {
        if self.poll(el).is_ready() {
            el.exit();
        }
    }
}

/// Run winit [`EventLoop`] and reactivity system
pub fn run<Fut: Future<Output = ()>>(mut el: EventLoopBuilder<()>, fut: Fut) {
    // TODO:: error handling
    let el = el.build().unwrap();
    let proxy = el.create_proxy();

    // Poll task on start
    let _ = proxy.send_event(());

    let waker = waker_fn(move || {
        let _ = proxy.send_event(());
    });

    let cx = Rc::pin(AppShared::new(Some(waker.clone())));

    let fut_cx = cx.clone();
    let fut = pin!(fut_cx.executor().run(fut));

    let mut app = App { cx, waker, fut };

    // TODO:: error handling
    el.run_app(&mut app).unwrap();
}
