use crate::definition::{BuiltinRules, Rule};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReference};

pub struct Validator<'a, S: SchemaDefinition, R: Rule<'a, S>> {
    schema_definition: &'a S,
    rule: R,
}

pub type BuiltinRulesValidator<'a, S> = Validator<'a, S, BuiltinRules<'a, S>>;

impl<'a, S: SchemaDefinition, R: Rule<'a, S>> Validator<'a, S, R> {
    fn new(schema_definition: &'a S) -> Self {
        Self {
            schema_definition,
            rule: Rule::new(schema_definition),
        }
    }

    fn visit(&mut self) {
        self.schema_definition.type_definitions().for_each(
            |type_definition| match type_definition {
                TypeDefinitionReference::InputObject(iotd) => {
                    self.visit_input_object_type_definition(iotd)
                }
                TypeDefinitionReference::Enum(etd) => self.visit_enum_type_definition(etd),
                _ => {}
            },
        )
    }

    fn visit_input_object_type_definition(
        &mut self,
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
    ) {
        self.rule
            .visit_input_object_type_definition(input_object_type_definition);
    }

    fn visit_enum_type_definition(&mut self, enum_type_definition: &'a S::EnumTypeDefinition) {
        self.rule.visit_enum_type_definition(enum_type_definition);
    }

    pub fn validate(schema_definition: &'a S) -> <Self as IntoIterator>::IntoIter {
        let mut instance = Self::new(schema_definition);
        instance.visit();
        instance.into_iter()
    }
}

impl<'a, S: SchemaDefinition, R: Rule<'a, S>> IntoIterator for Validator<'a, S, R> {
    type Item = R::Error;
    type IntoIter = <R as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.rule.into_iter()
    }
}
