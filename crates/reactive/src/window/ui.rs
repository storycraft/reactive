use core::pin::Pin;
use std::rc::Rc;

use taffy::Style;

use crate::{tree::Tree, Element, ElementId};

#[derive(Debug, Clone, Copy)]
pub struct Ui<'a> {
    tree: &'a Tree,
    current: ElementId,
}

impl<'a> Ui<'a> {
    pub fn new(tree: &'a Tree, current: ElementId) -> Self {
        Self { tree, current }
    }

    pub fn root(tree: &'a Tree) -> Self {
        Self {
            tree,
            current: tree.root(),
        }
    }

    pub fn tree(&self) -> &'a Tree {
        self.tree
    }

    pub fn current(&self) -> ElementId {
        self.current
    }

    pub fn append<T>(&self, layout: Style, element: T) -> (ElementId, Pin<Rc<T>>)
    where
        T: Element + 'static,
    {
        let (id, element) = self.tree.create(layout, element);
        self.tree.append(self.current, id);

        (id, element)
    }

    pub fn remove_child(&self, id: ElementId) {
        self.tree.remove_child(self.current, id)
    }

    pub fn remove(&self, id: ElementId) {
        self.tree.remove(id)
    }
}
