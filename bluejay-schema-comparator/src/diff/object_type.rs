use std::ops::Not;

use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
use crate::diff::field::FieldDiff;
use bluejay_core::definition::{
    DirectiveLocation, FieldDefinition, FieldsDefinition, InterfaceImplementation,
    InterfaceTypeDefinition, ObjectTypeDefinition, SchemaDefinition,
};
use bluejay_core::AsIter;

pub struct ObjectTypeDiff<'a, S: SchemaDefinition> {
    old_type_definition: &'a S::ObjectTypeDefinition,
    new_type_definition: &'a S::ObjectTypeDefinition,
}

impl<'a, S: SchemaDefinition + 'a> ObjectTypeDiff<'a, S> {
    pub fn new(
        old_type_definition: &'a S::ObjectTypeDefinition,
        new_type_definition: &'a S::ObjectTypeDefinition,
    ) -> Self {
        Self {
            old_type_definition,
            new_type_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        changes.extend(self.interface_additions().map(|interface_definition| {
            Change::ObjectInterfaceAddition {
                object_type_definition: self.old_type_definition,
                interface_type_definition: interface_definition,
            }
        }));
        changes.extend(self.interface_removals().map(|interface_definition| {
            Change::ObjectInterfaceRemoval {
                object_type_definition: self.old_type_definition,
                interface_type_definition: interface_definition,
            }
        }));

        changes.extend(
            self.field_additions()
                .map(|field_definition| Change::FieldAdded {
                    added_field_definition: field_definition,
                    type_name: self.new_type_definition.name(),
                }),
        );
        changes.extend(
            self.field_removals()
                .map(|field_definition| Change::FieldRemoved {
                    removed_field_definition: field_definition,
                    type_name: self.new_type_definition.name(),
                }),
        );

        changes.extend(
            directive_additions::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::Object,
                    member_name: self.old_type_definition.name(),
                },
            ),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_type_definition, self.new_type_definition).map(
                |directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::Object,
                    member_name: self.old_type_definition.name(),
                },
            ),
        );

        // diff common fields
        self.old_type_definition
            .fields_definition()
            .iter()
            .for_each(|old_field: &'a <S as SchemaDefinition>::FieldDefinition| {
                if let Some(new_field) = self
                    .new_type_definition
                    .fields_definition()
                    .get(old_field.name())
                {
                    changes.extend(
                        FieldDiff::new(self.new_type_definition.name(), old_field, new_field)
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

    fn field_additions(&self) -> impl Iterator<Item = &'a S::FieldDefinition> {
        self.new_type_definition
            .fields_definition()
            .iter()
            .filter(|new_field| {
                self.old_type_definition
                    .fields_definition()
                    .contains_field(new_field.name())
                    .not()
            })
    }

    fn field_removals(&self) -> impl Iterator<Item = &'a S::FieldDefinition> {
        self.old_type_definition
            .fields_definition()
            .iter()
            .filter(|old_field| {
                self.new_type_definition
                    .fields_definition()
                    .contains_field(old_field.name())
                    .not()
            })
    }

    fn interface_additions(&self) -> impl Iterator<Item = &'a S::InterfaceTypeDefinition> {
        self.new_type_definition
            .interface_implementations()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter_map(
                |new_interface_impl: &'a <S as SchemaDefinition>::InterfaceImplementation| {
                    self.old_type_definition
                        .interface_implementations()
                        .map_or(true, |interface_implementations| {
                            !interface_implementations.iter().any(|old_interface_impl| {
                                old_interface_impl.interface().name()
                                    == new_interface_impl.interface().name()
                            })
                        })
                        .then_some(new_interface_impl.interface())
                },
            )
    }

    fn interface_removals(&self) -> impl Iterator<Item = &'a S::InterfaceTypeDefinition> {
        self.old_type_definition
            .interface_implementations()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter_map(
                |old_interface_impl: &'a <S as SchemaDefinition>::InterfaceImplementation| {
                    self.new_type_definition
                        .interface_implementations()
                        .map_or(true, |interface_implementations| {
                            !interface_implementations.iter().any(|new_interface_impl| {
                                old_interface_impl.interface().name()
                                    == new_interface_impl.interface().name()
                            })
                        })
                        .then_some(old_interface_impl.interface())
                },
            )
    }
}
