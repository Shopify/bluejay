use bluejay_core::definition::{ScalarTypeDefinition, SchemaDefinition, TypeDefinitionReference};
use bluejay_parser::{
    ast::{
        definition::{DefinitionDocument, SchemaDefinition as ParserSchemaDefinition},
        Parse,
    },
    Error as ParserError,
};
use bluejay_validator::definition::BuiltinRulesValidator;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{parse_macro_input, spanned::Spanned};

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
use input::{Codec, DocumentInput, Input};
use input_object_type_definition::generate_input_object_type_definition;

pub(crate) struct Config<'a, S: SchemaDefinition> {
    borrow: bool,
    schema_definition: &'a S,
    custom_scalar_borrows: HashMap<String, bool>,
    codec: Codec,
}

impl<'a, S: SchemaDefinition> Config<'a, S> {
    pub(crate) fn schema_definition(&self) -> &'a S {
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

    pub(crate) fn codec(&self) -> Codec {
        self.codec
    }
}

fn generate_schema(input: Input, module: &mut syn::ItemMod) -> syn::Result<()> {
    let Input {
        ref schema,
        borrow,
        codec,
    } = input;

    if borrow && codec == Codec::Miniserde {
        return Err(syn::Error::new(
            module.span(),
            "Cannot borrow with miniserde codec",
        ));
    }

    let schema_contents = schema.read_to_string()?;

    let definition_document: DefinitionDocument = DefinitionDocument::parse(&schema_contents)
        .map_err(|errors| map_parser_errors(schema, &schema_contents, errors))?;
    let schema_definition = ParserSchemaDefinition::try_from(&definition_document)
        .map_err(|errors| map_parser_errors(schema, &schema_contents, errors))?;
    let schema_errors: Vec<_> = BuiltinRulesValidator::validate(&schema_definition).collect();
    if !schema_errors.is_empty() {
        return Err(map_parser_errors(schema, &schema_contents, schema_errors));
    }

    let custom_scalar_borrows = custom_scalar_borrows(module, &schema_definition, borrow)?;

    let config = Config {
        schema_definition: &schema_definition,
        borrow,
        custom_scalar_borrows,
        codec,
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
    schema_definition: &impl SchemaDefinition,
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

fn process_module_items<S: SchemaDefinition>(
    config: &Config<S>,
    items: Vec<syn::Item>,
) -> syn::Result<Vec<syn::Item>> {
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

fn process_module_item<S: SchemaDefinition>(
    config: &Config<S>,
    item: syn::Item,
) -> syn::Result<syn::Item> {
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
    span: &impl syn::spanned::Spanned,
    schema_contents: &str,
    errors: impl IntoIterator<Item = E>,
) -> syn::Error {
    syn::Error::new(
        span.span(),
        ParserError::format_errors(schema_contents, errors),
    )
}

/// Generates Rust types from GraphQL schema definitions and queries.
///
/// ### Arguments
///
/// **Positional:**
///
/// 1. String literal with path to the file containing the schema definition. If relative, should be with respect to
/// the project root (wherever `Cargo.toml` is located). Alternatively, include the schema contents enclosed in square
/// brackets.
///
/// **Optional keyword:**
///
/// _borrow_: Boolean literal indicating whether the generated types should borrow where possible. Defaults to `false`.
/// When `true`, deserializing must be done from a string as a opposed to `serde_json::Value` or a reader.
///
/// ### Trait implementations
///
/// By default, will implement `PartialEq`, `Eq`, `Clone`, and `Debug` for all types. Will implement `Copy` for enums.
/// For types corresponding to values returned from queries, `serde::Deserialize` is implemented. For types that would
/// be arguments to a query, `serde::Serialize` is implemented.
///
/// ### Usage
///
/// Must be used with a module. Inside the module, type aliases must be defined for any custom scalars in the schema.
/// To use a query, define a module within the aforementioned module, and annotate it with
/// `#[query("path/to/query.graphql")]`, where the argument is a string literal path to the query document, or the
/// query contents enclosed in square brackets.
///
/// ### Naming
///
/// To generate idiomatic Rust code, some renaming of types, enum variants, and fields is performed. Types are
/// renamed with `PascalCase`, as are enum variants. Fields are renamed with `snake_case`.
///
/// ### Query restrictions
///
/// In order to keep the type generation code relatively simple, there are some restrictions on the queries that are
/// permitted. This may be relaxed in future versions.
/// * Selection sets on object and interface types must contain either a single fragment spread, or entirely field
/// selections.
/// * Selection sets on union types must contain either a single fragment spread, or both an unaliased `__typename`
/// selection and inline fragments for all or a subset of the objects contained in the union.
///
/// ### Example
/// See top-level documentation of `bluejay-typegen` for an example.
#[proc_macro_attribute]
pub fn typegen(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(attr as Input);
    let mut module = parse_macro_input!(item as syn::ItemMod);

    if let Err(error) = generate_schema(input, &mut module) {
        return error.to_compile_error().into();
    }

    module.to_token_stream().into()
}
