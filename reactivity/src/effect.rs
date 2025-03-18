use core::{
    array,
    cell::{Cell, UnsafeCell},
    pin::{Pin, pin},
    ptr::{self},
};

use pin_project::pin_project;

use hkt_pin_list::Node;

#[derive(Debug)]
#[pin_project]
pub struct Effect<'a, const BINDINGS: usize, F> {
    #[pin]
    to_queue: Node<Inner<BINDINGS, F>, dyn EffectFn + 'a>,
}

impl<'a, const BINDINGS: usize, F> Effect<'a, BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>) + 'a,
{
    pub fn new(f: F) -> Self
    where
        'a: 'static,
    {
        Self {
            to_queue: Node::new(Inner::new(f)),
        }
    }

    /// # Safety
    /// Borrowed values must remain valid even if [`Effect`] is leaked
    pub unsafe fn new_unchecked(f: F) -> Self {
        unsafe {
            Self {
                to_queue: Node::new_unchecked(Inner::new(f)),
            }
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        let this = self.project();
        let node = this.to_queue.as_ref();
        let inner = node.value_pinned();

        // Initialize bindings
        for binding in inner.project_ref().bindings.iter() {
            binding.to_tracker().value().0.set(&raw const *node as _);
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
    pub(crate) fn to_tracker(self) -> Pin<&'a Node<TrackerBinding>> {
        self.0.project_ref().to_tracker
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
    f: UnsafeCell<F>,
}

impl<const BINDINGS: usize, F> Inner<BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>),
{
    fn new(f: F) -> Self {
        Self {
            bindings: BindingArray::new(),
            f: UnsafeCell::new(f),
        }
    }
}

pub(super) trait EffectFn {
    fn call(self: Pin<&Self>);
}

impl EffectFn for () {
    fn call(self: Pin<&Self>) {}
}

impl<const BINDINGS: usize, F> EffectFn for Inner<BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>),
{
    fn call(self: Pin<&Self>) {
        let this = self.project_ref();
        // SAFETY: F is valid to dereference, no multiple references can be obtained
        unsafe {
            (*this.f.get())(this.bindings);
        }
    }
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
pub(crate) struct TrackerBinding(Cell<*const Node<dyn EffectFn>>);

impl TrackerBinding {
    const fn new() -> Self {
        Self(Cell::new(ptr::null::<Node<(), dyn EffectFn>>()))
    }

    pub unsafe fn as_ref(self: Pin<&Self>) -> Pin<&Node<dyn EffectFn>> {
        unsafe { Pin::new_unchecked(&*self.0.get()) }
    }
}
