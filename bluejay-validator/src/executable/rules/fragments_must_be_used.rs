use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, FragmentSpread};
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub struct FragmentsMustBeUsed<'a, E: ExecutableDocument, S: SchemaDefinition> {
    unused_fragment_definitions: BTreeMap<&'a str, &'a E::FragmentDefinition>,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentsMustBeUsed<'a, E, S>
{
    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a <E as ExecutableDocument>::FragmentSpread,
    ) {
        self.unused_fragment_definitions
            .remove(fragment_spread.name());
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FragmentsMustBeUsed<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::iter::Map<
        std::collections::btree_map::IntoValues<&'a str, &'a E::FragmentDefinition>,
        fn(&'a E::FragmentDefinition) -> Error<'a, E, S>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.unused_fragment_definitions
            .into_values()
            .map(|fragment_definition| Error::FragmentDefinitionUnused {
                fragment_definition,
            })
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentsMustBeUsed<'a, E, S>
{
    fn new(executable_document: &'a E, _: &'a S) -> Self {
        Self {
            unused_fragment_definitions: BTreeMap::from_iter(
                executable_document
                    .fragment_definitions()
                    .iter()
                    .map(|fd| (fd.name(), fd)),
            ),
            schema_definition: Default::default(),
        }
    }
}
