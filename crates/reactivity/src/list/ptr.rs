use core::ptr::NonNull;

use super::Entry;

#[repr(transparent)]
#[derive(Debug)]
pub struct EntryPtr<T: ?Sized>(NonNull<Entry<T>>);

impl<T: ?Sized> EntryPtr<T> {
    pub fn new(node: &Entry<T>) -> Self {
        Self(NonNull::from(node))
    }

    #[inline]
    pub unsafe fn as_ref(&self) -> &Entry<T> {
        self.as_extended_ref()
    }

    pub unsafe fn as_extended_ref<'a>(self) -> &'a Entry<T> {
        // Original reference was pinned
        self.0.as_ref()
    }
}

impl<T: ?Sized> Clone for EntryPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for EntryPtr<T> {}
