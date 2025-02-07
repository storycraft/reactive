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
    list_ident: &'a Ident,
    bindings: usize,
}

impl<'a> BindingGen<'a> {
    fn check(&mut self, buf: &mut TokenStream, stream: TokenStream) {
        let binding_list = self.list_ident;

        for tt in stream {
            match tt {
                TokenTree::Punct(punct) if punct.as_char() == '$' => {
                    let index = self.bindings;
                    self.bindings += 1;

                    buf.extend(quote!(
                    unsafe {
                        ::core::pin::Pin::new_unchecked(&#binding_list[#index])
                    }));
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
            list_ident: ident,
            bindings: 0,
        };

        let mut buf = TokenStream::new();
        this.check(&mut buf, stream);

        (this.bindings, buf)
    }
}

pub fn gen_effect(EffectDef { closure }: EffectDef) -> TokenStream {
    let hidden = Ident::new("_effect", Span::mixed_site());

    let (len, tokens) = BindingGen::transform(&hidden, closure);

    quote!(
        let #hidden = ::reactive::__private::bindings::<#len>();
        let #hidden = &mut #tokens;
        let #hidden = ::core::pin::pin!(
            ::reactive::__private::Effect::new(#hidden)
        );
        ::reactive::__private::Effect::init(#hidden);
    )
}
