use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition};
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub struct FragmentNameUniqueness<'a, E: ExecutableDocument, S: SchemaDefinition> {
    fragment_definitions: BTreeMap<&'a str, Vec<&'a E::FragmentDefinition>>,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentNameUniqueness<'a, E, S>
{
    fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
        self.fragment_definitions
            .entry(fragment_definition.name())
            .or_default()
            .push(fragment_definition);
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FragmentNameUniqueness<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::iter::FilterMap<
        std::collections::btree_map::IntoIter<&'a str, Vec<&'a E::FragmentDefinition>>,
        fn((&'a str, Vec<&'a E::FragmentDefinition>)) -> Option<Error<'a, E, S>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.fragment_definitions
            .into_iter()
            .filter_map(|(name, fragment_definitions)| {
                (fragment_definitions.len() > 1).then_some(
                    Error::NonUniqueFragmentDefinitionNames {
                        name,
                        fragment_definitions,
                    },
                )
            })
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentNameUniqueness<'a, E, S>
{
    type Error = Error<'a, E, S>;

    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            fragment_definitions: BTreeMap::new(),
            schema_definition: Default::default(),
        }
    }
}
