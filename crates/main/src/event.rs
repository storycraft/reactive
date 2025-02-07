use core::{pin::{pin, Pin}, ptr::NonNull};
use pin_project::pin_project;
use reactivity::list::{List, Node};

#[derive(Debug)]
#[pin_project]
pub struct EventTarget<F: ?Sized> {
    #[pin]
    list: List<NonNull<F>>,
}

impl<F: ?Sized> EventTarget<F> {
    pub fn on(self: Pin<&Self>, listener: Pin<&EventListener<F>>) {
        self.project_ref()
            .list
            .push_front(listener.project_ref().node.entry());
    }
}

#[derive(derive_more::Debug)]
#[pin_project]
pub struct EventListener<F: ?Sized> {
    #[pin]
    node: Node<NonNull<F>>,
}

impl<F: ?Sized> EventListener<F> {}
