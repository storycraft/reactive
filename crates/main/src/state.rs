use core::{cell::Cell, pin::Pin};

use pin_project::pin_project;

use crate::{effect::binding::Binding, tracker::DependencyTracker};

#[pin_project]
pub struct StateCell<T: ?Sized> {
    #[pin]
    tracker: DependencyTracker,
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
    pub fn take(self: Pin<&Self>, binding: Pin<&Binding>) -> T {
        self.project_ref().tracker.register(binding);
        self.take_untracked()
    }

    pub fn take_untracked(self: Pin<&Self>) -> T {
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
