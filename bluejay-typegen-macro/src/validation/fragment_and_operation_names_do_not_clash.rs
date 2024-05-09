use crate::validation::Error;
use bluejay_core::{
    definition::SchemaDefinition,
    executable::{ExecutableDocument, OperationDefinition},
};
use bluejay_validator::executable::{
    document::{Rule, Visitor},
    Cache,
};

pub(crate) struct FragmentAndOperationNamesDoNotClash<
    'a,
    E: ExecutableDocument + 'a,
    S: SchemaDefinition + 'a,
> {
    errors: Vec<Error<'a, E, S>>,
    cache: &'a Cache<'a, E, S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentAndOperationNamesDoNotClash<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            cache,
        }
    }

    fn visit_operation_definition(
        &mut self,
        operation_definition: &'a <E as ExecutableDocument>::OperationDefinition,
    ) {
        let name_for_operation = operation_definition
            .as_ref()
            .name()
            .unwrap_or(crate::names::ANONYMOUS_OPERATION_STRUCT_NAME);
        if let Some(fragment_definition) = self.cache.fragment_definition(name_for_operation) {
            self.errors.push(Error::FragmentAndOperationNamesClash {
                operation_definition,
                fragment_definition,
            });
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Rule<'a, E, S>
    for FragmentAndOperationNamesDoNotClash<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Self::Error>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
