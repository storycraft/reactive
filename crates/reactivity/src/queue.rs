use core::{
    cell::Cell,
    pin::{pin, Pin},
    task::Waker,
};

use hkt_pin_list::{define_safe_list, Node};
use pin_project::pin_project;

use crate::effect::EffectFnPtr;

define_safe_list!(EffectFnList = EffectFnPtr);

#[pin_project]
pub struct Queue {
    waker: Cell<Option<Waker>>,
    #[pin]
    updates: EffectFnList,
}

impl Queue {
    pub fn new(waker: Option<Waker>) -> Self {
        Self {
            waker: Cell::new(waker),
            updates: EffectFnList::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    pub fn run(&self, waker: &Waker) {
        loop {
            if !self.updates.iter(|mut iter| {
                if let Some(entry) = iter.next() {
                    entry.unlink();
                    entry.value().call();
                    true
                } else {
                    false
                }
            }) {
                break;
            }
        }

        if let Some(current) = self.waker.take() {
            if current.will_wake(waker) {
                self.waker.set(Some(current));
                return;
            }
        }

        self.waker.set(Some(waker.clone()));
    }

    pub(crate) fn add(self: Pin<&Self>, node: Pin<&Node<EffectFnPtr>>) {
        self.project_ref().updates.push_front(node);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}
