use crate::executable::{ArgumentError, Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, Field};
use bluejay_core::{Argument, AsIter, Directive};
use std::collections::BTreeMap;

pub struct ArgumentUniqueness<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> ArgumentUniqueness<'a, E, S> {
    fn visit_arguments<
        const CONST: bool,
        F: Fn(ArgumentError<'a, CONST, E, S>) -> Error<'a, E, S>,
    >(
        &mut self,
        arguments: Option<&'a E::Arguments<CONST>>,
        build_error: F,
    ) {
        if let Some(arguments) = arguments {
            let indexed = arguments.iter().fold(
                BTreeMap::new(),
                |mut indexed: BTreeMap<&'a str, Vec<&'a E::Argument<CONST>>>, argument| {
                    indexed.entry(argument.name()).or_default().push(argument);
                    indexed
                },
            );

            self.errors
                .extend(indexed.into_iter().filter_map(|(name, arguments)| {
                    (arguments.len() > 1).then(|| {
                        build_error(ArgumentError::NonUniqueArgumentNames { name, arguments })
                    })
                }))
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for ArgumentUniqueness<'a, E, S>
{
    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        _: &'a S::FieldDefinition,
    ) {
        self.visit_arguments(field.arguments(), Error::InvalidVariableArgument)
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<false>,
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_arguments(directive.arguments(), Error::InvalidVariableArgument)
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<true>,
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_arguments(directive.arguments(), Error::InvalidConstArgument)
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for ArgumentUniqueness<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for ArgumentUniqueness<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self { errors: Vec::new() }
    }
}
