pub mod context;
pub mod handler;

use core::{
    future::{pending, Future},
    pin::{pin, Pin},
    task::{Context, Waker},
};
use std::rc::Rc;

use context::AppCx;
use never_say_never::Never;
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
    Fut: Future<Output = Never>,
{
    fn poll(&mut self) {
        AppCx::set(&self.cx, || {
            let _ = self
                .fut
                .as_mut()
                .poll(&mut Context::from_waker(&self.waker));

            let cx = self.cx.as_ref();
            let queue = cx.queue();
            if !queue.is_empty() {
                queue.run();

                for handler in cx.handlers().iter() {
                    handler.value().with(|handler| handler.request_redraw());
                }
            }
            queue.update_waker(&self.waker);
        });
    }
}

impl<Fut> ApplicationHandler for App<'_, Fut>
where
    Fut: Future<Output = Never>,
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
                entry
                    .value()
                    .with(|handler| handler.on_window_event(el, window_id, &mut event));
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

    fn user_event(&mut self, _: &ActiveEventLoop, _: ()) {
        self.poll();
    }
}

pub fn run<Fut: Future>(fut: Fut) {
    // TODO:: error handling
    let el = EventLoop::<()>::with_user_event().build().unwrap();

    let waker = waker_fn({
        let proxy = el.create_proxy();
        move || {
            let _ = proxy.send_event(());
        }
    });

    let cx = Rc::pin(AppCx::new(Some(waker.clone())));
    let fut = pin!({
        let cx = cx.clone();
        async move {
            cx.executor().run(fut).await;
            pending().await
        }
    });

    let mut app = App { cx, waker, fut };
    app.poll();

    // TODO:: error handling
    el.run_app(&mut app).unwrap();
}
