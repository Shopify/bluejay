use crate::changes::Change;
use crate::diff::directive_definition_argument::DirectiveDefinitionArgumentDiff;
use bluejay_core::definition::{
    DirectiveDefinition as _, DirectiveLocation, InputValueDefinition, SchemaDefinition,
};
use bluejay_core::AsIter;

pub struct DirectiveDefinitionDiff<'a, S: SchemaDefinition> {
    old_directive_definition: &'a S::DirectiveDefinition,
    new_directive_definition: &'a S::DirectiveDefinition,
}

impl<'a, S: SchemaDefinition + 'a> DirectiveDefinitionDiff<'a, S> {
    pub fn new(
        old_directive_definition: &'a S::DirectiveDefinition,
        new_directive_definition: &'a S::DirectiveDefinition,
    ) -> Self {
        Self {
            old_directive_definition,
            new_directive_definition,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes: Vec<Change<'a, S>> = Vec::new();

        if self.old_directive_definition.description()
            != self.new_directive_definition.description()
        {
            changes.push(Change::DirectiveDefinitionDescriptionChanged {
                old_directive_definition: self.old_directive_definition,
                new_directive_definition: self.new_directive_definition,
            });
        }

        changes.extend(self.location_additions().map(|location| {
            Change::DirectiveDefinitionLocationAdded {
                directive_definition: self.new_directive_definition,
                location,
            }
        }));

        changes.extend(self.location_removals().map(|location| {
            Change::DirectiveDefinitionLocationRemoved {
                directive_definition: self.new_directive_definition,
                location,
            }
        }));

        changes.extend(self.argument_additions().map(|argument_definition| {
            Change::DirectiveDefinitionArgumentAdded {
                directive_definition: self.new_directive_definition,
                argument_definition,
            }
        }));

        changes.extend(self.argument_removals().map(|argument_definition| {
            Change::DirectiveDefinitionArgumentRemoved {
                directive_definition: self.new_directive_definition,
                argument_definition,
            }
        }));

        // diff common directives
        self.old_directive_definition
            .arguments_definition()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .for_each(
                |old_argument: &'a <S as SchemaDefinition>::InputValueDefinition| {
                    let new_argument: Option<&'a <S as SchemaDefinition>::InputValueDefinition> =
                        self.new_directive_definition
                            .arguments_definition()
                            .map(|ii| ii.iter())
                            .into_iter()
                            .flatten()
                            .find(|new_argument| old_argument.name() == new_argument.name());

                    if let Some(new_argument) = new_argument {
                        changes.extend(
                            DirectiveDefinitionArgumentDiff::new(
                                self.new_directive_definition,
                                old_argument,
                                new_argument,
                            )
                            .diff(),
                        );
                    }
                },
            );

        changes
    }

    fn location_removals(&self) -> impl Iterator<Item = &'a DirectiveLocation> {
        self.old_directive_definition
            .locations()
            .iter()
            .filter(|&old_location| {
                self.new_directive_definition
                    .locations()
                    .iter()
                    .all(|new_location| new_location != old_location)
            })
    }

    fn location_additions(&self) -> impl Iterator<Item = &'a DirectiveLocation> {
        self.new_directive_definition
            .locations()
            .iter()
            .filter(|&new_location| {
                self.old_directive_definition
                    .locations()
                    .iter()
                    .all(|old_location| old_location != new_location)
            })
    }

    fn argument_removals(&self) -> impl Iterator<Item = &'a S::InputValueDefinition> {
        self.old_directive_definition
            .arguments_definition()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(|old_argument| {
                self.new_directive_definition
                    .arguments_definition()
                    .map_or(false, |args| {
                        !args
                            .iter()
                            .any(|new_argument| old_argument.name() == new_argument.name())
                    })
            })
    }

    fn argument_additions(&self) -> impl Iterator<Item = &'a S::InputValueDefinition> {
        self.new_directive_definition
            .arguments_definition()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(|new_argument| {
                self.old_directive_definition
                    .arguments_definition()
                    .map_or(false, |args| {
                        !args
                            .iter()
                            .any(|old_argument| old_argument.name() == new_argument.name())
                    })
            })
    }
}
