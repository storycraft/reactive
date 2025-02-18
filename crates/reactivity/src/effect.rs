use core::{
    array,
    cell::{Cell, UnsafeCell},
    pin::{pin, Pin},
    ptr::{self, NonNull},
};

use pin_project::{pin_project, pinned_drop};
use pinned_aliasable::Aliasable;

use hkt_pin_list::Node;

#[derive(Debug)]
#[pin_project(PinnedDrop)]
pub struct Effect<const BINDINGS: usize, F> {
    #[pin]
    to_queue: Node<EffectFnPtr>,
    #[pin]
    inner: Aliasable<EffectFn<BINDINGS, F>>,
}

impl<const BINDINGS: usize, F> Effect<BINDINGS, F>
where
    F: FnMut(Pin<&BindingArray<BINDINGS>>),
{
    pub fn new(f: F) -> Self {
        Self {
            to_queue: Node::new(EffectFnPtr::null()),
            inner: Aliasable::new(EffectFn {
                bindings: BindingArray::new(),
                f: UnsafeCell::new(f),
            }),
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        let mut this = self.project();
        // aliasable pin projection
        let inner = unsafe { Pin::new_unchecked(this.inner.as_ref().get()) };

        // Initialize node to queue
        this.to_queue.set(Node::new(EffectFnPtr::new(inner)));

        // Initialize bindings
        let node = this.to_queue.as_ref();
        for binding in inner.project_ref().bindings.iter() {
            binding.to_tracker().value().0.set(NonNull::from(&*node));
        }

        inner.call();
    }
}

// Although looks unnecessary, this drop implementation is a must.
// Lenient drop check allow closures to be invalid before dropping entire effect. Leading to undefined behavior!
// See https://doc.rust-lang.org/nomicon/dropck.html
#[pinned_drop]
impl<const BINDINGS: usize, F> PinnedDrop for Effect<BINDINGS, F> {
    fn drop(self: Pin<&mut Self>) {}
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
struct EffectFn<const BINDINGS: usize, F> {
    #[pin]
    bindings: BindingArray<BINDINGS>,
    #[pin]
    f: UnsafeCell<F>,
}

impl<const BINDINGS: usize, F> EffectFn<BINDINGS, F>
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

#[derive(Debug, Clone, Copy)]
/// Self referential pinned Effect fn pointer
pub(super) struct EffectFnPtr {
    call: unsafe fn(*const ()),
    ptr: *const (),
}

impl EffectFnPtr {
    fn new<const BINDINGS: usize, F>(effect: Pin<&EffectFn<BINDINGS, F>>) -> Self
    where
        F: FnMut(Pin<&BindingArray<BINDINGS>>),
    {
        unsafe fn call<const BINDINGS: usize, F>(this: *const ())
        where
            F: FnMut(Pin<&BindingArray<BINDINGS>>),
        {
            Pin::new_unchecked(&*this.cast::<EffectFn<BINDINGS, F>>()).call();
        }

        Self {
            call: call::<BINDINGS, F>,
            ptr: &raw const *effect as *const (),
        }
    }

    fn null() -> Self {
        Self {
            call: |_| {},
            ptr: ptr::null(),
        }
    }

    #[inline(always)]
    pub(super) fn call(self) {
        unsafe {
            (self.call)(self.ptr);
        }
    }
}

type EffectFnPtrNode = Node<EffectFnPtr>;

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct TrackerBinding(Cell<NonNull<EffectFnPtrNode>>);

impl TrackerBinding {
    pub const fn new() -> Self {
        Self(Cell::new(NonNull::dangling()))
    }

    pub fn get(&self) -> NonNull<EffectFnPtrNode> {
        self.0.get()
    }
}
