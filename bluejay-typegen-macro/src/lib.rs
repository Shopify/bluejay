use bluejay_core::definition::{
    ScalarTypeDefinition, SchemaDefinition as CoreSchemaDefinition, TypeDefinitionReference,
};
use bluejay_parser::{
    ast::definition::{DefinitionDocument, SchemaDefinition},
    Error as ParserError,
};
use bluejay_validator::definition::BuiltinRulesValidator;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{parse_macro_input, spanned::Spanned, LitStr};

mod attributes;
mod builtin_scalar;
mod enum_type_definition;
mod executable_definition;
mod input;
mod input_object_type_definition;
mod names;
mod types;
mod validation;

use attributes::doc_string;
use enum_type_definition::generate_enum_type_definition;
use executable_definition::generate_executable_definition;
use input::Input;
use input_object_type_definition::generate_input_object_type_definition;

pub(crate) struct Config<'a> {
    borrow: bool,
    schema_definition: &'a SchemaDefinition<'a>,
    custom_scalar_borrows: HashMap<String, bool>,
}

impl<'a> Config<'a> {
    pub(crate) fn schema_definition(&self) -> &'a SchemaDefinition<'a> {
        self.schema_definition
    }

    pub(crate) fn borrow(&self) -> bool {
        self.borrow
    }

    pub(crate) fn custom_scalar_borrows(&self, cstd: &impl ScalarTypeDefinition) -> bool {
        *self
            .custom_scalar_borrows
            .get(&names::type_name(cstd.name()))
            .expect("No type alias for custom scalar")
    }
}

fn read_file(filename: &LitStr) -> syn::Result<String> {
    let cargo_manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").map_err(|_| syn::Error::new(filename.span(), "Environment variable CARGO_MANIFEST_DIR was not set but is needed to resolve relative paths"))?;
    let base_path = std::path::PathBuf::from(cargo_manifest_dir);

    let file_path = base_path.join(filename.value());

    std::fs::read_to_string(file_path)
        .map_err(|err| syn::Error::new(filename.span(), format!("{}", err)))
}

fn generate_schema(input: Input, module: &mut syn::ItemMod) -> syn::Result<()> {
    let Input { schema, borrow } = input;

    let schema_contents = read_file(&schema)?;

    let definition_document: DefinitionDocument = DefinitionDocument::parse(&schema_contents)
        .map_err(|errors| map_parser_errors(&schema, &schema_contents, errors))?;
    let schema_definition = SchemaDefinition::try_from(&definition_document)
        .map_err(|errors| map_parser_errors(&schema, &schema_contents, errors))?;
    let schema_errors: Vec<_> = BuiltinRulesValidator::validate(&schema_definition).collect();
    if !schema_errors.is_empty() {
        return Err(map_parser_errors(&schema, &schema_contents, schema_errors));
    }

    let custom_scalar_borrows = custom_scalar_borrows(module, &schema_definition, borrow)?;

    let config = Config {
        schema_definition: &schema_definition,
        borrow,
        custom_scalar_borrows,
    };

    if let Some((_, items)) = module.content.take() {
        let new_items = process_module_items(&config, items)?;
        module.content = Some((syn::token::Brace::default(), new_items));
    } else {
        let new_items = process_module_items(&config, Vec::new())?;
        module.content = Some((syn::token::Brace::default(), new_items));
    }

    if let Some(description) = schema_definition.description() {
        module.attrs.push(doc_string(description));
    }

    Ok(())
}

fn custom_scalar_borrows(
    module: &syn::ItemMod,
    schema_definition: &impl CoreSchemaDefinition,
    borrow: bool,
) -> syn::Result<HashMap<String, bool>> {
    let items = module
        .content
        .as_ref()
        .map(|(_, items)| items.as_slice())
        .unwrap_or_default();

    let type_aliases = items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Type(ty) => Some(ty),
            _ => None,
        })
        .collect::<Vec<_>>();

    type_aliases.iter().try_for_each(|type_alias| {
        let generics = &type_alias.generics;

        if let Some(type_param) = generics.type_params().next() {
            return Err(syn::Error::new(
                type_param.span(),
                "Type aliases for custom scalars must not contain type parameters",
            ));
        }

        if let Some(const_param) = generics.const_params().next() {
            return Err(syn::Error::new(
                const_param.span(),
                "Type aliases for custom scalars must not contain const parameters",
            ));
        }

        if !borrow {
            if let Some(lifetime_param) = generics.lifetimes().next() {
                return Err(syn::Error::new(
                    lifetime_param.span(),
                    "Type aliases for custom scalars cannot contain lifetime parameters when `borrow` is set to true",
                ));
            }
        } else if let Some(lifetime_param) = generics.lifetimes().nth(1) {
            return Err(syn::Error::new(
                lifetime_param.span(),
                "Type aliases for custom scalars must contain at most one lifetime parameter",
            ));
        }

        let name = type_alias.ident.to_string();

        if !schema_definition.type_definitions().any(|type_definition| {
            matches!(type_definition, TypeDefinitionReference::CustomScalar(cstd) if names::type_name(cstd.name()) == name)
        }) {
            return Err(syn::Error::new(
                type_alias.ident.span(),
                format!("No custom scalar definition named {name}"),
            ));
        }

        Ok(())
    })?;

    let custom_scalars: HashMap<String, bool> = type_aliases
        .into_iter()
        .map(|type_alias| {
            (
                type_alias.ident.to_string(),
                type_alias.generics.lifetimes().next().is_some(),
            )
        })
        .collect();

    schema_definition
        .type_definitions()
        .try_for_each(|td| match td {
            TypeDefinitionReference::CustomScalar(cstd) => {
                let name = names::type_name(cstd.name());
                if custom_scalars.contains_key(&name) {
                    Ok(())
                } else {
                    Err(syn::Error::new(
                        module.span(),
                        format!("Missing type alias for custom scalar {name}"),
                    ))
                }
            }
            _ => Ok(()),
        })?;

    Ok(custom_scalars)
}

fn process_module_items(config: &Config, items: Vec<syn::Item>) -> syn::Result<Vec<syn::Item>> {
    config
        .schema_definition
        .type_definitions()
        .filter_map(|type_definition| match type_definition {
            TypeDefinitionReference::Enum(etd) => Some(generate_enum_type_definition(etd)),
            TypeDefinitionReference::InputObject(iotd) => {
                Some(generate_input_object_type_definition(iotd, config))
            }
            _ => None,
        })
        .flatten()
        .map(Ok)
        .chain(
            items
                .into_iter()
                .map(|item| process_module_item(config, item).map(Into::into)),
        )
        .collect()
}

fn process_module_item(config: &Config, item: syn::Item) -> syn::Result<syn::Item> {
    if let syn::Item::Mod(mut module) = item {
        if let Some((attribute, &mut [])) = module.attrs.split_first_mut() {
            if matches!(attribute.style, syn::AttrStyle::Inner(_)) {
                Err(syn::Error::new(
                    attribute.span(),
                    "Expected an outer attribute",
                ))
            } else if let syn::Meta::List(list) = &mut attribute.meta {
                if list.path.is_ident("query") {
                    if !matches!(list.delimiter, syn::MacroDelimiter::Bracket(_)) {
                        let items = generate_executable_definition(
                            config,
                            std::mem::take(&mut list.tokens),
                        )?;
                        module.content = Some((syn::token::Brace::default(), items));
                        module.attrs = Vec::new();
                        Ok(module.into())
                    } else {
                        Err(syn::Error::new(
                            list.delimiter.span().open(),
                            "Expected brackets",
                        ))
                    }
                } else {
                    Err(syn::Error::new(list.path.span(), "Expected `query`"))
                }
            } else {
                Err(syn::Error::new(
                    attribute.meta.span(),
                    "Expected a list meta attribute, e.g. `#[query(...)]`",
                ))
            }
        } else {
            Err(syn::Error::new(
                module.span(),
                "Expected a single `#[query(...)]` attribute",
            ))
        }
    } else if matches!(item, syn::Item::Type(_)) {
        Ok(item)
    } else {
        Err(syn::Error::new(item.span(), "Expected a module"))
    }
}

fn map_parser_errors<E: Into<ParserError>>(
    token: &LitStr,
    schema_contents: &str,
    errors: impl IntoIterator<Item = E>,
) -> syn::Error {
    syn::Error::new(
        token.span(),
        ParserError::format_errors(schema_contents, errors),
    )
}

#[proc_macro_attribute]
pub fn typegen(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // "fn answer() -> u32 { 42 }".parse().unwrap()
    let input = parse_macro_input!(attr as Input);
    let mut module = parse_macro_input!(item as syn::ItemMod);

    if let Err(error) = generate_schema(input, &mut module) {
        return error.to_compile_error().into();
    }

    module.to_token_stream().into()
}
