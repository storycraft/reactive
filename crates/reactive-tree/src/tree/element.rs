pub mod rect;
pub mod text;

use core::pin::Pin;

use nalgebra::Point3;
use pin_project::pin_project;
use reactive_event::EventTarget;
use rect::Rect;
use skia_safe::M44;
use taffy::Style;
use text::Text;
use winit::event::WindowEvent;

use crate::{transform::Transform, tree::node::Node};

#[derive(Debug)]
#[pin_project]
pub struct Element {
    pub(super) node: Node,

    pub transform: Transform,

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
    pub(super) fn new(style: Style) -> Self {
        Self {
            node: Node::new(style),

            transform: Transform::new(),

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

    pub fn transform_mut(self: Pin<&mut Self>) -> &mut Transform {
        self.project().transform
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

    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        let Some(ref rect) = self.rect else {
            return false;
        };
        let layout = self.node.layout();
        let transformed = self.node.matrix().transform_point(&Point3::new(
            x - layout.location.x,
            y - layout.location.y,
            0.0,
        ));

        rect.hit_test(
            layout.content_box_width(),
            layout.content_box_height(),
            transformed.x,
            transformed.y,
        )
    }

    pub(super) fn pre_draw(&self, canvas: &skia_safe::Canvas) {
        let matrix = self.node.matrix();
        canvas.set_matrix(&M44::new(
            matrix.m11, matrix.m21, matrix.m31, matrix.m41, matrix.m12, matrix.m22, matrix.m32,
            matrix.m42, matrix.m13, matrix.m23, matrix.m33, matrix.m43, matrix.m14, matrix.m24,
            matrix.m34, matrix.m44,
        ));
    }

    pub fn draw(&self, canvas: &skia_safe::Canvas) {
        let size = self.node.layout().content_box_size();
        if let Some(ref rect) = self.rect {
            rect.draw(canvas, size.width, size.height);
        }

        if let Some(ref text) = self.text {
            text.draw(canvas, size.height);
        }
    }

    pub(super) fn dispatch_event(&self, event: &mut WindowEvent) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            if !self.hit_test(position.x as _, position.y as _) {
                return;
            }

            self.on_mouse_move.emit_mut(&mut ());
        }
    }
}
