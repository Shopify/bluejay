use crate::executable::{
    document::{DirectiveError, Error, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;
use bluejay_core::Directive;

pub struct DirectivesAreDefined<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> DirectivesAreDefined<'a, E, S> {
    fn visit_directive<
        const CONST: bool,
        F: Fn(DirectiveError<'a, CONST, E, S>) -> Error<'a, E, S>,
    >(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<CONST>,
        build_error: F,
    ) {
        if self
            .schema_definition
            .get_directive_definition(directive.name())
            .is_none()
        {
            self.errors
                .push(build_error(DirectiveError::DirectiveDoesNotExist {
                    directive,
                }));
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for DirectivesAreDefined<'a, E, S>
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
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_directive(directive, Error::InvalidVariableDirective)
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a <E as ExecutableDocument>::Directive<true>,
        _: bluejay_core::definition::DirectiveLocation,
    ) {
        self.visit_directive(directive, Error::InvalidConstDirective)
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for DirectivesAreDefined<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for DirectivesAreDefined<'a, E, S>
{
    type Error = Error<'a, E, S>;
}
