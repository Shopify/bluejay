use quote::{ToTokens, TokenStreamExt};
use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(borrow);
    syn::custom_keyword!(enums_as_str);
}

pub enum DocumentInput {
    Path(syn::LitStr),
    Dsl {
        bracket: syn::token::Bracket,
        contents: proc_macro2::TokenStream,
    },
}

impl Parse for DocumentInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let contents;
            Ok(Self::Dsl {
                bracket: syn::bracketed!(contents in input),
                contents: contents.parse()?,
            })
        } else {
            input.parse().map(Self::Path)
        }
    }
}

impl ToTokens for DocumentInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Path(path) => tokens.append(path.token()),
            DocumentInput::Dsl { bracket, contents } => {
                bracket.surround(tokens, |tokens| tokens.extend(contents.clone()))
            }
        }
    }
}

impl DocumentInput {
    pub(crate) fn read_to_string_and_path(&self) -> syn::Result<(String, Option<String>)> {
        match self {
            Self::Path(path) => {
                Self::read_file(path).map(|contents| (contents, Some(path.value())))
            }
            Self::Dsl { contents, .. } => Ok((contents.to_string(), None)),
        }
    }

    fn read_file(filename: &syn::LitStr) -> syn::Result<String> {
        let cargo_manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").map_err(|_| syn::Error::new(filename.span(), "Environment variable CARGO_MANIFEST_DIR was not set but is needed to resolve relative paths"))?;
        let base_path = std::path::PathBuf::from(cargo_manifest_dir);

        let file_path = base_path.join(filename.value());

        std::fs::read_to_string(file_path)
            .map_err(|err| syn::Error::new(filename.span(), format!("{}", err)))
    }
}

pub struct Input {
    pub(crate) schema: DocumentInput,
    pub(crate) borrow: bool,
    pub enums_as_str: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let schema: DocumentInput = input.parse()?;

        let mut borrow: Option<syn::LitBool> = None;
        let mut enums_as_str = None;

        while !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::borrow) {
                parse_key_value(input, &mut borrow)?;
            } else if lookahead.peek(kw::enums_as_str) {
                parse_key_value_with(input, &mut enums_as_str, |input| {
                    let content;
                    syn::bracketed!(content in input);
                    syn::punctuated::Punctuated::parse_separated_nonempty(&content)
                })?;
            } else {
                return Err(lookahead.error());
            }
        }

        let borrow = borrow.is_some_and(|borrow| borrow.value);
        let enums_as_str = enums_as_str.unwrap_or_default();

        Ok(Self {
            schema,
            borrow,
            enums_as_str,
        })
    }
}

fn parse_key_value<V: syn::parse::Parse>(
    input: syn::parse::ParseStream,
    value: &mut Option<V>,
) -> syn::Result<()> {
    parse_key_value_with(input, value, syn::parse::Parse::parse)
}

pub(crate) fn parse_key_value_with<V>(
    input: syn::parse::ParseStream,
    value: &mut Option<V>,
    parser: fn(syn::parse::ParseStream<'_>) -> syn::Result<V>,
) -> syn::Result<()> {
    let key: syn::Ident = input.parse()?;

    if value.is_some() {
        return Err(syn::Error::new(
            key.span(),
            format!("Duplicate entry for `{}`", key),
        ));
    }

    input.parse::<syn::Token![=]>()?;
    *value = Some(parser(input)?);
    Ok(())
}
