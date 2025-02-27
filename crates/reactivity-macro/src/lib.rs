mod effect;

use effect::EffectDef;

#[proc_macro]
/// Create pinned local borrowed effect on the scope.
/// 
/// # Safety
/// Due to lack of drop guarantee in Rust,
/// it can be unsound if the effect is leaked before other depending states drop
/// unless drop guarantee is available in the future.
/// 
/// it is unsafe to manually poll boxed future containing effects and leaking it.
/// You can't have undefined behaviour by using well defined executors(like tokio or async_std) and future combinators
pub fn let_effect(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = EffectDef::new(item.into());
    effect::gen(input).into()
}
