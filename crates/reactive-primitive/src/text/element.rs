use core::pin::Pin;

use reactive::{Element, skia_safe, taffy};

pub struct TextElement {
    pub blob: Option<skia_safe::TextBlob>,
    pub font: Option<skia_safe::Font>,
    pub fill_paint: skia_safe::Paint,
    pub stroke_paint: skia_safe::Paint,
}

impl TextElement {
    pub fn new() -> Self {
        Self {
            blob: None,
            font: None,
            fill_paint: skia_safe::Paint::new(skia_safe::colors::BLACK, None),
            stroke_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
        }
    }
}

impl Element for TextElement {
    fn draw(self: Pin<&Self>, canvas: &skia_safe::Canvas, layout: &taffy::Layout) {
        if let Some(blob) = self.blob.as_ref() {
            let origin = skia_safe::Point::new(0.0, layout.size.height);
            if !self.fill_paint.nothing_to_draw() {
                canvas.draw_text_blob(blob, origin, &self.fill_paint);
            }

            if !self.stroke_paint.nothing_to_draw() {
                canvas.draw_text_blob(blob, origin, &self.stroke_paint);
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
