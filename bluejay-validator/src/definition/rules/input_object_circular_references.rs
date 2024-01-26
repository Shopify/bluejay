use crate::definition::{Error, Rule, Visitor};
use bluejay_core::definition::{
    BaseInputTypeReference, InputObjectTypeDefinition, InputType, InputTypeReference,
    InputValueDefinition, SchemaDefinition,
};
use bluejay_core::AsIter;
use std::collections::HashSet;

pub struct InputObjectCircularReferences<'a, S: SchemaDefinition + 'a> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, S>>,
}

impl<'a, S: SchemaDefinition> Visitor<'a, S> for InputObjectCircularReferences<'a, S> {
    fn visit_input_object_type_definition(
        &mut self,
        input_object_type_definition: &'a <S as SchemaDefinition>::InputObjectTypeDefinition,
    ) {
        let mut circular_references = Vec::new();
        Self::visit_for_circular_references(
            self.schema_definition,
            input_object_type_definition,
            input_object_type_definition,
            &mut circular_references,
            &mut HashSet::new(),
        );

        if !circular_references.is_empty() {
            self.errors
                .push(Error::InputObjectTypeDefinitionCircularReferences {
                    input_object_type_definition,
                    circular_references,
                });
        }
    }
}

impl<'a, S: SchemaDefinition + 'a> InputObjectCircularReferences<'a, S> {
    fn visit_for_circular_references(
        schema_definition: &'a S,
        target: &'a S::InputObjectTypeDefinition,
        iotd: &'a S::InputObjectTypeDefinition,
        circular_references: &mut Vec<&'a S::InputType>,
        encountered: &mut HashSet<&'a str>,
    ) {
        iotd.input_field_definitions().iter().for_each(|ivd| {
            match ivd.r#type().as_ref(schema_definition) {
                InputTypeReference::Base(inner, required) if required => {
                    if inner.name() == target.name() {
                        circular_references.push(ivd.r#type());
                    } else if let BaseInputTypeReference::InputObject(inner_iotd) = inner {
                        if encountered.insert(inner_iotd.name()) {
                            Self::visit_for_circular_references(
                                schema_definition,
                                target,
                                inner_iotd,
                                circular_references,
                                encountered,
                            );
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

impl<'a, S: SchemaDefinition> IntoIterator for InputObjectCircularReferences<'a, S> {
    type Item = Error<'a, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, S: SchemaDefinition> Rule<'a, S> for InputObjectCircularReferences<'a, S> {
    type Error = Error<'a, S>;

    fn new(schema_definition: &'a S) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
        }
    }
}
