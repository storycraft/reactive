use core::{
    cell::{Cell, RefCell},
    pin::Pin,
};
use std::rc::Rc;

use reactivity_winit::winit::{event::WindowEvent, event_loop::ActiveEventLoop};
use skia_safe::Canvas;
use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree, TraversePartialTree};

use crate::{Element, ElementId};

type TaffyElementTree = TaffyTree<Pin<Rc<dyn Element>>>;

#[derive(derive_more::Debug)]
pub struct Tree {
    #[debug(skip)]
    taffy: RefCell<TaffyElementTree>,
    size: Cell<(u32, u32)>,
    root: ElementId,
}

impl Tree {
    pub fn new() -> Self {
        let mut tree = TaffyTree::new();
        let root = ElementId(tree.new_leaf(Style {
            size: Size::from_percent(1.0, 1.0),
            ..Default::default()
        }).unwrap());

        Self {
            taffy: RefCell::new(tree),
            size: Cell::new((0, 0)),
            root,
        }
    }

    pub fn root(&self) -> ElementId {
        self.root
    }

    pub fn create<T: Element + 'static>(
        &self,
        layout: Style,
        element: T,
    ) -> (ElementId, Pin<Rc<T>>) {
        let element = Rc::pin(element);

        // It never failes, but why result lol
        let id = self
            .taffy
            .borrow_mut()
            .new_leaf_with_context(layout, element.clone())
            .unwrap();

        (ElementId(id), element)
    }

    // TODO:: error
    pub fn append(&self, parent: ElementId, child: ElementId) {
        let _ = self.taffy.borrow_mut().add_child(parent.0, child.0);
    }

    // TODO:: error
    pub fn remove_child(&self, parent: ElementId, child: ElementId) {
        let _ = self.taffy.borrow_mut().remove_child(parent.0, child.0);
    }

    // TODO:: error
    pub fn remove(&self, element: ElementId) {
        let _ = self.taffy.borrow_mut().remove(element.0);
    }

    pub fn window_event(&self, el: &ActiveEventLoop, event: &mut WindowEvent) {
        fn event_inner(
            el: &ActiveEventLoop,
            event: &mut WindowEvent,
            taffy: &TaffyElementTree,
            parent: NodeId,
        ) {
            for child in taffy.child_ids(parent) {
                let Some(cx) = taffy.get_node_context(child) else {
                    continue;
                };
                let cx = cx.as_ref();
                cx.on_event(el, event);

                event_inner(el, event, taffy, child);
            }
        }

        if let WindowEvent::Resized(size) = event {
            self.size.set((size.width, size.height));
        }

        let taffy = &mut *self.taffy.borrow_mut();
        event_inner(el, event, taffy, self.root.0);
    }

    pub fn redraw(&self, canvas: &Canvas) {
        fn redraw_inner(canvas: &Canvas, taffy: &TaffyElementTree, parent: NodeId) {
            for child in taffy.child_ids(parent) {
                let Some(cx) = taffy.get_node_context(child) else {
                    continue;
                };
                let layout = taffy.layout(child).unwrap();

                let cx = cx.as_ref();

                canvas.translate((layout.location.x, layout.location.y));
                cx.draw(canvas, layout.size.width, layout.size.height);

                cx.pre_child_draw(canvas);
                redraw_inner(canvas, taffy, child);
                cx.post_child_draw(canvas);

                canvas.translate((-layout.location.x, -layout.location.y));
            }
        }

        let (width, height) = self.size.get();
        let taffy = &mut *self.taffy.borrow_mut();
        taffy
            .compute_layout(
                self.root.0,
                Size {
                    width: AvailableSpace::Definite(width as _),
                    height: AvailableSpace::Definite(height as _),
                },
            )
            .unwrap();

        redraw_inner(canvas, taffy, self.root.0);
    }
}
