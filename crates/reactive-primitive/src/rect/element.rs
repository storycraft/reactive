use core::pin::Pin;
use reactive::{skia_safe, taffy, Element};

pub struct RectElement {
    pub fill_paint: skia_safe::Paint,
    pub stroke_paint: skia_safe::Paint,
    pub border_radius: [skia_safe::Point; 4],
}

impl RectElement {
    pub fn new() -> Self {
        Self {
            fill_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
            stroke_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
            border_radius: [skia_safe::Point::new(0.0, 0.0); 4],
        }
    }
}

impl Element for RectElement {
    fn draw(self: Pin<&Self>, canvas: &skia_safe::Canvas, layout: &taffy::Layout) {
        let rect = skia_safe::Rect::new(0.0, 0.0, layout.size.width, layout.size.height);

        let border_radius = &self.border_radius;
        let draw_rrect = !border_radius.iter().all(|radius| radius.is_zero());

        let fill_paint = &self.fill_paint;
        if !fill_paint.nothing_to_draw() {
            if draw_rrect {
                canvas.draw_rrect(
                    skia_safe::RRect::new_rect_radii(rect, border_radius),
                    fill_paint,
                );
            } else {
                canvas.draw_rect(rect, fill_paint);
            }
        }

        let stroke_paint = &self.stroke_paint;
        if !stroke_paint.nothing_to_draw() {
            if draw_rrect {
                canvas.draw_rrect(
                    skia_safe::RRect::new_rect_radii(rect, border_radius),
                    stroke_paint,
                );
            } else {
                canvas.draw_rect(rect, stroke_paint);
            }
        }
    }
}
