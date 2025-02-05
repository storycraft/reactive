use core::{marker::PhantomData, pin::Pin};

use super::{Entry, EntryPtr, List};

#[derive(Debug)]
pub struct Iter<'a, T: ?Sized> {
    next: Option<EntryPtr<T>>,
    _ph: PhantomData<&'a Entry<T>>,
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a Entry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            Some(next) => {
                let entry = unsafe { next.as_extended_ref::<'a>() };
                self.next = entry.next.get();
                Some(entry)
            }

            None => None,
        }
    }
}

impl<'a, T: ?Sized> From<Pin<&'a List<T>>> for Iter<'a, T> {
    fn from(list: Pin<&'a List<T>>) -> Self {
        Self {
            next: list.project_ref().start.get().get(),
            _ph: PhantomData,
        }
    }
}
