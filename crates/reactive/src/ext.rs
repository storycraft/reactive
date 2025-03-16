use crate::{SetupFn, Ui};
use core::{future::pending, pin::Pin};
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

macro_rules! wired {
    ($ty:ty, $name:ident, $expr:expr) => {
        pub fn $name($name: Pin<&StateCell<$ty>>) -> impl SetupFn<Output = Never> {
            async move |ui: Ui| {
                let_effect!({
                    ui.with_mut(|mut el| {
                        el.as_mut().$expr = $name.get($);
                    });
                    ui.request_redraw();
                });

                pending().await
            }
        }
    };
}

wired!(f32, rotation_z, transform_mut().rotation.z);
