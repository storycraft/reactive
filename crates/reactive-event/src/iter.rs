use core::cell::UnsafeCell;

use hkt_pin_list::Iter as ListIter;

use crate::hkt::ForLt;

pub struct Iter<'a, Hkt: ForLt> {
    pub(super) iter: ListIter<'a, UnsafeCell<Hkt::Of<'static>>>,
}

impl<'a, Hkt: ForLt + 'a> Iterator for Iter<'a, Hkt> {
    type Item = &'a mut Hkt::Of<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;

        Some(unsafe { &mut *next.value().get() })
    }
}
