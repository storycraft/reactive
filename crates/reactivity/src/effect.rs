pub mod handle;

use core::{marker::PhantomData, pin::Pin, ptr::NonNull};

use handle::EffectHandle;
use pin_project::pin_project;

use crate::binding::Binding;

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
            // SAFETY: f is valid mutable reference outliving EffectHandle
            handle: unsafe { EffectHandle::new(NonNull::new_unchecked(f as *mut _ as *mut _)) },
            _ph: PhantomData,
        }
    }

    pub fn init<'b>(self: Pin<&mut Self>, bindings: impl IntoIterator<Item = Pin<&'b Binding>>) {
        let handle = self.project().handle.into_ref();
        handle.init(bindings);
        handle.call_f();
    }
}
