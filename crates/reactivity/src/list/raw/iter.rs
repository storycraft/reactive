use core::marker::PhantomData;

use super::{EntryPtr, RawList};

#[derive(Debug)]
pub struct Iter<'a> {
    next: Option<EntryPtr>,
    _ph: PhantomData<&'a ()>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = EntryPtr;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            Some(next) => {
                self.next = unsafe { next.link().next.get() };
                Some(next)
            }

            None => None,
        }
    }
}

impl<'a> IntoIterator for &'a RawList {
    type Item = EntryPtr;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            next: self.start(),
            _ph: PhantomData,
        }
    }
}
