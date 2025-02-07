use core::{cell::Cell, pin::Pin, ptr::NonNull};

use derive_more::Deref;
use pin_project::pin_project;

use crate::{
    binding::Binding,
    list::{Entry, List, Node},
};

#[derive(Debug)]
#[pin_project]
pub struct EffectHandle {
    #[pin]
    pub(super) bindings: List<EffectFnPtrSlot>,
    #[pin]
    pub(super) to_queue: Node<NonNull<dyn FnMut()>>,
}

impl EffectHandle {
    pub fn f(self: Pin<&Self>) -> NonNull<dyn FnMut()> {
        *self.project_ref().to_queue.entry().value()
    }

    pub fn init<'a>(self: Pin<&Self>, bindings: impl IntoIterator<Item = Pin<&'a Binding>>) {
        let this = self.project_ref();
        let entry_ptr = NonNull::from(this.to_queue.entry());

        for binding in bindings {
            let to_handle = binding.to_handle();
            to_handle.value().set(entry_ptr);

            this.bindings.push_front(to_handle);
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Deref)]
pub(crate) struct EffectFnPtrSlot(Cell<NonNull<Entry<NonNull<dyn FnMut()>>>>);

impl EffectFnPtrSlot {
    pub const fn new(inner: NonNull<Entry<NonNull<dyn FnMut()>>>) -> Self {
        Self(Cell::new(inner))
    }
}
