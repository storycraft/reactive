use core::pin::Pin;

use reactive::{skia_safe, taffy, Element};

pub struct BlockElement {
    pub fill_paint: skia_safe::Paint,
    pub stroke_paint: skia_safe::Paint,
    pub border_radius: [skia_safe::Point; 4],
    pub text: Option<skia_safe::TextBlob>,
}

impl BlockElement {
    pub fn new() -> Self {
        Self {
            fill_paint: skia_safe::Paint::new(skia_safe::Color4f::from_bytes_rgba(0), None),
            stroke_paint: skia_safe::Paint::new(skia_safe::Color4f::from_bytes_rgba(0), None),
            border_radius: [skia_safe::Point::new(0.0, 0.0); 4],
            text: None,
        }
    }
}

impl Element for BlockElement {
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

        if let Some(text) = self.text.as_ref() {
            canvas.draw_text_blob(text, skia_safe::Point::new(0.0, 0.0), stroke_paint);
        }
    }

    fn measure(
        self: Pin<&Self>,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
        _style: &taffy::Style,
    ) -> taffy::Size<f32> {
        taffy::Size::ZERO
    }
}
