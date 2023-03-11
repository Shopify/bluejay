use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReferenceFromAbstract};
use bluejay_core::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};
use bluejay_core::OperationType;
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::executable::ExecutableDocument as ParserExecutableDocument,
    error::{Annotation, Error as ParserError},
    HasSpan,
};

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
        }
    }
}
