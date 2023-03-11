use crate::ast::definition::{
    BaseInputTypeReference, BaseOutputTypeReference, CustomScalarTypeDefinition,
    DirectiveDefinition, EnumTypeDefinition, ExplicitSchemaDefinition, FieldsDefinition,
    InputObjectTypeDefinition, InputValueDefinition, InterfaceImplementations,
    InterfaceTypeDefinition, ObjectTypeDefinition, SchemaDefinition, TypeDefinitionReference,
    UnionTypeDefinition,
};
use crate::ast::{FromTokens, ParseError, ScannerTokens, Tokens};
use crate::lexical_token::HasSpan;
use crate::scanner::LogosScanner;
use crate::Error;
use bluejay_core::definition::{
    DirectiveDefinition as CoreDirectiveDefinition, FieldDefinition as CoreFieldDefinition,
    InputObjectTypeDefinition as CoreInputObjectTypeDefinition,
    InputValueDefinition as CoreInputValueDefinition,
    InterfaceTypeDefinition as CoreInterfaceTypeDefinition,
    ObjectTypeDefinition as CoreObjectTypeDefinition,
    UnionTypeDefinition as CoreUnionTypeDefinition,
};
use bluejay_core::{AsIter, BuiltinScalarDefinition, IntoEnumIterator, OperationType};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

mod definition_document_error;
use definition_document_error::DefinitionDocumentError;

#[derive(Debug)]
pub struct DefinitionDocument<'a> {
    schema_definitions: Vec<ExplicitSchemaDefinition<'a>>,
    directive_definitions: Vec<DirectiveDefinition<'a>>,
    type_definition_references: Vec<TypeDefinitionReference<'a>>,
}

#[derive(Debug)]
pub struct ImplicitSchemaDefinition<'a> {
    query: &'a ObjectTypeDefinition<'a>,
    mutation: Option<&'a ObjectTypeDefinition<'a>>,
    subscription: Option<&'a ObjectTypeDefinition<'a>>,
}

impl<'a> DefinitionDocument<'a> {
    fn new() -> Self {
        Self {
            schema_definitions: Vec::new(),
            directive_definitions: vec![
                DirectiveDefinition::skip(),
                DirectiveDefinition::include(),
            ],
            type_definition_references: Vec::from_iter(
                BuiltinScalarDefinition::iter().map(TypeDefinitionReference::BuiltinScalarType),
            ),
        }
    }

    fn parse_definition<'b, S, T: FromTokens<'b> + Into<S>>(
        definitions: &mut Vec<S>,
        tokens: &mut impl Tokens<'b>,
        errors: &mut Vec<ParseError>,
        last_pass_had_error: &mut bool,
    ) {
        match T::from_tokens(tokens) {
            Ok(definition) => {
                definitions.push(definition.into());
                *last_pass_had_error = false;
            }
            Err(err) => {
                if !*last_pass_had_error {
                    errors.push(err);
                    *last_pass_had_error = true;
                }
            }
        }
    }

    pub fn parse(s: &'a str) -> Result<Self, Vec<Error>> {
        let scanner = LogosScanner::new(s);
        let mut tokens = ScannerTokens::new(scanner);

        let mut instance: Self = Self::new();
        let mut errors = Vec::new();
        let mut last_pass_had_error = false;

        loop {
            match Self::next_definition_identifier(&mut tokens) {
                Some(CustomScalarTypeDefinition::SCALAR_IDENTIFIER) => {
                    Self::parse_definition::<_, CustomScalarTypeDefinition>(
                        &mut instance.type_definition_references,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(ObjectTypeDefinition::TYPE_IDENTIFIER) => {
                    Self::parse_definition::<_, ObjectTypeDefinition>(
                        &mut instance.type_definition_references,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(InputObjectTypeDefinition::INPUT_IDENTIFIER) => {
                    Self::parse_definition::<_, InputObjectTypeDefinition>(
                        &mut instance.type_definition_references,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(EnumTypeDefinition::ENUM_IDENTIFIER) => {
                    Self::parse_definition::<_, EnumTypeDefinition>(
                        &mut instance.type_definition_references,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(UnionTypeDefinition::UNION_IDENTIFIER) => {
                    Self::parse_definition::<_, UnionTypeDefinition>(
                        &mut instance.type_definition_references,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(InterfaceTypeDefinition::INTERFACE_IDENTIFIER) => {
                    Self::parse_definition::<_, InterfaceTypeDefinition>(
                        &mut instance.type_definition_references,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(ExplicitSchemaDefinition::SCHEMA_IDENTIFIER) => {
                    Self::parse_definition::<_, ExplicitSchemaDefinition>(
                        &mut instance.schema_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(DirectiveDefinition::DIRECTIVE_IDENTIFIER) => {
                    Self::parse_definition::<_, DirectiveDefinition>(
                        &mut instance.directive_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                _ => {
                    if let Some(token) = tokens.next() {
                        if !last_pass_had_error {
                            errors.push(ParseError::UnexpectedToken { span: token.span() });
                            last_pass_had_error = true;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        let errors = if tokens.errors.is_empty() {
            if errors.is_empty() && instance.is_empty() {
                vec![ParseError::EmptyDocument.into()]
            } else {
                errors.into_iter().map(Into::into).collect()
            }
        } else {
            tokens.errors.into_iter().map(Into::into).collect()
        };

        if errors.is_empty() {
            Ok(instance)
        } else {
            Err(errors)
        }
    }

    fn is_empty(&self) -> bool {
        self.definition_count() == 0
    }

    fn next_definition_identifier(tokens: &mut impl Tokens<'a>) -> Option<&str> {
        let idx_to_peek = if tokens.peek_string_value(0) { 1 } else { 0 };
        tokens.peek_name(idx_to_peek).map(AsRef::as_ref)
    }

    pub fn definition_count(&self) -> usize {
        self.directive_definitions
            .iter()
            .filter(|dd| !dd.is_builtin())
            .count()
            + self.schema_definitions.len()
            + self
                .type_definition_references
                .iter()
                .filter(|tdr| !tdr.is_builtin())
                .count()
    }

    pub fn directive_definitions(&self) -> &[DirectiveDefinition<'a>] {
        &self.directive_definitions
    }

    fn index_directive_definitions(
        &'a self,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) -> BTreeMap<&str, &'a DirectiveDefinition<'a>> {
        let mut indexed: BTreeMap<&str, &DirectiveDefinition<'a>> = BTreeMap::new();
        let mut duplicates: BTreeMap<&str, Vec<&DirectiveDefinition<'a>>> = BTreeMap::new();

        self.directive_definitions
            .iter()
            .for_each(|directive_definition| {
                match indexed.entry(directive_definition.name().as_ref()) {
                    Entry::Vacant(entry) => {
                        entry.insert(directive_definition);
                    }
                    Entry::Occupied(entry) => {
                        duplicates
                            .entry(directive_definition.name().as_ref())
                            .or_insert_with(|| vec![entry.get()])
                            .push(directive_definition);
                    }
                }
            });

        errors.extend(duplicates.into_iter().map(|(name, definitions)| {
            DefinitionDocumentError::DuplicateDirectiveDefinitions { name, definitions }
        }));

        indexed
    }

    fn index_type_definitions(
        &'a self,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) -> BTreeMap<&str, &TypeDefinitionReference<'a>> {
        let mut indexed: BTreeMap<&str, &TypeDefinitionReference<'a>> = BTreeMap::new();
        let mut duplicates: BTreeMap<&str, Vec<&TypeDefinitionReference<'a>>> = BTreeMap::new();

        self.type_definition_references
            .iter()
            .for_each(|tdr| match indexed.entry(tdr.name()) {
                Entry::Vacant(entry) => {
                    entry.insert(tdr);
                }
                Entry::Occupied(entry) => {
                    duplicates
                        .entry(tdr.name())
                        .or_insert_with(|| vec![entry.get()])
                        .push(tdr);
                }
            });

        errors.extend(duplicates.into_iter().map(|(name, definitions)| {
            DefinitionDocumentError::DuplicateTypeDefinitions { name, definitions }
        }));

        indexed
    }

    fn implicit_schema_definition(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
    ) -> Result<Option<ImplicitSchemaDefinition<'a>>, Vec<DefinitionDocumentError<'a>>> {
        let mut errors = Vec::new();
        let query =
            Self::implicit_root_operation_type("Query", indexed_type_definitions, &mut errors);
        let mutation =
            Self::implicit_root_operation_type("Mutation", indexed_type_definitions, &mut errors);
        let subscription = Self::implicit_root_operation_type(
            "Subscription",
            indexed_type_definitions,
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(errors);
        }

        if let Some(query) = query {
            Ok(Some(ImplicitSchemaDefinition {
                query,
                mutation,
                subscription,
            }))
        } else if mutation.is_some() || subscription.is_some() {
            Err(vec![
                DefinitionDocumentError::ImplicitSchemaDefinitionMissingQuery,
            ])
        } else {
            Ok(None)
        }
    }

    fn implicit_root_operation_type(
        name: &str,
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) -> Option<&'a ObjectTypeDefinition<'a>> {
        match indexed_type_definitions.get(name) {
            Some(TypeDefinitionReference::ObjectType(o, _)) => Some(o),
            Some(definition) => {
                errors.push(
                    DefinitionDocumentError::ImplicitRootOperationTypeNotAnObject { definition },
                );
                None
            }
            None => None,
        }
    }

    fn explicit_schema_definition(
        &'a self,
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) -> Option<(
        &'a ExplicitSchemaDefinition<'a>,
        &'a ObjectTypeDefinition<'a>,
        Option<&'a ObjectTypeDefinition<'a>>,
        Option<&'a ObjectTypeDefinition<'a>>,
    )> {
        if let Some(first) = self.schema_definitions.first() {
            if self.schema_definitions.len() == 1 {
                let query = Self::explicit_operation_type_definition(
                    OperationType::Query,
                    first,
                    indexed_type_definitions,
                    errors,
                );
                let mutation = Self::explicit_operation_type_definition(
                    OperationType::Mutation,
                    first,
                    indexed_type_definitions,
                    errors,
                );
                let subscription = Self::explicit_operation_type_definition(
                    OperationType::Subscription,
                    first,
                    indexed_type_definitions,
                    errors,
                );
                if !errors.is_empty() {
                    return None;
                }
                if let Some(query) = query {
                    Some((first, query, mutation, subscription))
                } else {
                    errors.push(
                        DefinitionDocumentError::ExplicitSchemaDefinitionMissingQuery {
                            definition: first,
                        },
                    );
                    None
                }
            } else {
                errors.push(
                    DefinitionDocumentError::DuplicateExplicitSchemaDefinitions {
                        definitions: &self.schema_definitions,
                    },
                );
                None
            }
        } else {
            None
        }
    }

    fn explicit_operation_type_definition(
        operation_type: OperationType,
        explicit_schema_definition: &'a ExplicitSchemaDefinition<'a>,
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) -> Option<&'a ObjectTypeDefinition<'a>> {
        let root_operation_type_definitions: Vec<_> = explicit_schema_definition
            .root_operation_type_definitions()
            .iter()
            .filter(|rotd| rotd.operation_type() == operation_type)
            .collect();

        if let Some(first) = root_operation_type_definitions.first() {
            if root_operation_type_definitions.len() == 1 {
                match indexed_type_definitions.get(first.name().as_ref()) {
                    Some(TypeDefinitionReference::ObjectType(o, _)) => Some(o),
                    Some(_) => {
                        errors.push(
                            DefinitionDocumentError::ExplicitRootOperationTypeNotAnObject {
                                name: first.name(),
                            },
                        );
                        None
                    }
                    None => {
                        errors.push(
                            DefinitionDocumentError::ExplicitRootOperationTypeDoesNotExist {
                                root_operation_type_definition: first,
                            },
                        );
                        None
                    }
                }
            } else {
                errors.push(
                    DefinitionDocumentError::DuplicateExplicitRootOperationDefinitions {
                        operation_type,
                        root_operation_type_definitions,
                    },
                );
                None
            }
        } else {
            None
        }
    }

    fn resolve_type_definition_references(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
        indexed_directive_definitions: &BTreeMap<&str, &'a DirectiveDefinition<'a>>,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) {
        indexed_type_definitions
            .values()
            .for_each(|type_definition| match type_definition {
                TypeDefinitionReference::ObjectType(otd, _) => {
                    Self::resolve_fields_definition_type_references(
                        indexed_type_definitions,
                        otd.fields_definition(),
                        errors,
                    );
                    if let Some(interface_implementations) = otd.interface_implementations() {
                        Self::resolve_interface_implementation_references(
                            indexed_type_definitions,
                            interface_implementations,
                            errors,
                        );
                    }
                }
                TypeDefinitionReference::InterfaceType(itd, _) => {
                    Self::resolve_fields_definition_type_references(
                        indexed_type_definitions,
                        itd.fields_definition(),
                        errors,
                    );
                    if let Some(interface_implementations) = itd.interface_implementations() {
                        Self::resolve_interface_implementation_references(
                            indexed_type_definitions,
                            interface_implementations,
                            errors,
                        );
                    }
                }
                TypeDefinitionReference::UnionType(utd, _) => {
                    utd.union_member_types().iter().for_each(|member_type| {
                        match indexed_type_definitions.get(member_type.name().as_ref()) {
                            Some(TypeDefinitionReference::ObjectType(otd, _)) => {
                                member_type.set_type_reference(otd).unwrap();
                            }
                            Some(_) => errors.push(
                                DefinitionDocumentError::ReferencedUnionMemberTypeIsNotAnObject {
                                    name: member_type.name(),
                                },
                            ),
                            None => {
                                errors.push(DefinitionDocumentError::ReferencedTypeDoesNotExist {
                                    name: member_type.name(),
                                })
                            }
                        }
                    });
                }
                TypeDefinitionReference::InputObjectType(iotd, _) => {
                    Self::resolve_input_type_references(
                        indexed_type_definitions,
                        iotd.input_field_definitions().iter(),
                        errors,
                    )
                }
                TypeDefinitionReference::BuiltinScalarType(_)
                | TypeDefinitionReference::CustomScalarType(_, _)
                | TypeDefinitionReference::EnumType(_, _) => {}
            });

        indexed_directive_definitions
            .values()
            .for_each(|directive_definition| {
                if let Some(arguments_definition) = directive_definition.arguments_definition() {
                    Self::resolve_input_type_references(
                        indexed_type_definitions,
                        arguments_definition.iter(),
                        errors,
                    );
                }
            })
    }

    fn resolve_fields_definition_type_references(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
        fields_definition: &'a FieldsDefinition<'a>,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) {
        fields_definition.iter().for_each(|field_definition| {
            let t = field_definition.r#type().base();
            match indexed_type_definitions.get(t.name().as_ref()) {
                Some(&tdr) => {
                    match BaseOutputTypeReference::core_type_from_type_definition_reference(tdr) {
                        Ok(core_t) => t.set_type_reference(core_t).unwrap(),
                        Err(_) => {
                            errors.push(DefinitionDocumentError::ReferencedTypeIsNotAnOutputType {
                                name: t.name(),
                            })
                        }
                    }
                }
                None => errors
                    .push(DefinitionDocumentError::ReferencedTypeDoesNotExist { name: t.name() }),
            }

            if let Some(arguments_definition) = field_definition.arguments_definition() {
                Self::resolve_input_type_references(
                    indexed_type_definitions,
                    arguments_definition.iter(),
                    errors,
                )
            }
        })
    }

    fn resolve_interface_implementation_references(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
        interface_impelementations: &'a InterfaceImplementations<'a>,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) {
        interface_impelementations
            .iter()
            .for_each(|interface_implementation| {
                let name = interface_implementation.interface_name();
                match indexed_type_definitions.get(name.as_ref()) {
                    Some(TypeDefinitionReference::InterfaceType(itd, _)) => {
                        interface_implementation.set_type_reference(itd).unwrap();
                    }
                    Some(_) => errors
                        .push(DefinitionDocumentError::ReferencedTypeIsNotAnInterface { name }),
                    None => {
                        errors.push(DefinitionDocumentError::ReferencedTypeDoesNotExist { name })
                    }
                }
            })
    }

    fn resolve_input_type_references(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinitionReference<'a>>,
        input_value_definitions: impl Iterator<Item = &'a InputValueDefinition<'a>>,
        errors: &mut Vec<DefinitionDocumentError<'a>>,
    ) {
        input_value_definitions.for_each(|input_value_definition| {
            let t = input_value_definition.r#type().base();
            match indexed_type_definitions.get(t.name().as_ref()) {
                Some(&tdr) => {
                    match BaseInputTypeReference::core_type_from_type_definition_reference(tdr) {
                        Ok(core_t) => t.set_type_reference(core_t).unwrap(),
                        Err(_) => {
                            errors.push(DefinitionDocumentError::ReferencedTypeIsNotAnInputType {
                                name: t.name(),
                            })
                        }
                    }
                }
                None => errors
                    .push(DefinitionDocumentError::ReferencedTypeDoesNotExist { name: t.name() }),
            }
        })
    }
}

impl<'a> TryFrom<&'a DefinitionDocument<'a>> for SchemaDefinition<'a> {
    type Error = Vec<DefinitionDocumentError<'a>>;

    fn try_from(definition_document: &'a DefinitionDocument<'a>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let indexed_type_definitions = definition_document.index_type_definitions(&mut errors);

        let indexed_directive_definitions =
            definition_document.index_directive_definitions(&mut errors);

        DefinitionDocument::resolve_type_definition_references(
            &indexed_type_definitions,
            &indexed_directive_definitions,
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(errors);
        }

        let explicit_schema_definition =
            definition_document.explicit_schema_definition(&indexed_type_definitions, &mut errors);

        dbg!(&errors);
        dbg!(&explicit_schema_definition);

        let implicit_schema_definition =
            match DefinitionDocument::implicit_schema_definition(&indexed_type_definitions) {
                Ok(isd) => isd,
                Err(mut errs) => {
                    errors.append(&mut errs);
                    None
                }
            };

        if !errors.is_empty() {
            return Err(errors);
        }

        match (explicit_schema_definition, implicit_schema_definition) {
            (Some((explicit, _, mutation, subscription)), Some(implicit))
                if !(explicit.uses_implicit_names()
                    && mutation.is_some() == implicit.mutation.is_some()
                    && subscription.is_some() == implicit.mutation.is_some()) =>
            {
                Err(vec![
                    DefinitionDocumentError::ImplicitAndExplicitSchemaDefinitions {
                        implicit,
                        explicit,
                    },
                ])
            }
            (Some((explicit, query, mutation, subscription)), _) => Ok(Self::new(
                indexed_type_definitions,
                indexed_directive_definitions,
                explicit.description(),
                query,
                mutation,
                subscription,
                explicit.directives(),
            )),
            (None, Some(implicit)) => Ok(Self::new(
                indexed_type_definitions,
                indexed_directive_definitions,
                None,
                implicit.query,
                implicit.mutation,
                implicit.subscription,
                None,
            )),
            (None, None) => Err(vec![DefinitionDocumentError::NoSchemaDefinition]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DefinitionDocument;

    #[test]
    fn smoke_test() {
        let s = r#"
        """
        Description
        """
        type Object {
            foo: String!
        }
        "#;

        let document = DefinitionDocument::parse(s).unwrap();

        assert_eq!(1, document.definition_count());
    }
}
