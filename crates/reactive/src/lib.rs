mod ext;
mod macros;
pub mod window;

pub use ext::*;
pub use reactive_event as event;
pub use reactive_tree::{ElementId, tree};
pub use reactivity_winit;
pub use reactivity_winit::winit;
pub use skia_safe;
pub use taffy;

use core::future::{Future, pending};
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

#[inline]
pub fn div<F: SetupFn>(f: F) -> impl SetupFn<Output = F::Output> {
    styled_div(Style::DEFAULT, f)
}

pub fn styled_div<F: SetupFn>(style: Style, f: F) -> impl SetupFn<Output = F::Output> {
    |ui: Ui| async move {
        let id = ui.with_tree_mut(|tree| {
            let id = tree.create(style);
            tree.append_child(ui.current_id(), id);
            id
        });
        defer!(ui.with_tree_mut(|tree| tree.destroy(id)));

        f.show(ui.sub_ui(id)).await
    }
}
