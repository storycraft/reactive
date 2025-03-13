pub mod rect;
pub mod text;

use core::pin::Pin;

use pin_project::pin_project;
use reactive_event::EventTarget;
use rect::Rect;
use taffy::Style;
use text::Text;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

use crate::tree::node::Node;

#[derive(Debug)]
#[pin_project]
pub struct Element {
    pub(crate) node: Node,

    pub rect: Option<Rect>,
    pub text: Option<Text>,

    #[pin]
    on_click: EventTarget!(&mut ()),
    #[pin]
    on_mouse_move: EventTarget!(&mut ()),
    #[pin]
    on_mouse_down: EventTarget!(&mut ()),
    #[pin]
    on_mouse_up: EventTarget!(&mut ()),
    #[pin]
    on_enter: EventTarget!(&mut ()),
    #[pin]
    on_leave: EventTarget!(&mut ()),
    #[pin]
    on_drag: EventTarget!(&mut ()),
}

impl Element {
    pub fn new(style: Style) -> Self {
        Self {
            node: Node::new(style),

            rect: None,
            text: None,

            on_click: EventTarget::new(),
            on_mouse_move: EventTarget::new(),
            on_mouse_down: EventTarget::new(),
            on_mouse_up: EventTarget::new(),
            on_enter: EventTarget::new(),
            on_leave: EventTarget::new(),
            on_drag: EventTarget::new(),
        }
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn node_mut(self: Pin<&mut Self>) -> &mut Node {
        self.project().node
    }

    pub fn rect_mut(self: Pin<&mut Self>) -> &mut Option<Rect> {
        self.project().rect
    }

    pub fn text_mut(self: Pin<&mut Self>) -> &mut Option<Text> {
        self.project().text
    }

    pub fn on_mouse_move(self: Pin<&Self>) -> Pin<&EventTarget!(&mut ())> {
        self.project_ref().on_mouse_move
    }

    pub fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        if let Some(ref text) = self.text {
            return text.measure(known_dimensions, available_space, self.node.style());
        }

        taffy::Size::zero()
    }

    pub fn hittest(&self, x: f64, y: f64) -> bool {
        let Some(ref rect) = self.rect else {
            return false;
        };

        rect.hittest(x, y, self.node.layout())
    }

    pub(super) fn pre_child_draw(&self, canvas: &skia_safe::Canvas) {
        let layout = self.node.layout();
        canvas.translate((layout.location.x, layout.location.y));
    }

    pub(super) fn post_child_draw(&self, canvas: &skia_safe::Canvas) {
        let layout = self.node.layout();
        canvas.translate((-layout.location.x, -layout.location.y));
    }

    pub fn draw(&self, canvas: &skia_safe::Canvas) {
        if let Some(ref rect) = self.rect {
            rect.draw(canvas, self.node.layout());
        }

        if let Some(ref text) = self.text {
            text.draw(canvas, self.node.layout());
        }
    }

    pub(super) fn dispatch_event(&self, _: &ActiveEventLoop, event: &mut WindowEvent) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            if !self.hittest(position.x, position.y) {
                return;
            }

            self.on_mouse_move.emit_mut(&mut ());
        }
    }
}

impl Default for Element {
    fn default() -> Self {
        Self::new(Style::DEFAULT)
    }
}
