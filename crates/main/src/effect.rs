pub mod binding;
pub(crate) mod handle;

use core::{cell::UnsafeCell, mem, pin::Pin, ptr::NonNull};

use handle::{run_effect_handle, Handle};
use pin_project::pin_project;
use pinned_aliasable::Aliasable;

use crate::list::{List, Node};

#[derive(Debug)]
#[pin_project]
pub struct Effect<F> {
    #[pin]
    f: Aliasable<UnsafeCell<F>>,
    #[pin]
    inner: Node<Handle>,
}

impl<F: FnMut()> Effect<F> {
    pub fn new(f: F) -> Self {
        Self {
            f: Aliasable::new(UnsafeCell::new(f)),
            inner: Node::new(Handle {
                list: List::new(),
                f: NonNull::from(&|| {}),
            }),
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        let mut this = self.project();

        this.inner.set(Node::new(Handle {
            list: List::new(),
            f: unsafe {
                mem::transmute(
                    NonNull::new(this.f.as_ref().get().get() as *mut dyn FnMut()).unwrap(),
                )
            },
        }));

        run_effect_handle(this.inner.as_ref().entry());
    }
}
