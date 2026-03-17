use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
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

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        if self.old_argument_definition.description() != self.new_argument_definition.description()
        {
            changes.push(Change::FieldArgumentDescriptionChanged {
                type_name: self.type_name,
                field_definition: self.field_definition,
                old_argument_definition: self.old_argument_definition,
                new_argument_definition: self.new_argument_definition,
            });
        }

        if self.old_argument_definition.r#type().as_shallow_ref()
            != self.new_argument_definition.r#type().as_shallow_ref()
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

        diff_directives_into::<S, _>(
            self.old_argument_definition,
            self.new_argument_definition,
            DirectiveLocation::ArgumentDefinition,
            self.old_argument_definition.name(),
            changes,
        );
    }
}
