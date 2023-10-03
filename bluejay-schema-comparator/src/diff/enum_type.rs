use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
use crate::diff::enum_value::EnumValueDiff;
use bluejay_core::definition::{
    DirectiveLocation, EnumTypeDefinition, EnumValueDefinition, SchemaDefinition,
};
use bluejay_core::AsIter;

pub struct EnumTypeDiff<'a, S: SchemaDefinition> {
    old_type_definition: &'a S::EnumTypeDefinition,
    new_type_definition: &'a S::EnumTypeDefinition,
}

impl<'a, S: SchemaDefinition + 'a> EnumTypeDiff<'a, S> {
    pub fn new(
        old_type_definition: &'a S::EnumTypeDefinition,
        new_type_definition: &'a S::EnumTypeDefinition,
    ) -> Self {
        Self {
            old_type_definition,
            new_type_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        changes.extend(self.value_additions().map(|enum_value_definition| {
            Change::EnumValueAdded {
                enum_type_definition: self.new_type_definition,
                enum_value_definition,
            }
        }));

        changes.extend(self.value_removals().map(|enum_value_definition| {
            Change::EnumValueRemoved {
                enum_type_definition: self.old_type_definition,
                enum_value_definition,
            }
        }));

        changes.extend(
            directive_additions::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::Enum,
                    member_name: self.old_type_definition.name(),
                },
            ),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::Enum,
                    member_name: self.old_type_definition.name(),
                },
            ),
        );

        // diff common enum values
        self.old_type_definition
            .enum_value_definitions()
            .iter()
            .for_each(|old_enum_value| {
                let new_enum_value = self
                    .new_type_definition
                    .enum_value_definitions()
                    .iter()
                    .find(|new_enum_value| old_enum_value.name() == new_enum_value.name());

                if let Some(new_enum_value) = new_enum_value {
                    changes.extend(
                        EnumValueDiff::new(
                            self.old_type_definition,
                            old_enum_value,
                            new_enum_value,
                        )
                        .diff(),
                    );
                }
            });

        changes.extend(common_directive_changes(
            self.old_type_definition,
            self.new_type_definition,
        ));

        changes
    }

    fn value_additions(&self) -> impl Iterator<Item = &'a S::EnumValueDefinition> {
        self.new_type_definition
            .enum_value_definitions()
            .iter()
            .filter_map(|new_enum_value| {
                self.old_type_definition
                    .enum_value_definitions()
                    .iter()
                    .all(|old_enum_value| old_enum_value.name() != new_enum_value.name())
                    .then_some(new_enum_value)
            })
    }

    fn value_removals(&self) -> impl Iterator<Item = &'a S::EnumValueDefinition> {
        self.old_type_definition
            .enum_value_definitions()
            .iter()
            .filter_map(|old_enum_value| {
                self.new_type_definition
                    .enum_value_definitions()
                    .iter()
                    .all(|new_enum_value| old_enum_value.name() != new_enum_value.name())
                    .then_some(old_enum_value)
            })
    }
}
