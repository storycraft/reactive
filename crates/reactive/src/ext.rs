use crate::{SetupFn, Ui};
use core::{future::pending, pin::Pin};
use reactive_tree::tree::action::TreeActionExt;
use reactivity::let_effect;
use reactivity_winit::state::StateCell;

mod __private {
    pub trait Extract {
        type T;
    }

    impl<F: FnOnce() -> T, T> Extract for F {
        type T = T;
    }
}

type Never = <fn() -> ! as __private::Extract>::T;

pub fn rotation_z(rotation_z: Pin<&StateCell<f32>>) -> impl SetupFn<Output = Never> {
    async move |ui: Ui| {
        let_effect!({
            ui.with_tree_mut(|tree|{
                tree.transform_mut(ui.current_id()).rotation.z = rotation_z.get($);
            });
            ui.request_redraw();
        });
        pending().await
    }
}
