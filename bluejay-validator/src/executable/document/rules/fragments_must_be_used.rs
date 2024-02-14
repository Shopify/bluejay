use crate::executable::{
    document::{Error, Path, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, FragmentSpread};
use std::collections::BTreeMap;

pub struct FragmentsMustBeUsed<'a, E: ExecutableDocument> {
    unused_fragment_definitions: BTreeMap<&'a str, &'a E::FragmentDefinition>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentsMustBeUsed<'a, E>
{
    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            unused_fragment_definitions: BTreeMap::from_iter(
                executable_document
                    .fragment_definitions()
                    .iter()
                    .map(|fd| (fd.name(), fd)),
            ),
        }
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a <E as ExecutableDocument>::FragmentSpread,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        _path: &Path<'a, E>,
    ) {
        self.unused_fragment_definitions
            .remove(fragment_spread.name());
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentsMustBeUsed<'a, E>
{
    type Error = Error<'a, E, S>;
    type Errors = std::iter::Map<
        std::collections::btree_map::IntoValues<&'a str, &'a E::FragmentDefinition>,
        fn(&'a E::FragmentDefinition) -> Error<'a, E, S>,
    >;

    fn into_errors(self) -> Self::Errors {
        self.unused_fragment_definitions
            .into_values()
            .map(|fragment_definition| Error::FragmentDefinitionUnused {
                fragment_definition,
            })
    }
}
