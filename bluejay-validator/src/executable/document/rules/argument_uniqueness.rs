use crate::executable::{
    document::{ArgumentError, Error, Path, Rule, Visitor},
    Cache,
};
use crate::utils::duplicates;
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, Field};
use bluejay_core::{Argument, AsIter, Directive};

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
            self.errors
                .extend(
                    duplicates(arguments.iter(), Argument::name).map(|(name, arguments)| {
                        build_error(ArgumentError::NonUniqueArgumentNames { name, arguments })
                    }),
                );
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for ArgumentUniqueness<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self { errors: Vec::new() }
    }

    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        _: &'a S::FieldDefinition,
        _: &Path<'a, E>,
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

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for ArgumentUniqueness<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
