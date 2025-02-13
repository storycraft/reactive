use core::pin::Pin;

use reactive::{skia_safe, taffy, Element};

pub struct BlockElement {
    pub fill_paint: skia_safe::Paint,
    pub stroke_paint: skia_safe::Paint,
    pub border_radius: [skia_safe::Point; 4],
    pub blob: Option<skia_safe::TextBlob>,
    pub font: Option<skia_safe::Font>,
    pub text_fill_paint: skia_safe::Paint,
    pub text_stroke_paint: skia_safe::Paint,
}

impl BlockElement {
    pub fn new() -> Self {
        Self {
            fill_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
            stroke_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
            border_radius: [skia_safe::Point::new(0.0, 0.0); 4],
            blob: None,
            font: None,
            text_fill_paint: skia_safe::Paint::new(skia_safe::colors::BLACK, None),
            text_stroke_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
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

        if let Some(blob) = self.blob.as_ref() {
            let origin = skia_safe::Point::new(0.0, layout.content_box_height());
            if !self.text_fill_paint.nothing_to_draw() {
                canvas.draw_text_blob(blob, origin, &self.text_fill_paint);
            }

            if !self.text_stroke_paint.nothing_to_draw() {
                canvas.draw_text_blob(blob, origin, &self.text_stroke_paint);
            }
        }
    }

    fn measure(
        self: Pin<&Self>,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
        _style: &taffy::Style,
    ) -> taffy::Size<f32> {
        if let Some(blob) = self.blob.as_ref() {
            let rect = blob.bounds();
            return taffy::Size {
                width: rect.width(),
                height: rect.height(),
            };
        }

        taffy::Size::ZERO
    }
}
