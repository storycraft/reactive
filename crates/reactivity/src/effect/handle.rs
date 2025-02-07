use core::{cell::Cell, pin::Pin, ptr::NonNull};

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
            // This pointer is valid as long as linked
            to_handle.value().0.set(entry_ptr);

            this.bindings.push_front(to_handle);
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct EffectFnPtrSlot(Cell<NonNull<Entry<NonNull<dyn FnMut()>>>>);

impl EffectFnPtrSlot {
    pub const fn new(inner: NonNull<Entry<NonNull<dyn FnMut()>>>) -> Self {
        Self(Cell::new(inner))
    }
}

pub trait EffectFnPtrExt {
    fn to_queue(&self) -> Option<&Entry<NonNull<dyn FnMut()>>>;
}

impl EffectFnPtrExt for Entry<EffectFnPtrSlot> {
    fn to_queue(&self) -> Option<&Entry<NonNull<dyn FnMut()>>> {
        if self.linked() {
            // SAFETY: unless the entry is unlinked pointer is valid
            Some(unsafe { self.value().0.get().as_ref() })
        } else {
            None
        }
    }
}
