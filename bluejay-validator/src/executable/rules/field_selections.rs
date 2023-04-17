use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::{
    FieldsDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, SchemaDefinition,
    TypeDefinitionReference, TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{ExecutableDocument, Field, Selection};
use std::ops::Not;

pub struct FieldSelections<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for FieldSelections<'a, E, S>
{
    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        r#type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
        let fields_definition = match &r#type {
            TypeDefinitionReference::BuiltinScalarType(_)
            | TypeDefinitionReference::CustomScalarType(_)
            | TypeDefinitionReference::EnumType(_)
            | TypeDefinitionReference::UnionType(_)
            | TypeDefinitionReference::InputObjectType(_) => {
                return;
            }
            TypeDefinitionReference::InterfaceType(itd) => itd.fields_definition(),
            TypeDefinitionReference::ObjectType(otd) => otd.fields_definition(),
        };

        self.errors
            .extend(selection_set.as_ref().iter().filter_map(|selection| {
                if let Selection::Field(field) = selection.as_ref() {
                    let name = field.name();
                    fields_definition
                        .contains_field(name)
                        .not()
                        .then_some(Error::FieldDoesNotExistOnType { field, r#type })
                } else {
                    None
                }
            }))
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
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self { errors: Vec::new() }
    }
}
