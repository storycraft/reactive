use core::{
    array,
    cell::{Cell, UnsafeCell},
    future::{pending, Future},
    pin::{pin, Pin},
    ptr::NonNull,
    task::Context,
};

use noop_waker::noop_waker;
use pin_project::pin_project;
use pinned_aliasable::Aliasable;

use crate::list::{Entry, Node};

pub trait Effect {
    /// Initialize effect
    fn init(self: Pin<&mut Self>);
}

#[derive(Debug)]
#[pin_project]
/// Connection to dependency tracker from a effect
pub struct Binding {
    /// Node connected to dependency tracker
    #[pin]
    to_tracker: Node<TrackerBinding>,
}

impl Binding {
    pub(crate) fn new() -> Self {
        Self {
            to_tracker: Node::new(TrackerBinding::new()),
        }
    }

    /// Entry connecting to dependency tracker
    pub(crate) fn to_tracker(self: Pin<&Self>) -> &Entry<TrackerBinding> {
        self.project_ref().to_tracker.entry()
    }
}

impl Default for Binding {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct BindingArray<const SIZE: usize> {
    inner: [Binding; SIZE],
}

impl<const SIZE: usize> BindingArray<SIZE> {
    fn new() -> Self {
        Self {
            inner: array::from_fn(|_| Binding::new()),
        }
    }

    pub fn inner(&self) -> &[Binding; SIZE] {
        &self.inner
    }

    #[inline]
    pub fn get_const<const INDEX: usize>(self: Pin<&Self>) -> Pin<&Binding> {
        // SAFETY: perform structural pinning
        unsafe { Pin::new_unchecked(&self.get_ref().inner[INDEX]) }
    }

    pub fn iter(self: Pin<&Self>) -> impl Iterator<Item = Pin<&Binding>> {
        // SAFETY: perform structural pinning
        self.get_ref()
            .inner
            .iter()
            .map(|binding| unsafe { Pin::new_unchecked(binding) })
    }
}

#[must_use]
pub fn effect<const BINDINGS: usize, R>(
    mut f: impl FnMut(Pin<&BindingArray<BINDINGS>>) -> R,
) -> impl Effect {
    #[pin_project]
    struct ImplEffect<Fut> {
        #[pin]
        fut: Fut,
    }

    impl<Fut> Effect for ImplEffect<Fut>
    where
        Fut: Future,
    {
        fn init(self: Pin<&mut Self>) {
            let _ = self
                .project()
                .fut
                .poll(&mut Context::from_waker(&noop_waker()));
        }
    }

    ImplEffect {
        fut: async move {
            let bindings = pin!(BindingArray::<BINDINGS>::new());
            let bindings = bindings.into_ref();

            let f = pin!(Aliasable::new(UnsafeCell::new(|| {
                let _ = f(bindings);
            })));
            let f = f.into_ref().get().get() as *mut dyn FnMut();

            let to_queue = pin!(Node::new(EffectFnPtr(f)));
            let to_queue = to_queue.into_ref();

            for binding in bindings.iter() {
                binding
                    .to_tracker()
                    .value()
                    .0
                    .set(NonNull::from(to_queue.entry()));
            }
            to_queue.entry().value().call();

            // Freeze forever here
            pending::<()>().await;
        },
    }
}

#[repr(transparent)]
#[derive(Debug)]
/// Self contained Effect fn pointer
pub(crate) struct EffectFnPtr(*mut dyn FnMut());

impl EffectFnPtr {
    pub fn call(&self) {
        // SAFETY: pointer is always valid since entry is self referential with pointee
        unsafe { (*self.0)() }
    }
}

type EffectFnPtrEntry = Entry<EffectFnPtr>;

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct TrackerBinding(Cell<NonNull<EffectFnPtrEntry>>);

impl TrackerBinding {
    pub const fn new() -> Self {
        Self(Cell::new(NonNull::dangling()))
    }

    pub fn get(&self) -> NonNull<EffectFnPtrEntry> {
        self.0.get()
    }
}
