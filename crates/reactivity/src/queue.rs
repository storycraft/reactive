use core::{
    cell::Cell,
    pin::{pin, Pin},
    task::Waker,
};

use pin_project::pin_project;

use crate::{
    effect::handle::EffectFn,
    list::{Entry, List},
};

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

    pub fn update_waker(mut self: Pin<&mut Self>, waker: &Waker) {
        let current = self.as_mut().project().waker.get_mut();
        if let Some(current) = current {
            if current.will_wake(waker) {
                return;
            }
        }

        *current = Some(waker.clone());
    }

    pub fn run(self: Pin<&Self>, waker: &Waker) {
        let this = self.project_ref();

        'a: {
            if let Some(current) = this.waker.take() {
                if current.will_wake(waker) {
                    this.waker.set(Some(current));
                    break 'a;
                }
            }

            this.waker.set(Some(waker.clone()));
        }

        let updates = this.updates;
        while let Some(entry) = updates.iter().next() {
            entry.unlink();
            entry.value().call();
        }
    }

    pub(crate) fn add(self: Pin<&Self>, entry: &Entry<EffectFn>) {
        self.project_ref().updates.push_front(entry);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}
