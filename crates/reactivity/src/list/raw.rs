pub mod iter;

use core::{
    cell::Cell,
    fmt::{self, Debug, Formatter},
    pin::{pin, Pin},
    ptr::NonNull,
};

use iter::Iter;
use pin_project::{pin_project, pinned_drop};
use pinned_aliasable::Aliasable;

use super::{ptr::EntryPtr, Entry};

#[pin_project(PinnedDrop)]
/// Raw intrusive hkt linked list
pub struct RawList {
    #[pin]
    start: Aliasable<Next>,
}

impl RawList {
    pub fn new() -> Self {
        Self {
            start: Aliasable::new(Next::new(None)),
        }
    }

    fn start(&self) -> Option<EntryPtr> {
        // SAFETY: start is always unique and None if self is not pinned
        unsafe { Pin::new_unchecked(&self.start) }.get().get()
    }

    pub fn is_empty(&self) -> bool {
        self.start().is_none()
    }

    /// # Safety
    /// Item type must be a same type except lifetimes.
    pub unsafe fn push_front<T>(self: Pin<&Self>, entry: &Entry<T>) {
        let start = self.project_ref().start.get();
        entry.unlink();
        entry.link.next.set(start.get());
        entry.link.parent.set(Some(NonNull::from(start)));

        if let Some(old) = start.replace(Some(EntryPtr::new(entry))) {
            // SAFETY: replace parent of linked start node
            unsafe { old.link() }
                .parent
                .set(Some(NonNull::from(&entry.link.next)));
        }
    }

    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    pub fn take<R>(self: Pin<&Self>, f: impl FnOnce(Pin<&Self>) -> R) -> R {
        let list = pin!(Self::new());
        let list = list.as_ref();

        if let Some(ptr) = self.project_ref().start.get().take() {
            let new_start = list.project_ref().start.get();
            new_start.set(Some(ptr));

            let parent = unsafe { &ptr.link().parent };
            parent.set(Some(NonNull::from(new_start)));
        }

        f(list)
    }

    pub fn clear(&self) {
        for entry in self.iter() {
            unsafe { entry.link() }.unlink();
        }
    }
}

impl Default for RawList {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for RawList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[pinned_drop]
impl PinnedDrop for RawList {
    fn drop(self: Pin<&mut Self>) {
        // Unlink all entries before dropping list
        self.into_ref().clear();
    }
}

#[repr(transparent)]
#[derive(Debug, derive_more::Deref)]
struct Next(Cell<Option<EntryPtr>>);

impl Next {
    pub const fn new(inner: Option<EntryPtr>) -> Self {
        Self(Cell::new(inner))
    }
}

#[repr(transparent)]
#[derive(Debug, derive_more::Deref)]
struct Parent(Cell<Option<NonNull<Next>>>);

impl Parent {
    pub const fn new(inner: Option<NonNull<Next>>) -> Self {
        Self(Cell::new(inner))
    }
}

#[derive(Debug)]
pub struct Link {
    next: Next,
    parent: Parent,
}

impl Link {
    pub fn new() -> Self {
        Link {
            next: Next::new(None),
            parent: Parent::new(None),
        }
    }

    pub fn linked(&self) -> bool {
        self.parent.get().is_some()
    }

    pub fn unlink(&self) {
        if let Some(parent) = self.parent.take() {
            let next = self.next.take();

            if let Some(ref next) = next {
                // SAFETY: pointer is valid as long as linked
                unsafe { next.link() }.parent.set(Some(parent));
            }

            // SAFETY: pointer is valid as long as linked
            unsafe { parent.as_ref() }.set(next);
        }
    }
}

impl Default for Link {
    fn default() -> Self {
        Self::new()
    }
}
