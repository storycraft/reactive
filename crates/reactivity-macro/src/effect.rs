use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::quote;
use syn::Ident;

pub struct EffectDef {
    closure: TokenStream,
}

impl EffectDef {
    pub const fn new(closure: TokenStream) -> Self {
        Self { closure }
    }
}

struct BindingGen<'a> {
    array_ident: &'a Ident,
    bindings: usize,
}

impl<'a> BindingGen<'a> {
    fn check(&mut self, buf: &mut TokenStream, stream: TokenStream) {
        let binding_array = self.array_ident;

        for tt in stream {
            match tt {
                TokenTree::Punct(punct) if punct.as_char() == '$' => {
                    let index = self.bindings;
                    self.bindings += 1;

                    buf.extend(quote!(
                        #binding_array.get_const::<#index>()
                    ));
                }

                TokenTree::Group(group) => {
                    let mut group_buf = TokenStream::new();
                    self.check(&mut group_buf, group.stream());
                    let mut group = Group::new(group.delimiter(), group_buf);
                    group.set_span(group.span());

                    buf.extend([TokenTree::Group(group)]);
                }

                tt => {
                    buf.extend([tt]);
                }
            }
        }
    }

    fn transform(ident: &'a Ident, stream: TokenStream) -> (usize, TokenStream) {
        let mut this = Self {
            array_ident: ident,
            bindings: 0,
        };

        let mut buf = TokenStream::new();
        this.check(&mut buf, stream);

        (this.bindings, buf)
    }
}

pub fn gen(EffectDef { closure }: EffectDef) -> TokenStream {
    let effect = Ident::new("_effect", Span::mixed_site());
    let bindings = Ident::new("bindings", Span::mixed_site());
    let (len, tokens) = BindingGen::transform(&bindings, closure);

    quote!(
        let #effect = ::core::pin::pin!(
            ::reactivity::effect::effect::<#len>(|#bindings| ({ #tokens })())
        );
        ::reactivity::effect::Effect::init(#effect);
    )
}
