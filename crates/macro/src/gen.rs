use proc_macro2::TokenStream;
use quote::quote;

use crate::component::ComponentDef;

pub fn gen_impl(def: &ComponentDef) -> TokenStream {
    quote!({
        mod __gen {
            pub struct ImplComponent {}

            impl ImplComponent {
                pub const fn new() -> Self {
                    Self {}
                }
            }

            impl ::reactive::Component<()> for ImplComponent {
                fn update(&mut self, (): ()) {
                    
                }
            } 
        }

        __gen::ImplComponent::new()
    })
}
