use itertools::{Either, Itertools};
use quote::{ToTokens, TokenStreamExt};
use syn::{parse::Parse, spanned::Spanned};

mod kw {
    syn::custom_keyword!(borrow);
    syn::custom_keyword!(enums_as_str);
    syn::custom_keyword!(custom_scalar_overrides);
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
            .map_err(|err| syn::Error::new(filename.span(), format!("{err}")))
    }
}

pub(crate) struct CustomScalarOverride {
    graphql_path_token: syn::LitStr,
    pub(crate) graphql_path: Vec<String>,
    type_token: syn::Type,
    pub(crate) borrows: bool,
}

impl Parse for CustomScalarOverride {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let graphql_path_token = input.parse()?;
        let graphql_path = Self::graphql_path(&graphql_path_token);
        input.parse::<syn::Token![=>]>()?;
        let type_token = input.parse()?;

        let borrows = Self::type_borrows(&type_token)?;

        Ok(Self {
            graphql_path_token,
            graphql_path,
            type_token,
            borrows,
        })
    }
}

impl CustomScalarOverride {
    /// Parses the braced map that is the value of `custom_scalar_overrides`.
    pub(crate) fn parse_all(
        input: syn::parse::ParseStream,
    ) -> syn::Result<syn::punctuated::Punctuated<Self, syn::Token![,]>> {
        let content;
        syn::braced!(content in input);
        syn::punctuated::Punctuated::parse_terminated(&content)
    }

    /// Returns `overrides`, or an error for each whose path `is_custom_scalar_path` rejects or that borrows without the
    /// `borrow` option.
    pub(crate) fn validate_all(
        overrides: Option<syn::punctuated::Punctuated<Self, syn::Token![,]>>,
        borrow: bool,
        path_error: &str,
        is_custom_scalar_path: impl Fn(&[String]) -> bool,
    ) -> syn::Result<Vec<Self>> {
        let (valid, errors): (Vec<_>, Vec<syn::Error>) =
            overrides.into_iter().flatten().partition_map(|c| {
                if !is_custom_scalar_path(&c.graphql_path) {
                    Either::Right(syn::Error::new(c.graphql_path_token.span(), path_error))
                } else if c.borrows && !borrow {
                    Either::Right(syn::Error::new(
                        c.type_token.span(),
                        "Custom scalar overrides must not borrow if the `borrow` option is not enabled",
                    ))
                } else {
                    Either::Left(c)
                }
            });

        match errors.into_iter().reduce(|mut acc, error| {
            acc.combine(error);
            acc
        }) {
            Some(error) => Err(error),
            None => Ok(valid),
        }
    }

    fn graphql_path(lit_str: &syn::LitStr) -> Vec<String> {
        lit_str.value().split('.').map(|s| s.to_string()).collect()
    }

    fn type_borrows(ty: &syn::Type) -> syn::Result<bool> {
        let path = match ty {
            syn::Type::Path(path) => path,
            // allow the `()` type
            syn::Type::Tuple(tuple) if tuple.elems.is_empty() => return Ok(false),
            _ => {
                return Err(syn::Error::new(
                    ty.span(),
                    "Unsupported type for custom scalar overrides",
                ));
            }
        };

        let Some(last_segment) = path.path.segments.last() else {
            return Err(syn::Error::new(
                path.span(),
                "Path must have at least one segment",
            ));
        };

        let path_arguments = match &last_segment.arguments {
            syn::PathArguments::None => return Ok(false),
            syn::PathArguments::AngleBracketed(bracketed) => bracketed,
            syn::PathArguments::Parenthesized(parenthesized) => {
                return Err(syn::Error::new(
                    parenthesized.span(),
                    "Paths for custom scalar overrides must not contain parenthesized generic arguments",
                ));
            }
        };

        if path_arguments.args.len() != 1
            || !matches!(
                path_arguments.args.first(),
                Some(syn::GenericArgument::Lifetime(lifetime)) if lifetime.ident != "'a"
            )
        {
            return Err(syn::Error::new(
                ty.span(),
                "Paths for custom scalar overrides with generic arguments must contain a single lifetime parameter 'a",
            ));
        }

        Ok(true)
    }

    pub(crate) fn r#type(&self) -> &syn::Type {
        &self.type_token
    }
}

pub struct Input {
    pub(crate) schema: DocumentInput,
    pub borrow: Option<syn::LitBool>,
    pub enums_as_str: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]>,
    pub(crate) custom_scalar_overrides:
        Option<syn::punctuated::Punctuated<CustomScalarOverride, syn::Token![,]>>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let schema: DocumentInput = input.parse()?;

        let mut borrow: Option<syn::LitBool> = None;
        let mut enums_as_str = None;
        let mut custom_scalar_overrides = None;

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
            } else if lookahead.peek(kw::custom_scalar_overrides) {
                parse_key_value_with(
                    input,
                    &mut custom_scalar_overrides,
                    CustomScalarOverride::parse_all,
                )?;
            } else {
                return Err(lookahead.error());
            }
        }

        let enums_as_str = enums_as_str.unwrap_or_default();

        Ok(Self {
            schema,
            borrow,
            enums_as_str,
            custom_scalar_overrides,
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
            format!("Duplicate entry for `{key}`"),
        ));
    }

    input.parse::<syn::Token![=]>()?;
    *value = Some(parser(input)?);
    Ok(())
}
