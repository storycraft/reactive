pub mod context;
pub mod handler;

use core::{
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll, Waker},
};
use std::rc::Rc;

use context::AppCx;
use waker_fn::waker_fn;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

struct App<'a, Fut> {
    waker: Waker,
    cx: Pin<Rc<AppCx>>,
    fut: Pin<&'a mut Fut>,
}

impl<Fut> App<'_, Fut>
where
    Fut: Future<Output = ()>,
{
    fn poll(&mut self) -> Poll<Fut::Output> {
        AppCx::set(&self.cx, || {
            if self
                .fut
                .as_mut()
                .poll(&mut Context::from_waker(&self.waker))
                .is_ready()
            {
                return Poll::Ready(());
            }

            let cx = self.cx.as_ref();
            let queue = cx.queue();
            if !queue.is_empty() {
                for handler in cx.handlers().iter() {
                    handler.value().with(|handler| handler.request_redraw());
                }
            }
            queue.run(&self.waker);

            Poll::Pending
        })
    }
}

impl<Fut> ApplicationHandler for App<'_, Fut>
where
    Fut: Future<Output = ()>,
{
    fn resumed(&mut self, el: &ActiveEventLoop) {
        AppCx::set(&self.cx, || {
            for entry in self.cx.as_ref().handlers().iter() {
                entry.value().with(|handler| handler.resumed(el));
            }
        });
    }

    fn window_event(&mut self, el: &ActiveEventLoop, window_id: WindowId, mut event: WindowEvent) {
        AppCx::set(&self.cx, || {
            for entry in self.cx.as_ref().handlers().iter() {
                if entry.value().with(|handler| {
                    if handler
                        .window_id()
                        .map(|id| id == window_id)
                        .unwrap_or(false)
                    {
                        handler.on_window_event(el, &mut event);
                        true
                    } else {
                        false
                    }
                }) {
                    break;
                }
            }
        });
    }

    fn suspended(&mut self, el: &ActiveEventLoop) {
        AppCx::set(&self.cx, || {
            for entry in self.cx.as_ref().handlers().iter() {
                entry.value().with(|handler| handler.suspended(el));
            }
        });
    }

    fn user_event(&mut self, el: &ActiveEventLoop, _: ()) {
        if self.poll().is_ready() {
            el.exit();
        }
    }
}

pub fn run<Fut: Future<Output = ()>>(fut: Fut) {
    // TODO:: error handling
    let el = EventLoop::<()>::with_user_event().build().unwrap();

    let waker = waker_fn({
        let proxy = el.create_proxy();
        move || {
            let _ = proxy.send_event(());
        }
    });

    let cx = Rc::pin(AppCx::new(Some(waker.clone())));

    let fut_cx = cx.clone();
    let fut = pin!(fut_cx.executor().run(fut));

    let mut app = App { cx, waker, fut };
    if app.poll().is_ready() {
        return;
    }

    // TODO:: error handling
    el.run_app(&mut app).unwrap();
}
