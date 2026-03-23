use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
use crate::diff::input_field::InputFieldDiff;
use bluejay_core::definition::{
    DirectiveLocation, InputFieldsDefinition, InputObjectTypeDefinition, InputValueDefinition,
    SchemaDefinition,
};
use bluejay_core::AsIter;
use std::collections::HashMap;

const HASHMAP_THRESHOLD: usize = 64;

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

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        let old_hint = self
            .old_type_definition
            .input_field_definitions()
            .iter()
            .size_hint()
            .0;
        let new_hint = self
            .new_type_definition
            .input_field_definitions()
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
            DirectiveLocation::InputObject,
            self.old_type_definition.name(),
            changes,
        );
    }

    #[cold]
    #[inline(never)]
    fn diff_fields_hashmap(&self, changes: &mut Vec<Change<'a, S>>) {
        let old_fields: HashMap<&str, &'a S::InputValueDefinition> = self
            .old_type_definition
            .input_field_definitions()
            .iter()
            .map(|f| (f.name(), f))
            .collect();
        let new_fields: HashMap<&str, &'a S::InputValueDefinition> = self
            .new_type_definition
            .input_field_definitions()
            .iter()
            .map(|f| (f.name(), f))
            .collect();

        for (&name, &field) in &new_fields {
            if !old_fields.contains_key(name) {
                changes.push(Change::InputFieldAdded {
                    added_field_definition: field,
                    input_object_type_definition: self.new_type_definition,
                });
            }
        }

        for (&name, &old_field) in &old_fields {
            if let Some(&new_field) = new_fields.get(name) {
                InputFieldDiff::new(
                    self.old_type_definition,
                    self.new_type_definition,
                    old_field,
                    new_field,
                )
                .diff_into(changes);
            } else {
                changes.push(Change::InputFieldRemoved {
                    removed_field_definition: old_field,
                    input_object_type_definition: self.old_type_definition,
                });
            }
        }
    }

    #[inline]
    fn diff_fields_linear(&self, changes: &mut Vec<Change<'a, S>>) {
        // Additions
        changes.extend(
            self.new_type_definition
                .input_field_definitions()
                .iter()
                .filter(|new_field| {
                    self.old_type_definition
                        .input_field_definitions()
                        .get(new_field.name())
                        .is_none()
                })
                .map(|input_value_definition| Change::InputFieldAdded {
                    added_field_definition: input_value_definition,
                    input_object_type_definition: self.new_type_definition,
                }),
        );

        // Removals + common field diffs in a single pass
        self.old_type_definition
            .input_field_definitions()
            .iter()
            .for_each(
                |old_field: &'a <S as SchemaDefinition>::InputValueDefinition| {
                    if let Some(new_field) = self
                        .new_type_definition
                        .input_field_definitions()
                        .get(old_field.name())
                    {
                        InputFieldDiff::new(
                            self.old_type_definition,
                            self.new_type_definition,
                            old_field,
                            new_field,
                        )
                        .diff_into(changes);
                    } else {
                        changes.push(Change::InputFieldRemoved {
                            removed_field_definition: old_field,
                            input_object_type_definition: self.old_type_definition,
                        });
                    }
                },
            );
    }
}
