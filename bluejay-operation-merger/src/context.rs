use crate::{id::Id, IdGenerator};
use bluejay_core::{
    executable::{ExecutableDocument, FragmentDefinition, OperationDefinition, VariableDefinition},
    Argument, AsIter, Directive, Value,
};
use std::{borrow::Cow, collections::HashMap};

pub(crate) struct Context<'a, E: ExecutableDocument> {
    id_generator: IdGenerator,
    executable_document: &'a E,
    user_provided_context: &'a HashMap<String, String>,
    variable_name_mapping: HashMap<&'a str, Cow<'a, str>>,
}

impl<'a, E: ExecutableDocument> Context<'a, E> {
    pub(crate) fn new(
        id_generator: IdGenerator,
        executable_document: &'a E,
        operation_definition: &'a E::OperationDefinition,
        user_provided_context: &'a HashMap<String, String>,
    ) -> Self {
        let variable_name_mapping = operation_definition
            .as_ref()
            .variable_definitions()
            .map_or_else(HashMap::new, |variable_definitions| {
                variable_definitions
                    .iter()
                    .map(|variable_definition| {
                        (
                            variable_definition.variable(),
                            Self::name_for_variable_definition(
                                variable_definition,
                                user_provided_context,
                            ),
                        )
                    })
                    .collect()
            });
        Self {
            id_generator,
            executable_document,
            user_provided_context,
            variable_name_mapping,
        }
    }

    pub(crate) fn next_id(&self) -> Id {
        self.id_generator.next()
    }

    pub(crate) fn fragment_definition(&self, name: &str) -> Option<&'a E::FragmentDefinition> {
        self.executable_document
            .fragment_definitions()
            .find(|fd| fd.name() == name)
    }

    pub(crate) fn user_provided_context_for_key(&self, key: &str) -> Option<&'a str> {
        self.user_provided_context.get(key).map(String::as_str)
    }

    fn name_for_variable_definition(
        variable_definition: &'a E::VariableDefinition,
        user_provided_context: &'a HashMap<String, String>,
    ) -> Cow<'a, str> {
        let suffix_on_merge = variable_definition
            .directives()
            .iter()
            .find_map(|directive| {
                if directive.name() == "suffixOnMerge" {
                    directive.arguments().and_then(|arguments| {
                        arguments.iter().find_map(|arg| {
                            if arg.name() == "contextKey" {
                                if let Some(context_key) = arg.value().as_ref().as_string() {
                                    user_provided_context.get(*context_key).map(String::as_str)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    })
                } else {
                    None
                }
            });

        if let Some(suffix) = suffix_on_merge {
            format!("{}{}", variable_definition.variable(), suffix).into()
        } else {
            variable_definition.variable().into()
        }
    }

    pub(crate) fn variable_name(&self, variable: &'a str) -> Cow<'a, str> {
        self.variable_name_mapping
            .get(variable)
            .cloned()
            .unwrap_or(Cow::Borrowed(variable))
    }
}
