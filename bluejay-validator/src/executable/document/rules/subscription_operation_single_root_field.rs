use crate::executable::{
    document::{Error, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    ExecutableDocument, FragmentDefinition, FragmentSpread, InlineFragment, OperationDefinition,
    Selection, SelectionReference,
};
use bluejay_core::{AsIter, OperationType};

pub struct SubscriptionOperationSingleRootField<'a, E: ExecutableDocument> {
    invalid_operation_definitions: Vec<&'a E::OperationDefinition>,
    executable_document: &'a E,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for SubscriptionOperationSingleRootField<'a, E>
{
    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            invalid_operation_definitions: Vec::new(),
            executable_document,
        }
    }

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
                SelectionReference::Field(_) => Some(1),
                SelectionReference::FragmentSpread(fs) => {
                    let fragment_definition = self
                        .executable_document
                        .fragment_definitions()
                        .iter()
                        .find(|fd| fd.name() == fs.name());
                    fragment_definition.map(|fd| fd.selection_set().len())
                }
                SelectionReference::InlineFragment(inline_fragment) => {
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

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for SubscriptionOperationSingleRootField<'a, E>
{
    type Error = Error<'a, E, S>;
    type Errors = std::iter::Map<
        std::vec::IntoIter<&'a E::OperationDefinition>,
        fn(&'a E::OperationDefinition) -> Error<'a, E, S>,
    >;

    fn into_errors(self) -> Self::Errors {
        self.invalid_operation_definitions
            .into_iter()
            .map(|operation| Error::SubscriptionRootNotSingleField { operation })
    }
}
