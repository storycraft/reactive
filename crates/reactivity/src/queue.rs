use core::{
    cell::Cell,
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll, Waker},
};

use pin_project::pin_project;
use scoped_tls_hkt::scoped_thread_local;

use crate::{
    effect::handle::{run_effect_handle, Handle},
    list::{Entry, List},
};

scoped_thread_local!(static QUEUE: for<'a> Pin<&'a Queue>);

#[pin_project]
pub struct Queue {
    waker: Cell<Option<Waker>>,
    #[pin]
    list: List<Handle>,
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
            list: List::new(),
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

        let queue = self.as_ref();
        QUEUE.set(queue, || {
            let list = queue.project_ref().list;
            while let Some(entry) = list.iter().next() {
                entry.unlink();
                run_effect_handle(entry);
            }

            fut.poll(cx)
        })
    }

    pub(crate) fn add(self: Pin<&Self>, entry: &Entry<Handle>) {
        self.project_ref().list.push_front(entry);
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
