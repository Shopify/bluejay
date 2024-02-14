use crate::executable::{
    document::{Error, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{FieldsDefinition, SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::{ExecutableDocument, Field, Selection, SelectionReference};
use bluejay_core::AsIter;
use std::ops::Not;

pub struct FieldSelections<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for FieldSelections<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self { errors: Vec::new() }
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        r#type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
        if let Some(fields_definition) = r#type.fields_definition() {
            self.errors
                .extend(selection_set.iter().filter_map(|selection| {
                    if let SelectionReference::Field(field) = selection.as_ref() {
                        let name = field.name();
                        fields_definition
                            .contains_field(name)
                            .not()
                            .then_some(Error::FieldDoesNotExistOnType { field, r#type })
                    } else {
                        None
                    }
                }));
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FieldSelections<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FieldSelections<'a, E, S>
{
    type Error = Error<'a, E, S>;
}
