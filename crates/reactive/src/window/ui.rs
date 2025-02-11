use core::{cell::RefCell, pin::Pin};

use taffy::Style;

use crate::{tree::Tree, Element, ElementId};

#[derive(Debug, Clone, Copy)]
pub struct Ui<'a> {
    tree: &'a RefCell<Tree>,
    current: ElementId,
}

impl<'a> Ui<'a> {
    pub const fn new(tree: &'a RefCell<Tree>, current: ElementId) -> Self {
        Self { tree, current }
    }

    pub fn root(tree: &'a RefCell<Tree>) -> Self {
        let current = tree.borrow().root();
        Self::new(tree, current)
    }

    pub fn sub_ui(&self, child: ElementId) -> Ui<'a> {
        Self::new(self.tree, child)
    }

    pub fn current_id(&self) -> ElementId {
        self.current
    }

    pub fn append<T>(&self, layout: Style, element: T) -> ElementId
    where
        T: Element + 'static,
    {
        let mut tree = self.tree.borrow_mut();
        let id = tree.create(layout, element);
        tree.append(self.current, id);
        id
    }

    pub fn with_ref<T: Element, R>(
        &self,
        id: ElementId,
        f: impl FnOnce(Pin<&T>) -> R,
    ) -> Option<R> {
        Some(f(self.tree.borrow().get(id)?))
    }

    pub fn with_mut<T: Element, R>(
        &self,
        id: ElementId,
        f: impl FnOnce(Pin<&mut T>) -> R,
    ) -> Option<R> {
        Some(f(self.tree.borrow_mut().get_mut(id)?))
    }

    pub fn remove_child(&self, id: ElementId) {
        self.tree.borrow_mut().remove_child(self.current, id)
    }

    pub fn remove(&self, id: ElementId) {
        self.tree.borrow_mut().remove(id)
    }

    pub fn set_style(&self, id: ElementId, style: Style) {
        self.tree.borrow_mut().set_style(id, style);
    }
}
