pub mod element;
pub mod node;
mod relation;
pub mod split;
mod taffy;
pub mod visitor;

use core::pin::Pin;

use ::taffy::{AvailableSpace, Size, Style, compute_root_layout};
use element::Element;
use nalgebra::Matrix4;
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
        let mut map = SlotMap::with_key();
        let root = map.insert(Box::pin(Element::new(Style {
            size: Size::from_percent(1.0, 1.0),
            ..Style::DEFAULT
        })));

        let mut relations = SecondaryMap::new();
        relations.insert(
            root,
            Relation {
                parent: root,
                children: Vec::new(),
            },
        );

        Self {
            elements: map,
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
    pub fn split(&mut self) -> (Elements<'_>, Relations<'_>) {
        (Elements(&mut self.elements), Relations(&self.relations))
    }

    pub fn append(&mut self, parent: ElementId, child: Pin<Box<Element>>) -> Option<ElementId> {
        if !self.elements.contains_key(parent) {
            return None;
        }

        let id = self.elements.insert(child);

        self.relations.insert(
            id,
            Relation {
                parent,
                children: Vec::new(),
            },
        );
        self.relations[parent].children.push(id);
        Some(id)
    }

    fn remove_child_recursive(&mut self, id: ElementId) -> Option<(ElementId, Pin<Box<Element>>)> {
        let element = self.elements.remove(id)?;
        let mut relation = self.relations.remove(id).unwrap();

        for child in relation.children.drain(..) {
            self.remove_child_recursive(child);
        }

        Some((relation.parent, element))
    }

    pub fn remove(&mut self, id: ElementId) -> Option<Pin<Box<Element>>> {
        if id == self.root {
            return None;
        }

        let (parent, mut element) = self.remove_child_recursive(id)?;
        element.as_mut().node_mut().cleanup();

        self.relations[parent]
            .children
            .retain(|child_id| *child_id != id);

        Some(element)
    }

    #[inline]
    pub fn get(&self, id: ElementId) -> Pin<&Element> {
        self.elements[id].as_ref()
    }

    #[inline]
    pub fn get_mut(&mut self, id: ElementId) -> Pin<&mut Element> {
        self.elements[id].as_mut()
    }

    #[inline]
    pub fn try_get(&self, id: ElementId) -> Option<Pin<&Element>> {
        Some(self.elements.get(id)?.as_ref())
    }

    #[inline]
    pub fn try_get_mut(&mut self, id: ElementId) -> Option<Pin<&mut Element>> {
        Some(self.elements.get_mut(id)?.as_mut())
    }

    #[inline]
    pub fn children(&self, id: ElementId) -> &[ElementId] {
        Relations(&self.relations).children(id)
    }

    #[inline]
    pub fn parent(&self, id: ElementId) -> ElementId {
        Relations(&self.relations).parent(id)
    }

    #[inline]
    pub fn try_parent(&self, id: ElementId) -> Option<ElementId> {
        Relations(&self.relations).try_parent(id)
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
                elements[id].as_mut().node_mut().cache.clear();

                visitor::visit_mut(self, id, elements, relations);
            }
        }

        let (mut elements, relations) = self.split();
        MarkDirty.visit_mut(id, &mut elements, relations);
    }

    pub fn style_mut(&mut self, id: ElementId) -> &mut Style {
        self.mark_dirty(id);
        &mut self.elements[id].as_mut().node_mut().style
    }

    pub fn set_style(&mut self, id: ElementId, style: Style) {
        let Some(element) = self.elements.get_mut(id) else {
            return;
        };

        element.as_mut().node_mut().style = style;
        self.mark_dirty(id);
    }

    pub fn update(&mut self) {
        struct UpdateMatrix {
            matrix: Matrix4<f32>,
            inverse_matrix: Matrix4<f32>,
        }

        impl TreeVisitorMut for UpdateMatrix {
            fn visit_mut(&mut self, id: ElementId, elements: &mut Elements, relations: Relations) {
                let mut element = elements[id].as_mut();
                let matrix = self.matrix * element.transform.to_matrix();
                let inverse_matrix = self.inverse_matrix * element.transform.to_inverse_matrix();

                let node = element.as_mut().node_mut();
                node.matrix = matrix;
                node.inverse_matrix = inverse_matrix;

                visitor::visit_mut(
                    &mut Self {
                        matrix,
                        inverse_matrix,
                    },
                    id,
                    elements,
                    relations,
                );
            }
        }

        let (width, height) = self.screen.logical_size();
        compute_root_layout(
            self,
            self.root.to_taffy_id(),
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        );

        let root = self.root;
        let (mut elements, relations) = self.split();
        UpdateMatrix {
            matrix: Matrix4::identity(),
            inverse_matrix: Matrix4::identity(),
        }
        .visit_mut(root, &mut elements, relations);
    }
}

impl Default for UiTree {
    fn default() -> Self {
        Self::new()
    }
}
