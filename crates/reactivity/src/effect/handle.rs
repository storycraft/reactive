use core::{cell::Cell, pin::Pin, ptr::NonNull};

use derive_more::{Deref, DerefMut};
use pin_project::pin_project;
use scoped_tls_hkt::scoped_thread_local;

use crate::list::{Entry, List};

#[derive(Debug)]
#[pin_project]
pub struct Handle {
    #[pin]
    pub(super) list: List<Cell<HandleEntryPtr>>,
    pub(super) f: NonNull<dyn FnMut()>,
}

impl Handle {
    pub fn list(self: Pin<&Self>) -> Pin<&List<Cell<HandleEntryPtr>>> {
        self.project_ref().list
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Deref, DerefMut)]
pub(crate) struct HandleEntryPtr(Option<NonNull<Entry<Handle>>>);

impl HandleEntryPtr {
    pub const fn new(inner: Option<NonNull<Entry<Handle>>>) -> Self {
        Self(inner)
    }
}

impl Default for HandleEntryPtr {
    fn default() -> Self {
        Self::new(None)
    }
}

scoped_thread_local!(static CURRENT: Entry<Handle>);

pub fn with_handle(f: impl FnOnce(&Entry<Handle>)) {
    if CURRENT.is_set() {
        CURRENT.with(f);
    }
}

pub fn run_effect_handle(entry: &Entry<Handle>) {
    let mut f = entry.value().f;
    CURRENT.set(entry, unsafe { f.as_mut() });
}
