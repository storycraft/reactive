use core::array;

pub use pin_project;
pub use reactivity::effect::{Effect, binding::Binding};

/// Used in [`reactive_macro::let_effect`] macro codegen.
/// Not useful for safe uses.
pub fn bindings<const SIZE: usize>() -> [Binding; SIZE] {
    array::from_fn(|_| Binding::new())
}
