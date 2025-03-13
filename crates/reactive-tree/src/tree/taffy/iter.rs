use core::slice;

use taffy::NodeId;

use crate::ElementId;

pub struct Iter<'a> {
    pub(super) tree: slice::Iter<'a, ElementId>,
}

impl Iterator for Iter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.tree.next()?.to_taffy_id())
    }
}
