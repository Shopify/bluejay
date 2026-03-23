use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
use crate::diff::field::FieldDiff;
use bluejay_core::definition::{
    DirectiveLocation, FieldDefinition, FieldsDefinition, InterfaceImplementation,
    ObjectTypeDefinition, SchemaDefinition,
};
use bluejay_core::AsIter;
use std::collections::HashMap;

/// Use HashMap for types above this field count threshold.
/// size_hint().0 is O(1) for slice-backed iterators.
const HASHMAP_THRESHOLD: usize = 64;

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

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        changes.extend(self.interface_additions().map(|interface_implementation| {
            Change::ObjectInterfaceAddition {
                object_type_definition: self.old_type_definition,
                interface_implementation,
            }
        }));
        changes.extend(self.interface_removals().map(|interface_implementation| {
            Change::ObjectInterfaceRemoval {
                object_type_definition: self.old_type_definition,
                interface_implementation,
            }
        }));

        let old_hint = self
            .old_type_definition
            .fields_definition()
            .iter()
            .size_hint()
            .0;
        let new_hint = self
            .new_type_definition
            .fields_definition()
            .iter()
            .size_hint()
            .0;

        if old_hint >= HASHMAP_THRESHOLD || new_hint >= HASHMAP_THRESHOLD {
            self.diff_fields_hashmap(changes);
        } else {
            self.diff_fields_linear(changes);
        }

        diff_directives_into::<S, _>(
            self.old_type_definition,
            self.new_type_definition,
            DirectiveLocation::Object,
            self.old_type_definition.name(),
            changes,
        );
    }

    #[cold]
    #[inline(never)]
    fn diff_fields_hashmap(&self, changes: &mut Vec<Change<'a, S>>) {
        let old_fields: HashMap<&str, &'a S::FieldDefinition> = self
            .old_type_definition
            .fields_definition()
            .iter()
            .map(|f| (f.name(), f))
            .collect();
        let new_fields: HashMap<&str, &'a S::FieldDefinition> = self
            .new_type_definition
            .fields_definition()
            .iter()
            .map(|f| (f.name(), f))
            .collect();

        for (&name, &field) in &new_fields {
            if !old_fields.contains_key(name) {
                changes.push(Change::FieldAdded {
                    added_field_definition: field,
                    type_name: self.new_type_definition.name(),
                });
            }
        }

        for (&name, &old_field) in &old_fields {
            if let Some(&new_field) = new_fields.get(name) {
                FieldDiff::new(self.new_type_definition.name(), old_field, new_field)
                    .diff_into(changes);
            } else {
                changes.push(Change::FieldRemoved {
                    removed_field_definition: old_field,
                    type_name: self.new_type_definition.name(),
                });
            }
        }
    }

    #[inline]
    fn diff_fields_linear(&self, changes: &mut Vec<Change<'a, S>>) {
        // Additions
        changes.extend(
            self.new_type_definition
                .fields_definition()
                .iter()
                .filter(|new_field| {
                    !self
                        .old_type_definition
                        .fields_definition()
                        .contains_field(new_field.name())
                })
                .map(|field_definition| Change::FieldAdded {
                    added_field_definition: field_definition,
                    type_name: self.new_type_definition.name(),
                }),
        );

        // Removals + common field diffs in a single pass
        self.old_type_definition
            .fields_definition()
            .iter()
            .for_each(|old_field: &'a <S as SchemaDefinition>::FieldDefinition| {
                if let Some(new_field) = self
                    .new_type_definition
                    .fields_definition()
                    .get(old_field.name())
                {
                    FieldDiff::new(self.new_type_definition.name(), old_field, new_field)
                        .diff_into(changes);
                } else {
                    changes.push(Change::FieldRemoved {
                        removed_field_definition: old_field,
                        type_name: self.new_type_definition.name(),
                    });
                }
            });
    }

    fn interface_additions(&self) -> impl Iterator<Item = &'a S::InterfaceImplementation> {
        self.new_type_definition
            .interface_implementations()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(
                |new_interface_impl: &&'a <S as SchemaDefinition>::InterfaceImplementation| {
                    self.old_type_definition
                        .interface_implementations()
                        .is_none_or(|interface_implementations| {
                            !interface_implementations.iter().any(|old_interface_impl| {
                                old_interface_impl.name() == new_interface_impl.name()
                            })
                        })
                },
            )
    }

    fn interface_removals(&self) -> impl Iterator<Item = &'a S::InterfaceImplementation> {
        self.old_type_definition
            .interface_implementations()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(
                |old_interface_impl: &&'a <S as SchemaDefinition>::InterfaceImplementation| {
                    self.new_type_definition
                        .interface_implementations()
                        .is_none_or(|interface_implementations| {
                            !interface_implementations.iter().any(|new_interface_impl| {
                                old_interface_impl.name() == new_interface_impl.name()
                            })
                        })
                },
            )
    }
}
