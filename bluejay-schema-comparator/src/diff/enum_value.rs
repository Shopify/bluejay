use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
use bluejay_core::definition::{DirectiveLocation, EnumValueDefinition, SchemaDefinition};

pub struct EnumValueDiff<'a, S: SchemaDefinition> {
    enum_type_definition: &'a S::EnumTypeDefinition,
    old_value_definition: &'a S::EnumValueDefinition,
    new_value_definition: &'a S::EnumValueDefinition,
}

impl<'a, S: SchemaDefinition + 'a> EnumValueDiff<'a, S> {
    pub fn new(
        enum_type_definition: &'a S::EnumTypeDefinition,
        old_value_definition: &'a S::EnumValueDefinition,
        new_value_definition: &'a S::EnumValueDefinition,
    ) -> Self {
        Self {
            enum_type_definition,
            old_value_definition,
            new_value_definition,
        }
    }

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        if self.old_value_definition.description() != self.new_value_definition.description() {
            changes.push(Change::EnumValueDescriptionChanged {
                enum_type_definition: self.enum_type_definition,
                old_enum_value_definition: self.old_value_definition,
                new_enum_value_definition: self.new_value_definition,
            });
        }

        diff_directives_into::<S, _>(
            self.old_value_definition,
            self.new_value_definition,
            DirectiveLocation::EnumValue,
            self.old_value_definition.name(),
            changes,
        );
    }
}
