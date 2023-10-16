use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
use crate::diff::input_field::InputFieldDiff;
use bluejay_core::definition::{
    DirectiveLocation, InputFieldsDefinition, InputObjectTypeDefinition, InputValueDefinition,
    SchemaDefinition,
};
use bluejay_core::AsIter;

pub struct InputObjectTypeDiff<'a, S: SchemaDefinition> {
    old_type_definition: &'a S::InputObjectTypeDefinition,
    new_type_definition: &'a S::InputObjectTypeDefinition,
}

impl<'a, S: SchemaDefinition + 'a> InputObjectTypeDiff<'a, S> {
    pub fn new(
        old_type_definition: &'a S::InputObjectTypeDefinition,
        new_type_definition: &'a S::InputObjectTypeDefinition,
    ) -> Self {
        Self {
            old_type_definition,
            new_type_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        changes.extend(self.input_field_additions().map(|input_value_definition| {
            Change::InputFieldAdded {
                added_field_definition: input_value_definition,
                input_object_type_definition: self.new_type_definition,
            }
        }));
        changes.extend(self.input_field_removals().map(|input_value_definition| {
            Change::InputFieldRemoved {
                removed_field_definition: input_value_definition,
                input_object_type_definition: self.old_type_definition,
            }
        }));

        changes.extend(
            directive_additions::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::InputObject,
                    member_name: self.old_type_definition.name(),
                },
            ),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::InputObject,
                    member_name: self.old_type_definition.name(),
                },
            ),
        );

        // diff common input fields
        self.old_type_definition
            .input_field_definitions()
            .iter()
            .for_each(
                |old_field: &'a <S as SchemaDefinition>::InputValueDefinition| {
                    let new_field: Option<&'a <S as SchemaDefinition>::InputValueDefinition> = self
                        .new_type_definition
                        .input_field_definitions()
                        .get(old_field.name());

                    if let Some(new_field) = new_field {
                        changes.extend(
                            InputFieldDiff::new(
                                self.old_type_definition,
                                self.new_type_definition,
                                old_field,
                                new_field,
                            )
                            .diff(),
                        );
                    }
                },
            );

        changes.extend(common_directive_changes(
            self.old_type_definition,
            self.new_type_definition,
        ));

        changes
    }

    fn input_field_additions(&self) -> impl Iterator<Item = &'a S::InputValueDefinition> {
        self.new_type_definition
            .input_field_definitions()
            .iter()
            .filter(|new_input_field| {
                self.old_type_definition
                    .input_field_definitions()
                    .get(new_input_field.name())
                    .is_none()
            })
    }

    fn input_field_removals(&self) -> impl Iterator<Item = &'a S::InputValueDefinition> {
        self.old_type_definition
            .input_field_definitions()
            .iter()
            .filter(|old_input_field| {
                self.new_type_definition
                    .input_field_definitions()
                    .get(old_input_field.name())
                    .is_none()
            })
    }
}
