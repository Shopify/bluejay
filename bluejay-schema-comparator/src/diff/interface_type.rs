use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
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

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        // Field additions
        changes.extend(
            self.new_interface_definition
                .fields_definition()
                .iter()
                .filter(|field| {
                    !self
                        .old_interface_definition
                        .fields_definition()
                        .contains_field(field.name())
                })
                .map(|field_definition| Change::FieldAdded {
                    added_field_definition: field_definition,
                    type_name: self.old_interface_definition.name(),
                }),
        );

        // Field removals + common field diffs in a single pass
        self.old_interface_definition
            .fields_definition()
            .iter()
            .for_each(|old_field: &'a <S as SchemaDefinition>::FieldDefinition| {
                if let Some(new_field) = self
                    .new_interface_definition
                    .fields_definition()
                    .get(old_field.name())
                {
                    FieldDiff::new(self.new_interface_definition.name(), old_field, new_field)
                        .diff_into(changes);
                } else {
                    changes.push(Change::FieldRemoved {
                        removed_field_definition: old_field,
                        type_name: self.new_interface_definition.name(),
                    });
                }
            });

        diff_directives_into::<S, _>(
            self.old_interface_definition,
            self.new_interface_definition,
            DirectiveLocation::Interface,
            self.old_interface_definition.name(),
            changes,
        );
    }
}
