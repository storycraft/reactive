use skia_safe::{Contains, RRect};

#[non_exhaustive]
#[derive(Debug)]
pub struct Rect {
    pub fill_paint: skia_safe::Paint,
    pub stroke_paint: skia_safe::Paint,
    pub border_radius: [skia_safe::Point; 4],
}

impl Rect {
    pub fn new() -> Self {
        Self {
            fill_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
            stroke_paint: skia_safe::Paint::new(skia_safe::colors::TRANSPARENT, None),
            border_radius: [skia_safe::Point::new(0.0, 0.0); 4],
        }
    }

    pub fn is_rrect(&self) -> bool {
        !self.border_radius.iter().all(|radius| radius.is_zero())
    }

    // TODO:: cleanup code
    pub(super) fn hit_test(&self, x: f32, y: f32, layout: &taffy::Layout) -> bool {
        let rect = skia_safe::Rect::new(0.0, 0.0, layout.size.width, layout.size.height);

        if self.is_rrect() {
            const NEARLY_ZERO: f32 = 1.0_f32 / (1 << 12) as f32;
            // https://github.com/google/skia/blob/f4467ff38f1f0495307b3fe8cf4a3f564c33f7f3/modules/sksg/src/SkSGRect.cpp#L64
            // wth
            RRect::new_rect_radii(rect, &self.border_radius).contains(skia_safe::Rect::new(
                x - NEARLY_ZERO,
                y - NEARLY_ZERO,
                x + NEARLY_ZERO,
                y + NEARLY_ZERO,
            ))
        } else {
            rect.contains(skia_safe::Point::new(x, y))
        }
    }

    pub(super) fn draw(&self, canvas: &skia_safe::Canvas, layout: &taffy::Layout) {
        let rect = skia_safe::Rect::new(0.0, 0.0, layout.size.width, layout.size.height);

        let border_radius = &self.border_radius;
        let draw_rrect = self.is_rrect();

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

impl Default for Rect {
    fn default() -> Self {
        Self::new()
    }
}
