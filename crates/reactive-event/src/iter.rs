use core::cell::UnsafeCell;

use hkt_pin_list::{Iter as ListIter, hkt::ForLt};

pub struct Iter<'a, 'b, Hkt: ForLt> {
    pub(super) iter: ListIter<'a, UnsafeCell<Hkt::Of<'b>>>,
}

impl<'a, 'b, Hkt: ForLt + 'a> Iterator for Iter<'a, 'b, Hkt>
where
    <Hkt as ForLt>::Of<'b>: 'a,
{
    type Item = &'a mut Hkt::Of<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;

        Some(unsafe { &mut *(next.value().get() as *mut Hkt::Of<'b>) })
    }
}
