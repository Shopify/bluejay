use crate::executable::{ArgumentError, Cache, Error, Path, Rule, Visitor};
use bluejay_core::definition::{
    DirectiveDefinition, FieldDefinition, InputValueDefinition, SchemaDefinition,
};
use bluejay_core::executable::{ExecutableDocument, Field};
use bluejay_core::{Argument, AsIter, Directive};

pub struct ArgumentNames<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> ArgumentNames<'a, E, S> {
    fn visit_directive<
        const CONST: bool,
        F: Fn(ArgumentError<'a, CONST, E, S>) -> Error<'a, E, S>,
    >(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<CONST>,
        build_error: F,
    ) {
        if let Some((arguments, directive_definition)) = directive.arguments().zip(
            self.schema_definition
                .get_directive_definition(directive.name()),
        ) {
            self.visit_arguments(
                Some(arguments),
                directive_definition.arguments_definition(),
                |argument| {
                    build_error(ArgumentError::ArgumentDoesNotExistOnDirective {
                        argument,
                        directive_definition,
                    })
                },
            )
        }
    }

    fn visit_arguments<const CONST: bool, F: Fn(&'a E::Argument<CONST>) -> Error<'a, E, S>>(
        &mut self,
        arguments: Option<&'a E::Arguments<CONST>>,
        arguments_definition: Option<&'a S::ArgumentsDefinition>,
        build_error: F,
    ) {
        if let Some(arguments) = arguments {
            self.errors.extend(arguments.iter().filter_map(|argument| {
                let argument_definition = arguments_definition.and_then(|arguments_definition| {
                    arguments_definition
                        .iter()
                        .find(|ivd| ivd.name() == argument.name())
                });
                argument_definition.is_none().then(|| build_error(argument))
            }))
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for ArgumentNames<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
        }
    }

    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        field_definition: &'a S::FieldDefinition,
        _: &Path<'a, E>,
    ) {
        self.visit_arguments(
            field.arguments(),
            field_definition.arguments_definition(),
            |argument| {
                Error::InvalidVariableArgument(ArgumentError::ArgumentDoesNotExistOnField {
                    argument,
                    field_definition,
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
    for ArgumentNames<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for ArgumentNames<'a, E, S>
{
    type Error = Error<'a, E, S>;
}
