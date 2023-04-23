use crate::executable::{ArgumentError, Cache, Error, Rule, Visitor};
use bluejay_core::definition::{
    AbstractInputTypeReference, DirectiveDefinition, FieldDefinition, InputValueDefinition,
    SchemaDefinition,
};
use bluejay_core::executable::{ExecutableDocument, Field};
use bluejay_core::{Argument, AsIter, Directive};
use std::collections::HashMap;

pub struct RequiredArguments<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> RequiredArguments<'a, E, S> {
    fn visit_directive<
        const CONST: bool,
        F: Fn(ArgumentError<'a, CONST, E, S>) -> Error<'a, E, S>,
    >(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<CONST>,
        build_error: F,
    ) {
        if let Some(directive_definition) = self
            .schema_definition
            .get_directive_definition(directive.name())
        {
            self.visit_arguments(
                directive.arguments(),
                directive_definition.arguments_definition(),
                |missing_argument_definitions| {
                    build_error(ArgumentError::DirectiveMissingRequiredArguments {
                        directive,
                        directive_definition,
                        missing_argument_definitions,
                    })
                },
            )
        }
    }

    fn visit_arguments<
        const CONST: bool,
        F: Fn(Vec<&'a S::InputValueDefinition>) -> Error<'a, E, S>,
    >(
        &mut self,
        arguments: Option<&'a E::Arguments<CONST>>,
        arguments_definition: Option<&'a S::ArgumentsDefinition>,
        build_error: F,
    ) {
        if let Some(arguments_definition) = arguments_definition {
            let indexed_arguments = arguments
                .map(|arguments| {
                    arguments.iter().fold(
                        HashMap::new(),
                        |mut indexed_arguments: HashMap<&'a str, &'a E::Argument<CONST>>,
                         argument| {
                            indexed_arguments.insert(argument.name(), argument);
                            indexed_arguments
                        },
                    )
                })
                .unwrap_or_default();
            let missing_argument_definitions =
                Vec::from_iter(arguments_definition.iter().filter_map(|ivd| {
                    let argument = indexed_arguments.get(ivd.name()).copied();
                    (ivd.r#type().as_ref().is_required()
                        && ivd.default_value().is_none()
                        && argument.is_none())
                    .then_some(ivd)
                }));
            if !missing_argument_definitions.is_empty() {
                self.errors.push(build_error(missing_argument_definitions));
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for RequiredArguments<'a, E, S>
{
    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        field_definition: &'a S::FieldDefinition,
    ) {
        self.visit_arguments(
            field.arguments(),
            field_definition.arguments_definition(),
            |missing_argument_definitions| {
                Error::InvalidVariableArgument(ArgumentError::FieldMissingRequiredArguments {
                    field,
                    field_definition,
                    missing_argument_definitions,
                })
            },
        )
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<false>,
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_directive(directive, Error::InvalidVariableArgument)
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<true>,
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_directive(directive, Error::InvalidConstArgument)
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for RequiredArguments<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for RequiredArguments<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
        }
    }
}
