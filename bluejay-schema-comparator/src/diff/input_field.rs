use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
use bluejay_core::definition::{
    DirectiveLocation, InputType, InputValueDefinition, SchemaDefinition,
};
use bluejay_core::Value;

pub struct InputFieldDiff<'a, S: SchemaDefinition> {
    old_type_definition: &'a S::InputObjectTypeDefinition,
    new_type_definition: &'a S::InputObjectTypeDefinition,
    old_field_definition: &'a S::InputValueDefinition,
    new_field_definition: &'a S::InputValueDefinition,
}

impl<'a, S: SchemaDefinition + 'a> InputFieldDiff<'a, S> {
    pub fn new(
        old_type_definition: &'a S::InputObjectTypeDefinition,
        new_type_definition: &'a S::InputObjectTypeDefinition,
        old_field_definition: &'a S::InputValueDefinition,
        new_field_definition: &'a S::InputValueDefinition,
    ) -> Self {
        Self {
            old_type_definition,
            new_type_definition,
            old_field_definition,
            new_field_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        if self.old_field_definition.description() != self.new_field_definition.description() {
            changes.push(Change::InputFieldDescriptionChanged {
                input_object_type_definition: self.old_type_definition,
                old_field_definition: self.old_field_definition,
                new_field_definition: self.new_field_definition,
            });
        }

        if self.old_field_definition.r#type().as_shallow_ref()
            != self.new_field_definition.r#type().as_shallow_ref()
        {
            changes.push(Change::InputFieldTypeChanged {
                input_object_type_definition: self.old_type_definition,
                old_field_definition: self.old_field_definition,
                new_field_definition: self.new_field_definition,
            });
        }

        match (
            self.old_field_definition.default_value(),
            self.new_field_definition.default_value(),
        ) {
            (Some(old_default), Some(new_default)) => {
                if old_default.as_ref() != new_default.as_ref() {
                    changes.push(Change::InputFieldDefaultValueChanged {
                        input_object_type_definition: self.old_type_definition,
                        old_field_definition: self.old_field_definition,
                        new_field_definition: self.new_field_definition,
                    });
                }
            }
            (Some(_), None) => {
                changes.push(Change::InputFieldDefaultValueChanged {
                    input_object_type_definition: self.old_type_definition,
                    old_field_definition: self.old_field_definition,
                    new_field_definition: self.new_field_definition,
                });
            }
            (None, Some(_)) => {
                changes.push(Change::InputFieldDefaultValueChanged {
                    input_object_type_definition: self.old_type_definition,
                    old_field_definition: self.old_field_definition,
                    new_field_definition: self.new_field_definition,
                });
            }
            (None, None) => {}
        }

        changes.extend(
            directive_additions::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::InputFieldDefinition,
                    member_name: self.old_field_definition.name(),
                },
            ),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::InputFieldDefinition,
                    member_name: self.old_field_definition.name(),
                },
            ),
        );

        changes.extend(common_directive_changes(
            self.old_field_definition,
            self.new_field_definition,
        ));

        changes
    }
}
