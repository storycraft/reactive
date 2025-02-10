use core::{
    cell::Cell,
    pin::{pin, Pin},
    task::Waker,
};

use pin_project::pin_project;

use crate::{
    effect::EffectFnPtr, list::{Entry, List}
};

#[pin_project]
pub struct Queue {
    waker: Cell<Option<Waker>>,
    #[pin]
    updates: List<EffectFnPtr>,
}

impl Queue {
    pub fn new(waker: Option<Waker>) -> Self {
        Self {
            waker: Cell::new(waker),
            updates: List::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    pub fn run(self: Pin<&Self>, waker: &Waker) {
        let this = self.project_ref();

        while let Some(entry) = this.updates.iter().next() {
            entry.unlink();
            entry.value().call();
        }

        if let Some(current) = self.waker.take() {
            if current.will_wake(waker) {
                self.waker.set(Some(current));
                return;
            }
        }

        self.waker.set(Some(waker.clone()));
    }

    pub(crate) fn add(self: Pin<&Self>, entry: &Entry<EffectFnPtr>) {
        self.project_ref().updates.push_front(entry);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}
