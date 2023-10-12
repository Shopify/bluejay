use crate::changes::Change;
use bluejay_core::definition::{InputType, InputValueDefinition, SchemaDefinition};
use bluejay_core::Value;

pub struct DirectiveDefinitionArgumentDiff<'a, S: SchemaDefinition> {
    directive_definition: &'a S::DirectiveDefinition,
    old_argument_definition: &'a S::InputValueDefinition,
    new_argument_definition: &'a S::InputValueDefinition,
}

impl<'a, S: SchemaDefinition + 'a> DirectiveDefinitionArgumentDiff<'a, S> {
    pub fn new(
        directive_definition: &'a S::DirectiveDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    ) -> Self {
        Self {
            directive_definition,
            old_argument_definition,
            new_argument_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        if self.old_argument_definition.description() != self.new_argument_definition.description()
        {
            changes.push(Change::DirectiveDefinitionArgumentDescriptionChanged {
                directive_definition: self.directive_definition,
                old_argument_definition: self.old_argument_definition,
                new_argument_definition: self.new_argument_definition,
            });
        }

        if self
            .old_argument_definition
            .r#type()
            .as_ref()
            .display_name()
            != self
                .new_argument_definition
                .r#type()
                .as_ref()
                .display_name()
        {
            changes.push(Change::DirectiveDefinitionArgumentTypeChanged {
                directive_definition: self.directive_definition,
                old_argument_definition: self.old_argument_definition,
                new_argument_definition: self.new_argument_definition,
            });
        }

        match (
            self.old_argument_definition.default_value(),
            self.new_argument_definition.default_value(),
        ) {
            (Some(old_default), Some(new_default)) => {
                if old_default.as_ref() != new_default.as_ref() {
                    changes.push(Change::DirectiveDefinitionArgumentDefaultValueChanged {
                        directive_definition: self.directive_definition,
                        old_argument_definition: self.old_argument_definition,
                        new_argument_definition: self.new_argument_definition,
                    });
                }
            }
            (Some(_), None) | (None, Some(_)) => {
                changes.push(Change::DirectiveDefinitionArgumentDefaultValueChanged {
                    directive_definition: self.directive_definition,
                    old_argument_definition: self.old_argument_definition,
                    new_argument_definition: self.new_argument_definition,
                });
            }
            _ => {}
        }

        changes
    }
}
