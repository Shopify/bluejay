use crate::executable::{
    document::{Error, Path, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::{ExecutableDocument, FragmentSpread};

pub struct FragmentSpreadTargetDefined<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    cache: &'a Cache<'a, E, S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentSpreadTargetDefined<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            cache,
        }
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a <E as ExecutableDocument>::FragmentSpread,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        _path: &Path<'a, E>,
    ) {
        if self.cache.fragment_definition(fragment_spread.name()).is_none() {
            self.errors
                .push(Error::FragmentSpreadTargetUndefined { fragment_spread });
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentSpreadTargetDefined<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
