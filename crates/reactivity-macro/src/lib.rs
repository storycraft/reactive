mod component;
mod effect;

use effect::{gen_effect, EffectDef};
use quote::quote;

#[proc_macro]
pub fn let_effect(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = EffectDef::new(item.into());
    gen_effect(input).into()
}

#[proc_macro_derive(Component, attributes(state))]
pub fn component_derive(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote!().into()
}
