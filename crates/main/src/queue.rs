use core::{
    cell::Cell,
    future::{poll_fn, Future},
    marker::PhantomData,
    pin::{pin, Pin},
    task::Waker,
};

use pin_project::pin_project;
use scoped_tls_hkt::scoped_thread_local;

use crate::{
    effect::handle::{run_effect_handle, Handle},
    list::{Entry, List},
};

scoped_thread_local!(static QUEUE: for<'a> Pin<&'a Queue>);

#[pin_project]
pub(crate) struct Queue {
    waker: Cell<Option<Waker>>,
    #[pin]
    list: List<Handle>,
    _ph: PhantomData<*mut ()>,
}

impl Queue {
    pub(crate) async fn run<F: Future>(app: F) -> F::Output {
        let mut queue = pin!(Self {
            waker: Cell::new(None),
            list: List::new(),
            _ph: PhantomData,
        });

        let mut app = pin!(app);
        poll_fn(|cx| {
            'a: {
                let waker = queue.as_mut().project().waker.get_mut();
                if let Some(waker) = waker {
                    if waker.will_wake(cx.waker()) {
                        break 'a;
                    }
                }

                *waker = Some(cx.waker().clone());
            }

            let queue = queue.as_ref();
            QUEUE.set(queue, || {
                let list = queue.project_ref().list;
                while let Some(entry) = list.iter().next() {
                    entry.unlink();
                    run_effect_handle(entry);
                }

                app.as_mut().poll(cx)
            })
        })
        .await
    }

    pub(crate) fn add(self: Pin<&Self>, entry: &Entry<Handle>) {
        self.project_ref().list.push_front(entry);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    pub fn with(f: impl FnOnce(Pin<&Self>)) {
        if QUEUE.is_set() {
            QUEUE.with(f)
        }
    }
}
