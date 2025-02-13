mod element;
pub(crate) mod util;

pub use palette;

use crate::element::BlockElement;
use bon::Builder;
use core::pin::Pin;
use palette::Srgba;
use reactive::{
    create_element,
    reactivity_winit::state::{StateCell, StateRefCell},
    skia_safe, taffy,
    window::ui::Ui,
    SetupFn, WithChild,
};
use reactivity::let_effect;
use util::use_mut;

#[derive(Builder)]
pub struct Block<'a> {
    layout: Option<Pin<&'a StateRefCell<taffy::Style>>>,
    #[builder(default)]
    fill: Fill<'a>,
    #[builder(default)]
    border: Border<'a>,
    #[builder(default)]
    text: Text<'a>,
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

#[derive(Builder, Default)]
pub struct Text<'a> {
    content: Option<Pin<&'a StateCell<String>>>,
}

impl<'a> Block<'a> {
    fn start<Child: SetupFn + 'a>(self, child: Child) -> impl SetupFn<Output = Child::Output> + 'a {
        create_element(
            BlockElement::new(),
            taffy::Style::DEFAULT,
            move |ui: Ui| async move {
                let id = ui.current_id();

                macro_rules! wire_ui {
                    ($name:ident = $prop:expr => $($tt:tt)*) => {
                        let_effect!({
                            if let Some($name) = $prop {
                                ui.request_redraw();

                                $($tt)*
                            }
                        });
                    };

                    ($element:ident, $name:ident = $prop:expr => $($tt:tt)*) => {
                        wire_ui!($name = $prop => {
                            use_mut::<BlockElement>(
                                &ui,
                                id,
                                |#[allow(unused_mut)] mut $element| {
                                    $($tt)*
                                }
                            )
                        });
                    };
                }

                wire_ui!(layout = self.layout => {
                    ui.set_style(id, layout.get($).clone());
                });

                wire_ui!(element, color = self.fill.color => {
                    let color = color.get($);
                    element.fill_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                wire_ui!(element, color = self.border.color => {
                    let color = color.get($);
                    element.stroke_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                wire_ui!(element, thickness = self.border.thickness => {
                    element.stroke_paint.set_stroke_width(thickness.get($));
                });

                wire_ui!(element, text = self.text.content => {
                    // element.text = Some(skia_safe::TextBlobBuilder::new().alloc_run_text(font, count, offset, text_byte_count, bounds))
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
        self.start(child)
    }
}
