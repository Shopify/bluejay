use crate::{directives::EmptyDirectives, Error, MergedVariableDefinition};
use bluejay_core::{
    executable::{ExecutableDocument, VariableDefinition, VariableDefinitions, VariableType},
    AsIter,
};
use indexmap::{map::Entry, IndexMap};

pub struct MergedVariableDefinitions<'a, E: ExecutableDocument> {
    variable_definitions: IndexMap<&'a str, MergedVariableDefinition<'a, E>>,
}

impl<'a, E: ExecutableDocument> Default for MergedVariableDefinitions<'a, E> {
    fn default() -> Self {
        Self {
            variable_definitions: IndexMap::new(),
        }
    }
}

impl<'a, E: ExecutableDocument> VariableDefinitions for MergedVariableDefinitions<'a, E> {
    type VariableDefinition = MergedVariableDefinition<'a, E>;
}

impl<'a, E: ExecutableDocument> AsIter for MergedVariableDefinitions<'a, E> {
    type Item = MergedVariableDefinition<'a, E>;
    type Iterator<'b> = indexmap::map::Values<'b, &'b str, MergedVariableDefinition<'a, E>> where 'a: 'b, E: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.variable_definitions.values()
    }
}

impl<'a, E: ExecutableDocument + 'a> MergedVariableDefinitions<'a, E> {
    pub(crate) fn merge(
        &mut self,
        variable_definitions: &'a E::VariableDefinitions,
    ) -> Result<(), Vec<Error<'a, E>>> {
        variable_definitions
            .iter()
            .try_for_each(|variable_definition| {
                let name = variable_definition.variable();

                EmptyDirectives::ensure_empty(variable_definition.directives())?;

                match self.variable_definitions.entry(name) {
                    Entry::Occupied(entry) => {
                        if entry.get().r#type().as_ref() != variable_definition.r#type().as_ref() {
                            return Err(vec![Error::VariableTypeMismatch {
                                variable_name: name,
                            }]);
                        }

                        if entry.get().default_value().map(bluejay_core::Value::as_ref)
                            != variable_definition
                                .default_value()
                                .map(bluejay_core::Value::as_ref)
                        {
                            return Err(vec![Error::VariableDefaultValueMismatch {
                                variable_name: name,
                                first: entry.get().default_value(),
                                second: variable_definition.default_value(),
                            }]);
                        }

                        Ok(())
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(MergedVariableDefinition::new(
                            variable_definition.variable(),
                            variable_definition.r#type(),
                            variable_definition.default_value(),
                        ));
                        Ok(())
                    }
                }
            })
    }
}
