use crate::prop::Prop;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Brace,
    Expr, ExprPath, Ident, Path, Token,
};

pub struct ComponentDef {
    pub props: Punctuated<Prop, Token![,]>,
    pub body: ComponentBody,
}

impl Parse for ComponentDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut props = Punctuated::new();
        input.parse::<Token![|]>()?;
        loop {
            if input.peek(Token![|]) {
                break;
            }
            props.push_value(Prop::parse(input)?);
            if input.peek(Token![|]) {
                break;
            }
            props.push_punct(input.parse::<Token![,]>()?);
        }
        input.parse::<Token![|]>()?;

        let body = {
            let content;
            syn::braced!(content in input);
            ComponentBody::parse(&content)?
        };

        Ok(Self { props, body })
    }
}

pub struct ComponentBody {
    pub states: Punctuated<State, Token![,]>,
    pub children: Punctuated<Component, Token![,]>,
}

impl Parse for ComponentBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut states = Punctuated::new();
        let mut children = Punctuated::new();

        while !input.is_empty() && !input.peek2(Brace) {
            states.push_value(State::parse(input)?);
            if input.peek(Token![,]) {
                states.push_punct(input.parse::<Token![,]>()?);
            }
        }

        while !input.is_empty() {
            children.push_value(Component::parse(input)?);
            if input.peek(Token![,]) {
                children.push_punct(input.parse::<Token![,]>()?);
            }
        }

        Ok(Self { states, children })
    }
}

pub struct Component {
    pub path: Path,
    pub body: ComponentBody,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = Path::parse(input)?;
        let body = {
            let content;
            syn::braced!(content in input);

            ComponentBody::parse(&content)?
        };

        Ok(Self { path, body })
    }
}

pub enum State {
    Unnamed(Expr),
    Named(Ident, Expr),
}

impl Parse for State {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            input.parse::<Token![:]>()?;
            let expr = Expr::parse(input)?;

            Ok(State::Unnamed(expr))
        } else {
            let ident = Ident::parse(input)?;

            let expr = if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                Expr::parse(input)?
            } else {
                Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: Path::from(ident.clone()),
                })
            };

            Ok(State::Named(ident, expr))
        }
    }
}
