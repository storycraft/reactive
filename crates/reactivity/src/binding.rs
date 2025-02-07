use crate::{
    effect::handle::EffectFnPtrSlot,
    list::{Entry, Node},
};
use core::{pin::Pin, ptr::NonNull};
use pin_project::pin_project;

#[derive(Debug)]
#[pin_project]
/// Connection to dependency tracker and effect handle
pub struct Binding {
    /// Node connected to dependency tracker
    #[pin]
    to_tracker: Node<TrackerBinding>,
}

impl Binding {
    pub fn new() -> Self {
        Self {
            to_tracker: Node::new(TrackerBinding {
                to_handle: Node::new(EffectFnPtrSlot::new(NonNull::dangling())),
            }),
        }
    }

    /// Entry connecting to dependency tracker
    pub(crate) fn to_tracker(self: Pin<&Self>) -> &Entry<TrackerBinding> {
        self.project_ref().to_tracker.entry()
    }

    /// Entry connecting to handle
    pub(crate) fn to_handle(self: Pin<&Self>) -> &Entry<EffectFnPtrSlot> {
        self.to_tracker().value_pinned().to_handle()
    }
}

impl Default for Binding {
    fn default() -> Self {
        Self::new()
    }
}

#[pin_project]
#[derive(Debug)]
pub(crate) struct TrackerBinding {
    /// Node connected to handle
    #[pin]
    pub(crate) to_handle: Node<EffectFnPtrSlot>,
}

impl TrackerBinding {
    pub(crate) fn to_handle(self: Pin<&Self>) -> &Entry<EffectFnPtrSlot> {
        self.project_ref().to_handle.entry()
    }
}
