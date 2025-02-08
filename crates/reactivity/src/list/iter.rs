use core::marker::PhantomData;

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

impl<'a, T: ?Sized> IntoIterator for &'a List<T> {
    type Item = &'a Entry<T>;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            next: self.start(),
            _ph: PhantomData,
        }
    }
}
