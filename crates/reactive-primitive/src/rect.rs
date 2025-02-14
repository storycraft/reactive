mod element;

use crate::util::create_wire_macro;
use bon::Builder;
use element::RectElement;
use core::pin::Pin;
use palette::Srgba;
use reactive::{
    create_element,
    reactivity_winit::state::{StateCell, StateRefCell},
    skia_safe, taffy,
    window::ui::Ui,
    SetupFn, WithChild,
};

#[derive(Builder)]
pub struct Rect<'a> {
    layout: Option<Pin<&'a StateRefCell<taffy::Style>>>,
    #[builder(default)]
    fill: Fill<'a>,
    border_color: Option<Pin<&'a StateCell<Srgba>>>,
    border_thickness: Option<Pin<&'a StateCell<f32>>>,
}

#[derive(Builder, Default)]
pub struct Fill<'a> {
    color: Option<Pin<&'a StateCell<Srgba>>>,
}

impl<'a> Rect<'a> {
    fn show<Child: SetupFn + 'a>(self, child: Child) -> impl SetupFn<Output = Child::Output> + 'a {
        create_element(
            RectElement::new(),
            taffy::Style::DEFAULT,
            move |ui: Ui| async move {
                create_wire_macro!(wire, ui);

                wire!(layout = self.layout => {
                    ui.set_style(layout.get($).clone());
                });

                wire!(element: RectElement, color = self.fill.color => {
                    let color = color.get($);
                    element.fill_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                wire!(element: RectElement, color = self.border_color => {
                    let color = color.get($);
                    element.stroke_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                wire!(element: RectElement, thickness = self.border_thickness => {
                    element.stroke_paint.set_stroke_width(thickness.get($));
                });

                child.show(ui.clone()).await
            },
        )
    }
}

impl<'a, Child> WithChild<Child> for Rect<'a>
where
    Child: SetupFn + 'a,
{
    type Output = Child::Output;

    fn child(self, child: Child) -> impl SetupFn<Output = Self::Output> {
        self.show(child)
    }
}
