use syn::{
    parse::{Parse, ParseStream},
    Ident, Token, Type,
};

pub struct Prop {
    pub ident: Ident,
    pub ty: Type,
}

impl Parse for Prop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = Ident::parse(input)?;
        input.parse::<Token![:]>()?;
        let ty = Type::parse(input)?;

        Ok(Self { ident, ty })
    }
}
