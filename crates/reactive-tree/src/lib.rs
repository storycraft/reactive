pub mod element;
pub mod event;
pub mod tree;
pub mod draw;
pub mod screen;

use slotmap::{KeyData, new_key_type};
use taffy::NodeId;

new_key_type! { pub struct ElementId; }

impl ElementId {
    #[inline]
    pub(crate) fn to_taffy_id(self) -> NodeId {
        NodeId::new(self.0.as_ffi())
    }

    #[inline]
    pub(crate) fn from_taffy_id(id: NodeId) -> Self {
        ElementId(KeyData::from_ffi(id.into()))
    }
}
