use crate::ElementId;

use super::{
    UiTree,
    entry::{Elements, Relations},
};

/// Visit immutable elements on tree
/// 
/// Default implementation visit children recursively
pub trait TreeVisitor {
    fn visit(&mut self, id: ElementId, tree: &UiTree) {
        visit(self, id, tree);
    }
}

#[inline]
/// Default implementation for [TreeVisitor::visit]
pub fn visit(this: &mut (impl TreeVisitor + ?Sized), id: ElementId, tree: &UiTree) {
    for &id in tree.children(id) {
        this.visit(id, tree);
    }
}

/// Visit mutable elements on tree
/// 
/// Default implementation visit children recursively
pub trait TreeVisitorMut {
    fn visit_mut(&mut self, id: ElementId, elements: &mut Elements, relations: Relations) {
        visit_mut(self, id, elements, relations);
    }
}

#[inline]
/// Default implementation for [TreeVisitorMut::visit_mut]
pub fn visit_mut(
    this: &mut (impl TreeVisitorMut + ?Sized),
    id: ElementId,
    elements: &mut Elements,
    relations: Relations,
) {
    for &id in relations.children(id) {
        this.visit_mut(id, elements, relations);
    }
}
