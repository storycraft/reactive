pub(crate) mod handle;

use core::{marker::PhantomData, pin::Pin, ptr::NonNull};

use handle::EffectHandle;
use pin_project::pin_project;

use crate::{
    binding::Binding,
    list::{List, Node},
};

#[derive(Debug)]
#[pin_project]
pub struct Effect<'a> {
    #[pin]
    handle: EffectHandle,
    _ph: PhantomData<&'a ()>,
}

impl<'a> Effect<'a> {
    pub fn new(f: &'a mut dyn FnMut()) -> Self {
        Self {
            handle: EffectHandle {
                bindings: List::new(),
                // `f` is exclusively borrowed during effect's lifetime.
                // It will never move or dropped before the effect holding is.
                to_queue: Node::new(
                    NonNull::new(f as *mut _ as *mut (dyn FnMut() + 'static)).unwrap(),
                ),
            },
            _ph: PhantomData,
        }
    }

    pub fn init<'b>(self: Pin<&mut Self>, bindings: impl IntoIterator<Item = Pin<&'b Binding>>) {
        let handle = self.project().handle.into_ref();
        handle.init(bindings);

        // SAFETY: Safe to deref mut due to a constraint in constructor
        (unsafe { handle.f().as_mut() })();
    }
}
