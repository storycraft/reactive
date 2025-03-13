mod macros;
pub mod window;

pub use reactive_event as event;
pub use reactive_tree::{ElementId, element};
pub use reactivity_winit;
pub use reactivity_winit::winit;
pub use skia_safe;
pub use taffy;

use core::future::{Future, pending};
use reactive_tree::element::Element;
use scopeguard::defer;
use taffy::Style;
use window::ui::Ui;

/// Representation of a functional component.
///
/// This trait is implemented for all `FnOnce(Ui<'a>) -> impl Future + 'a` types.
pub trait SetupFn {
    type Output;

    fn show(self, ui: Ui) -> impl Future<Output = Self::Output>;
}

// For function components without children
impl<F, Fut> SetupFn for F
where
    F: FnOnce(Ui) -> Fut,
    Fut: Future,
{
    type Output = Fut::Output;

    fn show(self, ui: Ui) -> impl Future<Output = Self::Output> {
        self(ui)
    }
}

impl SetupFn for () {
    type Output = ();

    async fn show(self, _: Ui) {
        pending::<()>().await;
    }
}

/// Representation of a functional component with a child
pub trait WithChild<Child> {
    type Output;

    fn child(self, child: Child) -> impl SetupFn<Output = Self::Output>;
}

impl<F, Child, Fut> WithChild<Child> for F
where
    F: FnOnce(Ui, Child) -> Fut,
    Fut: Future,
{
    type Output = Fut::Output;

    fn child(self, child: Child) -> impl SetupFn<Output = Fut::Output> {
        |ui| self(ui, child)
    }
}

#[easy_ext::ext(SetupFnWithChildExt)]
pub impl<F> F
where
    F: WithChild<()>,
{
    fn show(self, ui: Ui) -> impl Future<Output = F::Output> {
        self.child(()).show(ui)
    }
}

pub fn create_element<F: SetupFn>(style: Style, f: F) -> impl SetupFn<Output = F::Output> {
    |ui: Ui| async move {
        let id = ui.append(Element::new(style)).unwrap();
        defer!({
            _ = ui.remove(id);
        });

        f.show(ui.sub_ui(id)).await
    }
}
