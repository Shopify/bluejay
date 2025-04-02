use bluejay_core::{
    definition::{prelude::*, SchemaDefinition, TypeDefinitionReference},
    BuiltinScalarDefinition,
};
use bluejay_parser::{
    ast::{
        definition::{DefinitionDocument, SchemaDefinition as ParserSchemaDefinition},
        Parse,
    },
    Error as ParserError,
};
use bluejay_validator::definition::BuiltinRulesValidator;
use std::collections::{HashMap, HashSet};
use syn::{parse_quote, spanned::Spanned};

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
use enum_type_definition::EnumTypeDefinitionBuilder;
use executable_definition::generate_executable_definition;
use input::DocumentInput;
pub use input::{Codec, Input};
use input_object_type_definition::InputObjectTypeDefinitionBuilder;

pub(crate) struct Config<'a, S: SchemaDefinition> {
    borrow: bool,
    schema_definition: &'a S,
    custom_scalar_borrows: HashMap<String, bool>,
    codec: Codec,
    enums_as_str: HashSet<String>,
}

impl<'a, S: SchemaDefinition> Config<'a, S> {
    pub(crate) fn schema_definition(&self) -> &'a S {
        self.schema_definition
    }

    pub(crate) fn borrow(&self) -> bool {
        self.borrow
    }

    pub(crate) fn custom_scalar_borrows(&self, cstd: &S::CustomScalarTypeDefinition) -> bool {
        *self
            .custom_scalar_borrows
            .get(&names::type_name(cstd.name()))
            .expect("No type alias for custom scalar")
    }

    pub(crate) fn builtin_scalar_borrows(&self, bstd: BuiltinScalarDefinition) -> bool {
        self.borrow && builtin_scalar::scalar_is_reference(bstd)
    }

    pub(crate) fn codec(&self) -> Codec {
        self.codec
    }

    pub(crate) fn enum_as_str(&self, etd: &S::EnumTypeDefinition) -> bool {
        self.enums_as_str.contains(etd.name())
    }
}

pub fn generate_schema(
    input: Input,
    module: &mut syn::ItemMod,
    known_custom_scalar_types: HashMap<String, KnownCustomScalarType>,
) -> syn::Result<()> {
    let Input {
        ref schema,
        borrow,
        codec,
        enums_as_str,
    } = input;

    if borrow && codec == Codec::Miniserde {
        return Err(syn::Error::new(
            module.span(),
            "Cannot borrow with miniserde codec",
        ));
    }

    let (schema_contents, schema_path) = schema.read_to_string_and_path()?;

    let definition_document: DefinitionDocument = DefinitionDocument::parse(&schema_contents)
        .map_err(|errors| {
            map_parser_errors(schema, &schema_contents, schema_path.as_deref(), errors)
        })?;
    let schema_definition =
        ParserSchemaDefinition::try_from(&definition_document).map_err(|errors| {
            map_parser_errors(schema, &schema_contents, schema_path.as_deref(), errors)
        })?;
    let schema_errors: Vec<_> = BuiltinRulesValidator::validate(&schema_definition).collect();
    if !schema_errors.is_empty() {
        return Err(map_parser_errors(
            schema,
            &schema_contents,
            schema_path.as_deref(),
            schema_errors,
        ));
    }

    let custom_scalar_borrows = custom_scalar_borrows(
        module,
        &schema_definition,
        borrow,
        known_custom_scalar_types,
    )?;

    let enums_as_str = validate_enums_as_str(enums_as_str, &schema_definition)?;

    let config = Config {
        schema_definition: &schema_definition,
        borrow,
        custom_scalar_borrows,
        codec,
        enums_as_str,
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
    module: &mut syn::ItemMod,
    schema_definition: &impl SchemaDefinition,
    borrow: bool,
    known_custom_scalar_types: HashMap<String, KnownCustomScalarType>,
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

    let mut custom_scalars: HashMap<String, bool> = type_aliases
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
                #[allow(clippy::map_entry)]
                if custom_scalars.contains_key(&name) {
                    Ok(())
                } else if let Some(known_custom_scalar_type) = known_custom_scalar_types.get(&name)
                {
                    let (ty, lifetime): (_, Option<syn::Generics>) =
                        match known_custom_scalar_type.type_for_borrowed.as_ref() {
                            Some(ty) if borrow => (ty, Some(parse_quote! { <'a> })),
                            _ => (&known_custom_scalar_type.type_for_owned, None),
                        };
                    let ident = quote::format_ident!("{}", name);
                    let alias: syn::ItemType = parse_quote! {
                        type #ident #lifetime = #ty;
                    };
                    if let Some((_, items)) = module.content.as_mut() {
                        items.push(alias.into());
                    }
                    custom_scalars.insert(
                        name,
                        borrow && known_custom_scalar_type.type_for_borrowed.is_some(),
                    );
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

fn validate_enums_as_str(
    enums_as_str: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]>,
    schema_definition: &impl SchemaDefinition,
) -> syn::Result<HashSet<String>> {
    let mut enum_names = HashSet::new();
    enums_as_str.iter().try_for_each(|lit| {
        let name: String = lit.value();
        if matches!(
            schema_definition.get_type_definition(&name),
            Some(TypeDefinitionReference::Enum(_))
        ) {
            if enum_names.insert(name.clone()) {
                Ok(())
            } else {
                Err(syn::Error::new(
                    lit.span(),
                    format!("Duplicate enum definition named {name}"),
                ))
            }
        } else {
            Err(syn::Error::new(
                lit.span(),
                format!("No enum definition named {name}"),
            ))
        }
    })?;
    Ok(enum_names)
}

fn process_module_items<S: SchemaDefinition>(
    config: &Config<S>,
    items: Vec<syn::Item>,
) -> syn::Result<Vec<syn::Item>> {
    config
        .schema_definition
        .type_definitions()
        .filter_map(|type_definition| match type_definition {
            TypeDefinitionReference::Enum(etd) if !config.enum_as_str(etd) => {
                Some(EnumTypeDefinitionBuilder::build(etd, config))
            }
            TypeDefinitionReference::InputObject(iotd) => {
                Some(InputObjectTypeDefinitionBuilder::build(iotd, config))
            }
            _ => None,
        })
        .flatten()
        .map(Ok)
        .chain(
            items
                .into_iter()
                .map(|item| process_module_item(config, item)),
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
    schema_path: Option<&str>,
    errors: impl IntoIterator<Item = E>,
) -> syn::Error {
    syn::Error::new(
        span.span(),
        ParserError::format_errors(schema_contents, schema_path, errors),
    )
}

#[derive(Clone)]
pub struct KnownCustomScalarType {
    pub type_for_owned: syn::Type,
    pub type_for_borrowed: Option<syn::Type>,
}
