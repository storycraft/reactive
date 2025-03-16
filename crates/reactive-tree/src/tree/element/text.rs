#[non_exhaustive]
#[derive(Debug)]
pub struct Text {
    pub blob: Option<skia_safe::TextBlob>,
    pub font: Option<skia_safe::Font>,
    pub fill_paint: skia_safe::Paint,
    pub stroke_paint: skia_safe::Paint,
}

impl Text {
    pub fn new() -> Self {
        Self {
            blob: None,
            font: None,
            fill_paint: skia_safe::Paint::new(skia_safe::colors::BLACK, None),
            stroke_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
        }
    }

    pub fn draw(&self, canvas: &skia_safe::Canvas, height: f32) {
        if let Some(blob) = self.blob.as_ref() {
            let origin = skia_safe::Point::new(0.0, height);
            if !self.fill_paint.nothing_to_draw() {
                canvas.draw_text_blob(blob, origin, &self.fill_paint);
            }

            if !self.stroke_paint.nothing_to_draw() {
                canvas.draw_text_blob(blob, origin, &self.stroke_paint);
            }
        }
    }

    pub fn measure(
        &self,
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

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}
