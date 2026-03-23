use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
use bluejay_core::definition::{
    DirectiveLocation, InputType, InputValueDefinition, SchemaDefinition,
};
use bluejay_core::Value;

pub struct InputFieldDiff<'a, S: SchemaDefinition> {
    old_type_definition: &'a S::InputObjectTypeDefinition,
    old_field_definition: &'a S::InputValueDefinition,
    new_field_definition: &'a S::InputValueDefinition,
}

impl<'a, S: SchemaDefinition + 'a> InputFieldDiff<'a, S> {
    pub fn new(
        old_type_definition: &'a S::InputObjectTypeDefinition,
        _new_type_definition: &'a S::InputObjectTypeDefinition,
        old_field_definition: &'a S::InputValueDefinition,
        new_field_definition: &'a S::InputValueDefinition,
    ) -> Self {
        Self {
            old_type_definition,
            old_field_definition,
            new_field_definition,
        }
    }

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
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
            (Some(_), None) | (None, Some(_)) => {
                changes.push(Change::InputFieldDefaultValueChanged {
                    input_object_type_definition: self.old_type_definition,
                    old_field_definition: self.old_field_definition,
                    new_field_definition: self.new_field_definition,
                });
            }
            (None, None) => {}
        }

        diff_directives_into::<S, _>(
            self.old_field_definition,
            self.new_field_definition,
            DirectiveLocation::InputFieldDefinition,
            self.old_field_definition.name(),
            changes,
        );
    }
}
