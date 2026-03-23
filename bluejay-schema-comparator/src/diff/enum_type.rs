use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
use crate::diff::enum_value::EnumValueDiff;
use bluejay_core::definition::{
    DirectiveLocation, EnumTypeDefinition, EnumValueDefinition, SchemaDefinition,
};
use bluejay_core::AsIter;
use std::collections::HashMap;

const HASHMAP_THRESHOLD: usize = 64;

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

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        let old_hint = self
            .old_type_definition
            .enum_value_definitions()
            .iter()
            .size_hint()
            .0;
        let new_hint = self
            .new_type_definition
            .enum_value_definitions()
            .iter()
            .size_hint()
            .0;

        if old_hint >= HASHMAP_THRESHOLD || new_hint >= HASHMAP_THRESHOLD {
            self.diff_values_hashmap(changes);
        } else {
            self.diff_values_linear(changes);
        }

        diff_directives_into::<S, _>(
            self.old_type_definition,
            self.new_type_definition,
            DirectiveLocation::Enum,
            self.old_type_definition.name(),
            changes,
        );
    }

    #[cold]
    #[inline(never)]
    fn diff_values_hashmap(&self, changes: &mut Vec<Change<'a, S>>) {
        let old_values: HashMap<&str, &'a S::EnumValueDefinition> = self
            .old_type_definition
            .enum_value_definitions()
            .iter()
            .map(|v| (v.name(), v))
            .collect();
        let new_values: HashMap<&str, &'a S::EnumValueDefinition> = self
            .new_type_definition
            .enum_value_definitions()
            .iter()
            .map(|v| (v.name(), v))
            .collect();

        for (&name, &value) in &new_values {
            if !old_values.contains_key(name) {
                changes.push(Change::EnumValueAdded {
                    enum_type_definition: self.new_type_definition,
                    enum_value_definition: value,
                });
            }
        }

        for (&name, &old_value) in &old_values {
            if let Some(&new_value) = new_values.get(name) {
                EnumValueDiff::new(self.old_type_definition, old_value, new_value)
                    .diff_into(changes);
            } else {
                changes.push(Change::EnumValueRemoved {
                    enum_type_definition: self.old_type_definition,
                    enum_value_definition: old_value,
                });
            }
        }
    }

    #[inline]
    fn diff_values_linear(&self, changes: &mut Vec<Change<'a, S>>) {
        // Additions
        changes.extend(
            self.new_type_definition
                .enum_value_definitions()
                .iter()
                .filter(|new_enum_value| {
                    self.old_type_definition
                        .enum_value_definitions()
                        .iter()
                        .all(|old_enum_value| old_enum_value.name() != new_enum_value.name())
                })
                .map(|enum_value_definition| Change::EnumValueAdded {
                    enum_type_definition: self.new_type_definition,
                    enum_value_definition,
                }),
        );

        // Removals + common value diffs in a single pass
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
                    EnumValueDiff::new(self.old_type_definition, old_enum_value, new_enum_value)
                        .diff_into(changes);
                } else {
                    changes.push(Change::EnumValueRemoved {
                        enum_type_definition: self.old_type_definition,
                        enum_value_definition: old_enum_value,
                    });
                }
            });
    }
}
