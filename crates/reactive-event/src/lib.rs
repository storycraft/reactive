#![no_std]

#[doc(hidden)]
pub mod __private;
mod iter;
mod macros;

use core::{
    cell::{RefCell, UnsafeCell},
    pin::Pin,
};

use hkt_pin_list::{ForLt, LinkedList, Node, hkt::ForLt};
use iter::Iter;
use pin_project::pin_project;

#[derive(derive_more::Debug)]
#[pin_project]
pub struct EventTarget<Hkt: ForLt> {
    guard: RefCell<()>,
    #[debug(skip)]
    #[pin]
    list: LinkedList<ForLt!(for<'a> UnsafeCell<Hkt::Of<'a>>)>,
}

impl<Hkt: ForLt> EventTarget<Hkt> {
    pub const fn new() -> Self {
        Self {
            guard: RefCell::new(()),
            list: LinkedList::new(),
        }
    }

    pub fn bind(self: Pin<&Self>, listener: Pin<&Listener<Hkt::Of<'_>>>) {
        let this = self.project_ref();
        let node = listener.project_ref().node;
        this.list.push_front(node);
    }
}

impl<Hkt: ForLt> EventTarget<Hkt> {
    pub fn iter<R>(&self, f: impl FnOnce(iter::Iter<Hkt>) -> R) -> R {
        let _guard = self.guard.borrow_mut();
        self.list.iter(|iter| f(Iter { iter }))
    }

    pub fn emit_empty(&self)
    where
        for<'a> Hkt::Of<'a>: FnMut() -> bool,
    {
        self.iter(|iter| {
            for f in iter {
                if !f() {
                    break;
                }
            }
        });
    }

    pub fn emit<T>(&self, value: T)
    where
        for<'a> Hkt::Of<'a>: FnMut(T) -> bool,
        T: Copy,
    {
        self.iter(|iter| {
            for f in iter {
                if !f(value) {
                    break;
                }
            }
        });
    }

    pub fn emit_ref<T>(&self, value: &T)
    where
        for<'a> Hkt::Of<'a>: FnMut(&T) -> bool,
    {
        self.iter(|iter| {
            for f in iter {
                if !f(value) {
                    break;
                }
            }
        });
    }

    pub fn emit_mut<T>(&self, value: &mut T)
    where
        for<'a> Hkt::Of<'a>: FnMut(&mut T) -> bool,
    {
        self.iter(|iter| {
            for f in iter {
                if !f(value) {
                    break;
                }
            }
        });
    }
}

#[derive(Debug)]
#[pin_project]
pub struct Listener<F: ?Sized, Dyn: ?Sized = F> {
    #[pin]
    node: Node<UnsafeCell<F>, UnsafeCell<Dyn>>,
}

impl<F: ?Sized, Dyn: ?Sized> Listener<F, Dyn> {
    pub fn new(f: F) -> Self
    where
        F: Sized,
    {
        Self {
            // TODO:: Fix safety
            node: unsafe { Node::new_unchecked(UnsafeCell::new(f)) },
        }
    }

    pub fn unbind(self: Pin<&Self>) {
        self.node.unlink();
    }
}
