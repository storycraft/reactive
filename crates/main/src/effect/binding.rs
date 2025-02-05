use super::handle::HandleEntryPtr;
use crate::list::{Entry, Node};
use core::{cell::Cell, pin::Pin};
use pin_project::pin_project;

#[derive(Debug)]
#[pin_project]
pub struct Binding {
    #[pin]
    inner: Node<HandleBinding>,
}

impl Binding {
    pub fn new() -> Self {
        Self {
            inner: Node::new(HandleBinding {
                node: Node::new(Cell::new(HandleEntryPtr::new(None))),
            }),
        }
    }

    pub(crate) fn binding_entry(self: Pin<&Self>) -> &Entry<HandleBinding> {
        self.project_ref().inner.entry()
    }

    pub(crate) fn handle_entry(self: Pin<&Self>) -> &Entry<Cell<HandleEntryPtr>> {
        self.binding_entry().value_pinned().handle_entry()
    }
}

impl Default for Binding {
    fn default() -> Self {
        Self::new()
    }
}

#[pin_project]
#[derive(Debug)]
pub(crate) struct HandleBinding {
    #[pin]
    pub(crate) node: Node<Cell<HandleEntryPtr>>,
}

impl HandleBinding {
    pub(crate) fn handle_entry(self: Pin<&Self>) -> &Entry<Cell<HandleEntryPtr>> {
        self.project_ref().node.entry()
    }
}
