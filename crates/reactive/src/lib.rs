mod macros;
pub mod window;
mod ext;

pub use reactive_event as event;
pub use reactive_tree::{ElementId, element};
pub use reactivity_winit;
pub use reactivity_winit::winit;
pub use skia_safe;
pub use taffy;
pub use ext::*;

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

pub fn div<F: SetupFn>(f: F) -> impl SetupFn<Output = F::Output> {
    |ui: Ui| async move {
        let id = ui.append(Element::new(Style::DEFAULT)).unwrap();
        defer!({
            _ = ui.remove(id);
        });

        f.show(ui.sub_ui(id)).await
    }
}
