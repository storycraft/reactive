use core::pin::Pin;

use reactive::{
    skia_safe::{Canvas, Color4f, Paint, Point, RRect, Rect},
    taffy::Layout,
    Element,
};

pub struct BlockElement {
    pub fill_paint: Paint,
    pub stroke_paint: Paint,
    pub border_radius: [Point; 4],
}

impl BlockElement {
    pub fn new() -> Self {
        Self {
            fill_paint: Paint::new(Color4f::from_bytes_rgba(0), None),
            stroke_paint: Paint::new(Color4f::from_bytes_rgba(0), None),
            border_radius: [Point::new(0.0, 0.0); 4],
        }
    }
}

impl Element for BlockElement {
    fn draw(self: Pin<&Self>, canvas: &Canvas, layout: &Layout) {
        let fill_paint = &self.fill_paint;
        if fill_paint.nothing_to_draw() {
            return;
        }

        let rect = Rect::new(0.0, 0.0, layout.size.width, layout.size.height);

        let border_radius = &self.border_radius;
        let draw_rrect = !border_radius.iter().all(|radius| radius.is_zero());
        if draw_rrect {
            canvas.draw_rrect(RRect::new_rect_radii(rect, border_radius), fill_paint);
        } else {
            canvas.draw_rect(rect, fill_paint);
        }

        let stroke_paint = &self.stroke_paint;
        if stroke_paint.nothing_to_draw() {
            return;
        }

        if draw_rrect {
            canvas.draw_rrect(RRect::new_rect_radii(rect, border_radius), stroke_paint);
        } else {
            canvas.draw_rect(rect, stroke_paint);
        }
    }
}
