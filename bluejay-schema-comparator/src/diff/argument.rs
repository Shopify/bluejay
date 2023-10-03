use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
use bluejay_core::definition::{
    DirectiveLocation, InputType, InputValueDefinition, SchemaDefinition,
};
use bluejay_core::Value;

pub struct ArgumentDiff<'a, S: SchemaDefinition> {
    type_name: &'a str,
    field_definition: &'a S::FieldDefinition,
    old_argument_definition: &'a S::InputValueDefinition,
    new_argument_definition: &'a S::InputValueDefinition,
}

impl<'a, S: SchemaDefinition + 'a> ArgumentDiff<'a, S> {
    pub fn new(
        type_name: &'a str,
        field_definition: &'a S::FieldDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    ) -> Self {
        Self {
            type_name,
            field_definition,
            old_argument_definition,
            new_argument_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        if self.old_argument_definition.description() != self.new_argument_definition.description()
        {
            changes.push(Change::FieldArgumentDescriptionChanged {
                type_name: self.type_name,
                field_definition: self.field_definition,
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
            changes.push(Change::FieldArgumentTypeChanged {
                type_name: self.type_name,
                field_definition: self.field_definition,
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
                    changes.push(Change::FieldArgumentDefaultValueChanged {
                        type_name: self.type_name,
                        field_definition: self.field_definition,
                        old_argument_definition: self.old_argument_definition,
                        new_argument_definition: self.new_argument_definition,
                    });
                }
            }
            (Some(_), None) | (None, Some(_)) => {
                changes.push(Change::FieldArgumentDefaultValueChanged {
                    type_name: self.type_name,
                    field_definition: self.field_definition,
                    old_argument_definition: self.old_argument_definition,
                    new_argument_definition: self.new_argument_definition,
                });
            }
            (None, None) => {}
        }

        changes.extend(
            directive_additions::<S, _>(self.old_argument_definition, self.new_argument_definition)
                .map(|directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::ArgumentDefinition,
                    member_name: self.old_argument_definition.name(),
                }),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_argument_definition, self.new_argument_definition)
                .map(|directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::ArgumentDefinition,
                    member_name: self.old_argument_definition.name(),
                }),
        );

        changes.extend(common_directive_changes(
            self.old_argument_definition,
            self.new_argument_definition,
        ));

        changes
    }
}
