use crate::{
    operation_definition::ImplicitMergedOperationDefinition,
    variable_definitions::MergedVariableDefinitions, Context, EmptyDirectives, Error, IdGenerator,
    MergedField, MergedFragmentDefinition, MergedFragmentSpread, MergedInlineFragment,
    MergedOperationDefinition, MergedSelection, MergedSelectionSet, MergedVariableDefinition,
};
use bluejay_core::executable::{
    ExecutableDocument, ExplicitOperationDefinition, OperationDefinition,
};
use indexmap::IndexMap;

pub struct MergedExecutableDocument<'a, E: ExecutableDocument> {
    operation_definitions: IndexMap<Option<&'a str>, MergedOperationDefinition<'a, E>>,
}

impl<'a, E: ExecutableDocument> ExecutableDocument for MergedExecutableDocument<'a, E> {
    type Argument<const CONST: bool> = E::Argument<CONST>;
    type Arguments<const CONST: bool> = E::Arguments<CONST>;
    type Directive<const CONST: bool> = E::Directive<CONST>;
    type Directives<const CONST: bool> = EmptyDirectives<CONST, E>;
    type Field = MergedField<'a, E>;
    type FragmentSpread = MergedFragmentSpread<'a, E>;
    type InlineFragment = MergedInlineFragment<'a, E>;
    type ExplicitOperationDefinition = MergedOperationDefinition<'a, E>;
    type ImplicitOperationDefinition = ImplicitMergedOperationDefinition<'a, E>;
    type OperationDefinition = MergedOperationDefinition<'a, E>;
    type FragmentDefinition = MergedFragmentDefinition<'a, E>;
    type Selection = MergedSelection<'a, E>;
    type SelectionSet = MergedSelectionSet<'a, E>;
    type Value<const CONST: bool> = E::Value<CONST>;
    type VariableDefinition = MergedVariableDefinition<'a, E>;
    type VariableDefinitions = MergedVariableDefinitions<'a, E>;
    type VariableType = E::VariableType;
    type FragmentDefinitions<'b> = std::iter::Empty<&'b Self::FragmentDefinition> where Self: 'b;
    type OperationDefinitions<'b> = indexmap::map::Values<'b, Option<&'a str>, MergedOperationDefinition<'a, E>> where Self: 'b;

    fn fragment_definitions(&self) -> Self::FragmentDefinitions<'_> {
        std::iter::empty()
    }

    fn operation_definitions(&self) -> Self::OperationDefinitions<'_> {
        self.operation_definitions.values()
    }
}

impl<'a, E: ExecutableDocument> MergedExecutableDocument<'a, E> {
    pub fn new(
        executable_documents: impl IntoIterator<Item = &'a E>,
    ) -> Result<Self, Vec<Error<'a, E>>> {
        let mut merged = Self {
            operation_definitions: IndexMap::new(),
        };
        let mut errors = Vec::new();
        let id_generator = IdGenerator::default();

        executable_documents
            .into_iter()
            .for_each(|executable_document| {
                let context = Context::new(id_generator.clone(), executable_document);
                executable_document
                    .operation_definitions()
                    .for_each(|operation_definition| {
                        let operation_definition_reference = operation_definition.as_ref();
                        let name = operation_definition_reference.name();
                        let merged_operation =
                            merged.operation_definitions.entry(name).or_insert_with(|| {
                                MergedOperationDefinition::new(
                                    operation_definition_reference.operation_type(),
                                    operation_definition_reference.name(),
                                    &context,
                                )
                            });

                        if let Some(directives) = operation_definition_reference.directives() {
                            if let Err(errs) = EmptyDirectives::<false, E>::ensure_empty(directives)
                            {
                                errors.extend(errs);
                                return;
                            }
                        }

                        if merged_operation.operation_type()
                            != operation_definition_reference.operation_type()
                        {
                            errors.push(Error::OperationTypeMismatch {
                                operation_name: name,
                            });
                            return;
                        }

                        // merge variables
                        if let Some(variable_definitions) =
                            operation_definition_reference.variable_definitions()
                        {
                            let merged_variable_definitions = merged_operation
                                .variable_definitions_mut()
                                .get_or_insert_with(Default::default);

                            match merged_variable_definitions.merge(variable_definitions) {
                                Ok(()) => {}
                                Err(errs) => errors.extend(errs),
                            }
                        }

                        // merge selections
                        match merged_operation
                            .selection_set_mut()
                            .merge(operation_definition_reference.selection_set(), &context)
                        {
                            Ok(()) => {}
                            Err(errs) => errors.extend(errs),
                        }
                    });
            });

        if errors.is_empty() {
            Ok(merged)
        } else {
            Err(errors)
        }
    }
}
