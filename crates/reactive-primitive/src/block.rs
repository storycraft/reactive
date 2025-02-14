mod element;

use crate::util::create_wire_macro;
use bon::Builder;
use element::BlockElement;
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
pub struct Block<'a> {
    layout: Option<Pin<&'a StateRefCell<taffy::Style>>>,
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
    fn show<Child: SetupFn + 'a>(self, child: Child) -> impl SetupFn<Output = Child::Output> + 'a {
        create_element(
            BlockElement::new(),
            taffy::Style::DEFAULT,
            move |ui: Ui| async move {
                let id = ui.current_id();

                create_wire_macro!(wire, ui, id);

                wire!(layout = self.layout => {
                    ui.set_style(id, layout.get($).clone());
                });

                wire!(element: BlockElement, color = self.fill.color => {
                    let color = color.get($);
                    element.fill_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                wire!(element: BlockElement, color = self.border.color => {
                    let color = color.get($);
                    element.stroke_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                wire!(element: BlockElement, thickness = self.border.thickness => {
                    element.stroke_paint.set_stroke_width(thickness.get($));
                });

                child.show(ui.clone()).await
            },
        )
    }
}

impl<'a, Child> WithChild<Child> for Block<'a>
where
    Child: SetupFn + 'a,
{
    type Output = Child::Output;

    fn child(self, child: Child) -> impl SetupFn<Output = Self::Output> {
        self.show(child)
    }
}
