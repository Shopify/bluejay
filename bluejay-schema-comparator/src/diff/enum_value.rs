use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
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

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        if self.old_value_definition.description() != self.new_value_definition.description() {
            changes.push(Change::EnumValueDescriptionChanged {
                enum_type_definition: self.enum_type_definition,
                old_enum_value_definition: self.old_value_definition,
                new_enum_value_definition: self.new_value_definition,
            });
        }

        changes.extend(
            directive_additions::<S, _>(self.old_value_definition, self.new_value_definition).map(
                |directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::EnumValue,
                    member_name: self.old_value_definition.name(),
                },
            ),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_value_definition, self.new_value_definition).map(
                |directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::EnumValue,
                    member_name: self.old_value_definition.name(),
                },
            ),
        );

        changes.extend(common_directive_changes(
            self.old_value_definition,
            self.new_value_definition,
        ));

        changes
    }
}
