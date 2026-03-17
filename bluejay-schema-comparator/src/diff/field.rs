use crate::changes::Change;
use crate::diff::argument::ArgumentDiff;
use crate::diff::directive::diff_directives_into;
use bluejay_core::definition::{
    DirectiveLocation, FieldDefinition, InputValueDefinition, OutputType, SchemaDefinition,
};
use bluejay_core::AsIter;

pub struct FieldDiff<'a, S: SchemaDefinition> {
    type_name: &'a str,
    old_field_definition: &'a S::FieldDefinition,
    new_field_definition: &'a S::FieldDefinition,
}

impl<'a, S: SchemaDefinition + 'a> FieldDiff<'a, S> {
    pub fn new(
        type_name: &'a str,
        old_field_definition: &'a S::FieldDefinition,
        new_field_definition: &'a S::FieldDefinition,
    ) -> Self {
        Self {
            type_name,
            old_field_definition,
            new_field_definition,
        }
    }

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        if self.old_field_definition.description() != self.new_field_definition.description() {
            changes.push(Change::FieldDescriptionChanged {
                type_name: self.type_name,
                old_field_definition: self.old_field_definition,
                new_field_definition: self.new_field_definition,
            });
        }

        if self.old_field_definition.r#type().as_shallow_ref()
            != self.new_field_definition.r#type().as_shallow_ref()
        {
            changes.push(Change::FieldTypeChanged {
                type_name: self.type_name,
                old_field_definition: self.old_field_definition,
                new_field_definition: self.new_field_definition,
            });
        }

        let old_args = self.old_field_definition.arguments_definition();
        let new_args = self.new_field_definition.arguments_definition();

        // Fast path: skip argument diffing when neither field has arguments
        if old_args.is_some() || new_args.is_some() {
            // Argument additions
            new_args
                .map(|ii| ii.iter())
                .into_iter()
                .flatten()
                .for_each(|new_arg| {
                    let found_in_old = old_args
                        .map(|ii| ii.iter())
                        .into_iter()
                        .flatten()
                        .any(|old_arg| old_arg.name() == new_arg.name());
                    if !found_in_old {
                        changes.push(Change::FieldArgumentAdded {
                            type_name: self.type_name,
                            field_definition: self.new_field_definition,
                            argument_definition: new_arg,
                        });
                    }
                });

            // Argument removals + common argument diffs in a single pass
            old_args
                .map(|ii| ii.iter())
                .into_iter()
                .flatten()
                .for_each(|old_argument| {
                    let new_argument = new_args
                        .map(|ii| ii.iter())
                        .into_iter()
                        .flatten()
                        .find(|new_argument| old_argument.name() == new_argument.name());

                    if let Some(new_argument) = new_argument {
                        ArgumentDiff::new(
                            self.type_name,
                            self.old_field_definition,
                            old_argument,
                            new_argument,
                        )
                        .diff_into(changes);
                    } else {
                        changes.push(Change::FieldArgumentRemoved {
                            type_name: self.type_name,
                            field_definition: self.old_field_definition,
                            argument_definition: old_argument,
                        });
                    }
                });
        }

        diff_directives_into::<S, _>(
            self.old_field_definition,
            self.new_field_definition,
            DirectiveLocation::FieldDefinition,
            self.old_field_definition.name(),
            changes,
        );
    }
}
