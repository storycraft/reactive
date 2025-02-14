mod effect;

use effect::EffectDef;

#[proc_macro]
pub fn let_effect(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = EffectDef::new(item.into());
    effect::gen(input).into()
}
