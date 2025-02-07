use core::{array, pin::Pin};

pub use pin_project;

use crate::{binding::Binding, effect::Effect};

/// Used in [`reactive_macro::let_effect`] macro codegen.
/// Not useful for safe uses.
pub fn bindings<const SIZE: usize>() -> [Binding; SIZE] {
    array::from_fn(|_| Binding::new())
}

pub fn init_effect(effect: Pin<&mut Effect>, bindings: Pin<&[Binding]>) {
    // SAFETY: Perform structural pinning
    effect.init(
        bindings
            .into_iter()
            .map(|binding| unsafe { Pin::new_unchecked(binding) }),
    );
}
