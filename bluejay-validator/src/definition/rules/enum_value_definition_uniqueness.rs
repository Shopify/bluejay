use crate::definition::{Error, Rule, Visitor};
use bluejay_core::definition::{EnumTypeDefinition, EnumValueDefinition, SchemaDefinition};
use bluejay_core::AsIter;
use std::collections::BTreeMap;

pub struct EnumValueDefinitionUniqueness<'a, S: SchemaDefinition + 'a> {
    errors: Vec<Error<'a, S>>,
}

impl<'a, S: SchemaDefinition> Visitor<'a, S> for EnumValueDefinitionUniqueness<'a, S> {
    fn visit_enum_type_definition(
        &mut self,
        enum_type_definition: &'a <S as SchemaDefinition>::EnumTypeDefinition,
    ) {
        let indexed = enum_type_definition.enum_value_definitions().iter().fold(
            BTreeMap::new(),
            |mut indexed: BTreeMap<&'a str, Vec<&'a S::EnumValueDefinition>>,
             evd: &'a S::EnumValueDefinition| {
                indexed.entry(evd.name()).or_default().push(evd);
                indexed
            },
        );

        self.errors.extend(
            indexed
                .into_iter()
                .filter_map(|(name, enum_value_definitions)| {
                    (enum_value_definitions.len() > 1).then_some(
                        Error::NonUniqueEnumValueDefinitionNames {
                            name,
                            enum_value_definitions,
                        },
                    )
                }),
        );
    }
}

impl<'a, S: SchemaDefinition> IntoIterator for EnumValueDefinitionUniqueness<'a, S> {
    type Item = Error<'a, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, S: SchemaDefinition> Rule<'a, S> for EnumValueDefinitionUniqueness<'a, S> {
    type Error = Error<'a, S>;

    fn new(_: &'a S) -> Self {
        Self { errors: Vec::new() }
    }
}
