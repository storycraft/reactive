use core::{
    array,
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    pin::{pin, Pin},
    ptr::{self},
};

use pin_project::pin_project;

use hkt_pin_list::Node;

#[derive(Debug)]
#[pin_project]
pub struct Effect<'a, const BINDINGS: usize, F> {
    #[pin]
    to_queue: Node<Inner<'a, BINDINGS, F>, dyn EffectFn + 'a>,
}

impl<'a, const BINDINGS: usize, F> Effect<'a, BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>) + 'a,
{
    pub fn new(f: F) -> Self {
        Self {
            to_queue: Node::new(Inner::new(f)),
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
struct Inner<'a, const BINDINGS: usize, F> {
    #[pin]
    bindings: BindingArray<BINDINGS>,
    #[pin]
    f: UnsafeCell<F>,
    _ph: PhantomData<&'a ()>,
}

impl<'a, const BINDINGS: usize, F> Inner<'a, BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>) + 'a,
{
    fn new(f: F) -> Self {
        Self {
            bindings: BindingArray::new(),
            f: UnsafeCell::new(f),
            _ph: PhantomData,
        }
    }
}

pub(super) trait EffectFn {
    fn call(self: Pin<&Self>);
}

impl EffectFn for () {
    fn call(self: Pin<&Self>) {}
}

impl<const BINDINGS: usize, F> EffectFn for Inner<'_, BINDINGS, F>
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

    pub fn get(&self) -> *const Node<dyn EffectFn> {
        self.0.get()
    }
}
