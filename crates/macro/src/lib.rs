mod component;
mod effect;
mod expr;
mod gen;
mod prop;

use component::ComponentDef;
use effect::{gen_effect, EffectDef};
use gen::gen_impl;
use syn::parse_macro_input;

#[proc_macro]
pub fn component(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ComponentDef);
    gen_impl(&input).into()
}

#[proc_macro]
pub fn let_effect(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = EffectDef::new(item.into());
    gen_effect(input).into()
}

#[proc_macro_derive(Component, attributes(state))]
pub fn component_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    todo!()
}
