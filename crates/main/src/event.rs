use core::{
    marker::PhantomData,
    pin::{pin, Pin},
};
use pin_project::pin_project;
use reactivity::list::{List, Node};

#[derive(Debug)]
#[pin_project]
pub struct EventTarget<T: ?Sized> {
    #[pin]
    list: List<*mut dyn FnMut(&mut T) -> bool>,
}

impl<T: ?Sized> Default for EventTarget<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> EventTarget<T> {
    pub fn new() -> Self {
        Self { list: List::new() }
    }

    pub fn on(self: Pin<&Self>, listener: Pin<&EventListener<T>>) {
        self.project_ref()
            .list
            .push_front(listener.project_ref().node.entry());
    }

    pub fn emit(self: Pin<&Self>, value: &mut T) {
        for entry in self.project_ref().list.iter() {
            if (unsafe { &mut **entry.value() })(value) {
                entry.unlink();
            }
        }
    }
}

#[derive(derive_more::Debug)]
#[pin_project]
pub struct EventListener<'a, T: ?Sized> {
    #[pin]
    node: Node<*mut dyn FnMut(&mut T) -> bool>,
    _ph: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> EventListener<'a, T> {
    pub fn new(f: &'a mut dyn FnMut(&mut T) -> bool) -> Self {
        Self {
            node: Node::new(f as *mut _ as *mut _),
            _ph: PhantomData,
        }
    }
}
