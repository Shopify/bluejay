use std::ops::Not;

use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
use crate::diff::field::FieldDiff;
use bluejay_core::definition::{
    DirectiveLocation, FieldDefinition, FieldsDefinition, InterfaceTypeDefinition, SchemaDefinition,
};
use bluejay_core::AsIter;

pub struct InterfaceTypeDiff<'a, S: SchemaDefinition> {
    old_interface_definition: &'a S::InterfaceTypeDefinition,
    new_interface_definition: &'a S::InterfaceTypeDefinition,
}

impl<'a, S: SchemaDefinition + 'a> InterfaceTypeDiff<'a, S> {
    pub fn new(
        old_interface_definition: &'a S::InterfaceTypeDefinition,
        new_interface_definition: &'a S::InterfaceTypeDefinition,
    ) -> Self {
        Self {
            old_interface_definition,
            new_interface_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes: Vec<Change<'a, S>> = Vec::new();

        changes.extend(
            self.field_additions()
                .map(|field_definition| Change::FieldAdded {
                    added_field_definition: field_definition,
                    type_name: self.old_interface_definition.name(),
                }),
        );
        changes.extend(
            self.field_removals()
                .map(|field_definition| Change::FieldRemoved {
                    removed_field_definition: field_definition,
                    type_name: self.new_interface_definition.name(),
                }),
        );

        changes.extend(
            directive_additions::<S, _>(
                self.old_interface_definition,
                self.new_interface_definition,
            )
            .map(|directive| Change::DirectiveAdded {
                directive,
                location: DirectiveLocation::Interface,
                member_name: self.old_interface_definition.name(),
            }),
        );

        changes.extend(
            directive_removals::<S, _>(
                self.old_interface_definition,
                self.new_interface_definition,
            )
            .map(|directive| Change::DirectiveRemoved {
                directive,
                location: DirectiveLocation::Interface,
                member_name: self.old_interface_definition.name(),
            }),
        );

        // diff common fields
        self.old_interface_definition
            .fields_definition()
            .iter()
            .for_each(|old_field: &'a <S as SchemaDefinition>::FieldDefinition| {
                let new_field: Option<&'a <S as SchemaDefinition>::FieldDefinition> = self
                    .new_interface_definition
                    .fields_definition()
                    .get(old_field.name());

                if let Some(new_field) = new_field {
                    changes.extend(
                        FieldDiff::new(self.new_interface_definition.name(), old_field, new_field)
                            .diff(),
                    );
                }
            });

        changes.extend(common_directive_changes(
            self.old_interface_definition,
            self.new_interface_definition,
        ));

        changes
    }

    fn field_additions(&self) -> impl Iterator<Item = &'a S::FieldDefinition> {
        self.new_interface_definition
            .fields_definition()
            .iter()
            .filter_map(|field: &'a <S as SchemaDefinition>::FieldDefinition| {
                self.old_interface_definition
                    .fields_definition()
                    .contains_field(field.name())
                    .not()
                    .then_some(field)
            })
    }

    fn field_removals(&self) -> impl Iterator<Item = &'a S::FieldDefinition> {
        self.old_interface_definition
            .fields_definition()
            .iter()
            .filter_map(|field: &'a <S as SchemaDefinition>::FieldDefinition| {
                self.new_interface_definition
                    .fields_definition()
                    .contains_field(field.name())
                    .not()
                    .then_some(field)
            })
    }
}
