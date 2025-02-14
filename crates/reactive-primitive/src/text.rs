mod element;

use core::pin::Pin;

use crate::util::{create_wire_macro, use_mut};
use bon::Builder;
use element::TextElement;
use palette::Srgba;
use reactive::{
    create_element,
    reactivity_winit::state::{StateCell, StateRefCell},
    skia_safe, taffy,
    window::ui::Ui,
    SetupFn, WithChild,
};

#[derive(Builder)]
pub struct Text<'a> {
    content: Option<Pin<&'a StateRefCell<String>>>,
    color: Option<Pin<&'a StateCell<Srgba>>>,
    stroke_color: Option<Pin<&'a StateCell<Srgba>>>,
    size: Option<Pin<&'a StateCell<f32>>>,
}

impl<'a> Text<'a> {
    fn show<Child: SetupFn + 'a>(self, child: Child) -> impl SetupFn<Output = Child::Output> + 'a {
        create_element(
            TextElement::new(),
            taffy::Style::DEFAULT,
            move |ui: Ui| async move {
                let id = ui.current_id();

                create_wire_macro!(wire, ui, id);

                wire!(text = self.content => {
                    use_mut(&ui, id, |mut element: Pin<&mut TextElement>| {
                        let text = &*text.get($);
                        let font = element.font.get_or_insert_with(|| skia_safe::Font::from_typeface(
                            skia_safe::FontMgr::new()
                                .legacy_make_typeface(None, skia_safe::FontStyle::normal()).unwrap(),
                                self.size.map(|cell| cell.get_untracked())
                        ));

                        element.blob = skia_safe::TextBlob::from_str(
                            text,
                            font
                        );
                    });

                    ui.request_layout();
                });

                wire!(element: TextElement, size = self.size => {
                    let size = size.get($);
                    if let Some(font) = element.font.as_mut() {
                        font.set_size(size);
                    }
                });

                wire!(element: TextElement, color = self.color => {
                    let color = color.get($);
                    element.fill_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                wire!(element: TextElement, color = self.stroke_color => {
                    let color = color.get($);
                    element.stroke_paint.set_color4f(
                        skia_safe::Color4f::new(color.red, color.green, color.blue, color.alpha),
                        None
                    );
                });

                child.show(ui.clone()).await
            },
        )
    }
}

impl<'a, Child> WithChild<Child> for Text<'a>
where
    Child: SetupFn + 'a,
{
    type Output = Child::Output;

    fn child(self, child: Child) -> impl SetupFn<Output = Self::Output> {
        self.show(child)
    }
}
