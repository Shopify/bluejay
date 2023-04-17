use crate::executable::VariableDefinitionInputTypeReference;
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, VariableDefinition};
use bluejay_core::AsIter;
use std::collections::HashMap;

pub struct Cache<'a, E: ExecutableDocument, S: SchemaDefinition> {
    variable_definition_input_type_references: HashMap<
        &'a E::TypeReference,
        VariableDefinitionInputTypeReference<'a, S::BaseInputTypeReference>,
    >,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Cache<'a, E, S> {
    pub fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
        let variable_definition_input_type_references =
            HashMap::from_iter(executable_document.operation_definitions().iter().flat_map(
                |operation_definition: &'a E::OperationDefinition| {
                    let variable_definitions_iterator = operation_definition
                        .as_ref()
                        .variable_definitions()
                        .map(|variable_definitions: &'a E::VariableDefinitions| -> <E::VariableDefinitions as AsIter>::Iterator<'a> {
                            variable_definitions.iter()
                        });

                    variable_definitions_iterator
                        .into_iter()
                        .flatten()
                        .filter_map(|variable_definition| {
                            let type_reference = variable_definition.r#type();
                            VariableDefinitionInputTypeReference::try_from((
                                schema_definition,
                                type_reference,
                            ))
                            .ok()
                            .map(|vditr| (type_reference, vditr))
                        })
                },
            ));
        Self {
            variable_definition_input_type_references,
        }
    }

    pub fn variable_definition_input_type_reference(
        &self,
        type_reference: &E::TypeReference,
    ) -> Option<&VariableDefinitionInputTypeReference<'a, S::BaseInputTypeReference>> {
        self.variable_definition_input_type_references
            .get(type_reference)
    }
}
