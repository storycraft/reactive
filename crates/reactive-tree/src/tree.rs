pub mod node;
mod relation;
mod taffy;

use core::pin::Pin;

use ::taffy::{AvailableSpace, Size, Style, compute_root_layout};
use nalgebra::Matrix4;
use relation::Relation;
use skia_safe::Canvas;
use slotmap::{SecondaryMap, SlotMap};
use winit::event::WindowEvent;

use crate::{ElementId, element::Element, screen::ScreenRect};

type ElementMap = SlotMap<ElementId, Pin<Box<Element>>>;
type RelationMap = SecondaryMap<ElementId, Relation>;

#[derive(Debug)]
pub struct UiTree {
    map: ElementMap,
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
                parent: None,
                children: Vec::new(),
            },
        );

        Self {
            map,
            relations,
            screen: ScreenRect::ZERO,
            root,
        }
    }

    pub fn root(&self) -> ElementId {
        self.root
    }

    pub fn append(&mut self, parent_id: ElementId, child: Pin<Box<Element>>) -> Option<ElementId> {
        if !self.map.contains_key(parent_id) {
            return None;
        }

        let id = self.map.insert(child);

        self.relations.insert(
            id,
            Relation {
                parent: Some(parent_id),
                children: Vec::new(),
            },
        );
        self.relations[parent_id].children.push(id);
        Some(id)
    }

    pub fn remove(&mut self, id: ElementId) -> Option<Pin<Box<Element>>> {
        if id == self.root {
            return None;
        }

        let mut element = self.map.remove(id)?;
        element.as_mut().node_mut().cache.clear();
        let mut relation = self.relations.remove(id).unwrap();
        for child in relation.children.drain(..) {
            self.remove(child);
        }

        if let Some(parent_id) = relation.parent {
            self.relations[parent_id]
                .children
                .retain(|child_id| *child_id != id);
        }

        Some(element)
    }

    pub fn get(&self, id: ElementId) -> Option<Pin<&Element>> {
        Some(self.map.get(id)?.as_ref())
    }

    pub fn get_mut(&mut self, id: ElementId) -> Option<Pin<&mut Element>> {
        Some(self.map.get_mut(id)?.as_mut())
    }

    pub fn children(&mut self, id: ElementId) -> &[ElementId] {
        if let Some(relation) = self.relations.get(id) {
            &relation.children
        } else {
            &[]
        }
    }

    pub fn parent(&mut self, id: ElementId) -> Option<ElementId> {
        if let Some(relation) = self.relations.get(id) {
            relation.parent
        } else {
            None
        }
    }

    pub fn window_event(&self, event: &mut WindowEvent) {
        fn event_inner(tree: &UiTree, event: &mut WindowEvent, id: ElementId) {
            let element = &tree.map[id];
            element.dispatch_event(event);
            for child in &tree.relations[id].children {
                event_inner(tree, event, *child);
            }
        }

        event_inner(self, event, self.root);
    }

    pub fn draw(&self, canvas: &Canvas) {
        fn draw_inner(tree: &UiTree, canvas: &Canvas, id: ElementId) {
            let element = &tree.map[id];
            element.pre_draw(canvas);
            element.draw(canvas);

            let children = &tree.relations[id].children;
            if !children.is_empty() {
                for child in children {
                    draw_inner(tree, canvas, *child);
                }
            }
            element.post_draw(canvas);
        }

        draw_inner(self, canvas, self.root);
    }

    pub fn mark_dirty(&mut self, id: ElementId) {
        fn clear_inner(elements: &mut ElementMap, relations: &RelationMap, id: ElementId) {
            let Some(element) = elements.get_mut(id) else {
                return;
            };
            element.as_mut().node_mut().cache.clear();

            for child in &relations[id].children {
                clear_inner(elements, relations, *child);
            }
        }

        clear_inner(&mut self.map, &self.relations, id);
    }

    pub fn style_mut(&mut self, id: ElementId) -> &mut Style {
        self.mark_dirty(id);
        &mut self.map[id].as_mut().node_mut().style
    }

    pub fn set_style(&mut self, id: ElementId, style: Style) {
        let Some(element) = self.map.get_mut(id) else {
            return;
        };

        element.as_mut().node_mut().style = style;
        self.mark_dirty(id);
    }

    pub fn update(&mut self) {
        fn update_matrix_inner(
            elements: &mut ElementMap,
            relations: &RelationMap,
            parent_matrix: Matrix4<f32>,
            id: ElementId,
        ) {
            let Some(element) = elements.get_mut(id) else {
                return;
            };
            let matrix = parent_matrix * element.transform.to_matrix();
            element.as_mut().node_mut().matrix = matrix;

            for &child in &relations[id].children {
                update_matrix_inner(elements, relations, matrix, child);
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
        update_matrix_inner(
            &mut self.map,
            &self.relations,
            Matrix4::identity(),
            self.root,
        );
    }
}

impl Default for UiTree {
    fn default() -> Self {
        Self::new()
    }
}
