use core::{
    array,
    cell::{Cell, UnsafeCell},
    pin::{pin, Pin},
    ptr::NonNull,
};

use pin_project::pin_project;
use pinned_aliasable::Aliasable;

use crate::list::{Entry, Node};

#[derive(Debug)]
#[pin_project]
pub struct Effect<const BINDINGS: usize, F> {
    #[pin]
    to_queue: Node<EffectFnPtr>,
    #[pin]
    inner: Inner<BINDINGS, F>,
}

impl<const BINDINGS: usize, F> Effect<BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>),
{
    pub fn new(f: F) -> Self {
        Self {
            to_queue: Node::new(EffectFnPtr(NonNull::from(&mut ()))),
            inner: Inner {
                bindings: BindingArray::new(),
                f: Aliasable::new(UnsafeCell::new(f)),
            },
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        let mut this = self.project();
        let inner = this.inner.as_ref();

        // Initialize node to queue
        this.to_queue.set(Node::new(EffectFnPtr(
            NonNull::new(&*inner as *const dyn EffectFn as *mut dyn EffectFn).unwrap(),
        )));

        // Initialize bindings
        let entry = this.to_queue.as_ref().entry();
        for binding in inner.project_ref().bindings.iter() {
            binding.to_tracker().value().0.set(NonNull::from(entry));
        }

        inner.call();
    }
}

/// Pinned connection to dependency tracker from a effect
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Binding<'a>(Pin<&'a RawBinding>);

impl<'a> Binding<'a> {
    /// Entry connecting to dependency tracker
    pub(crate) fn to_tracker(self) -> &'a Entry<TrackerBinding> {
        self.0.project_ref().to_tracker.entry()
    }
}

#[derive(Debug)]
pub struct BindingArray<const SIZE: usize> {
    inner: [RawBinding; SIZE],
}

impl<const SIZE: usize> BindingArray<SIZE> {
    fn new() -> Self {
        Self {
            inner: array::from_fn(|_| RawBinding {
                to_tracker: Node::new(TrackerBinding::new()),
            }),
        }
    }

    #[inline]
    pub fn get_const<const INDEX: usize>(self: Pin<&Self>) -> Binding {
        // SAFETY: perform structural pinning
        Binding(unsafe { Pin::new_unchecked(&self.get_ref().inner[INDEX]) })
    }

    fn iter(self: Pin<&Self>) -> impl Iterator<Item = Binding> {
        // SAFETY: perform structural pinning
        self.get_ref()
            .inner
            .iter()
            .map(|binding| Binding(unsafe { Pin::new_unchecked(binding) }))
    }
}

#[derive(Debug)]
#[pin_project]
struct Inner<const BINDINGS: usize, F> {
    #[pin]
    bindings: BindingArray<BINDINGS>,
    #[pin]
    f: Aliasable<UnsafeCell<F>>,
}

trait EffectFn {
    // Call effect
    fn call(self: Pin<&Self>);
}

impl<const BINDINGS: usize, F> EffectFn for Inner<BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>),
{
    fn call(self: Pin<&Self>) {
        let this = self.project_ref();
        // SAFETY: F is valid to dereference, no multiple references can be obtained
        unsafe {
            (*this.f.get().get())(this.bindings);
        }
    }
}

impl EffectFn for () {
    fn call(self: Pin<&Self>) {}
}

#[derive(Debug)]
#[pin_project]
/// Unpinned binding
struct RawBinding {
    /// Node connected to dependency tracker
    #[pin]
    to_tracker: Node<TrackerBinding>,
}

#[repr(transparent)]
#[derive(Debug)]
/// Self contained and pinned Effect fn pointer
pub(super) struct EffectFnPtr(NonNull<dyn EffectFn>);

impl EffectFnPtr {
    pub fn call(&self) {
        // SAFETY: pointer is always valid since entry is self referential with pointee
        unsafe { Pin::new_unchecked(self.0.as_ref()).call() }
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
