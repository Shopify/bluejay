//! Intermediate representation of the executable document. This takes the AST of the executable document and converts it into
//! data structures that have baked into them many of the assumptions that are validated by the validator. This allows the type
//! generation to be simpler because we don't need to do so much coercion of the AST while generating the types.

use crate::{
    builtin_scalar::builtin_scalar_type,
    executable_definition::CustomScalarOverride,
    names::{module_ident, type_ident, ANONYMOUS_OPERATION_STRUCT_NAME},
    types, CodeGenerator, Config,
};
use bluejay_core::{
    definition::{prelude::*, BaseOutputTypeReference, OutputTypeReference, SchemaDefinition},
    executable::{
        ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment,
        OperationDefinition, Selection, SelectionReference,
    },
    AsIter, BuiltinScalarDefinition, OperationType,
};
use itertools::Either;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::marker::PhantomData;
use syn::parse_quote;

#[derive(Clone)]
enum PathRoot<'a> {
    Operation { name: Option<&'a str> },
    Fragment { name: &'a str },
}

impl<'a> PathRoot<'a> {
    fn name(&self) -> Option<&'a str> {
        match self {
            PathRoot::Operation { name } => *name,
            PathRoot::Fragment { name } => Some(name),
        }
    }
}

#[derive(Clone)]
struct Path<'a> {
    root: PathRoot<'a>,
    fields: Vec<&'a str>,
}

impl<'a> Path<'a> {
    fn new(root: PathRoot<'a>) -> Self {
        Self {
            root,
            fields: Vec::new(),
        }
    }

    fn with_field(&self, field: &'a str) -> Self {
        let mut clone = self.clone();
        clone.fields.push(field);
        clone
    }
}

pub enum ExecutableType<'a> {
    /// Any part of a query that maps to a Rust struct
    Struct(ExecutableStruct<'a>),
    /// Any part of a query that maps to a Rust enum
    Enum(ExecutableEnum<'a>),
    /// A selection set that contains only a single named fragment spread
    /// Note: the `borrows` field may incorrectly be false during construction, but
    /// will be updated later
    FragmentDefinitionReference { name: &'a str, borrows: bool },
    BuiltinScalar {
        bstd: BuiltinScalarDefinition,
        borrows: bool,
    },
    /// A custom scalar or GraphQL enum
    Leaf {
        /// The type of the leaf. If it is a `syn::Type::Path` and there is not a leading `::`, then
        /// it is relative to the root module of the schema definition.
        r#type: syn::Type,
        borrows: bool,
    },
}

impl<'a> ExecutableType<'a> {
    pub(crate) fn for_executable_document<
        E: ExecutableDocument,
        S: SchemaDefinition,
        C: CodeGenerator,
    >(
        executable_document: &'a E,
        config: &'a Config<'a, S, C>,
        custom_scalar_overrides: Vec<CustomScalarOverride>,
    ) -> Vec<Self> {
        ExecutableDocumentToExecutableTypes::convert(
            executable_document,
            config,
            custom_scalar_overrides,
        )
    }

    fn update_fragment_definition_references_borrow(
        &mut self,
        fragment_definitions_contains_reference_types: &HashMap<&'a str, bool>,
    ) {
        match self {
            Self::Struct(es) => {
                es.fields.iter_mut().for_each(|field| {
                    field
                        .r#type
                        .base_mut()
                        .update_fragment_definition_references_borrow(
                            fragment_definitions_contains_reference_types,
                        );
                });
            }
            Self::Enum(ee) => {
                ee.variants.iter_mut().for_each(|variant| {
                    variant.fields.iter_mut().for_each(|field| {
                        field
                            .r#type
                            .base_mut()
                            .update_fragment_definition_references_borrow(
                                fragment_definitions_contains_reference_types,
                            );
                    });
                });
            }
            Self::FragmentDefinitionReference { name, borrows } => {
                *borrows = fragment_definitions_contains_reference_types[name];
            }
            Self::Leaf { .. } | Self::BuiltinScalar { .. } => {}
        }
    }

    /// whether the type contains any fields that borrow
    pub fn borrows(&self) -> bool {
        match self {
            Self::Leaf { borrows, .. } => *borrows,
            Self::BuiltinScalar { borrows, .. } => *borrows,
            Self::FragmentDefinitionReference { borrows, .. } => *borrows,
            Self::Struct(es) => es.borrows(),
            Self::Enum(ee) => ee.borrows(),
        }
    }
}

pub struct ExecutableStruct<'a> {
    description: Option<&'a str>,
    parent_name: &'a str,
    fields: Vec<ExecutableField<'a>>,
    /// depth within the module for the executable document
    depth: usize,
}

impl ExecutableStruct<'_> {
    /// description of the struct from the schema definition
    pub fn description(&self) -> Option<&str> {
        self.description
    }

    /// name of either the operation, fragment, or field that owns the selection set that this struct represents
    pub fn parent_name(&self) -> &str {
        self.parent_name
    }

    /// fields of the struct
    pub fn fields(&self) -> &[ExecutableField<'_>] {
        &self.fields
    }

    /// whether the struct contains any fields that borrow
    pub fn borrows(&self) -> bool {
        self.fields
            .iter()
            .any(|field| field.r#type.base().borrows())
    }

    /// Computes the type path for a base type, relative to where the struct is defined.
    pub fn compute_base_type(&self, base: &ExecutableType<'_>) -> syn::Type {
        match base {
            ExecutableType::Leaf { r#type, .. } => {
                let type_is_relative_to_schema_module =
                    matches!(r#type, syn::Type::Path(path) if path.path.leading_colon.is_none());
                if type_is_relative_to_schema_module {
                    let prefix = self.prefix_for_schema_definition_module();
                    parse_quote! { #(#prefix::)* #r#type }
                } else {
                    r#type.clone()
                }
            }
            ExecutableType::BuiltinScalar { bstd, borrows } => builtin_scalar_type(*bstd, *borrows),
            ExecutableType::FragmentDefinitionReference { name, borrows } => {
                let prefix = self.prefix_for_executable_document_module();
                let type_ident = type_ident(name);
                let lifetime: Option<syn::Generics> = borrows.then(|| parse_quote! { <'a> });
                parse_quote! { #(#prefix::)* #type_ident #lifetime }
            }
            ExecutableType::Struct(es) => {
                let prefix = module_ident(self.parent_name);
                let type_ident = type_ident(es.parent_name);
                let lifetime: Option<syn::Generics> = es.borrows().then(|| parse_quote! { <'a> });
                parse_quote! { #prefix:: #type_ident #lifetime }
            }
            ExecutableType::Enum(ee) => {
                let prefix = module_ident(self.parent_name);
                let type_ident = type_ident(ee.parent_name);
                let lifetime: Option<syn::Generics> = ee.borrows().then(|| parse_quote! { <'a> });
                parse_quote! { #prefix:: #type_ident #lifetime }
            }
        }
    }

    /// Computes the type path for a type, relative to where the struct is defined.
    pub fn compute_type(&self, r#type: &WrappedExecutableType<'_>) -> syn::Type {
        match r#type {
            WrappedExecutableType::Base(base) => self.compute_base_type(base),
            WrappedExecutableType::Optional(inner) => types::option(self.compute_type(inner)),
            WrappedExecutableType::Vec(inner) => types::vec(self.compute_type(inner)),
        }
    }

    fn prefix_for_schema_definition_module(&self) -> impl Iterator<Item = syn::Token![super]> {
        // root is one level higher than the executable/query module
        std::iter::repeat(Default::default()).take(self.depth + 1)
    }

    fn prefix_for_executable_document_module(&self) -> impl Iterator<Item = syn::Token![super]> {
        std::iter::repeat(Default::default()).take(self.depth)
    }
}

pub struct ExecutableEnum<'a> {
    description: Option<&'a str>,
    parent_name: &'a str,
    variants: Vec<ExecutableStruct<'a>>,
}

impl ExecutableEnum<'_> {
    /// whether the enum contains any variants that borrow
    pub fn borrows(&self) -> bool {
        self.variants.iter().any(|variant| variant.borrows())
    }

    /// description of the enum from the schema definition
    pub fn description(&self) -> Option<&str> {
        self.description
    }

    /// name of either the operation, fragment, or field that owns the selection set that this struct represents
    pub fn parent_name(&self) -> &str {
        self.parent_name
    }

    /// variants of the enum
    pub fn variants(&self) -> &[ExecutableStruct<'_>] {
        &self.variants
    }
}

pub enum WrappedExecutableType<'a> {
    /// a required type, unless wrapped in an `Optional`
    Base(ExecutableType<'a>),
    /// an optional type
    Optional(Box<WrappedExecutableType<'a>>),
    /// a list type, required unless wrapped in an `Optional`
    Vec(Box<WrappedExecutableType<'a>>),
}

impl<'a> WrappedExecutableType<'a> {
    pub fn base(&self) -> &ExecutableType<'a> {
        match self {
            WrappedExecutableType::Base(base) => base,
            WrappedExecutableType::Optional(inner) => inner.base(),
            WrappedExecutableType::Vec(inner) => inner.base(),
        }
    }

    fn base_mut(&mut self) -> &mut ExecutableType<'a> {
        match self {
            WrappedExecutableType::Base(base) => base,
            WrappedExecutableType::Optional(inner) => inner.base_mut(),
            WrappedExecutableType::Vec(inner) => inner.base_mut(),
        }
    }
}

pub struct ExecutableField<'a> {
    description: Option<&'a str>,
    graphql_name: &'a str,
    r#type: WrappedExecutableType<'a>,
}

impl<'a> ExecutableField<'a> {
    /// description of the field from the schema definition
    pub fn description(&self) -> Option<&str> {
        self.description
    }

    /// name of the field
    pub fn graphql_name(&self) -> &str {
        self.graphql_name
    }

    /// type of the field
    pub fn r#type(&self) -> &WrappedExecutableType<'a> {
        &self.r#type
    }
}

struct ExecutableDocumentToExecutableTypes<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    C: CodeGenerator,
> {
    executable_document_type: PhantomData<&'a E>,
    config: &'a Config<'a, S, C>,
    custom_scalar_overrides: Vec<CustomScalarOverride>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, C: CodeGenerator>
    ExecutableDocumentToExecutableTypes<'a, E, S, C>
{
    fn convert(
        executable_document: &'a E,
        config: &'a Config<'a, S, C>,
        custom_scalar_overrides: Vec<CustomScalarOverride>,
    ) -> Vec<ExecutableType<'a>> {
        let instance = Self {
            executable_document_type: PhantomData,
            config,
            custom_scalar_overrides,
        };

        let named_fragment_definition_types = executable_document
            .fragment_definitions()
            .map(|fragment_definition| {
                (
                    fragment_definition.name(),
                    instance.build_fragment_definition(fragment_definition),
                )
            })
            .collect::<BTreeMap<&'a str, ExecutableType<'a>>>();

        // build a map of fragment definitions to whether they contain reference types
        let fragment_definitions_contains_reference_types = named_fragment_definition_types
            .iter()
            .map(|(name, fragment_definition)| {
                (
                    *name,
                    Self::type_borrows(
                        fragment_definition,
                        &named_fragment_definition_types,
                        &mut HashSet::new(),
                    ),
                )
            })
            .collect::<HashMap<&'a str, bool>>();

        let mut types = executable_document
            .operation_definitions()
            .map(|operation_definition| instance.build_operation_definition(operation_definition))
            .chain(named_fragment_definition_types.into_values())
            .collect::<Vec<ExecutableType<'a>>>();

        // walk through the types and update the `borrows` field for `FragmentDefinitionReference` types so that it is correct
        types.iter_mut().for_each(|executable_type| {
            executable_type.update_fragment_definition_references_borrow(
                &fragment_definitions_contains_reference_types,
            );
        });

        types
    }

    fn build_operation_definition(
        &self,
        operation_definition: &'a E::OperationDefinition,
    ) -> ExecutableType<'a> {
        let object_type_definition = match operation_definition.as_ref().operation_type() {
            OperationType::Query => self.config.schema_definition().query(),
            OperationType::Mutation => self
                .config
                .schema_definition()
                .mutation()
                .expect("Unsupported operation type used"),
            OperationType::Subscription => self
                .config
                .schema_definition()
                .subscription()
                .expect("Unsupported operation type used"),
        };
        let path = Path::new(PathRoot::Operation {
            name: operation_definition.as_ref().name(),
        });
        self.build_base_type(
            operation_definition
                .as_ref()
                .name()
                .unwrap_or(ANONYMOUS_OPERATION_STRUCT_NAME),
            Some(operation_definition.as_ref().selection_set()),
            BaseOutputTypeReference::Object(object_type_definition),
            0,
            path,
        )
    }

    fn build_fragment_definition(
        &self,
        fragment_definition: &'a E::FragmentDefinition,
    ) -> ExecutableType<'a> {
        let target_type = self
            .config
            .schema_definition()
            .get_type_definition(fragment_definition.type_condition())
            .expect("Fragment type condition not found");
        let path = Path::new(PathRoot::Fragment {
            name: fragment_definition.name(),
        });
        self.build_base_type(
            fragment_definition.name(),
            Some(fragment_definition.selection_set()),
            target_type
                .try_into()
                .expect("Fragment type not an output type"),
            0,
            path,
        )
    }

    fn fields_or_fragment_spread(
        &self,
        selection_set: &'a E::SelectionSet,
    ) -> Either<Vec<&'a E::Field>, &'a E::FragmentSpread> {
        let fragment_spread = selection_set.iter().find_map(|selection| {
            if let SelectionReference::FragmentSpread(fragment_spread) = selection.as_ref() {
                Some(fragment_spread)
            } else {
                None
            }
        });
        match fragment_spread {
            Some(fragment_spread) if selection_set.len() == 1 => Either::Right(fragment_spread),
            Some(_) => {
                panic!("Selection set contains a fragment spread and more than one selection")
            }
            None => Either::Left(
                selection_set
                    .iter()
                    .map(|selection| match selection.as_ref() {
                        SelectionReference::Field(field) => field,
                        SelectionReference::InlineFragment(_)
                        | SelectionReference::FragmentSpread(_) => {
                            panic!("Selection set does not contains exclusively fields")
                        }
                    })
                    .collect(),
            ),
        }
    }

    fn inline_fragments_or_fragment_spread(
        &self,
        selection_set: &'a E::SelectionSet,
    ) -> Either<Vec<&'a E::InlineFragment>, &'a E::FragmentSpread> {
        let fragment_spread = selection_set.iter().find_map(|selection| {
            if let SelectionReference::FragmentSpread(fragment_spread) = selection.as_ref() {
                Some(fragment_spread)
            } else {
                None
            }
        });
        match fragment_spread {
            Some(fragment_spread) if selection_set.len() == 1 => Either::Right(fragment_spread),
            Some(_) => {
                panic!("Selection set contains a fragment spread and more than one selection")
            }
            None => Either::Left(
                selection_set
                    .iter()
                    .filter_map(|selection| match selection.as_ref() {
                        SelectionReference::Field(_) => None, // there should be a `__typename` selection but we don't need it
                        SelectionReference::InlineFragment(inline_fragment) => {
                            Some(inline_fragment)
                        }
                        SelectionReference::FragmentSpread(_) => unreachable!(),
                    })
                    .collect(),
            ),
        }
    }

    fn build_field(
        &self,
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        depth: usize,
        path: Path<'a>,
    ) -> ExecutableField<'a> {
        let r#type = self.build_field_type(
            field,
            field_definition
                .r#type()
                .as_ref(self.config.schema_definition()),
            depth,
            path,
        );

        ExecutableField {
            description: field_definition.description(),
            graphql_name: field.response_name(),
            r#type,
        }
    }

    fn build_field_type(
        &self,
        field: &'a E::Field,
        output_type: OutputTypeReference<'a, S::OutputType>,
        depth: usize,
        path: Path<'a>,
    ) -> WrappedExecutableType<'a> {
        match output_type {
            OutputTypeReference::List(list, required) => {
                let list_type = WrappedExecutableType::Vec(Box::new(self.build_field_type(
                    field,
                    list.as_ref(self.config.schema_definition()),
                    depth,
                    path,
                )));
                if required {
                    list_type
                } else {
                    WrappedExecutableType::Optional(Box::new(list_type))
                }
            }
            OutputTypeReference::Base(inner, required) => {
                let base_type = WrappedExecutableType::Base(self.build_base_type(
                    field.response_name(),
                    field.selection_set(),
                    inner,
                    depth,
                    path,
                ));
                if required {
                    base_type
                } else {
                    WrappedExecutableType::Optional(Box::new(base_type))
                }
            }
        }
    }

    fn build_base_type(
        &self,
        parent_name: &'a str,
        selection_set: Option<&'a E::SelectionSet>,
        base_output_type: BaseOutputTypeReference<'a, S::OutputType>,
        depth: usize,
        path: Path<'a>,
    ) -> ExecutableType<'a> {
        match base_output_type {
            BaseOutputTypeReference::BuiltinScalar(bstd) => ExecutableType::BuiltinScalar {
                bstd,
                borrows: self.config.builtin_scalar_borrows(bstd),
            },
            BaseOutputTypeReference::CustomScalar(cstd) => {
                if let Some(custom_scalar_override) = self.custom_scalar_override_for_path(&path) {
                    ExecutableType::Leaf {
                        r#type: custom_scalar_override.r#type().clone(),
                        borrows: custom_scalar_override.borrows,
                    }
                } else {
                    let borrows = self.config.custom_scalar_borrows(cstd);
                    let lifetime: Option<syn::Generics> = borrows.then(|| parse_quote! { <'a> });
                    let ident = type_ident(cstd.name());
                    let r#type = parse_quote! { #ident #lifetime };
                    ExecutableType::Leaf { r#type, borrows }
                }
            }
            BaseOutputTypeReference::Enum(etd) => {
                if self.config.enum_as_str(etd) {
                    // kind of a hack because it's not a builtin scalar
                    ExecutableType::BuiltinScalar {
                        bstd: BuiltinScalarDefinition::String,
                        borrows: self.config.borrow(),
                    }
                } else {
                    let ident = type_ident(etd.name());
                    let r#type = parse_quote! { #ident };
                    ExecutableType::Leaf {
                        r#type,
                        borrows: false,
                    }
                }
            }
            BaseOutputTypeReference::Object(otd) => {
                let selection_set = selection_set.expect("No selections for object type");
                match self.fields_or_fragment_spread(selection_set) {
                    Either::Left(fields) => {
                        let fields_definition = otd.fields_definition();
                        ExecutableType::Struct(ExecutableStruct {
                            description: otd.description(),
                            parent_name,
                            fields: fields
                                .iter()
                                .map(|field| {
                                    self.build_field(
                                        field,
                                        fields_definition
                                            .get(field.name())
                                            .expect("Field not found"),
                                        depth + 1,
                                        path.with_field(field.response_name()),
                                    )
                                })
                                .collect(),
                            depth,
                        })
                    }
                    Either::Right(fragment_spread) => ExecutableType::FragmentDefinitionReference {
                        name: fragment_spread.name(),
                        borrows: false, // will get updated later
                    },
                }
            }
            BaseOutputTypeReference::Interface(itd) => {
                let selection_set = selection_set.expect("No selections for interface type");
                match self.fields_or_fragment_spread(selection_set) {
                    Either::Left(fields) => {
                        let fields_definition = itd.fields_definition();
                        ExecutableType::Struct(ExecutableStruct {
                            description: itd.description(),
                            parent_name,
                            fields: fields
                                .iter()
                                .map(|field| {
                                    self.build_field(
                                        field,
                                        fields_definition
                                            .get(field.name())
                                            .expect("Field not found"),
                                        depth + 1,
                                        path.with_field(field.response_name()),
                                    )
                                })
                                .collect(),
                            depth,
                        })
                    }
                    Either::Right(fragment_spread) => ExecutableType::FragmentDefinitionReference {
                        name: fragment_spread.name(),
                        borrows: false, // will get updated later
                    },
                }
            }
            BaseOutputTypeReference::Union(utd) => {
                let selection_set = selection_set.expect("No selections for union type");
                match self.inline_fragments_or_fragment_spread(selection_set) {
                    Either::Left(inline_fragments) => {
                        ExecutableType::Enum(ExecutableEnum {
                            description: utd.description(),
                            parent_name,
                            variants: inline_fragments.iter().map(|inline_fragment| {
                                let type_condition = inline_fragment.type_condition().expect("No type condition for inline fragment on union type");
                                let target_type = utd
                                    .union_member_types()
                                    .get(type_condition)
                                    .expect("Union member type not found")
                                    .member_type(self.config.schema_definition());
                                ExecutableStruct {
                                    description: target_type.description(),
                                    parent_name: type_condition,
                                    fields: inline_fragment
                                        .selection_set()
                                        .iter()
                                        .map(|selection| match selection.as_ref() {
                                            SelectionReference::Field(field) => {
                                                self.build_field(
                                                    field,
                                                    target_type
                                                        .fields_definition()
                                                        .get(field.name())
                                                        .expect("Field not found"),
                                                    depth + 2,
                                                    path.with_field(field.response_name()),
                                                )
                                            }
                                            SelectionReference::InlineFragment(_)
                                            | SelectionReference::FragmentSpread(_) => {
                                                panic!("Selection set does not contain exclusively fields")
                                            }
                                        }).collect(),
                                    depth: depth + 1,
                                }
                            }).collect(),
                        })
                    }
                    Either::Right(fragment_spread) => ExecutableType::FragmentDefinitionReference {
                        name: fragment_spread.name(),
                        borrows: false, // will get updated later
                    },
                }
            }
        }
    }

    fn type_borrows(
        executable_type: &ExecutableType<'a>,
        types_for_fragment_definitions: &BTreeMap<&'a str, ExecutableType<'a>>,
        visited_fragment_definitions: &mut HashSet<&'a str>,
    ) -> bool {
        match executable_type {
            ExecutableType::Struct(es) => es.fields.iter().any(|field| {
                Self::type_borrows(
                    field.r#type.base(),
                    types_for_fragment_definitions,
                    visited_fragment_definitions,
                )
            }),
            ExecutableType::Enum(ee) => ee.variants.iter().any(|variant| {
                variant.fields.iter().any(|field| {
                    Self::type_borrows(
                        field.r#type.base(),
                        types_for_fragment_definitions,
                        visited_fragment_definitions,
                    )
                })
            }),
            ExecutableType::FragmentDefinitionReference { name, .. } => {
                // cannot rely on the `borrows` value in the `FragmentDefinitionReference` yet
                if visited_fragment_definitions.insert(name) {
                    Self::type_borrows(
                        types_for_fragment_definitions
                            .get(name)
                            .expect("Fragment definition not found"),
                        types_for_fragment_definitions,
                        visited_fragment_definitions,
                    )
                } else {
                    false
                }
            }
            ExecutableType::BuiltinScalar { borrows, .. } => *borrows,
            ExecutableType::Leaf { borrows, .. } => *borrows,
        }
    }

    fn custom_scalar_override_for_path(&self, path: &Path<'a>) -> Option<&CustomScalarOverride> {
        self.custom_scalar_overrides
            .iter()
            .find(|custom_scalar_override| {
                custom_scalar_override
                    .graphql_path
                    .split_first()
                    .is_some_and(|(operation_name, fields)| {
                        path.root.name() == Some(operation_name) && path.fields == fields
                    })
            })
    }
}
