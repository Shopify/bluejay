use crate::ast::definition::{
    definition_document::ImplicitSchemaDefinition, type_definition_reference, DirectiveDefinition,
    ExplicitSchemaDefinition, RootOperationTypeDefinition, TypeDefinitionReference,
};
use crate::error::{Annotation, Error};
use crate::lexical_token::{HasSpan, Name};
use bluejay_core::OperationType;

#[derive(Debug)]
pub enum DefinitionDocumentError<'a> {
    DuplicateDirectiveDefinitions {
        name: &'a str,
        definitions: Vec<&'a DirectiveDefinition<'a>>,
    },
    DuplicateTypeDefinitions {
        name: &'a str,
        definitions: Vec<&'a TypeDefinitionReference<'a>>,
    },
    ImplicitRootOperationTypeNotAnObject {
        definition: &'a TypeDefinitionReference<'a>,
    },
    ExplicitRootOperationTypeNotAnObject {
        name: &'a Name<'a>,
    },
    ImplicitSchemaDefinitionMissingQuery,
    ExplicitSchemaDefinitionMissingQuery {
        definition: &'a ExplicitSchemaDefinition<'a>,
    },
    DuplicateExplicitSchemaDefinitions {
        definitions: &'a [ExplicitSchemaDefinition<'a>],
    },
    ImplicitAndExplicitSchemaDefinitions {
        implicit: ImplicitSchemaDefinition<'a>,
        explicit: &'a ExplicitSchemaDefinition<'a>,
    },
    DuplicateExplicitRootOperationDefinitions {
        operation_type: OperationType,
        root_operation_type_definitions: Vec<&'a RootOperationTypeDefinition<'a>>,
    },
    ExplicitRootOperationTypeDoesNotExist {
        root_operation_type_definition: &'a RootOperationTypeDefinition<'a>,
    },
    NoSchemaDefinition,
    ReferencedTypeDoesNotExist {
        name: &'a Name<'a>,
    },
    ReferencedTypeIsNotAnOutputType {
        name: &'a Name<'a>,
    },
    ReferencedTypeIsNotAnInputType {
        name: &'a Name<'a>,
    },
    ReferencedUnionMemberTypeIsNotAnObject {
        name: &'a Name<'a>,
    },
    ReferencedTypeIsNotAnInterface {
        name: &'a Name<'a>,
    },
}

impl<'a> From<DefinitionDocumentError<'a>> for Error {
    fn from(value: DefinitionDocumentError) -> Self {
        match value {
            DefinitionDocumentError::DuplicateDirectiveDefinitions { name, definitions } => Error {
                message: format!("Multiple directive definitions with name `@{name}`"),
                primary_annotation: None,
                secondary_annotations: definitions
                    .into_iter()
                    .map(|definition| Annotation {
                        message: format!("Directive definition with name `@{name}`"),
                        span: definition.name().span(),
                    })
                    .collect(),
            },
            DefinitionDocumentError::DuplicateExplicitRootOperationDefinitions {
                operation_type,
                root_operation_type_definitions,
            } => Error {
                message: format!("Multiple root operation type definitions for `{operation_type}`"),
                primary_annotation: None,
                secondary_annotations: root_operation_type_definitions
                    .into_iter()
                    .map(|rotd| Annotation {
                        message: format!("Root operation type definition for `{operation_type}`"),
                        span: rotd.name().span(),
                    })
                    .collect(),
            },
            DefinitionDocumentError::DuplicateExplicitSchemaDefinitions { definitions } => Error {
                message: "Multiple schema definitions".to_string(),
                primary_annotation: None,
                secondary_annotations: definitions
                    .into_iter()
                    .map(|definition| Annotation {
                        message: "Schema definition".to_string(),
                        span: definition.schema_identifier_span(),
                    })
                    .collect(),
            },
            DefinitionDocumentError::DuplicateTypeDefinitions { name, definitions } => Error {
                message: format!("Multiple type definitions with name `{name}`"),
                primary_annotation: None,
                secondary_annotations: definitions
                    .into_iter()
                    .map(|definition| Annotation {
                        message: format!("Type definition with name `{name}`"),
                        span: type_definition_reference::name(definition).unwrap().span(),
                    })
                    .collect(),
            },
            DefinitionDocumentError::ExplicitRootOperationTypeDoesNotExist {
                root_operation_type_definition,
            } => Error {
                message: format!(
                    "Referenced type `{}` does not exist",
                    root_operation_type_definition.name().as_ref()
                ),
                primary_annotation: Some(Annotation {
                    message: "No definition for referenced type".to_string(),
                    span: root_operation_type_definition.name().span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ExplicitSchemaDefinitionMissingQuery { definition } => Error {
                message: "Schema definition does not contain a query".to_string(),
                primary_annotation: Some(Annotation {
                    message: "Does not contain a query".to_string(),
                    span: definition.root_operation_type_definitions_span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ImplicitAndExplicitSchemaDefinitions {
                implicit,
                explicit,
            } => Error {
                message: "Document uses implicit and explicit schema definitions".to_string(),
                primary_annotation: Some(Annotation {
                    message: "Explicit schema definition".to_string(),
                    span: explicit.schema_identifier_span(),
                }),
                secondary_annotations: {
                    let mut annotations = vec![Annotation {
                        message: "Query of implicit schema definition".to_string(),
                        span: implicit.query.name().span(),
                    }];

                    if let Some(mutation) = implicit.mutation {
                        annotations.push(Annotation {
                            message: "Mutation of implicit schema definition".to_string(),
                            span: mutation.name().span(),
                        });
                    }

                    if let Some(subscription) = implicit.subscription {
                        annotations.push(Annotation {
                            message: "Subscription of implicit schema definition".to_string(),
                            span: subscription.name().span(),
                        });
                    }

                    annotations
                },
            },
            DefinitionDocumentError::ImplicitSchemaDefinitionMissingQuery => Error {
                message: "Implicit schema definition missing query".to_string(),
                primary_annotation: None,
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::NoSchemaDefinition => Error {
                message: "Document does not contain a schema definition".to_string(),
                primary_annotation: None,
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ReferencedTypeDoesNotExist { name } => Error {
                message: format!("Referenced type `{}` does not exist", name.as_ref()),
                primary_annotation: Some(Annotation {
                    message: "No definition for referenced type".to_string(),
                    span: name.span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ReferencedTypeIsNotAnInputType { name } => Error {
                message: format!("Referenced type `{}` is not an input type", name.as_ref()),
                primary_annotation: Some(Annotation {
                    message: "Not an input type".to_string(),
                    span: name.span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ReferencedTypeIsNotAnInterface { name } => Error {
                message: format!("Referenced type `{}` is not an interface", name.as_ref()),
                primary_annotation: Some(Annotation {
                    message: "Not an interface".to_string(),
                    span: name.span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ReferencedTypeIsNotAnOutputType { name } => Error {
                message: format!("Referenced type `{}` is not an output type", name.as_ref()),
                primary_annotation: Some(Annotation {
                    message: "Not an output type".to_string(),
                    span: name.span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ReferencedUnionMemberTypeIsNotAnObject { name } => Error {
                message: format!("Referenced type `{}` is not an object", name.as_ref()),
                primary_annotation: Some(Annotation {
                    message: "Not an object type".to_string(),
                    span: name.span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ImplicitRootOperationTypeNotAnObject { definition } => Error {
                message: format!("Referenced type `{}` is not an object", definition.name()),
                primary_annotation: Some(Annotation {
                    message: "Not an object type".to_string(),
                    // ok to unwrap because builtin scalar cannot be an implicit schema definition member
                    span: type_definition_reference::name(definition).unwrap().span(),
                }),
                secondary_annotations: Vec::new(),
            },
            DefinitionDocumentError::ExplicitRootOperationTypeNotAnObject { name } => Error {
                message: format!("Referenced type `{}` is not an object", name.as_ref()),
                primary_annotation: Some(Annotation {
                    message: "Not an object type".to_string(),
                    span: name.span(),
                }),
                secondary_annotations: Vec::new(),
            },
        }
    }
}
