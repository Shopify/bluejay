use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::{
    AbstractInputTypeReference, DirectiveDefinition, FieldDefinition, InputValueDefinition,
    SchemaDefinition,
};
use bluejay_core::executable::{ExecutableDocument, Field};
use bluejay_core::{
    AbstractValue, Argument, ArgumentWrapper, AsIter, Directive, DirectiveWrapper, Value,
};
use std::collections::HashMap;

pub struct RequiredArguments<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> RequiredArguments<'a, E, S> {
    fn visit_directive<
        const CONST: bool,
        F: Fn(
            &'a S::DirectiveDefinition,
            Vec<&'a S::InputValueDefinition>,
            Vec<&'a E::Argument<CONST>>,
        ) -> Error<'a, E, S>,
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
                |missing_argument_definitions, arguments_with_null_values| {
                    build_error(
                        directive_definition,
                        missing_argument_definitions,
                        arguments_with_null_values,
                    )
                },
            )
        }
    }

    fn visit_arguments<
        const CONST: bool,
        F: Fn(Vec<&'a S::InputValueDefinition>, Vec<&'a E::Argument<CONST>>) -> Error<'a, E, S>,
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
            let (missing_argument_definitions, arguments_with_null_values) =
                arguments_definition.iter().fold(
                    (Vec::new(), Vec::new()),
                    |(mut missing_argument_definitions, mut arguments_with_null_values): (
                        Vec<&'a S::InputValueDefinition>,
                        Vec<&'a E::Argument<CONST>>,
                    ),
                     ivd| {
                        let argument = indexed_arguments.get(ivd.name()).copied();
                        let argument_null = match argument {
                            Some(argument) => {
                                matches!(argument.value().as_ref(), Value::<CONST, _, _>::Null,)
                            }
                            None => false,
                        };
                        let argument_missing_or_null = argument.is_none() || argument_null;
                        if ivd.r#type().as_ref().is_required()
                            && ivd.default_value().is_none()
                            && argument_missing_or_null
                        {
                            missing_argument_definitions.push(ivd);
                            if argument_null {
                                arguments_with_null_values.push(argument.unwrap());
                            }
                        }
                        (missing_argument_definitions, arguments_with_null_values)
                    },
                );
            if !missing_argument_definitions.is_empty() {
                self.errors.push(build_error(
                    missing_argument_definitions,
                    arguments_with_null_values,
                ));
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
            |missing_argument_definitions, arguments_with_null_values| {
                Error::FieldMissingRequiredArguments {
                    field,
                    field_definition,
                    missing_argument_definitions,
                    arguments_with_null_values,
                }
            },
        )
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<false>,
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_directive(
            directive,
            |directive_definition, missing_argument_definitions, arguments_with_null_values| {
                Error::DirectiveMissingRequiredArguments {
                    directive: DirectiveWrapper::Variable(directive),
                    directive_definition,
                    missing_argument_definitions,
                    arguments_with_null_values: arguments_with_null_values
                        .into_iter()
                        .map(ArgumentWrapper::Variable)
                        .collect(),
                }
            },
        )
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<true>,
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_directive(
            directive,
            |directive_definition, missing_argument_definitions, arguments_with_null_values| {
                Error::DirectiveMissingRequiredArguments {
                    directive: DirectiveWrapper::Constant(directive),
                    directive_definition,
                    missing_argument_definitions,
                    arguments_with_null_values: arguments_with_null_values
                        .into_iter()
                        .map(ArgumentWrapper::Constant)
                        .collect(),
                }
            },
        )
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
    fn new(_: &'a E, schema_definition: &'a S) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
        }
    }
}
