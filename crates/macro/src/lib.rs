mod component;
mod expr;
mod gen;
mod prop;

use component::ComponentDef;
use gen::gen_impl;
use syn::parse_macro_input;

#[proc_macro]
pub fn component(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ComponentDef);
    gen_impl(&input).into()
}
