pub(crate) mod components;

use core::{
    future::{pending, Future},
    pin::{pin, Pin},
    task::{Context, Waker},
};

use components::ComponentKey;
use never_say_never::Never;
use reactivity::{list::List, queue::Queue};
use waker_fn::waker_fn;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::Component;

struct WinitApp<'a, Fut> {
    waker: Waker,
    queue: Pin<&'a mut Queue>,
    components: Pin<&'a List<ComponentKey>>,
    fut: Pin<&'a mut Fut>,
}

impl<Fut> WinitApp<'_, Fut>
where
    Fut: Future<Output = Never>,
{
    fn poll(&mut self) {
        ComponentKey::set(self.components.as_ref(), || {
            let _ = self
                .queue
                .as_mut()
                .poll(self.fut.as_mut(), &mut Context::from_waker(&self.waker));
        });
    }
}

impl<Fut> ApplicationHandler for WinitApp<'_, Fut>
where
    Fut: Future<Output = Never>,
{
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.queue.as_ref().set(|| {
            for entry in self.components.iter() {
                entry.value().component().resumed(el);
            }
        });
    }

    fn window_event(&mut self, el: &ActiveEventLoop, window_id: WindowId, mut event: WindowEvent) {
        self.queue.as_ref().set(|| {
            for entry in self.components.iter() {
                entry
                    .value()
                    .component()
                    .on_window_event(el, window_id, &mut event);
            }
        });
    }

    fn suspended(&mut self, el: &ActiveEventLoop) {
        self.queue.as_ref().set(|| {
            for entry in self.components.iter() {
                entry.value().component().suspended(el);
            }
        });
    }

    fn user_event(&mut self, _: &ActiveEventLoop, _: ()) {
        self.poll();
    }
}

pub async fn render<T: for<'a> Component<'a>>(component: Pin<&T>) -> ! {
    ComponentKey::register(component).await
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
    let components = components.into_ref();

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
