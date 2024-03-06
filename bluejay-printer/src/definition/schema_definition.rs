use crate::{
    definition::{
        directive_definition::DirectiveDefinitionPrinter,
        enum_type_definition::EnumTypeDefinitionPrinter,
        input_object_type_definition::InputObjectTypeDefinitionPrinter,
        interface_type_definition::InterfaceTypeDefinitionPrinter,
        object_type_definition::ObjectTypeDefinitionPrinter,
        scalar_type_definition::ScalarTypeDefinitionPrinter,
        union_type_definition::UnionTypeDefinitionPrinter,
    },
    directive::DirectivesPrinter,
    string_value::BlockStringValuePrinter,
};
use bluejay_core::{
    definition::{
        DirectiveDefinition, ObjectTypeDefinition, SchemaDefinition, TypeDefinitionReference,
    },
    AsIter,
};
use std::fmt::{Display, Formatter, Result};

pub struct SchemaDefinitionPrinter<'a, S: SchemaDefinition>(&'a S);

impl<'a, S: SchemaDefinition> SchemaDefinitionPrinter<'a, S> {
    pub fn new(schema_definition: &'a S) -> Self {
        Self(schema_definition)
    }

    pub fn to_string(schema_definition: &'a S) -> String {
        Self::new(schema_definition).to_string()
    }

    fn is_implicit(schema_definition: &S) -> bool {
        schema_definition.description().is_none()
            && schema_definition.query().name() == "Query"
            && schema_definition
                .mutation()
                .map(|mutation| mutation.name() == "Mutation")
                .unwrap_or(true)
            && schema_definition
                .subscription()
                .map(|subscription| subscription.name() == "Subscription")
                .unwrap_or(true)
            && schema_definition
                .directives()
                .map(AsIter::is_empty)
                .unwrap_or(true)
    }

    fn fmt_explicit_schema_definition(schema_definition: &S, f: &mut Formatter<'_>) -> Result {
        if let Some(description) = schema_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "schema")?;

        if let Some(directives) = schema_definition.directives() {
            if !directives.is_empty() {
                write!(f, " {}", DirectivesPrinter::new(directives))?;
            }
        }

        writeln!(f, " {{\n  query: {}", schema_definition.query().name())?;

        if let Some(mutation) = schema_definition.mutation() {
            writeln!(f, "  mutation: {}", mutation.name())?;
        }

        if let Some(subscription) = schema_definition.subscription() {
            writeln!(f, "  subscription: {}", subscription.name())?;
        }

        writeln!(f, "}}")
    }
}

impl<'a, S: SchemaDefinition> Display for SchemaDefinitionPrinter<'a, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(schema_definition) = *self;
        schema_definition
            .directive_definitions()
            .filter(|dd| !dd.is_builtin())
            .enumerate()
            .try_for_each(|(idx, dd)| {
                if idx != 0 {
                    writeln!(f)?;
                }
                write!(f, "{}", DirectiveDefinitionPrinter::new(dd))
            })?;

        let had_directives_to_output = schema_definition
            .directive_definitions()
            .any(|dd| !dd.is_builtin());

        schema_definition
            .type_definitions()
            .filter(|tdr| !tdr.is_builtin())
            .enumerate()
            .try_for_each(|(idx, tdr)| {
                if had_directives_to_output || idx != 0 {
                    writeln!(f)?;
                }
                match tdr {
                    TypeDefinitionReference::BuiltinScalar(_) => Ok(()),
                    TypeDefinitionReference::CustomScalar(cstd) => {
                        write!(f, "{}", ScalarTypeDefinitionPrinter::new(cstd))
                    }
                    TypeDefinitionReference::Enum(etd) => {
                        write!(f, "{}", EnumTypeDefinitionPrinter::new(etd))
                    }
                    TypeDefinitionReference::InputObject(iotd) => {
                        write!(f, "{}", InputObjectTypeDefinitionPrinter::new(iotd))
                    }
                    TypeDefinitionReference::Interface(itd) => {
                        write!(f, "{}", InterfaceTypeDefinitionPrinter::new(itd))
                    }
                    TypeDefinitionReference::Object(otd) => {
                        write!(f, "{}", ObjectTypeDefinitionPrinter::new(otd))
                    }
                    TypeDefinitionReference::Union(utd) => {
                        write!(f, "{}", UnionTypeDefinitionPrinter::new(utd))
                    }
                }
            })?;

        if Self::is_implicit(schema_definition) {
            Ok(())
        } else {
            if had_directives_to_output
                || schema_definition
                    .type_definitions()
                    .any(|td| !td.is_builtin())
            {
                writeln!(f)?;
            }
            Self::fmt_explicit_schema_definition(schema_definition, f)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaDefinitionPrinter;
    use bluejay_parser::ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        Parse,
    };

    #[test]
    fn test_schema_dump() {
        insta::glob!("test_data/schema_definition/*.graphql", |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let document: DefinitionDocument = DefinitionDocument::parse(input.as_str()).unwrap();
            let schema_definition = SchemaDefinition::try_from(&document).unwrap();
            similar_asserts::assert_eq!(
                input,
                SchemaDefinitionPrinter(&schema_definition).to_string()
            );
        });
    }
}
