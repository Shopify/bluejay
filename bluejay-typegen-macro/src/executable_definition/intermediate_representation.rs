//! Intermediate representation of the executable document. This takes the AST of the executable document and converts it into
//! data structures that have baked into them many of the assumptions that are validated by the validator. This allows the type
//! generation to be simpler because we don't need to do so much coercion of the AST while generating the types.

use crate::{names::ANONYMOUS_OPERATION_STRUCT_NAME, Config};
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

pub(crate) enum ExecutableType<'a> {
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
    Leaf { name: &'a str, borrows: bool },
}

impl<'a> ExecutableType<'a> {
    pub(crate) fn for_executable_document<E: ExecutableDocument, S: SchemaDefinition>(
        executable_document: &'a E,
        config: &'a Config<'a, S>,
    ) -> Vec<Self> {
        ExecutableDocumentToExecutableTypes::convert(executable_document, config)
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

    pub(crate) fn borrows(&self) -> bool {
        match self {
            Self::Leaf { borrows, .. } => *borrows,
            Self::BuiltinScalar { borrows, .. } => *borrows,
            Self::FragmentDefinitionReference { borrows, .. } => *borrows,
            Self::Struct(es) => es.borrows(),
            Self::Enum(ee) => ee.borrows(),
        }
    }
}

pub(crate) struct ExecutableStruct<'a> {
    pub(crate) description: Option<&'a str>,
    /// name of either the operation, fragment, or field that owns the selection set that this struct represents
    pub(crate) parent_name: &'a str,
    pub(crate) fields: Vec<ExecutableField<'a>>,
}

impl<'a> ExecutableStruct<'a> {
    pub(crate) fn borrows(&self) -> bool {
        self.fields
            .iter()
            .any(|field| field.r#type.base().borrows())
    }
}

pub(crate) struct ExecutableEnum<'a> {
    pub(crate) description: Option<&'a str>,
    /// name of either the operation, fragment, or field that owns the selection set that this struct represents
    pub(crate) parent_name: &'a str,
    pub(crate) variants: Vec<ExecutableEnumVariant<'a>>,
}

impl<'a> ExecutableEnum<'a> {
    pub(crate) fn borrows(&self) -> bool {
        self.variants.iter().any(|variant| variant.borrows())
    }
}

pub(crate) enum WrappedExecutableType<'a> {
    Base(ExecutableType<'a>),
    Optional(Box<WrappedExecutableType<'a>>),
    Vec(Box<WrappedExecutableType<'a>>),
}

impl<'a> WrappedExecutableType<'a> {
    pub(crate) fn base(&self) -> &ExecutableType<'a> {
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

pub(crate) struct ExecutableField<'a> {
    pub(crate) description: Option<&'a str>,
    pub(crate) graphql_name: &'a str,
    pub(crate) r#type: WrappedExecutableType<'a>,
}

pub(crate) struct ExecutableEnumVariant<'a> {
    pub(crate) description: Option<&'a str>,
    pub(crate) name: &'a str,
    pub(crate) fields: Vec<ExecutableField<'a>>,
}

impl<'a> ExecutableEnumVariant<'a> {
    pub(crate) fn borrows(&self) -> bool {
        self.fields
            .iter()
            .any(|field| field.r#type.base().borrows())
    }
}

struct ExecutableDocumentToExecutableTypes<'a, E: ExecutableDocument, S: SchemaDefinition> {
    executable_document_type: PhantomData<&'a E>,
    config: &'a Config<'a, S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> ExecutableDocumentToExecutableTypes<'a, E, S> {
    fn convert(executable_document: &'a E, config: &'a Config<'a, S>) -> Vec<ExecutableType<'a>> {
        let instance = Self {
            executable_document_type: PhantomData,
            config,
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
        self.build_base_type(
            operation_definition
                .as_ref()
                .name()
                .unwrap_or(ANONYMOUS_OPERATION_STRUCT_NAME),
            Some(operation_definition.as_ref().selection_set()),
            BaseOutputTypeReference::Object(object_type_definition),
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
        self.build_base_type(
            fragment_definition.name(),
            Some(fragment_definition.selection_set()),
            target_type
                .try_into()
                .expect("Fragment type not an output type"),
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
    ) -> ExecutableField<'a> {
        let r#type = self.build_field_type(
            field,
            field_definition
                .r#type()
                .as_ref(self.config.schema_definition()),
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
    ) -> WrappedExecutableType<'a> {
        match output_type {
            OutputTypeReference::List(list, required) => {
                let list_type = WrappedExecutableType::Vec(Box::new(
                    self.build_field_type(field, list.as_ref(self.config.schema_definition())),
                ));
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
    ) -> ExecutableType<'a> {
        match base_output_type {
            BaseOutputTypeReference::BuiltinScalar(bstd) => ExecutableType::BuiltinScalar {
                bstd,
                borrows: self.config.builtin_scalar_borrows(bstd),
            },
            BaseOutputTypeReference::CustomScalar(cstd) => ExecutableType::Leaf {
                name: cstd.name(),
                borrows: self.config.custom_scalar_borrows(cstd),
            },
            BaseOutputTypeReference::Enum(etd) => ExecutableType::Leaf {
                name: etd.name(),
                borrows: false,
            },
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
                                    )
                                })
                                .collect(),
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
                                    )
                                })
                                .collect(),
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
                                ExecutableEnumVariant {
                                    description: target_type.description(),
                                    name: type_condition,
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
                                                )
                                            }
                                            SelectionReference::InlineFragment(_)
                                            | SelectionReference::FragmentSpread(_) => {
                                                panic!("Selection set does not contains exclusively fields")
                                            }
                                        }).collect()
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
}
