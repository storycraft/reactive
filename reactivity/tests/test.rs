use core::{
    cell::Cell,
    future::{Future, poll_fn},
    pin::{Pin, pin},
};

use pin_project::pin_project;
use reactivity::{effect::Binding, let_effect, queue::Queue, tracker::DependencyTracker};
use scoped_tls_hkt::scoped_thread_local;

scoped_thread_local!(static QUEUE: for<'a> Pin<&'a Queue>);

fn run_system<Fut: Future>(fut: Fut) -> Fut::Output {
    let queue = pin!(Queue::new(None));
    let queue = queue.as_ref();
    let mut fut = pin!(fut);

    pollster::block_on(poll_fn(|cx| {
        QUEUE.set(queue, || {
            let v = fut.as_mut().poll(cx);
            queue.run(cx.waker());
            v
        })
    }))
}

#[derive(derive_more::Debug)]
#[pin_project]
pub struct State<T: ?Sized> {
    #[pin]
    tracker: DependencyTracker,
    #[debug(skip)]
    value: Cell<T>,
}

impl<T> State<T> {
    pub fn new(value: T) -> Self {
        Self {
            tracker: DependencyTracker::new(),
            value: Cell::new(value),
        }
    }

    #[inline]
    pub fn set(self: Pin<&Self>, value: T) {
        self.value.set(value);
        QUEUE.with(|queue| self.project_ref().tracker.notify(queue));
    }
}

impl<T: Copy> State<T> {
    #[inline]
    pub fn get(self: Pin<&Self>, binding: Binding) -> T {
        self.project_ref().tracker.register(binding);
        self.value.get()
    }
}

#[test]
fn effects() {
    run_system(async {
        let a = pin!(State::new(2));
        let a = a.into_ref();
        let b = pin!(State::new(0));
        let b = b.into_ref();

        let_effect!(b.set(a.get($) + 1));

        a.set(50);
    });
}
