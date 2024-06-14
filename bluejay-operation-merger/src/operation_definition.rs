use crate::{Context, EmptyDirectives, Id, MergedSelectionSet, MergedVariableDefinitions, Never};
use bluejay_core::{
    executable::{
        ExecutableDocument, ExplicitOperationDefinition, ImplicitOperationDefinition,
        OperationDefinition, OperationDefinitionReference,
    },
    Indexable, OperationType,
};

pub struct MergedOperationDefinition<'a, E: ExecutableDocument> {
    operation_type: OperationType,
    name: Option<&'a str>,
    selection_set: MergedSelectionSet<'a>,
    variable_definitions: Option<MergedVariableDefinitions<'a, E>>,
    id: Id,
}

impl<'a, E: ExecutableDocument> OperationDefinition for MergedOperationDefinition<'a, E> {
    type ExplicitOperationDefinition = Self;
    type ImplicitOperationDefinition = ImplicitMergedOperationDefinition<'a>;

    fn as_ref(&self) -> OperationDefinitionReference<'_, Self> {
        OperationDefinitionReference::Explicit(self)
    }
}

impl<'a, E: ExecutableDocument> Indexable for MergedOperationDefinition<'a, E> {
    type Id = Id;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl<'a, E: ExecutableDocument> ExplicitOperationDefinition for MergedOperationDefinition<'a, E> {
    type Directives = EmptyDirectives<'a>;
    type SelectionSet = MergedSelectionSet<'a>;
    type VariableDefinitions = MergedVariableDefinitions<'a, E>;

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }

    fn name(&self) -> Option<&str> {
        self.name
    }

    fn operation_type(&self) -> OperationType {
        self.operation_type
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }

    fn variable_definitions(&self) -> Option<&Self::VariableDefinitions> {
        self.variable_definitions.as_ref()
    }
}

impl<'a, E: ExecutableDocument> MergedOperationDefinition<'a, E> {
    pub(crate) fn new(
        operation_type: OperationType,
        name: Option<&'a str>,
        context: &Context<'a, E>,
    ) -> Self {
        Self {
            operation_type,
            name,
            id: context.next_id(),
            selection_set: MergedSelectionSet::new(context),
            variable_definitions: None,
        }
    }

    pub(crate) fn selection_set_mut(&mut self) -> &mut MergedSelectionSet<'a> {
        &mut self.selection_set
    }

    pub(crate) fn variable_definitions_mut(
        &mut self,
    ) -> &mut Option<MergedVariableDefinitions<'a, E>> {
        &mut self.variable_definitions
    }
}

/// This is never instantiated because we will always use explicit operation definitions in the merged document.
/// But to conform to the core traits, we need to provide a type that implements `ImplicitOperationDefinition`.
pub struct ImplicitMergedOperationDefinition<'a> {
    selection_set: MergedSelectionSet<'a>,
    /// This field is never used, but its presence ensures this will never be instantiated
    _never: Never,
}

impl<'a> ImplicitOperationDefinition for ImplicitMergedOperationDefinition<'a> {
    type SelectionSet = MergedSelectionSet<'a>;

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}
