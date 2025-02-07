use core::{
    cell::Cell,
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll, Waker},
};

use pin_project::pin_project;
use scoped_tls_hkt::scoped_thread_local;

use crate::{
    effect::handle::EffectFn,
    list::{Entry, List},
};

// TODO:: Use static fallback for single threaded no-std
scoped_thread_local!(static QUEUE: for<'a> Pin<&'a Queue>);

#[pin_project]
pub struct Queue {
    waker: Cell<Option<Waker>>,
    #[pin]
    updates: List<EffectFn>,
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}

impl Queue {
    pub fn new() -> Self {
        Self {
            waker: Cell::new(None),
            updates: List::new(),
        }
    }

    pub fn set<R>(self: Pin<&Self>, f: impl FnOnce() -> R) -> R {
        QUEUE.set(self, f)
    }

    pub fn poll<F: Future>(
        mut self: Pin<&mut Self>,
        fut: Pin<&mut F>,
        cx: &mut Context,
    ) -> Poll<F::Output> {
        'a: {
            let waker = self.as_mut().project().waker.get_mut();
            if let Some(waker) = waker {
                if waker.will_wake(cx.waker()) {
                    break 'a;
                }
            }

            *waker = Some(cx.waker().clone());
        }

        let queue = self.into_ref();
        queue.set(move || {
            let updates = queue.project_ref().updates;
            while let Some(entry) = updates.iter().next() {
                entry.unlink();
                entry.value().call();
            }

            fut.poll(cx)
        })
    }

    pub(crate) fn add(self: Pin<&Self>, entry: &Entry<EffectFn>) {
        self.project_ref().updates.push_front(entry);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    pub(crate) fn with(f: impl FnOnce(Pin<&Self>)) {
        if QUEUE.is_set() {
            QUEUE.with(f)
        }
    }
}
