use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, InlineFragment};

pub struct FragmentsOnCompositeTypes<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    schema_definition: &'a S,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for FragmentsOnCompositeTypes<'a, E, S>
{
    fn visit_fragment_definition(
        &mut self,
        fragment_definition: &'a <E as ExecutableDocument>::FragmentDefinition,
    ) {
        if matches!(
            self
                .schema_definition
                .get_type_definition(fragment_definition.type_condition()),
            Some(tdr) if !tdr.is_composite(),
        ) {
            self.errors
                .push(Error::FragmentDefinitionTargetTypeNotComposite {
                    fragment_definition,
                });
        }
    }

    fn visit_inline_fragment(
        &mut self,
        inline_fragment: &'a <E as ExecutableDocument>::InlineFragment,
    ) {
        if let Some(type_condition) = inline_fragment.type_condition() {
            if matches!(
                self
                    .schema_definition
                    .get_type_definition(type_condition),
                Some(tdr) if !tdr.is_composite(),
            ) {
                self.errors
                    .push(Error::InlineFragmentTargetTypeNotComposite { inline_fragment })
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FragmentsOnCompositeTypes<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentsOnCompositeTypes<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S) -> Self {
        Self {
            errors: Vec::new(),
            schema_definition,
        }
    }
}
