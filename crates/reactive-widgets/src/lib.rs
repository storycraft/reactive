pub(crate) mod util;

pub use palette;

use core::pin::Pin;

use bon::Builder;
use palette::Srgba;
use reactive::{
    reactivity_winit::state::{StateCell, StateRefCell},
    skia_safe::{self, Canvas, Color4f, Paint, Point, RRect, Rect},
    taffy::{Layout, Style},
    wrap_element, Element, SetupFn, SetupFnWithChild,
};
use reactivity::let_effect;
use util::use_mut;

#[derive(Builder)]
pub struct Block<'a> {
    layout: Option<Pin<&'a StateRefCell<Style>>>,
    #[builder(default)]
    fill: Fill<'a>,
    #[builder(default)]
    border: Border<'a>,
}

#[derive(Builder, Default)]
pub struct Fill<'a> {
    color: Option<Pin<&'a StateCell<Srgba>>>,
}

#[derive(Builder, Default)]
pub struct Border<'a> {
    color: Option<Pin<&'a StateCell<Srgba>>>,
    thickness: Option<Pin<&'a StateCell<f32>>>,
}

impl<'a> Block<'a> {
    fn show<Child: SetupFn<'a>>(self, child: Child) -> impl SetupFn<'a, Output = Child::Output> {
        wrap_element(Style::DEFAULT, BlockElement::new(), move |ui| async move {
            let id = ui.current_id();

            macro_rules! wire {
                ($name:ident = $prop:expr => $($tt:tt)*) => {
                    let_effect!({
                        if let Some($name) = $prop {
                            $($tt)*
                        }
                    });
                };

                ($element:ident, $name:ident = $prop:expr => $($tt:tt)*) => {
                    wire!($name = $prop => {
                        use_mut::<BlockElement>(
                            ui,
                            id,
                            |#[allow(unused_mut)] mut $element| {
                                $($tt)*
                            }
                        )
                    });
                };
            }

            wire!(layout = self.layout => {
                ui.set_style(id, layout.get($).clone());
            });

            wire!(element, color = self.fill.color => {
                let color = color.get($);
                element.fill_paint.set_color4f(
                    skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                    None
                );
            });

            wire!(element, color = self.border.color => {
                let color = color.get($);
                element.stroke_paint.set_color4f(
                    skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                    None
                );
            });

            wire!(element, thickness = self.border.thickness => {
                element.stroke_paint.set_stroke_width(thickness.get($));
            });

            child.show(ui).await
        })
    }
}

impl<'a, Child> SetupFnWithChild<'a, Child> for Block<'a>
where
    Child: SetupFn<'a>,
{
    type Output = Child::Output;

    fn child(self, child: Child) -> impl SetupFn<'a, Output = Self::Output> {
        self.show(child)
    }
}

struct BlockElement {
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

impl Default for BlockElement {
    fn default() -> Self {
        Self::new()
    }
}
