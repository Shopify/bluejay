use bluejay_core::definition::{
    DirectiveDefinition, FieldDefinition, InputValueDefinition, SchemaDefinition,
    TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};
use bluejay_core::{
    call_const_wrapper_method, ArgumentWrapper, Directive, DirectiveWrapper, OperationType,
};
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::executable::ExecutableDocument as ParserExecutableDocument,
    error::{Annotation, Error as ParserError},
    HasSpan,
};
use itertools::join;

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
        r#type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    },
    FieldSelectionsDoNotMerge {
        selection_set: &'a E::SelectionSet,
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
    ArgumentDoesNotExistOnField {
        argument: &'a E::Argument<false>,
        field_definition: &'a S::FieldDefinition,
    },
    ArgumentDoesNotExistOnDirective {
        argument: ArgumentWrapper<'a, E::Argument<true>, E::Argument<false>>,
        directive_definition: &'a S::DirectiveDefinition,
    },
    NonUniqueArgumentNames {
        arguments: Vec<ArgumentWrapper<'a, E::Argument<true>, E::Argument<false>>>,
        name: &'a str,
    },
    FieldMissingRequiredArguments {
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        missing_argument_definitions: Vec<&'a S::InputValueDefinition>,
        arguments_with_null_values: Vec<&'a E::Argument<false>>,
    },
    DirectiveMissingRequiredArguments {
        directive: DirectiveWrapper<'a, E::Directive<true>, E::Directive<false>>,
        directive_definition: &'a S::DirectiveDefinition,
        missing_argument_definitions: Vec<&'a S::InputValueDefinition>,
        arguments_with_null_values: Vec<ArgumentWrapper<'a, E::Argument<true>, E::Argument<false>>>,
    },
    NonUniqueFragmentDefinitionNames {
        name: &'a str,
        fragment_definitions: Vec<&'a E::FragmentDefinition>,
    },
}

#[cfg(feature = "parser-integration")]
impl<'a, S: SchemaDefinition> From<Error<'a, ParserExecutableDocument<'a>, S>> for ParserError {
    fn from(value: Error<'a, ParserExecutableDocument<'a>, S>) -> Self {
        match value {
            Error::NonUniqueOperationNames { name, operations } => Self {
                message: format!("Multiple operation definitions named `{name}`"),
                primary_annotation: None,
                secondary_annotations: operations
                    .iter()
                    .filter_map(|operation| {
                        operation.name().map(|operation_name| Annotation {
                            message: format!("Operation definition with name `{name}`"),
                            span: operation_name.span(),
                        })
                    })
                    .collect(),
            },
            Error::NotLoneAnonymousOperation {
                anonymous_operations,
            } => Self {
                message: "Anonymous operation not lone operation in document".to_string(),
                primary_annotation: None,
                secondary_annotations: anonymous_operations
                    .iter()
                    .map(|operation| Annotation {
                        message: "Anonymous operation definition".to_string(),
                        span: operation.selection_set().span(),
                    })
                    .collect(),
            },
            Error::SubscriptionRootNotSingleField { operation } => Self {
                message: "Subscription root is not a single field".to_string(),
                primary_annotation: Some(Annotation {
                    message: "Selection set contains multiple fields".to_string(),
                    span: operation.selection_set().span(),
                }),
                secondary_annotations: Vec::new(),
            },
            Error::FieldDoesNotExistOnType { field, r#type } => Self {
                message: format!(
                    "Field `{}` does not exist on type `{}`",
                    field.name().as_ref(),
                    r#type.name()
                ),
                primary_annotation: Some(Annotation {
                    message: format!("Field does not exist on type `{}`", r#type.name()),
                    span: field.name().span(),
                }),
                secondary_annotations: Vec::new(),
            },
            Error::OperationTypeNotDefined { operation } => Self {
                message: format!(
                    "Schema does not define a {} root",
                    OperationType::from(operation.operation_type()),
                ),
                primary_annotation: Some(Annotation {
                    message: format!(
                        "Schema does not define a {} root",
                        OperationType::from(operation.operation_type()),
                    ),
                    span: operation.operation_type().span(),
                }),
                secondary_annotations: Vec::new(),
            },
            Error::FieldSelectionsDoNotMerge { selection_set } => Self {
                message: "Field selections do not merge".to_string(),
                primary_annotation: Some(Annotation {
                    message: "Field selections do not merge".to_string(),
                    span: selection_set.span(),
                }),
                secondary_annotations: Vec::new(),
            },
            Error::LeafFieldSelectionNotEmpty {
                selection_set,
                r#type,
            } => Self {
                message: format!(
                    "Selection on field of leaf type `{}` was not empty",
                    r#type.as_ref().display_name()
                ),
                primary_annotation: Some(Annotation {
                    message: "Selection set on field of leaf type must be empty".to_string(),
                    span: selection_set.span(),
                }),
                secondary_annotations: Vec::new(),
            },
            Error::NonLeafFieldSelectionEmpty { field, r#type } => Self {
                message: format!(
                    "No selection on field of non-leaf type `{}`",
                    r#type.as_ref().display_name()
                ),
                primary_annotation: Some(Annotation {
                    message: "Fields of non-leaf types must have a selection".to_string(),
                    span: field.name().span(),
                }),
                secondary_annotations: Vec::new(),
            },
            Error::ArgumentDoesNotExistOnField {
                argument,
                field_definition,
            } => Self {
                message: format!(
                    "Field `{}` does not define an argument named `{}`",
                    field_definition.name(),
                    argument.name().as_ref(),
                ),
                primary_annotation: Some(Annotation {
                    message: "No argument definition with this name".to_string(),
                    span: argument.name().span(),
                }),
                secondary_annotations: Vec::new(),
            },
            Error::ArgumentDoesNotExistOnDirective {
                argument,
                directive_definition,
            } => {
                let name = call_const_wrapper_method!(ArgumentWrapper, argument, name);
                Self {
                    message: format!(
                        "Directive `{}` does not define an argument named `{}`",
                        directive_definition.name(),
                        name.as_ref(),
                    ),
                    primary_annotation: Some(Annotation {
                        message: "No argument definition with this name".to_string(),
                        span: name.span(),
                    }),
                    secondary_annotations: Vec::new(),
                }
            }
            Error::NonUniqueArgumentNames { arguments, name } => Self {
                message: format!("Multiple arguments with name `{name}`"),
                primary_annotation: None,
                secondary_annotations: arguments
                    .into_iter()
                    .map(|argument| Annotation {
                        message: format!("Argument with name `{name}`"),
                        span: call_const_wrapper_method!(ArgumentWrapper, argument, name).span(),
                    })
                    .collect(),
            },
            Error::FieldMissingRequiredArguments {
                field,
                field_definition: _,
                missing_argument_definitions,
                arguments_with_null_values,
            } => {
                let missing_argument_names = join(
                    missing_argument_definitions
                        .into_iter()
                        .map(InputValueDefinition::name),
                    ", ",
                );
                let span = match field.arguments() {
                    Some(arguments) => field.name().span().merge(&arguments.span()),
                    None => field.name().span(),
                };
                Self {
                    message: format!(
                        "Field `{}` missing argument(s): {missing_argument_names}",
                        field.response_key()
                    ),
                    primary_annotation: Some(Annotation {
                        message: format!("Missing argument(s): {missing_argument_names}"),
                        span,
                    }),
                    secondary_annotations: arguments_with_null_values
                        .into_iter()
                        .map(|argument| Annotation {
                            message: "`null` value provided for required argument".to_string(),
                            span: argument.span(),
                        })
                        .collect(),
                }
            }
            Error::DirectiveMissingRequiredArguments {
                directive,
                directive_definition: _,
                missing_argument_definitions,
                arguments_with_null_values,
            } => {
                let missing_argument_names = join(
                    missing_argument_definitions
                        .into_iter()
                        .map(InputValueDefinition::name),
                    ", ",
                );
                let directive_name = call_const_wrapper_method!(DirectiveWrapper, directive, name);
                let span = call_const_wrapper_method!(DirectiveWrapper, directive, span);
                Self {
                    message: format!(
                        "Directive `{directive_name}` missing argument(s): {missing_argument_names}",
                    ),
                    primary_annotation: Some(Annotation {
                        message: format!("Missing argument(s): {missing_argument_names}"),
                        span,
                    }),
                    secondary_annotations: arguments_with_null_values
                        .into_iter()
                        .map(|argument| Annotation {
                            message: "`null` value provided for required argument".to_string(),
                            span: call_const_wrapper_method!(ArgumentWrapper, argument, span),
                        })
                        .collect(),
                }
            }
            Error::NonUniqueFragmentDefinitionNames {
                name,
                fragment_definitions,
            } => Self {
                message: format!("Multiple fragment definitions named `{name}`"),
                primary_annotation: None,
                secondary_annotations: fragment_definitions
                    .iter()
                    .map(|fragment_definition| Annotation {
                        message: format!("Fragment definition with name `{name}`"),
                        span: fragment_definition.name().span(),
                    })
                    .collect(),
            },
        }
    }
}
