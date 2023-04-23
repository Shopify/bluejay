use crate::value::input_coercion::Error as InputCoercionError;
use bluejay_core::definition::{
    AbstractOutputTypeReference, FieldDefinition, SchemaDefinition,
    TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};
use bluejay_core::OperationType;
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::executable::ExecutableDocument as ParserExecutableDocument,
    error::{Annotation, Error as ParserError},
    HasSpan,
};

mod argument_error;
mod directive_error;

pub use argument_error::ArgumentError;
pub use directive_error::DirectiveError;

pub enum Error<'a, E: ExecutableDocument, S: SchemaDefinition> {
    NonUniqueOperationNames {
        name: &'a str,
        operations: Vec<&'a E::ExplicitOperationDefinition>,
    },
    NotLoneAnonymousOperation {
        anonymous_operations: Vec<&'a OperationDefinitionFromExecutableDocument<E>>,
    },
    SubscriptionRootNotSingleField {
        operation: &'a OperationDefinitionFromExecutableDocument<E>,
    },
    FieldDoesNotExistOnType {
        field: &'a E::Field,
        r#type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    },
    OperationTypeNotDefined {
        operation: &'a E::ExplicitOperationDefinition,
    },
    LeafFieldSelectionNotEmpty {
        selection_set: &'a E::SelectionSet,
        r#type: &'a S::OutputTypeReference,
    },
    NonLeafFieldSelectionEmpty {
        field: &'a E::Field,
        r#type: &'a S::OutputTypeReference,
    },
    NonUniqueFragmentDefinitionNames {
        name: &'a str,
        fragment_definitions: Vec<&'a E::FragmentDefinition>,
    },
    FragmentDefinitionTargetTypeDoesNotExist {
        fragment_definition: &'a E::FragmentDefinition,
    },
    InlineFragmentTargetTypeDoesNotExist {
        inline_fragment: &'a E::InlineFragment,
    },
    FragmentDefinitionTargetTypeNotComposite {
        fragment_definition: &'a E::FragmentDefinition,
    },
    InlineFragmentTargetTypeNotComposite {
        inline_fragment: &'a E::InlineFragment,
    },
    FragmentDefinitionUnused {
        fragment_definition: &'a E::FragmentDefinition,
    },
    FragmentSpreadTargetUndefined {
        fragment_spread: &'a E::FragmentSpread,
    },
    FragmentSpreadCycle {
        fragment_definition: &'a E::FragmentDefinition,
        fragment_spread: &'a E::FragmentSpread,
    },
    FieldSelectionsDoNotMergeIncompatibleTypes {
        selection_set: &'a E::SelectionSet,
        field_a: &'a E::Field,
        field_definition_a: &'a S::FieldDefinition,
        field_b: &'a E::Field,
        field_definition_b: &'a S::FieldDefinition,
    },
    FieldSelectionsDoNotMergeDifferingNames {
        selection_set: &'a E::SelectionSet,
        field_a: &'a E::Field,
        field_b: &'a E::Field,
    },
    FieldSelectionsDoNotMergeDifferingArguments {
        selection_set: &'a E::SelectionSet,
        field_a: &'a E::Field,
        field_b: &'a E::Field,
    },
    FragmentSpreadIsNotPossible {
        fragment_spread: &'a E::FragmentSpread,
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    },
    InlineFragmentSpreadIsNotPossible {
        inline_fragment: &'a E::InlineFragment,
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    },
    InvalidConstValue(InputCoercionError<'a, true, E::Value<true>>),
    InvalidVariableValue(InputCoercionError<'a, false, E::Value<false>>),
    InvalidConstDirective(DirectiveError<'a, true, E>),
    InvalidVariableDirective(DirectiveError<'a, false, E>),
    InvalidConstArgument(ArgumentError<'a, true, E, S>),
    InvalidVariableArgument(ArgumentError<'a, false, E, S>),
}

#[cfg(feature = "parser-integration")]
impl<'a, S: SchemaDefinition> From<Error<'a, ParserExecutableDocument<'a>, S>> for ParserError {
    fn from(value: Error<'a, ParserExecutableDocument<'a>, S>) -> Self {
        match value {
            Error::NonUniqueOperationNames { name, operations } => Self::new(
                format!("Multiple operation definitions named `{name}`"),
                None,
                operations
                    .iter()
                    .filter_map(|operation| {
                        operation.name().map(|operation_name| {
                            Annotation::new(
                                format!("Operation definition with name `{name}`"),
                                operation_name.span().clone(),
                            )
                        })
                    })
                    .collect(),
            ),
            Error::NotLoneAnonymousOperation {
                anonymous_operations,
            } => Self::new(
                "Anonymous operation not lone operation in document",
                None,
                anonymous_operations
                    .iter()
                    .map(|operation| {
                        Annotation::new(
                            "Anonymous operation definition",
                            operation.selection_set().span().clone(),
                        )
                    })
                    .collect(),
            ),
            Error::SubscriptionRootNotSingleField { operation } => Self::new(
                "Subscription root is not a single field",
                Some(Annotation::new(
                    "Selection set contains multiple fields",
                    operation.selection_set().span().clone(),
                )),
                Vec::new(),
            ),
            Error::FieldDoesNotExistOnType { field, r#type } => Self::new(
                format!(
                    "Field `{}` does not exist on type `{}`",
                    field.name().as_ref(),
                    r#type.name()
                ),
                Some(Annotation::new(
                    format!("Field does not exist on type `{}`", r#type.name()),
                    field.name().span().clone(),
                )),
                Vec::new(),
            ),
            Error::OperationTypeNotDefined { operation } => Self::new(
                format!(
                    "Schema does not define a {} root",
                    OperationType::from(operation.operation_type()),
                ),
                Some(Annotation::new(
                    format!(
                        "Schema does not define a {} root",
                        OperationType::from(operation.operation_type()),
                    ),
                    operation.operation_type().span().clone(),
                )),
                Vec::new(),
            ),
            Error::LeafFieldSelectionNotEmpty {
                selection_set,
                r#type,
            } => Self::new(
                format!(
                    "Selection on field of leaf type `{}` was not empty",
                    r#type.as_ref().display_name()
                ),
                Some(Annotation::new(
                    "Selection set on field of leaf type must be empty",
                    selection_set.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NonLeafFieldSelectionEmpty { field, r#type } => Self::new(
                format!(
                    "No selection on field of non-leaf type `{}`",
                    r#type.as_ref().display_name()
                ),
                Some(Annotation::new(
                    "Fields of non-leaf types must have a selection",
                    field.name().span().clone(),
                )),
                Vec::new(),
            ),
            Error::NonUniqueFragmentDefinitionNames {
                name,
                fragment_definitions,
            } => Self::new(
                format!("Multiple fragment definitions named `{name}`"),
                None,
                fragment_definitions
                    .iter()
                    .map(|fragment_definition| {
                        Annotation::new(
                            format!("Fragment definition with name `{name}`"),
                            fragment_definition.name().span().clone(),
                        )
                    })
                    .collect(),
            ),
            Error::FragmentDefinitionTargetTypeDoesNotExist {
                fragment_definition,
            } => Self::new(
                format!(
                    "No type definition with name `{}`",
                    fragment_definition.type_condition().named_type().as_ref()
                ),
                Some(Annotation::new(
                    "No type with this name",
                    fragment_definition
                        .type_condition()
                        .named_type()
                        .span()
                        .clone(),
                )),
                Vec::new(),
            ),
            Error::InlineFragmentTargetTypeDoesNotExist { inline_fragment } => Self::new(
                format!(
                    "No type definition with name `{}`",
                    inline_fragment
                        .type_condition()
                        .map(|tc| tc.named_type().as_ref())
                        .unwrap_or_default()
                ),
                inline_fragment.type_condition().map(|tc| {
                    Annotation::new("No type with this name", tc.named_type().span().clone())
                }),
                Vec::new(),
            ),
            Error::FragmentDefinitionTargetTypeNotComposite {
                fragment_definition,
            } => Self::new(
                format!(
                    "`{}` is not a composite type",
                    fragment_definition.type_condition().named_type().as_ref()
                ),
                Some(Annotation::new(
                    "Fragment definition target types must be composite types",
                    fragment_definition
                        .type_condition()
                        .named_type()
                        .span()
                        .clone(),
                )),
                Vec::new(),
            ),
            Error::InlineFragmentTargetTypeNotComposite { inline_fragment } => Self::new(
                format!(
                    "`{}` is not a composite type",
                    inline_fragment
                        .type_condition()
                        .map(|tc| tc.named_type().as_ref())
                        .unwrap_or_default()
                ),
                inline_fragment.type_condition().map(|tc| {
                    Annotation::new(
                        "Inline fragment target types must be composite types",
                        tc.named_type().span().clone(),
                    )
                }),
                Vec::new(),
            ),
            Error::FragmentDefinitionUnused {
                fragment_definition,
            } => Self::new(
                format!(
                    "Fragment definition `{}` is unused",
                    fragment_definition.name().as_ref()
                ),
                Some(Annotation::new(
                    "Fragment definition is unused",
                    fragment_definition.name().span().clone(),
                )),
                Vec::new(),
            ),
            Error::FragmentSpreadTargetUndefined { fragment_spread } => Self::new(
                format!(
                    "No fragment defined with name `{}`",
                    fragment_spread.name().as_ref()
                ),
                Some(Annotation::new(
                    "No fragment defined with this name",
                    fragment_spread.name().span().clone(),
                )),
                Vec::new(),
            ),
            Error::FragmentSpreadCycle {
                fragment_definition,
                fragment_spread,
            } => Self::new(
                format!(
                    "Cycle detected in fragment `{}`",
                    fragment_definition.name().as_ref()
                ),
                Some(Annotation::new(
                    "Cycle introduced by fragment spread",
                    fragment_spread.name().span().clone(),
                )),
                vec![Annotation::new(
                    "Affected fragment definition",
                    fragment_definition.name().span().clone(),
                )],
            ),
            Error::FieldSelectionsDoNotMergeDifferingArguments {
                selection_set,
                field_a,
                field_b,
            } => Self::new(
                "Fields in selection set do not merge due to unequal arguments",
                Some(Annotation::new(
                    "Fields in selection set do not merge",
                    selection_set.span().clone(),
                )),
                vec![
                    Annotation::new("First field", field_a.name().span().clone()),
                    Annotation::new("Second field", field_b.name().span().clone()),
                ],
            ),
            Error::FieldSelectionsDoNotMergeDifferingNames {
                selection_set,
                field_a,
                field_b,
            } => Self::new(
                "Fields in selection set do not merge due to unequal field names",
                Some(Annotation::new(
                    "Fields in selection set do not merge",
                    selection_set.span().clone(),
                )),
                vec![
                    Annotation::new("First field", field_a.name().span().clone()),
                    Annotation::new("Second field", field_b.name().span().clone()),
                ],
            ),
            Error::FieldSelectionsDoNotMergeIncompatibleTypes {
                selection_set,
                field_a,
                field_definition_a,
                field_b,
                field_definition_b,
            } => Self::new(
                "Fields in selection set do not merge due to incompatible types",
                Some(Annotation::new(
                    "Fields in selection set do not merge",
                    selection_set.span().clone(),
                )),
                vec![
                    Annotation::new(
                        format!(
                            "First field has type {}",
                            field_definition_a.r#type().as_ref().display_name(),
                        ),
                        field_a.name().span().clone(),
                    ),
                    Annotation::new(
                        format!(
                            "Second field has type {}",
                            field_definition_b.r#type().as_ref().display_name(),
                        ),
                        field_b.name().span().clone(),
                    ),
                ],
            ),
            Error::FragmentSpreadIsNotPossible {
                fragment_spread,
                parent_type,
            } => Self::new(
                format!(
                    "Fragment `{}` cannot be spread for type {}",
                    fragment_spread.name().as_ref(),
                    parent_type.name()
                ),
                Some(Annotation::new(
                    format!("Cannot be spread for type {}", parent_type.name()),
                    fragment_spread.name().span().clone(),
                )),
                Vec::new(),
            ),
            Error::InlineFragmentSpreadIsNotPossible {
                inline_fragment,
                parent_type,
            } => Self::new(
                format!(
                    "Fragment targeting type {} cannot be spread for type {}",
                    inline_fragment
                        .type_condition()
                        .map(|type_condition| type_condition.named_type().as_ref())
                        .unwrap_or_else(|| parent_type.name()),
                    parent_type.name(),
                ),
                Some(Annotation::new(
                    format!("Cannot be spread for type {}", parent_type.name()),
                    inline_fragment.span().clone(),
                )),
                Vec::new(),
            ),
            Error::InvalidConstValue(error) => Self::from(error),
            Error::InvalidVariableValue(error) => Self::from(error),
            Error::InvalidConstDirective(error) => Self::from(error),
            Error::InvalidVariableDirective(error) => Self::from(error),
            Error::InvalidConstArgument(error) => Self::from(error),
            Error::InvalidVariableArgument(error) => Self::from(error),
        }
    }
}
