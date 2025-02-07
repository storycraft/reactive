pub mod iter;
mod ptr;

use core::{
    cell::Cell,
    fmt::{self, Debug, Formatter},
    pin::{pin, Pin},
    ptr::NonNull,
};

use iter::Iter;
use pin_project::{pin_project, pinned_drop};
use pinned_aliasable::Aliasable;
use ptr::EntryPtr;

#[pin_project(PinnedDrop)]
pub struct List<T: ?Sized> {
    #[pin]
    start: Aliasable<Next<T>>,
}

impl<T: ?Sized> List<T> {
    pub fn new() -> Self {
        Self {
            start: Aliasable::new(Next::new(None)),
        }
    }

    pub fn push_front(self: Pin<&Self>, entry: &Entry<T>) {
        let start = self.project_ref().start.get();
        entry.unlink();
        entry.next.set(start.get());
        entry.parent.set(Some(NonNull::from(start)));

        if let Some(old) = start.replace(Some(EntryPtr::new(entry))) {
            unsafe { old.as_ref() }
                .parent
                .set(Some(NonNull::from(&entry.next)));
        }
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

    pub fn take<R>(self: Pin<&Self>, f: impl FnOnce(Pin<&Self>) -> R) -> R {
        let list = pin!(Self::new());
        let list = list.as_ref();

        if let Some(ptr) = self.project_ref().start.get().take() {
            let new_start = list.project_ref().start.get();
            new_start.set(Some(ptr));
            unsafe { ptr.as_ref() }
                .parent
                .set(Some(NonNull::from(new_start)));
        }

        f(list)
    }

    pub fn clear(&self) {
        for entry in self.iter() {
            entry.unlink();
        }
    }
}

impl<T: ?Sized> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Debug> Debug for List<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[pinned_drop]
impl<T: ?Sized> PinnedDrop for List<T> {
    fn drop(self: Pin<&mut Self>) {
        self.as_ref().clear();
    }
}

#[pin_project(PinnedDrop)]
#[derive(Debug)]
pub struct Entry<T: ?Sized> {
    next: Next<T>,
    parent: Parent<T>,
    #[pin]
    value: T,
}

impl<T: ?Sized> Entry<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_pinned(&self) -> Pin<&T> {
        // Safety: Reference to Entry is obtained only from pinned Node
        unsafe { Pin::new_unchecked(self) }.project_ref().value
    }

    pub fn linked(&self) -> bool {
        self.parent.get().is_some()
    }

    pub fn unlink(&self) {
        if let Some(parent) = self.parent.take() {
            let next = self.next.take();

            if let Some(next) = next {
                unsafe { next.as_ref() }.parent.set(Some(parent));
            }
            unsafe { parent.as_ref() }.set(next);
        }
    }
}

#[pinned_drop]
impl<T: ?Sized> PinnedDrop for Entry<T> {
    fn drop(self: Pin<&mut Self>) {
        self.unlink();
    }
}

#[derive(derive_more::Debug)]
#[pin_project]
pub struct Node<T> {
    #[pin]
    #[debug(skip)]
    inner: Aliasable<Entry<T>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Aliasable::new(Entry {
                next: Next::new(None),
                parent: Parent::new(None),
                value,
            }),
        }
    }

    #[inline]
    pub fn entry(self: Pin<&Self>) -> &Entry<T> {
        self.project_ref().inner.get()
    }
}

#[repr(transparent)]
#[derive(Debug, derive_more::Deref)]
struct Next<T: ?Sized>(Cell<Option<EntryPtr<T>>>);

impl<T: ?Sized> Next<T> {
    pub const fn new(inner: Option<EntryPtr<T>>) -> Self {
        Self(Cell::new(inner))
    }
}

#[repr(transparent)]
#[derive(Debug, derive_more::Deref)]
struct Parent<T: ?Sized>(Cell<Option<NonNull<Next<T>>>>);

impl<T: ?Sized> Parent<T> {
    pub const fn new(inner: Option<NonNull<Next<T>>>) -> Self {
        Self(Cell::new(inner))
    }
}

#[cfg(test)]
mod tests {
    use core::pin::pin;

    use crate::list::Entry;

    use super::{List, Node};

    #[test]
    fn test() {
        let list = pin!(List::new());
        let list = list.as_ref();
        let list2 = pin!(List::new());
        let list2 = list2.as_ref();

        let node1 = pin!(Node::new(1234));
        let node2 = pin!(Node::new(5678));
        let entry1 = node1.as_ref().entry();
        let entry2 = node2.as_ref().entry();
        list.push_front(entry2);
        list.push_front(entry1);

        list2.push_front(entry1);
        entry1.unlink();
        list.push_front(entry1);

        list.take(|list| {
            let mut iter = list.iter();
            assert_eq!(iter.next().map(Entry::value), Some(&1234));
            let _a = entry1;
            let _b = entry2;
            assert_eq!(iter.next().map(Entry::value), Some(&5678));
            assert_eq!(iter.next().map(Entry::value), None);
        });
    }
}
