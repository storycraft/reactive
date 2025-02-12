pub(crate) mod util;

use core::{future::pending, pin::Pin};

use reactive::{
    reactivity_winit::state::{StateCell, StateRefCell},
    skia_safe::{Canvas, Color, Color4f, Paint, Point, RRect, Rect},
    taffy::{Layout, Style},
    with_children, wrap_element, Element, SetupFn, SetupFnWithChild,
};
use reactivity::let_effect;
use util::use_mut;

#[derive(Debug, Default)]
pub struct BlockProp<'a> {
    pub layout: Option<Pin<&'a StateRefCell<Style>>>,
    pub color: Option<Pin<&'a StateCell<u32>>>,
}

pub fn block<'a, Child: SetupFn<'a>>(prop: BlockProp<'a>) -> impl SetupFnWithChild<'a, Child> {
    with_children::<Child, _>(move |child| {
        wrap_element(Style::DEFAULT, Block::new(), move |ui| async move {
            let id = ui.current_id();

            let_effect!(use_mut::<Block>(
                ui,
                id,
                |mut element| {
                    if let Some(color) = prop.color {
                        element.paint.set_color(Color::new(color.get($)));
                    }
                }
            ));

            let_effect!({
                if let Some(layout) = prop.layout {
                    ui.set_style(id, layout.get($).clone());
                }
            });

            child.show(ui).await;
            pending::<()>().await;
        })
    })
}

#[derive(Debug)]
#[non_exhaustive]
struct Block {
    pub paint: Paint,
    pub border_radius: [Point; 4],
}

impl Block {
    pub fn new() -> Self {
        Self {
            paint: Paint::new(Color4f::from_bytes_rgba(0), None),
            border_radius: [Point::new(0.0, 0.0); 4],
        }
    }
}

impl Element for Block {
    fn draw(self: Pin<&Self>, canvas: &Canvas, layout: &Layout) {
        let paint = &self.paint;
        if paint.nothing_to_draw() {
            return;
        }

        let rect = Rect::new(0.0, 0.0, layout.size.width, layout.size.height);

        let border_radius = &self.border_radius;
        if !border_radius.iter().all(|radius| radius.is_zero()) {
            canvas.draw_rrect(RRect::new_rect_radii(rect, border_radius), paint);
        } else {
            canvas.draw_rect(rect, paint);
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}
