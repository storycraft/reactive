use core::{
    future::{pending, Future},
    mem,
    pin::{pin, Pin},
    ptr::NonNull,
    task::{Context, Waker},
};

use never_say_never::Never;
use reactivity::{
    list::{List, Node},
    queue::Queue,
};
use scoped_tls_hkt::scoped_thread_local;
use waker_fn::waker_fn;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::Component;

scoped_thread_local!(static COMPONENTS: for<'a> Pin<&'a List<NonNull<dyn for<'b> Component<'b>>>>);

struct WinitApp<'a, Fut> {
    waker: Waker,
    queue: Pin<&'a mut Queue>,
    components: Pin<&'a List<NonNull<dyn for<'b> Component<'b>>>>,
    fut: Pin<&'a mut Fut>,
}

impl<'a, Fut> WinitApp<'a, Fut>
where
    Fut: Future<Output = Never>,
{
    fn poll(&mut self) {
        COMPONENTS.set(self.components.as_ref(), || {
            let _ = self
                .queue
                .as_mut()
                .poll(self.fut.as_mut(), &mut Context::from_waker(&self.waker));
        });
    }
}

impl<'a, Fut> ApplicationHandler for WinitApp<'a, Fut>
where
    Fut: Future<Output = Never>,
{
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.queue.as_ref().set(|| {
            for entry in self.components.iter() {
                let component = unsafe { Pin::new_unchecked(entry.value().as_ref()) };
                component.resumed(el);
            }
        });
    }

    fn window_event(&mut self, el: &ActiveEventLoop, window_id: WindowId, mut event: WindowEvent) {
        self.queue.as_ref().set(|| {
            for entry in self.components.iter() {
                let component = unsafe { Pin::new_unchecked(entry.value().as_ref()) };
                component.on_event(el, window_id, &mut event);
            }
        });
    }

    fn suspended(&mut self, el: &ActiveEventLoop) {
        self.queue.as_ref().set(|| {
            for entry in self.components.iter() {
                let component = unsafe { Pin::new_unchecked(entry.value().as_ref()) };
                component.suspended(el);
            }
        });
    }

    fn user_event(&mut self, _: &ActiveEventLoop, _: ()) {
        self.poll();
    }
}

pub async fn render<T: for<'a> Component<'a>>(component: Pin<&T>) -> ! {
    let ptr = unsafe {
        mem::transmute::<NonNull<dyn Component>, NonNull<dyn for<'a> Component<'a>>>(NonNull::from(
            &*component,
        ))
    };
    let node = pin!(Node::new(ptr));

    if COMPONENTS.is_set() {
        COMPONENTS.with(|list| {
            list.push_front(node.as_ref().entry());
        })
    }

    component.setup().await
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

    let components = pin!(List::new());
    let components = components.as_ref();

    let mut app = WinitApp {
        waker,
        queue: pin!(Queue::new()),
        components,
        fut: pin!(async {
            fut.await;
            pending().await
        }),
    };
    app.poll();

    // TODO:: error handling
    el.run_app(&mut app).unwrap();
}
