use crate::executable::{
    document::{DirectiveError, Error, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{DirectiveDefinition, DirectiveLocation, SchemaDefinition};
use bluejay_core::executable::ExecutableDocument;
use bluejay_core::{AsIter, Directive};

pub struct DirectivesAreInValidLocations<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a>
    DirectivesAreInValidLocations<'a, E, S>
{
    fn visit_directive<
        const CONST: bool,
        F: Fn(DirectiveError<'a, CONST, E, S>) -> Error<'a, E, S>,
    >(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<CONST>,
        location: DirectiveLocation,
        build_error: F,
    ) {
        if let Some(directive_definition) = self
            .schema_definition
            .get_directive_definition(directive.name())
        {
            if directive_definition
                .locations()
                .iter()
                .all(|&definition_location| definition_location != location)
            {
                self.errors
                    .push(build_error(DirectiveError::DirectiveInInvalidLocation {
                        directive,
                        directive_definition,
                        location,
                    }));
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for DirectivesAreInValidLocations<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
        }
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<false>,
        location: DirectiveLocation,
    ) {
        self.visit_directive(directive, location, Error::InvalidVariableDirective)
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<true>,
        location: DirectiveLocation,
    ) {
        self.visit_directive(directive, location, Error::InvalidConstDirective)
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for DirectivesAreInValidLocations<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
