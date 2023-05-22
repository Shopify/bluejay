use crate::executable::{Cache, DirectiveError, Error, Rule, Visitor};
use bluejay_core::definition::{DirectiveDefinition, DirectiveLocation, SchemaDefinition};
use bluejay_core::executable::ExecutableDocument;
use bluejay_core::{AsIter, Directive};
use std::collections::BTreeMap;

pub struct DirectivesAreUniquePerLocation<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a>
    DirectivesAreUniquePerLocation<'a, E, S>
{
    fn visit_directives<
        const CONST: bool,
        F: Fn(DirectiveError<'a, CONST, E, S>) -> Error<'a, E, S>,
    >(
        &mut self,
        directives: &'a <E as ExecutableDocument>::Directives<CONST>,
        build_error: F,
    ) {
        let indexed_directives: BTreeMap<&str, Vec<&'a E::Directive<CONST>>> = directives
            .iter()
            .fold(BTreeMap::new(), |mut indexed_directives, directive| {
                indexed_directives
                    .entry(directive.name())
                    .or_default()
                    .push(directive);
                indexed_directives
            });

        self.errors
            .extend(
                indexed_directives
                    .into_iter()
                    .filter_map(|(directive_name, directives)| {
                        self.schema_definition
                            .get_directive_definition(directive_name)
                            .and_then(|directive_definition| {
                                (!directive_definition.is_repeatable() && directives.len() > 1)
                                    .then(|| {
                                        build_error(
                                            DirectiveError::DirectivesNotUniquePerLocation {
                                                directives,
                                                directive_definition,
                                            },
                                        )
                                    })
                            })
                    }),
            );
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for DirectivesAreUniquePerLocation<'a, E, S>
{
    fn visit_variable_directives(
        &mut self,
        directives: &'a <E as ExecutableDocument>::Directives<false>,
        _: DirectiveLocation,
    ) {
        self.visit_directives(directives, Error::InvalidVariableDirective)
    }

    fn visit_const_directives(
        &mut self,
        directives: &'a <E as ExecutableDocument>::Directives<true>,
        _: DirectiveLocation,
    ) {
        self.visit_directives(directives, Error::InvalidConstDirective)
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for DirectivesAreUniquePerLocation<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for DirectivesAreUniquePerLocation<'a, E, S>
{
    type Error = Error<'a, E, S>;

    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
        }
    }
}
