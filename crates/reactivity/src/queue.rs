use core::{
    cell::Cell,
    pin::{pin, Pin},
    task::Waker,
};

use hkt_pin_list::{LinkedList, Node};
use pin_project::pin_project;

use crate::effect::EffectFn;

#[pin_project]
pub struct Queue {
    waker: Cell<Option<Waker>>,
    #[pin]
    updates: LinkedList!(for<'a> dyn EffectFn + 'a),
}

impl Queue {
    pub fn new(waker: Option<Waker>) -> Self {
        Self {
            waker: Cell::new(waker),
            updates: LinkedList::new(),
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
                    entry.value_pinned().call();
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

    pub(crate) fn add(self: Pin<&Self>, node: Pin<&Node<dyn EffectFn>>) {
        self.project_ref().updates.push_front(node);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}
