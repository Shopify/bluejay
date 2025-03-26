use crate::ast::definition::{
    Context, Directive, DirectiveDefinition, ExplicitSchemaDefinition, RootOperationTypeDefinition,
    TypeDefinition,
};
use crate::error::{Annotation, Error};
use crate::lexical_token::Name;
use crate::HasSpan;
use bluejay_core::definition::{
    DirectiveDefinition as CoreDirectiveDefinition, TypeDefinition as CoreTypeDefinition,
};
use bluejay_core::{Directive as _, OperationType};

#[derive(Debug)]
pub enum DefinitionDocumentError<'a, C: Context> {
    DuplicateDirectiveDefinitions {
        name: &'a str,
        definitions: Vec<&'a DirectiveDefinition<'a, C>>,
    },
    DuplicateTypeDefinitions {
        name: &'a str,
        definitions: Vec<&'a TypeDefinition<'a, C>>,
    },
    ImplicitRootOperationTypeNotAnObject {
        definition: &'a TypeDefinition<'a, C>,
    },
    ExplicitRootOperationTypeNotAnObject {
        name: &'a Name<'a>,
    },
    ImplicitSchemaDefinitionMissingQuery,
    ExplicitSchemaDefinitionMissingQuery {
        definition: &'a ExplicitSchemaDefinition<'a, C>,
    },
    DuplicateExplicitSchemaDefinitions {
        definitions: &'a [ExplicitSchemaDefinition<'a, C>],
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
    ReferencedDirectiveDoesNotExist {
        directive: &'a Directive<'a, C>,
    },
}

impl<C: Context> From<DefinitionDocumentError<'_, C>> for Error {
    fn from(value: DefinitionDocumentError<C>) -> Self {
        match value {
            DefinitionDocumentError::DuplicateDirectiveDefinitions { name, definitions } => {
                let message = if definitions
                    .iter()
                    .copied()
                    .any(CoreDirectiveDefinition::is_builtin)
                {
                    format!("Cannot redefine builtin directive @{name}")
                } else {
                    format!("Multiple directive definitions with name `@{name}`")
                };

                Error::new(
                    message,
                    None,
                    definitions
                        .into_iter()
                        .filter(|definition| !definition.is_builtin())
                        .map(|definition| {
                            Annotation::new(
                                format!("Directive definition with name `@{name}`"),
                                definition.name_token().span().clone(),
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
                            rotd.name_token().span().clone(),
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
                                definition.schema_identifier_span().clone(),
                            )
                        })
                        .collect(),
                )
            }
            DefinitionDocumentError::DuplicateTypeDefinitions { name, definitions } => {
                let message = if definitions
                    .iter()
                    .any(|definition| definition.as_ref().is_builtin())
                {
                    format!("Cannot redefine builtin type {name}")
                } else {
                    format!("Multiple type definitions with name `{name}`")
                };

                Error::new(
                    message,
                    None,
                    definitions
                        .into_iter()
                        .filter(|definition| !definition.as_ref().is_builtin())
                        .map(|definition| {
                            Annotation::new(
                                format!("Type definition with name `{name}`"),
                                definition.name_token().unwrap().span().clone(),
                            )
                        })
                        .collect(),
                )
            }
            DefinitionDocumentError::ExplicitRootOperationTypeDoesNotExist {
                root_operation_type_definition,
            } => Error::new(
                format!(
                    "Referenced type `{}` does not exist",
                    root_operation_type_definition.name()
                ),
                Some(Annotation::new(
                    "No definition for referenced type",
                    root_operation_type_definition.name_token().span().clone(),
                )),
                Vec::new(),
            ),
            DefinitionDocumentError::ExplicitSchemaDefinitionMissingQuery { definition } => {
                Error::new(
                    "Schema definition does not contain a query",
                    Some(Annotation::new(
                        "Does not contain a query",
                        definition.root_operation_type_definitions_span().clone(),
                    )),
                    Vec::new(),
                )
            }
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
                    name.span().clone(),
                )),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedTypeIsNotAnInputType { name } => Error::new(
                format!("Referenced type `{}` is not an input type", name.as_ref()),
                Some(Annotation::new("Not an input type", name.span().clone())),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedTypeIsNotAnInterface { name } => Error::new(
                format!("Referenced type `{}` is not an interface", name.as_ref()),
                Some(Annotation::new("Not an interface", name.span().clone())),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedTypeIsNotAnOutputType { name } => Error::new(
                format!("Referenced type `{}` is not an output type", name.as_ref()),
                Some(Annotation::new("Not an output type", name.span().clone())),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedUnionMemberTypeIsNotAnObject { name } => Error::new(
                format!("Referenced type `{}` is not an object", name.as_ref()),
                Some(Annotation::new("Not an object type", name.span().clone())),
                Vec::new(),
            ),
            DefinitionDocumentError::ImplicitRootOperationTypeNotAnObject { definition } => {
                Error::new(
                    format!(
                        "Referenced type `{}` is not an object",
                        definition.as_ref().name()
                    ),
                    Some(Annotation::new(
                        "Not an object type",
                        // ok to unwrap because builtin scalar cannot be an implicit schema definition member
                        definition.name_token().unwrap().span().clone(),
                    )),
                    Vec::new(),
                )
            }
            DefinitionDocumentError::ExplicitRootOperationTypeNotAnObject { name } => Error::new(
                format!("Referenced type `{}` is not an object", name.as_ref()),
                Some(Annotation::new("Not an object type", name.span().clone())),
                Vec::new(),
            ),
            DefinitionDocumentError::ReferencedDirectiveDoesNotExist { directive } => Error::new(
                format!(
                    "Referenced directive `@{}` does not exist",
                    directive.name()
                ),
                Some(Annotation::new(
                    "No definition for referenced directive",
                    directive.span().clone(),
                )),
                Vec::new(),
            ),
        }
    }
}
