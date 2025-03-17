pub mod action;
pub mod element;
pub mod node;
pub mod pass;
mod relation;
mod split;
mod taffy;
mod visitor;

use core::pin::Pin;

use ::taffy::{AvailableSpace, Size, Style, compute_root_layout, round_layout};
use element::Element;
use pass::{cleanup, update};
use relation::Relation;
use skia_safe::Canvas;
use slotmap::{SecondaryMap, SlotMap};
use split::{Elements, Relations};
use visitor::{TreeVisitor, TreeVisitorMut};
use winit::event::WindowEvent;

use crate::{ElementId, screen::ScreenRect};

type ElementMap = SlotMap<ElementId, Pin<Box<Element>>>;
type RelationMap = SecondaryMap<ElementId, Relation>;

#[derive(Debug)]
pub struct UiTree {
    elements: ElementMap,
    relations: RelationMap,
    pub screen: ScreenRect,
    root: ElementId,
}

impl UiTree {
    pub fn new() -> Self {
        let mut elements = SlotMap::with_key();
        let root = elements.insert(Box::pin(Element::new(Style {
            size: Size::from_percent(1.0, 1.0),
            ..Style::DEFAULT
        })));

        let mut relations = SecondaryMap::new();
        relations.insert(
            root,
            Relation {
                parent: None,
                children: Vec::new(),
            },
        );

        Self {
            elements,
            relations,
            screen: ScreenRect::ZERO,
            root,
        }
    }

    #[inline]
    pub fn root(&self) -> ElementId {
        self.root
    }

    #[inline]
    /// Split elements and relations for better lifetime utilization
    fn split(&mut self) -> (Elements<'_>, Relations<'_>) {
        (Elements(&mut self.elements), Relations(&self.relations))
    }

    /// Create a element in the tree
    pub fn create(&mut self, style: Style) -> ElementId {
        let id = self.elements.insert(Box::pin(Element::new(style)));
        self.relations.insert(
            id,
            Relation {
                parent: None,
                children: Vec::new(),
            },
        );

        id
    }

    #[inline]
    /// Check if id is valid for this tree
    pub fn contains(&self, id: ElementId) -> bool {
        self.elements.contains_key(id)
    }

    /// Append an element to parent
    pub fn append_child(&mut self, parent: ElementId, child: ElementId) {
        self.remove_child(child);

        self.relations[parent].children.push(child);
        self.relations[child].parent = Some(parent);
        self.mark_dirty(child);
    }

    /// Remove an element from the parent
    pub fn remove_child(&mut self, id: ElementId) {
        let parent = self.parent(id).take();
        let (mut elements, relations) = self.split();
        cleanup(id, &mut elements, relations);

        if let Some(parent) = parent {
            self.relations[parent]
                .children
                .retain(|child_id| *child_id != id);
            self.mark_dirty(parent);
        }
    }

    /// Completely remove a element from the tree except root element
    pub fn destroy(&mut self, id: ElementId) {
        if id == self.root {
            return;
        }

        self.remove_child(id);
        self.elements.remove(id);
        self.relations.remove(id);
    }

    #[inline]
    pub fn get(&self, id: ElementId) -> Pin<&Element> {
        self.elements[id].as_ref()
    }

    #[inline]
    pub fn try_get(&self, id: ElementId) -> Option<Pin<&Element>> {
        Some(self.elements.get(id)?.as_ref())
    }

    #[inline]
    pub fn children(&self, id: ElementId) -> &[ElementId] {
        Relations(&self.relations).children(id)
    }

    #[inline]
    pub fn parent(&self, id: ElementId) -> Option<ElementId> {
        Relations(&self.relations).parent(id)
    }

    pub fn window_event(&self, event: &mut WindowEvent) {
        struct Visitor<'a>(&'a mut WindowEvent);
        impl TreeVisitor for Visitor<'_> {
            fn visit(&mut self, id: ElementId, tree: &UiTree) {
                let element = &tree.elements[id];
                element.dispatch_event(self.0);

                visitor::visit(self, id, tree);
            }
        }

        Visitor(event).visit(self.root, self);
    }

    pub fn draw(&self, canvas: &Canvas) {
        struct Draw<'a>(&'a Canvas);
        impl TreeVisitor for Draw<'_> {
            fn visit(&mut self, id: ElementId, tree: &UiTree) {
                let element = &tree.elements[id];
                element.pre_draw(self.0);
                element.draw(self.0);

                visitor::visit(self, id, tree);
            }
        }

        Draw(canvas).visit(self.root, self);
        canvas.reset_matrix();
    }

    pub fn mark_dirty(&mut self, id: ElementId) {
        struct MarkDirty;
        impl TreeVisitorMut for MarkDirty {
            fn visit_mut(&mut self, id: ElementId, elements: &mut Elements, relations: Relations) {
                elements[id].as_mut().project().node.cache.clear();

                if let Some(id) = relations.parent(id) {
                    self.visit_mut(id, elements, relations);
                }
            }
        }

        let (mut elements, relations) = self.split();
        MarkDirty.visit_mut(id, &mut elements, relations);
    }

    pub fn update(&mut self) {
        let (width, height) = self.screen.logical_size();
        compute_root_layout(
            self,
            self.root.to_taffy_id(),
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        );
        round_layout(self, self.root.to_taffy_id());

        let root = self.root;
        let (mut elements, relations) = self.split();
        update(root, &mut elements, relations);
    }
}

impl Default for UiTree {
    fn default() -> Self {
        Self::new()
    }
}
