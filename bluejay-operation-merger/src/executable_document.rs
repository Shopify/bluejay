use crate::{
    operation_definition::ImplicitMergedOperationDefinition,
    variable_definitions::MergedVariableDefinitions, Context, EmptyDirectives, Error, IdGenerator,
    MergedArgument, MergedArguments, MergedDirective, MergedField, MergedFragmentDefinition,
    MergedFragmentSpread, MergedInlineFragment, MergedOperationDefinition, MergedSelection,
    MergedSelectionSet, MergedValue, MergedVariableDefinition,
};
use bluejay_core::executable::{
    ExecutableDocument, ExplicitOperationDefinition, OperationDefinition,
};
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct ExecutableDocumentEntry<'a, E: ExecutableDocument> {
    executable_document: &'a E,
    context: &'a HashMap<String, String>,
}

impl<'a, E: ExecutableDocument> ExecutableDocumentEntry<'a, E> {
    pub fn new(executable_document: &'a E, context: &'a HashMap<String, String>) -> Self {
        Self {
            executable_document,
            context,
        }
    }
}

pub struct MergedExecutableDocument<'a, E: ExecutableDocument> {
    operation_definitions: IndexMap<Option<&'a str>, MergedOperationDefinition<'a, E>>,
}

impl<'a, E: ExecutableDocument> ExecutableDocument for MergedExecutableDocument<'a, E> {
    type Argument<const CONST: bool> = MergedArgument<'a, CONST>;
    type Arguments<const CONST: bool> = MergedArguments<'a, CONST>;
    type Directive<const CONST: bool> = MergedDirective<'a>;
    type Directives<const CONST: bool> = EmptyDirectives<'a>;
    type Field = MergedField<'a>;
    type FragmentSpread = MergedFragmentSpread<'a>;
    type InlineFragment = MergedInlineFragment<'a>;
    type ExplicitOperationDefinition = MergedOperationDefinition<'a, E>;
    type ImplicitOperationDefinition = ImplicitMergedOperationDefinition<'a>;
    type OperationDefinition = MergedOperationDefinition<'a, E>;
    type FragmentDefinition = MergedFragmentDefinition<'a>;
    type Selection = MergedSelection<'a>;
    type SelectionSet = MergedSelectionSet<'a>;
    type Value<const CONST: bool> = MergedValue<'a, CONST>;
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

impl<'a, E: ExecutableDocument + 'a> MergedExecutableDocument<'a, E> {
    pub fn new(
        entries: impl IntoIterator<Item = ExecutableDocumentEntry<'a, E>>,
    ) -> Result<Self, Vec<Error<'a>>> {
        let mut merged = Self {
            operation_definitions: IndexMap::new(),
        };
        let mut errors = Vec::new();
        let id_generator = IdGenerator::default();

        entries.into_iter().for_each(
            |ExecutableDocumentEntry {
                 executable_document,
                 context,
             }| {
                executable_document
                    .operation_definitions()
                    .for_each(|operation_definition| {
                        let context = Context::new(
                            id_generator.clone(),
                            executable_document,
                            operation_definition,
                            context,
                        );
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
                            if let Err(errs) = EmptyDirectives::ensure_empty::<false, E>(
                                directives,
                                operation_definition_reference
                                    .operation_type()
                                    .associated_directive_location(),
                            ) {
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

                            match merged_variable_definitions.merge(variable_definitions, &context)
                            {
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
            },
        );

        if errors.is_empty() {
            Ok(merged)
        } else {
            Err(errors)
        }
    }
}
