use core::{
    cell::{self, Cell, RefCell},
    fmt::Debug,
    pin::Pin,
};

use pin_project::pin_project;

use reactivity::{binding::Binding, tracker::DependencyTracker};

use crate::event_loop::context::AppCx;

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
    pub fn take_get(self: Pin<&Self>, binding: Pin<&Binding>) -> T {
        self.project_ref().tracker.register(binding);
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
        notify(self.project_ref().tracker);
        self.set_untracked(value);
    }

    pub fn set_untracked(self: Pin<&Self>, value: T) {
        *self.value.borrow_mut() = value;
    }

    pub fn get(self: Pin<&Self>, binding: Pin<&Binding>) -> cell::Ref<'_, T> {
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

#[derive(Debug, derive_more::Deref, derive_more::DerefMut)]
pub struct Guard<'a, T> {
    tracker: Pin<&'a DependencyTracker>,
    #[deref]
    #[deref_mut]
    inner: cell::RefMut<'a, T>,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        notify(self.tracker);
    }
}

fn notify(tracker: Pin<&DependencyTracker>) {
    if AppCx::is_set() {
        AppCx::with(move |cx| {
            tracker.notify(cx.as_ref().queue());
        });
    }
}
