use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    ExecutableDocument, FragmentDefinition, FragmentSpread, InlineFragment,
    OperationDefinitionFromExecutableDocument, Selection,
};
use bluejay_core::OperationType;
use std::marker::PhantomData;

pub struct SubscriptionOperationSingleRootField<'a, E: ExecutableDocument, S: SchemaDefinition> {
    invalid_operation_definitions: Vec<&'a OperationDefinitionFromExecutableDocument<E>>,
    executable_document: &'a E,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for SubscriptionOperationSingleRootField<'a, E, S>
{
    fn visit_operation_definition(
        &mut self,
        operation_definition: &'a OperationDefinitionFromExecutableDocument<E>,
    ) {
        if !matches!(
            operation_definition.operation_type(),
            OperationType::Subscription
        ) {
            return;
        }
        let selection_set = operation_definition.selection_set();
        if selection_set.as_ref().len() != 1 {
            self.invalid_operation_definitions
                .push(operation_definition);
        } else if let Some(first_selection) = selection_set.as_ref().first() {
            let fields_count = match first_selection.as_ref() {
                Selection::Field(_) => Some(1),
                Selection::FragmentSpread(fs) => {
                    let fragment_definition = self
                        .executable_document
                        .fragment_definitions()
                        .iter()
                        .find(|fd| fd.name() == fs.name());
                    fragment_definition.map(|fd| fd.selection_set().as_ref().len())
                }
                Selection::InlineFragment(inline_fragment) => {
                    Some(inline_fragment.selection_set().as_ref().len())
                }
            };
            if matches!(fields_count, Some(fc) if fc != 1) {
                self.invalid_operation_definitions
                    .push(operation_definition);
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for SubscriptionOperationSingleRootField<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::iter::Map<
        std::vec::IntoIter<&'a OperationDefinitionFromExecutableDocument<E>>,
        fn(&'a OperationDefinitionFromExecutableDocument<E>) -> Error<'a, E, S>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.invalid_operation_definitions
            .into_iter()
            .map(|operation| Error::SubscriptionRootNotSingleField { operation })
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for SubscriptionOperationSingleRootField<'a, E, S>
{
    fn new(executable_document: &'a E, _: &'a S) -> Self {
        Self {
            invalid_operation_definitions: Vec::new(),
            executable_document,
            schema_definition: Default::default(),
        }
    }
}
