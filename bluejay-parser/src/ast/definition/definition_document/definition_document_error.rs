use crate::ast::definition::{
    definition_document::ImplicitSchemaDefinition, type_definition_reference, DirectiveDefinition,
    ExplicitSchemaDefinition, RootOperationTypeDefinition, TypeDefinitionReference,
};
use crate::error::{Annotation, Error};
use crate::lexical_token::Name;
use crate::HasSpan;
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
            DefinitionDocumentError::DuplicateDirectiveDefinitions { name, definitions } => {
                Error::new(
                    format!("Multiple directive definitions with name `@{name}`"),
                    None,
                    definitions
                        .into_iter()
                        .map(|definition| {
                            Annotation::new(
                                format!("Directive definition with name `@{name}`"),
                                definition.name().span(),
                            )
                        })
                        .collect(),
                )
            }
            DefinitionDocumentError::DuplicateExplicitRootOperationDefinitions {
                operation_type,
                root_operation_type_definitions,
            } => Error::new(
                format!("Multiple root operation type definitions for `{operation_type}`"),
                None,
                root_operation_type_definitions
                    .into_iter()
                    .map(|rotd| {
                        Annotation::new(
                            format!("Root operation type definition for `{operation_type}`"),
                            rotd.name().span(),
                        )
                    })
                    .collect(),
            ),
            DefinitionDocumentError::DuplicateExplicitSchemaDefinitions { definitions } => {
                Error::new(
                    "Multiple schema definitions",
                    None,
                    definitions
                        .iter()
                        .map(|definition| {
                            Annotation::new(
                                "Schema definition",
                                definition.schema_identifier_span(),
                            )
                        })
                        .collect(),
                )
            }
            DefinitionDocumentError::DuplicateTypeDefinitions { name, definitions } => Error::new(
                format!("Multiple type definitions with name `{name}`"),
                None,
                definitions
                    .into_iter()
                    .map(|definition| {
                        Annotation::new(
                            format!("Type definition with name `{name}`"),
                            type_definition_reference::name(definition).unwrap().span(),
                        )
                    })
                    .collect(),
            ),
            DefinitionDocumentError::ExplicitRootOperationTypeDoesNotExist {
                root_operation_type_definition,
            } => Error::new(
                format!(
                    "Referenced type `{}` does not exist",
                    root_operation_type_definition.name().as_ref()
                ),
                Some(Annotation::new(
                    "No definition for referenced type",
                    root_operation_type_definition.name().span(),
                )),
                Vec::new(),
            ),
            DefinitionDocumentError::ExplicitSchemaDefinitionMissingQuery { definition } => {
                Error::new(
                    "Schema definition does not contain a query",
                    Some(Annotation::new(
                        "Does not contain a query",
                        definition.root_operation_type_definitions_span(),
                    )),
                    Vec::new(),
                )
            }
            DefinitionDocumentError::ImplicitAndExplicitSchemaDefinitions {
                implicit,
                explicit,
            } => Error::new(
                "Document uses implicit and explicit schema definitions",
                Some(Annotation::new(
                    "Explicit schema definition",
                    explicit.schema_identifier_span(),
                )),
                {
                    let mut annotations = vec![Annotation::new(
                        "Query of implicit schema definition",
                        implicit.query.name().span(),
                    )];

                    if let Some(mutation) = implicit.mutation {
                        annotations.push(Annotation::new(
                            "Mutation of implicit schema definition",
                            mutation.name().span(),
                        ));
                    }

                    if let Some(subscription) = implicit.subscription {
                        annotations.push(Annotation::new(
                            "Subscription of implicit schema definition",
                            subscription.name().span(),
                        ));
                    }

                    annotations
                },
            ),
            DefinitionDocumentError::ImplicitSchemaDefinitionMissingQuery => {
                Error::new("Implicit schema definition missing query", None, Vec::new())
            }
            DefinitionDocumentError::NoSchemaDefinition => Error::new(
                "Document does not contain a schema definition",
                None,
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedTypeDoesNotExist { name } => Error::new(
                format!("Referenced type `{}` does not exist", name.as_ref()),
                Some(Annotation::new(
                    "No definition for referenced type",
                    name.span(),
                )),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedTypeIsNotAnInputType { name } => Error::new(
                format!("Referenced type `{}` is not an input type", name.as_ref()),
                Some(Annotation::new("Not an input type", name.span())),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedTypeIsNotAnInterface { name } => Error::new(
                format!("Referenced type `{}` is not an interface", name.as_ref()),
                Some(Annotation::new("Not an interface", name.span())),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedTypeIsNotAnOutputType { name } => Error::new(
                format!("Referenced type `{}` is not an output type", name.as_ref()),
                Some(Annotation::new("Not an output type", name.span())),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedUnionMemberTypeIsNotAnObject { name } => Error::new(
                format!("Referenced type `{}` is not an object", name.as_ref()),
                Some(Annotation::new("Not an object type", name.span())),
                Vec::new(),
            ),
            DefinitionDocumentError::ImplicitRootOperationTypeNotAnObject { definition } => {
                Error::new(
                    format!("Referenced type `{}` is not an object", definition.name()),
                    Some(Annotation::new(
                        "Not an object type",
                        // ok to unwrap because builtin scalar cannot be an implicit schema definition member
                        type_definition_reference::name(definition).unwrap().span(),
                    )),
                    Vec::new(),
                )
            }
            DefinitionDocumentError::ExplicitRootOperationTypeNotAnObject { name } => Error::new(
                format!("Referenced type `{}` is not an object", name.as_ref()),
                Some(Annotation::new("Not an object type", name.span())),
                Vec::new(),
            ),
        }
    }
}
