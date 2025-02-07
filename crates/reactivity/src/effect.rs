pub mod binding;
pub(crate) mod handle;

use core::{marker::PhantomData, pin::Pin, ptr::NonNull};

use handle::{run_effect_handle, Handle};
use pin_project::pin_project;

use crate::list::{List, Node};

#[derive(Debug)]
#[pin_project]
pub struct Effect<'a> {
    #[pin]
    inner: Node<Handle>,
    _ph: PhantomData<&'a ()>,
}

impl<'a> Effect<'a> {
    pub fn new(f: &'a mut dyn FnMut()) -> Self {
        Self {
            inner: Node::new(Handle {
                list: List::new(),
                // `f` is exclusively borrowed during effect's lifetime.
                // It will never move or dropped before the effect holding is.
                f: NonNull::new(f as *mut _ as *mut (dyn FnMut() + 'static)).unwrap(),
            }),
            _ph: PhantomData,
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        run_effect_handle(self.project().inner.as_ref().entry());
    }
}
