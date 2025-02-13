use core::{
    cell::{self, Cell, RefCell},
    fmt::Debug,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use pin_project::pin_project;

use reactivity::{effect::Binding, tracker::DependencyTracker};

use crate::event_loop::context;

#[derive(derive_more::Debug)]
#[pin_project]
pub struct StateCell<T: ?Sized> {
    #[pin]
    tracker: DependencyTracker,
    #[debug(skip)]
    value: Cell<T>,
}

impl<T> StateCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            tracker: DependencyTracker::new(),
            value: Cell::new(value),
        }
    }

    #[inline]
    pub fn set(self: Pin<&Self>, value: T) {
        self.set_untracked(value);
        notify(self.project_ref().tracker);
    }

    pub fn set_untracked(self: Pin<&Self>, value: T) {
        self.value.set(value);
    }
}

impl<T: Default> StateCell<T> {
    #[inline]
    pub fn take_get(self: Pin<&Self>, binding: Binding) -> T {
        self.project_ref().tracker.register(binding);
        self.take_get_untracked()
    }

    #[inline]
    pub fn take_get_untracked(self: Pin<&Self>) -> T {
        self.value.take()
    }

    #[inline]
    pub fn take(self: Pin<&Self>) -> T {
        notify(self.project_ref().tracker);
        self.value.take()
    }

    pub fn update(self: Pin<&Self>, f: impl FnOnce(T) -> T) {
        self.value.set(f(self.value.take()));
        notify(self.project_ref().tracker);
    }
}

impl<T: Copy> StateCell<T> {
    #[inline]
    pub fn get(self: Pin<&Self>, binding: Binding) -> T {
        self.project_ref().tracker.register(binding);
        self.get_untracked()
    }

    pub fn get_untracked(self: Pin<&Self>) -> T {
        self.value.get()
    }
}

#[derive(Debug)]
#[pin_project]
pub struct StateRefCell<T: ?Sized> {
    #[pin]
    tracker: DependencyTracker,
    value: RefCell<T>,
}

impl<T> StateRefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            tracker: DependencyTracker::new(),
            value: RefCell::new(value),
        }
    }

    #[inline]
    pub fn set(self: Pin<&Self>, value: T) {
        notify(self.project_ref().tracker);
        self.set_untracked(value);
    }

    pub fn set_untracked(self: Pin<&Self>, value: T) {
        *self.value.borrow_mut() = value;
    }

    pub fn get(self: Pin<&Self>, binding: Binding) -> cell::Ref<'_, T> {
        let this = self.project_ref();
        this.tracker.register(binding);
        this.value.borrow()
    }

    pub fn get_untracked(&self) -> cell::Ref<'_, T> {
        self.value.borrow()
    }

    pub fn get_mut(self: Pin<&Self>) -> Guard<'_, T> {
        let this = self.project_ref();
        Guard {
            tracker: this.tracker,
            inner: this.value.borrow_mut(),
        }
    }

    pub fn get_mut_untracked(&self) -> cell::RefMut<'_, T> {
        self.value.borrow_mut()
    }
}

#[derive(Debug)]
pub struct Guard<'a, T> {
    tracker: Pin<&'a DependencyTracker>,
    inner: cell::RefMut<'a, T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        notify(self.tracker);
    }
}

fn notify(tracker: Pin<&DependencyTracker>) {
    if context::is_set() {
        context::with(move |cx| {
            tracker.notify(cx.app.queue());
        });
    }
}
