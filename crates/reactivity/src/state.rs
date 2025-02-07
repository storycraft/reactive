use core::{
    cell::{Cell, Ref, RefCell},
    fmt::Debug,
    pin::Pin,
};

use pin_project::pin_project;

use crate::{effect::binding::Binding, tracker::DependencyTracker};

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
        self.project_ref().tracker.notify();
        self.set_untracked(value);
    }

    pub fn set_untracked(self: Pin<&Self>, value: T) {
        self.value.set(value);
    }
}

impl<T: Default> StateCell<T> {
    #[inline]
    pub fn take_get(self: Pin<&Self>, binding: Pin<&Binding>) -> T {
        self.project_ref().tracker.register(binding);
        self.value.take()
    }

    #[inline]
    pub fn take(self: Pin<&Self>) -> T {
        self.project_ref().tracker.notify();
        self.value.take()
    }
}

impl<T: Copy> StateCell<T> {
    #[inline]
    pub fn get(self: Pin<&Self>, binding: Pin<&Binding>) -> T {
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
        self.project_ref().tracker.notify();
        self.set_untracked(value);
    }

    pub fn set_untracked(self: Pin<&Self>, value: T) {
        *self.value.borrow_mut() = value;
    }

    pub fn get(self: Pin<&Self>, binding: Pin<&Binding>) -> Ref<'_, T> {
        let this = self.project_ref();
        this.tracker.register(binding);
        this.value.borrow()
    }

    pub fn get_untracked(&self) -> Ref<'_, T> {
        self.value.borrow()
    }
}
