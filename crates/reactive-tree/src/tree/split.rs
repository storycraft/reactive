use core::pin::Pin;

use crate::ElementId;

use super::{ElementMap, RelationMap, element::Element};

#[derive(Debug)]
pub struct Elements<'a>(pub(super) &'a mut ElementMap);

impl Elements<'_> {
    #[inline]
    pub fn get(&self, id: ElementId) -> Pin<&Element> {
        self.0[id].as_ref()
    }

    #[inline]
    pub fn try_get(&self, id: ElementId) -> Option<Pin<&Element>> {
        Some(self.0.get(id)?.as_ref())
    }

    #[inline]
    pub fn get_mut(&mut self, id: ElementId) -> Pin<&mut Element> {
        self.0[id].as_mut()
    }

    #[inline]
    pub fn try_get_mut(&mut self, id: ElementId) -> Option<Pin<&mut Element>> {
        Some(self.0.get_mut(id)?.as_mut())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Relations<'a>(pub(super) &'a RelationMap);

impl<'a> Relations<'a> {
    #[inline]
    pub fn children(self, id: ElementId) -> &'a [ElementId] {
        if let Some(relation) = self.0.get(id) {
            &relation.children
        } else {
            &[]
        }
    }

    #[inline]
    pub fn parent(self, id: ElementId) -> Option<ElementId> {
        self.0.get(id)?.parent
    }
}
