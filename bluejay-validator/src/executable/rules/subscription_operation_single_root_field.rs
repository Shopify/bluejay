use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    AbstractOperationDefinition, ExecutableDocument, FragmentDefinition, FragmentSpread,
    InlineFragment, Selection,
};
use bluejay_core::{AsIter, OperationType};
use std::marker::PhantomData;

pub struct SubscriptionOperationSingleRootField<'a, E: ExecutableDocument, S: SchemaDefinition> {
    invalid_operation_definitions: Vec<&'a E::OperationDefinition>,
    executable_document: &'a E,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for SubscriptionOperationSingleRootField<'a, E, S>
{
    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        let core_operation_definition = operation_definition.as_ref();
        if !matches!(
            core_operation_definition.operation_type(),
            OperationType::Subscription
        ) {
            return;
        }
        let selection_set = core_operation_definition.selection_set();
        if selection_set.len() != 1 {
            self.invalid_operation_definitions
                .push(operation_definition);
        } else if let Some(first_selection) = selection_set.iter().next() {
            let fields_count = match first_selection.as_ref() {
                Selection::Field(_) => Some(1),
                Selection::FragmentSpread(fs) => {
                    let fragment_definition = self
                        .executable_document
                        .fragment_definitions()
                        .iter()
                        .find(|fd| fd.name() == fs.name());
                    fragment_definition.map(|fd| fd.selection_set().len())
                }
                Selection::InlineFragment(inline_fragment) => {
                    Some(inline_fragment.selection_set().len())
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
        std::vec::IntoIter<&'a E::OperationDefinition>,
        fn(&'a E::OperationDefinition) -> Error<'a, E, S>,
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
    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            invalid_operation_definitions: Vec::new(),
            executable_document,
            schema_definition: Default::default(),
        }
    }
}
