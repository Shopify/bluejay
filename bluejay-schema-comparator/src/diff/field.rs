use crate::changes::Change;
use crate::diff::argument::ArgumentDiff;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
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

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

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

        changes.extend(self.argument_additions().map(|argument_definition| {
            Change::FieldArgumentAdded {
                type_name: self.type_name,
                field_definition: self.new_field_definition,
                argument_definition,
            }
        }));

        changes.extend(self.argument_removals().map(|argument_definition| {
            Change::FieldArgumentRemoved {
                type_name: self.type_name,
                field_definition: self.old_field_definition,
                argument_definition,
            }
        }));

        // diff common arguments
        self.old_field_definition
            .arguments_definition()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .for_each(|old_argument| {
                let new_argument = self
                    .new_field_definition
                    .arguments_definition()
                    .map(|ii| ii.iter())
                    .into_iter()
                    .flatten()
                    .find(|new_argument| old_argument.name() == new_argument.name());

                if let Some(new_argument) = new_argument {
                    changes.extend(
                        ArgumentDiff::new(
                            self.type_name,
                            self.old_field_definition,
                            old_argument,
                            new_argument,
                        )
                        .diff(),
                    );
                }
            });

        changes.extend(
            directive_additions::<S, _>(self.old_field_definition, self.new_field_definition).map(
                |directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::FieldDefinition,
                    member_name: self.old_field_definition.name(),
                },
            ),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_field_definition, self.new_field_definition).map(
                |directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::FieldDefinition,
                    member_name: self.old_field_definition.name(),
                },
            ),
        );

        changes.extend(common_directive_changes(
            self.old_field_definition,
            self.new_field_definition,
        ));

        changes
    }

    fn argument_additions(&self) -> impl Iterator<Item = &'a S::InputValueDefinition> {
        self.new_field_definition
            .arguments_definition()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(|new_arg| {
                self.old_field_definition
                    .arguments_definition()
                    .map_or(true, |args| {
                        !args.iter().any(|old_arg| old_arg.name() == new_arg.name())
                    })
            })
    }

    fn argument_removals(&self) -> impl Iterator<Item = &'a S::InputValueDefinition> {
        self.old_field_definition
            .arguments_definition()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(|new_arg| {
                self.new_field_definition
                    .arguments_definition()
                    .map_or(true, |args| {
                        !args.iter().any(|old_arg| old_arg.name() == new_arg.name())
                    })
            })
    }
}
