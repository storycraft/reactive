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
    bindings: List<EffectFnPtrSlot>,
    #[pin]
    to_queue: Node<EffectFn>,
}

impl EffectHandle {
    /// # Safety
    /// Pointer to `f` must be valid to dereference *mutable* until [`EffectHandle`] drops.
    pub unsafe fn new(f: NonNull<dyn FnMut()>) -> Self {
        Self {
            bindings: List::new(),
            to_queue: Node::new(EffectFn(f)),
        }
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

        this.to_queue.entry().value().call();
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct EffectFn(NonNull<dyn FnMut()>);

impl EffectFn {
    pub fn call(&self) {
        let mut ptr = self.0;
        unsafe { ptr.as_mut()() }
    }
}

type EffectFnPtrEntry = Entry<EffectFn>;

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct EffectFnPtrSlot(Cell<NonNull<EffectFnPtrEntry>>);

impl EffectFnPtrSlot {
    pub const fn new(inner: NonNull<EffectFnPtrEntry>) -> Self {
        Self(Cell::new(inner))
    }
}

pub trait EffectFnPtrExt {
    fn to_queue(&self) -> Option<&Entry<EffectFn>>;
}

impl EffectFnPtrExt for Entry<EffectFnPtrSlot> {
    fn to_queue(&self) -> Option<&Entry<EffectFn>> {
        if self.linked() {
            // SAFETY: unless the entry is unlinked pointer is valid
            Some(unsafe { self.value().0.get().as_ref() })
        } else {
            None
        }
    }
}
