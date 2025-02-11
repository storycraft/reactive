pub mod iter;
mod ptr;
pub mod raw;

use core::{
    fmt::Debug,
    pin::{pin, Pin},
};

use pin_project::{pin_project, pinned_drop};
use pinned_aliasable::Aliasable;
use raw::Link;

#[macro_export]
/// Define a new Safe wrapper around [`raw::RawList`]
macro_rules! define_safe_list {
    ($vis:vis $name:ident = for<$($lt:lifetime),*> $ty:ty) => {
        #[derive(Debug)]
        #[$crate::__private::pin_project::pin_project]
        #[repr(transparent)]
        $vis struct $name {
            #[pin]
            raw: $crate::list::raw::RawList,
        }

        #[allow(unused)]
        impl $name {
            /// Create a new List
            pub fn new() -> Self {
                Self {
                    raw: $crate::list::raw::RawList::new(),
                }
            }

            /// Check if list is empty
            pub fn is_empty(&self) -> bool {
                self.raw.is_empty()
            }

            /// Link a entry to start
            pub fn push_front<$($lt),*>(
                self: ::core::pin::Pin<&Self>,
                entry: &$crate::list::Entry<$ty>
            ) {
                unsafe {
                    self.project_ref().raw.push_front(entry);
                }
            }

            /// Traverse list from the start until `f` returns true
            pub fn iter<R>(
                &self,
                f: impl for<$($lt),*> ::core::ops::FnOnce(
                    $crate::list::iter::Iter<'_, $ty>
                ) -> R
            ) -> R
            {
                // SAFETY: hide unbound lifetimes in higher kinded closure
                f(unsafe { $crate::list::iter::Iter::from(self.raw.iter()) })
            }

            pub fn take<R>(
                self: ::core::pin::Pin<&Self>,
                f: impl FnOnce(::core::pin::Pin<&Self>) -> R
            ) -> R {
                // SAFETY: casting transparent struct
                    self.project_ref().raw.take(
                        move |inner| f(
                            unsafe {
                                *(&inner as *const _ as *const ::core::pin::Pin<&Self>)
                            }
                        )
                    )
            }

            /// Clear the list
            pub fn clear(&self) {
                self.raw.clear();
            }
        }

        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    ($vis:vis $name:ident = $ty:ty) => {
        $crate::define_safe_list!($vis $name = for<> $ty);
    };
}

#[derive(Debug)]
#[pin_project(PinnedDrop)]
#[repr(C)]
pub struct Entry<T> {
    link: Link,
    #[pin]
    value: T,
}

impl<T> Entry<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_pinned(&self) -> Pin<&T> {
        // SAFETY: Reference to Entry is obtained only from pinned Node
        unsafe { Pin::new_unchecked(self) }.project_ref().value
    }

    pub fn linked(&self) -> bool {
        self.link.linked()
    }

    pub fn unlink(&self) {
        self.link.unlink();
    }
}

#[pinned_drop]
impl<T> PinnedDrop for Entry<T> {
    fn drop(self: Pin<&mut Self>) {
        // unlink from list before drop
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
                link: Link::new(),
                value,
            }),
        }
    }

    #[inline]
    pub fn entry(self: Pin<&Self>) -> &Entry<T> {
        self.project_ref().inner.get()
    }
}

#[cfg(test)]
mod tests {
    use core::pin::pin;

    use crate::{define_safe_list, list::Entry};

    use super::Node;

    #[test]
    fn test() {
        define_safe_list!(List = &mut i32);

        let mut list = pin!(List::new());
        let list2 = pin!(List::new());
        let list2 = list2.into_ref();

        let mut a = 1234;
        let mut b = 5678;

        let node1 = pin!(Node::new(&mut a));
        let node2 = pin!(Node::new(&mut b));
        let entry1 = node1.into_ref().entry();
        let entry2 = node2.into_ref().entry();
        list.as_ref().push_front(entry2);
        list.as_ref().push_front(entry1);

        let list = list.as_mut();

        list2.push_front(entry1);
        entry1.unlink();
        list.as_ref().push_front(entry1);

        list.as_ref().take(|list| {
            list.iter(|mut iter| {
                assert_eq!(iter.next().map(Entry::value), Some(&&mut 1234));
                let _a = entry1;
                let _b = entry2;
                assert_eq!(iter.next().map(Entry::value), Some(&&mut 5678));
                assert_eq!(iter.next().map(Entry::value), None);
            });
        });
    }
}
