use core::ptr::{addr_of, NonNull};

use super::{Entry, Link};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EntryPtr(NonNull<Entry<()>>);

impl EntryPtr {
    pub fn new<T>(node: &Entry<T>) -> Self {
        Self(NonNull::from(node).cast())
    }

    pub unsafe fn get_extended_ref<'a, T>(self) -> &'a Entry<T> {
        self.0.cast().as_ref()
    }

    pub unsafe fn as_ref<T>(&self) -> &Entry<T> {
        self.0.cast().as_ref()
    }

    pub unsafe fn link(&self) -> &Link {
        &*addr_of!((*self.0.as_ptr()).link)
    }
}
