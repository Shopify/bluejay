use crate::validation::SelectionsAreValid;
use crate::{
    builtin_scalar::{builtin_scalar_type, scalar_is_reference},
    map_parser_errors,
    names::{module_ident, type_ident},
    Config, DocumentInput,
};
use bluejay_core::{
    definition::{
        prelude::*, BaseOutputTypeReference, OutputTypeReference, SchemaDefinition,
        TypeDefinitionReference,
    },
    executable::{
        Field, FragmentDefinition, FragmentSpread, InlineFragment, OperationDefinition, Selection,
        SelectionReference, SelectionSet,
    },
    OperationType,
};
use bluejay_parser::ast::executable::ExecutableDocument;
use bluejay_validator::executable::{BuiltinRulesValidator, Cache, Validator};
use std::collections::HashSet;
use syn::{parse::Parse, parse2, parse_quote};

mod field_selection;
mod union_selection;

use field_selection::{
    fields_and_definitions, generate_interface_type_definition, generate_object_type_definition,
    named_fields, nested_module,
};
use union_selection::{generate_union_type_definition, inline_fragments_and_definitions};

struct Input {
    query: DocumentInput,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let query = input.parse()?;
        Ok(Self { query })
    }
}

#[derive(Clone)]
pub(crate) struct Context<'a, S: SchemaDefinition> {
    name: &'a str,
    enum_variant: Option<&'a str>,
    config: &'a Config<'a, S>,
    executable_document: &'a ExecutableDocument<'a>,
    depth: usize,
}

impl<'a, S: SchemaDefinition> Context<'a, S> {
    pub(crate) fn new(
        name: &'a str,
        config: &'a Config<'a, S>,
        executable_document: &'a ExecutableDocument<'a>,
    ) -> Self {
        Self {
            name,
            enum_variant: None,
            config,
            executable_document,
            depth: 0,
        }
    }

    pub(crate) fn config(&self) -> &'a Config<'a, S> {
        self.config
    }

    pub(crate) fn schema_definition(&self) -> &'a S {
        self.config.schema_definition()
    }

    pub(crate) fn name(&self) -> &'a str {
        self.name
    }

    pub(crate) fn enum_variant(&self) -> Option<&'a str> {
        self.enum_variant
    }

    pub(crate) fn dive(&self, new_name: &'a str) -> Self {
        let Self {
            config,
            executable_document,
            depth,
            enum_variant,
            ..
        } = self;
        Self {
            name: new_name,
            enum_variant: None,
            config,
            executable_document,
            depth: depth + 1 + usize::from(enum_variant.is_some()),
        }
    }

    pub(crate) fn with_variant(&self, enum_variant: &'a str) -> Self {
        let Self {
            name,
            config,
            executable_document,
            depth,
            ..
        } = self;
        Self {
            name,
            enum_variant: Some(enum_variant),
            config,
            executable_document,
            depth: *depth,
        }
    }

    pub(crate) fn prefix_for_contained_type(&self) -> impl Iterator<Item = syn::Ident> {
        std::iter::once(module_ident(self.name)).chain(self.enum_variant.map(module_ident))
    }

    fn prefix_for_root(&self) -> impl Iterator<Item = syn::Token![super]> {
        std::iter::repeat(Default::default()).take(self.depth + 1)
    }

    fn prefix_for_fragment(&self) -> impl Iterator<Item = syn::Token![super]> {
        std::iter::repeat(Default::default()).take(self.depth)
    }

    pub(crate) fn type_for_output_type(
        &self,
        ty: OutputTypeReference<S::OutputType>,
        field: &impl Field,
    ) -> syn::TypePath {
        match ty {
            OutputTypeReference::Base(base, required) => {
                let inner = self.type_for_base_output_type(base, field);
                if required {
                    inner
                } else {
                    crate::types::option(inner)
                }
            }
            OutputTypeReference::List(inner, required) => {
                let inner_ty = crate::types::vec(
                    self.type_for_output_type(inner.as_ref(self.schema_definition()), field),
                );
                if required {
                    inner_ty
                } else {
                    crate::types::option(inner_ty)
                }
            }
        }
    }

    fn type_for_base_output_type(
        &self,
        ty: BaseOutputTypeReference<S::OutputType>,
        field: &impl Field,
    ) -> syn::TypePath {
        match ty {
            BaseOutputTypeReference::BuiltinScalar(bstd) => {
                builtin_scalar_type(bstd, self.config())
            }
            BaseOutputTypeReference::CustomScalar(cstd) => {
                let ident = type_ident(cstd.name());
                let lifetime: Option<syn::Generics> = self
                    .config
                    .custom_scalar_borrows(cstd)
                    .then(|| parse_quote! { <'a> });
                let prefix = self.prefix_for_root();
                parse_quote! { #(#prefix::)* #ident #lifetime }
            }
            BaseOutputTypeReference::Enum(etd) => {
                let ident = type_ident(etd.name());
                let prefix = self.prefix_for_root();
                parse_quote! { #(#prefix::)* #ident }
            }
            BaseOutputTypeReference::Interface(itd) => {
                self.field_selection_type(field, itd.fields_definition())
            }
            BaseOutputTypeReference::Object(otd) => {
                self.field_selection_type(field, otd.fields_definition())
            }
            BaseOutputTypeReference::Union(utd) => {
                let selection_set = field.selection_set().unwrap();

                self.fragment_type(selection_set).unwrap_or_else(|| {
                    let prefix = self.prefix_for_contained_type();
                    let type_ident = type_ident(field.response_name());
                    let inline_fragments_and_definitions =
                        inline_fragments_and_definitions(utd, selection_set, self);
                    let lifetime: Option<syn::Generics> = self
                        .inline_fragments_contains_reference_types(
                            &inline_fragments_and_definitions,
                            &mut HashSet::new(),
                        )
                        .then(|| parse_quote! { <'a> });

                    parse_quote! { #(#prefix::)*#type_ident #lifetime }
                })
            }
        }
    }

    fn fragment_type(&self, selection_set: &impl SelectionSet) -> Option<syn::TypePath> {
        fragment_spread(selection_set).map(|fragment_spread| {
            let fragment_ident = type_ident(fragment_spread.name());
            let lifetime = self.lifetime_for_fragment_definition(fragment_spread.name());
            let prefix = self.prefix_for_fragment();
            parse_quote! { #(#prefix::)* #fragment_ident #lifetime }
        })
    }

    fn field_selection_type(
        &self,
        field: &impl Field,
        fields_definition: &S::FieldsDefinition,
    ) -> syn::TypePath {
        let selection_set = field.selection_set().unwrap();

        self.fragment_type(selection_set).unwrap_or_else(|| {
            let prefix = self.prefix_for_contained_type();
            let type_ident = type_ident(field.response_name());
            let fields_and_definitions = fields_and_definitions(selection_set, fields_definition);
            let lifetime = self.lifetime(&fields_and_definitions);

            parse_quote! { #(#prefix::)*#type_ident #lifetime }
        })
    }

    fn lifetime_for_fragment_definition(&self, name: &str) -> Option<syn::Generics> {
        let fragment_definition = self
            .executable_document
            .fragment_definitions()
            .iter()
            .find(|fd| fd.name() == name)
            .unwrap();
        // TODO: handle non-object types
        let fields_definition = self
            .config
            .schema_definition()
            .get_type_definition(fragment_definition.type_condition().named_type().as_ref())
            .unwrap()
            .fields_definition()
            .unwrap();
        let fields_and_definitions =
            fields_and_definitions(fragment_definition.selection_set(), fields_definition);

        self.lifetime(&fields_and_definitions)
    }

    fn lifetime(
        &self,
        fields_and_field_definitions: &[(&impl Field, &S::FieldDefinition)],
    ) -> Option<syn::Generics> {
        (self.selection_set_contains_reference_types(
            fields_and_field_definitions,
            &mut HashSet::new(),
        ))
        .then(|| parse_quote! { <'a> })
    }

    fn selection_set_contains_reference_types(
        &self,
        fields_and_field_definitions: &[(&'a impl Field, &'a S::FieldDefinition)],
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        fields_and_field_definitions
            .iter()
            .any(|(f, fd)| self.contains_reference_types(*f, *fd, visited))
    }

    fn inline_fragments_contains_reference_types(
        &self,
        inline_fragments_and_object_type_definitions: &[(
            &'a impl InlineFragment,
            &'a S::ObjectTypeDefinition,
        )],
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        inline_fragments_and_object_type_definitions
            .iter()
            .any(|(inline_fragment, otd)| {
                let fields_and_definitions = fields_and_definitions(
                    inline_fragment.selection_set(),
                    otd.fields_definition(),
                );
                self.selection_set_contains_reference_types(&fields_and_definitions, visited)
            })
    }

    fn contains_reference_types(
        &self,
        field: &'a impl Field,
        field_definition: &'a S::FieldDefinition,
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        let ty = field_definition
            .r#type()
            .as_ref(self.schema_definition())
            .base(self.schema_definition());
        if !self.config.borrow() || !visited.insert(ty.name()) {
            return false;
        }

        match ty {
            BaseOutputTypeReference::BuiltinScalar(bstd) => scalar_is_reference(bstd),
            BaseOutputTypeReference::CustomScalar(cstd) => self.config.custom_scalar_borrows(cstd),
            BaseOutputTypeReference::Enum(_) => false,
            BaseOutputTypeReference::Interface(itd) => {
                let fields_and_definitions =
                    fields_and_definitions(field.selection_set().unwrap(), itd.fields_definition());

                self.selection_set_contains_reference_types(&fields_and_definitions, visited)
            }
            BaseOutputTypeReference::Object(otd) => {
                let fields_and_definitions =
                    fields_and_definitions(field.selection_set().unwrap(), otd.fields_definition());

                self.selection_set_contains_reference_types(&fields_and_definitions, visited)
            }
            BaseOutputTypeReference::Union(utd) => {
                let inline_fragments_and_definitions =
                    inline_fragments_and_definitions(utd, field.selection_set().unwrap(), self);

                self.inline_fragments_contains_reference_types(
                    &inline_fragments_and_definitions,
                    visited,
                )
            }
        }
    }
}

pub(crate) fn generate_executable_definition<S: SchemaDefinition>(
    config: &Config<S>,
    configuration: proc_macro2::TokenStream,
) -> syn::Result<Vec<syn::Item>> {
    let Input { query } = parse2(configuration)?;

    let contents = query.read_to_string()?;

    let executable_document = ExecutableDocument::parse(&contents)
        .map_err(|errors| map_parser_errors(&query, &contents, errors))?;
    let validation_cache = Cache::new(&executable_document, config.schema_definition());
    let validation_errors: Vec<_> = BuiltinRulesValidator::validate(
        &executable_document,
        config.schema_definition(),
        &validation_cache,
    )
    .collect();
    if !validation_errors.is_empty() {
        return Err(map_parser_errors(&query, &contents, validation_errors));
    }
    let validation_errors: Vec<_> = Validator::<_, _, SelectionsAreValid<_, _>>::validate(
        &executable_document,
        config.schema_definition(),
        &validation_cache,
    )
    .collect();
    if !validation_errors.is_empty() {
        return Err(map_parser_errors(&query, &contents, validation_errors));
    }

    Ok(executable_document
        .operation_definitions()
        .iter()
        .flat_map(|operation_definition| {
            let context = Context::new(
                operation_definition.as_ref().name().unwrap_or("Root"),
                config,
                &executable_document,
            );
            generate_operation_definition(operation_definition, context)
        })
        .chain(
            executable_document
                .fragment_definitions()
                .iter()
                .flat_map(|fragment_definition| {
                    let context = Context::new(
                        fragment_definition.name().as_ref(),
                        config,
                        &executable_document,
                    );
                    generate_fragment_definition(fragment_definition, context)
                }),
        )
        .collect())
}

fn generate_operation_definition<S: SchemaDefinition>(
    operation_definition: &impl OperationDefinition,
    context: Context<S>,
) -> Vec<syn::Item> {
    let schema_definition = context.config.schema_definition();
    let operation_definition = operation_definition.as_ref();
    let object_type_definition = match operation_definition.operation_type() {
        OperationType::Query => schema_definition.query(),
        OperationType::Mutation => schema_definition.mutation().unwrap(),
        OperationType::Subscription => schema_definition.subscription().unwrap(),
    };

    generate_object_type_definition(
        object_type_definition,
        operation_definition.selection_set(),
        context,
    )
}

fn generate_fragment_definition<S: SchemaDefinition>(
    fragment_definition: &impl FragmentDefinition,
    context: Context<S>,
) -> Vec<syn::Item> {
    let selection_set = fragment_definition.selection_set();
    match context
        .config()
        .schema_definition()
        .get_type_definition(fragment_definition.type_condition())
        .unwrap()
    {
        TypeDefinitionReference::Object(otd) => {
            generate_object_type_definition(otd, selection_set, context)
        }
        TypeDefinitionReference::Union(utd) => {
            generate_union_type_definition(utd, selection_set, context)
        }
        TypeDefinitionReference::Interface(itd) => {
            generate_interface_type_definition(itd, selection_set, context)
        }
        _ => unreachable!(),
    }
}

fn fragment_spread<S: SelectionSet>(
    selection_set: &S,
) -> Option<&<S::Selection as Selection>::FragmentSpread> {
    selection_set.iter().find_map(|selection| {
        if let SelectionReference::FragmentSpread(fs) = selection.as_ref() {
            Some(fs)
        } else {
            None
        }
    })
}
