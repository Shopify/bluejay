use std::marker::PhantomData;
use crate::validation::executable::{Visitor, Error, Rule};
use crate::executable::{
    ExecutableDocument,
    OperationDefinitionFromExecutableDocument,
    Selection,
    InlineFragment,
    FragmentDefinition,
    FragmentSpread,
};
use crate::OperationType;
use crate::definition::SchemaDefinition;

pub struct SubscriptionOperationSingleRootField<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> {
    invalid_operation_definitions: Vec<&'a OperationDefinitionFromExecutableDocument<'a, E>>,
    executable_document: &'a E,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> Visitor<'a, E, S> for SubscriptionOperationSingleRootField<'a, E, S> {
    fn visit_operation(&mut self, operation_definition: &'a OperationDefinitionFromExecutableDocument<'a, E>) {
        if !matches!(operation_definition.operation_type(), OperationType::Subscription) { return; }
        let selection_set = operation_definition.selection_set();
        if selection_set.as_ref().len() != 1 {
            self.invalid_operation_definitions.push(operation_definition);
        } else if let Some(first_selection) = selection_set.as_ref().first() {
            let fields_count = match first_selection.as_ref() {
                Selection::Field(_) => Some(1),
                Selection::FragmentSpread(fs) => {
                    let fragment_definition = self.executable_document.fragment_definitions().iter().find(|fd| fd.name() == fs.name());
                    fragment_definition.map(|fd| fd.selection_set().as_ref().len())
                },
                Selection::InlineFragment(inline_fragment) => Some(inline_fragment.selection_set().as_ref().len()),
            };
            if matches!(fields_count, Some(fc) if fc != 1) {
                self.invalid_operation_definitions.push(operation_definition);
            }
        }
    }
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> IntoIterator for SubscriptionOperationSingleRootField<'a, E, S> {
    type Item = Error<'a, E, S>;
    type IntoIter = std::iter::Map<std::vec::IntoIter<&'a OperationDefinitionFromExecutableDocument<'a, E>>, fn(&'a OperationDefinitionFromExecutableDocument<'a, E>) -> Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.invalid_operation_definitions.into_iter().map(|operation| Error::SubscriptionRootNotSingleField { operation })
    }
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> Rule<'a, E, S> for SubscriptionOperationSingleRootField<'a, E, S> {
    fn new(executable_document: &'a E, _: &'a S) -> Self {
        Self { invalid_operation_definitions: Vec::new(), executable_document, schema_definition: Default::default() }
    }
}
