use core::marker::PhantomData;

use super::{raw, Entry};

#[derive(Debug)]
pub struct Iter<'a, T> {
    inner: raw::iter::Iter<'a>,
    _ph: PhantomData<T>,
}

impl<'a, T> Iter<'a, T> {
    #[doc(hidden)]
    pub unsafe fn from(raw: raw::iter::Iter<'a>) -> Self {
        Self {
            inner: raw,
            _ph: PhantomData,
        }
    }

    pub fn next(&mut self) -> Option<&Entry<T>> {
        Some(unsafe { self.inner.next()?.get_extended_ref() })
    }
}
