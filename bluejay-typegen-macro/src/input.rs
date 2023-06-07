use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(borrow);
}

pub(crate) struct Input {
    pub(crate) schema: syn::LitStr,
    pub(crate) borrow: bool,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let schema: syn::LitStr = input.parse()?;

        let mut borrow: Option<syn::LitBool> = None;

        while !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::borrow) {
                Self::parse_key_value(&input, &mut borrow)?;
            } else {
                return Err(lookahead.error());
            }
        }

        let borrow = borrow.map_or(false, |borrow| borrow.value);

        Ok(Self { schema, borrow })
    }
}

impl Input {
    fn parse_key_value<V: syn::parse::Parse>(
        input: &syn::parse::ParseStream,
        value: &mut Option<V>,
    ) -> syn::Result<()> {
        let key: syn::Ident = input.parse()?;

        if value.is_some() {
            return Err(syn::Error::new(
                key.span(),
                format!("Duplicate entry for `{}`", key),
            ));
        }

        input.parse::<syn::Token![=]>()?;
        *value = Some(input.parse()?);
        Ok(())
    }
}
