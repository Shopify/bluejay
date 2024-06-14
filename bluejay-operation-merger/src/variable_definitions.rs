use crate::{value::MergedValue, Context, EmptyDirectives, Error, MergedVariableDefinition};
use bluejay_core::{
    definition::DirectiveLocation,
    executable::{ExecutableDocument, VariableDefinition, VariableDefinitions, VariableType},
    AsIter,
};
use indexmap::{map::Entry, IndexMap};
use std::borrow::Cow;

pub struct MergedVariableDefinitions<'a, E: ExecutableDocument> {
    variable_definitions: IndexMap<Cow<'a, str>, MergedVariableDefinition<'a, E>>,
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
    type Iterator<'b> = indexmap::map::Values<'b, Cow<'b, str>, MergedVariableDefinition<'a, E>> where 'a: 'b, E: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.variable_definitions.values()
    }
}

impl<'a, E: ExecutableDocument + 'a> MergedVariableDefinitions<'a, E> {
    pub(crate) fn merge(
        &mut self,
        variable_definitions: &'a E::VariableDefinitions,
        context: &Context<'a, E>,
    ) -> Result<(), Vec<Error<'a>>> {
        variable_definitions
            .iter()
            .try_for_each(|variable_definition| {
                let name = context.variable_name(variable_definition.variable());

                EmptyDirectives::ensure_empty::<true, E>(
                    variable_definition.directives(),
                    DirectiveLocation::VariableDefinition,
                )?;

                match self.variable_definitions.entry(name.clone()) {
                    Entry::Occupied(entry) => {
                        if entry.get().r#type().as_ref() != variable_definition.r#type().as_ref() {
                            return Err(vec![Error::VariableTypeMismatch {
                                variable_name: name,
                            }]);
                        }

                        let existing_default_value = entry.get().default_value();
                        let new_default_value = variable_definition
                            .default_value()
                            .map(|v| MergedValue::new(v, context));

                        if existing_default_value != new_default_value.as_ref() {
                            return Err(vec![Error::VariableDefaultValueMismatch {
                                variable_name: name,
                            }]);
                        }

                        Ok(())
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(MergedVariableDefinition::new(
                            name,
                            variable_definition.r#type(),
                            variable_definition
                                .default_value()
                                .map(|v| MergedValue::new(v, context)),
                        ));
                        Ok(())
                    }
                }
            })
    }
}
