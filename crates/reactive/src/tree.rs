use core::pin::Pin;

use reactivity_winit::winit::{event::WindowEvent, event_loop::ActiveEventLoop};
use skia_safe::Canvas;
use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree, TraversePartialTree};

use crate::{Element, ElementId};

type TaffyElementTree = TaffyTree<Pin<Box<dyn Element>>>;

#[derive(derive_more::Debug)]
pub struct Tree {
    #[debug(skip)]
    taffy: TaffyElementTree,
    size: (u32, u32),
    root: ElementId,
}

impl Tree {
    pub fn new() -> Self {
        let mut taffy = TaffyTree::new();
        let root = ElementId(
            taffy
                .new_leaf(Style {
                    size: Size::from_percent(1.0, 1.0),
                    ..Default::default()
                })
                .unwrap(),
        );

        Self {
            taffy,
            size: (0, 0),
            root,
        }
    }

    pub fn root(&self) -> ElementId {
        self.root
    }

    pub fn create<T: Element + 'static>(&mut self, layout: Style, element: T) -> ElementId {
        // It never failes, but why result lol
        let id = self
            .taffy
            .new_leaf_with_context(layout, Box::pin(element))
            .unwrap();

        ElementId(id)
    }

    // TODO:: error
    pub fn append(&mut self, parent: ElementId, child: ElementId) {
        let _ = self.taffy.add_child(parent.0, child.0);
    }

    // TODO:: error
    pub fn remove_child(&mut self, parent: ElementId, child: ElementId) {
        let _ = self.taffy.remove_child(parent.0, child.0);
    }

    // TODO:: error
    pub fn remove(&mut self, element: ElementId) {
        let _ = self.taffy.remove(element.0);
    }

    // TODO:: error
    pub fn set_style(&mut self, id: ElementId, style: Style) {
        let _ = self.taffy.set_style(id.0, style);
    }

    pub fn get<T: Element>(&self, id: ElementId) -> Option<Pin<&T>> {
        let context = self.taffy.get_node_context(id.0)?;
        context.as_ref().downcast_ref()
    }

    pub fn get_mut<T: Element>(&mut self, id: ElementId) -> Option<Pin<&mut T>> {
        let context = self.taffy.get_node_context_mut(id.0)?;
        context.as_mut().downcast_mut()
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

        event_inner(el, event, &self.taffy, self.root.0);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    pub fn draw(&mut self, canvas: &Canvas) {
        fn draw_inner(canvas: &Canvas, taffy: &TaffyElementTree, parent: NodeId) {
            for child in taffy.child_ids(parent) {
                let Some(cx) = taffy.get_node_context(child) else {
                    continue;
                };
                let layout = taffy.layout(child).unwrap();

                let cx = cx.as_ref();

                canvas.translate((layout.location.x, layout.location.y));
                cx.draw(canvas, layout);

                cx.pre_child_draw(canvas);
                draw_inner(canvas, taffy, child);
                cx.post_child_draw(canvas);

                canvas.translate((-layout.location.x, -layout.location.y));
            }
        }

        let (width, height) = self.size;
        self.taffy
            .compute_layout_with_measure(
                self.root.0,
                Size {
                    width: AvailableSpace::Definite(width as _),
                    height: AvailableSpace::Definite(height as _),
                },
                measure_element,
            )
            .unwrap();

        draw_inner(canvas, &self.taffy, self.root.0);
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

fn measure_element(
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    _: NodeId,
    node_context: Option<&mut Pin<Box<dyn Element>>>,
    style: &Style,
) -> Size<f32> {
    let Some(node_context) = node_context else {
        return Size::ZERO;
    };

    node_context
        .as_ref()
        .measure(known_dimensions, available_space, style)
}
