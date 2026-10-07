use crate::names::{type_name, variables_type_name, ANONYMOUS_OPERATION_STRUCT_NAME};
use crate::validation::Error;
use bluejay_core::{
    definition::SchemaDefinition,
    executable::{ExecutableDocument, FragmentDefinition, OperationDefinition},
    AsIter,
};
use bluejay_validator::executable::{
    document::{Rule, Visitor},
    Cache,
};

/// The struct for the variables of an operation sits next to the structs for operations and fragments, so its name
/// must not be the name of any of those.
pub(crate) struct VariablesStructNamesDoNotClash<
    'a,
    E: ExecutableDocument + 'a,
    S: SchemaDefinition + 'a,
> {
    executable_document: &'a E,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for VariablesStructNamesDoNotClash<'a, E, S>
{
    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            executable_document,
            errors: Vec::new(),
        }
    }

    fn visit_operation_definition(
        &mut self,
        operation_definition: &'a <E as ExecutableDocument>::OperationDefinition,
    ) {
        let operation_definition_reference = operation_definition.as_ref();
        if operation_definition_reference
            .variable_definitions()
            .is_none_or(AsIter::is_empty)
        {
            return;
        }
        let name = variables_type_name(operation_definition_reference.name());
        let executable_document = self.executable_document;

        self.errors.extend(
            executable_document
                .fragment_definitions()
                .filter(|fragment_definition| type_name(fragment_definition.name()) == name)
                .map(
                    |fragment_definition| Error::VariablesStructAndFragmentNamesClash {
                        operation_definition,
                        fragment_definition,
                        name: name.clone(),
                    },
                ),
        );

        self.errors.extend(
            executable_document
                .operation_definitions()
                .filter(|other_operation_definition| {
                    type_name(
                        other_operation_definition
                            .as_ref()
                            .name()
                            .unwrap_or(ANONYMOUS_OPERATION_STRUCT_NAME),
                    ) == name
                })
                .map(
                    |other_operation_definition| Error::VariablesStructAndOperationNamesClash {
                        operation_definition,
                        other_operation_definition,
                        name: name.clone(),
                    },
                ),
        );
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Rule<'a, E, S>
    for VariablesStructNamesDoNotClash<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Self::Error>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
