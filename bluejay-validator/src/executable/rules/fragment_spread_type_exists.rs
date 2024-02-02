use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, InlineFragment};

pub struct FragmentSpreadTypeExists<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    schema_definition: &'a S,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for FragmentSpreadTypeExists<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            schema_definition,
        }
    }

    fn visit_fragment_definition(
        &mut self,
        fragment_definition: &'a <E as ExecutableDocument>::FragmentDefinition,
    ) {
        if self
            .schema_definition
            .get_type_definition(fragment_definition.type_condition())
            .is_none()
        {
            self.errors
                .push(Error::FragmentDefinitionTargetTypeDoesNotExist {
                    fragment_definition,
                });
        }
    }

    fn visit_inline_fragment(
        &mut self,
        inline_fragment: &'a <E as ExecutableDocument>::InlineFragment,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
        if let Some(type_condition) = inline_fragment.type_condition() {
            if self
                .schema_definition
                .get_type_definition(type_condition)
                .is_none()
            {
                self.errors
                    .push(Error::InlineFragmentTargetTypeDoesNotExist { inline_fragment })
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FragmentSpreadTypeExists<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentSpreadTypeExists<'a, E, S>
{
    type Error = Error<'a, E, S>;
}
