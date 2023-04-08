use crate::{
    definition::{
        directive_definition::DisplayDirectiveDefinition,
        enum_type_definition::DisplayEnumTypeDefinition,
        input_object_type_definition::DisplayInputObjectTypeDefinition,
        interface_type_definition::DisplayInterfaceTypeDefinition,
        object_type_definition::DisplayObjectTypeDefinition,
        scalar_type_definition::DisplayScalarTypeDefinition,
        union_type_definition::DisplayUnionTypeDefinition,
    },
    directive::DisplayDirectives,
    string_value::DisplayStringValue,
};
use bluejay_core::{
    definition::{
        DirectiveDefinition, ObjectTypeDefinition, SchemaDefinition, TypeDefinitionReference,
    },
    AsIter,
};
use std::fmt::{Error, Write};

pub struct DisplaySchemaDefinition;

impl DisplaySchemaDefinition {
    pub fn fmt<T: SchemaDefinition, W: Write>(
        schema_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        schema_definition
            .directive_definitions()
            .filter(|dd| !dd.is_builtin())
            .enumerate()
            .try_for_each(|(idx, dd)| {
                if idx != 0 {
                    writeln!(f)?;
                }
                DisplayDirectiveDefinition::fmt(dd, f)
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
                    TypeDefinitionReference::BuiltinScalarType(_) => Ok(()),
                    TypeDefinitionReference::CustomScalarType(cstd) => {
                        DisplayScalarTypeDefinition::fmt(cstd, f)
                    }
                    TypeDefinitionReference::EnumType(etd) => {
                        DisplayEnumTypeDefinition::fmt(etd, f)
                    }
                    TypeDefinitionReference::InputObjectType(iotd) => {
                        DisplayInputObjectTypeDefinition::fmt(iotd, f)
                    }
                    TypeDefinitionReference::InterfaceType(itd) => {
                        DisplayInterfaceTypeDefinition::fmt(itd, f)
                    }
                    TypeDefinitionReference::ObjectType(otd) => {
                        DisplayObjectTypeDefinition::fmt(otd, f)
                    }
                    TypeDefinitionReference::UnionType(utd) => {
                        DisplayUnionTypeDefinition::fmt(utd, f)
                    }
                }
            })?;

        if Self::is_implicit(schema_definition) {
            Ok(())
        } else {
            if had_directives_to_output
                || schema_definition
                    .type_definitions()
                    .any(|tdr| !tdr.is_builtin())
            {
                writeln!(f)?;
            }
            Self::fmt_explicit_schema_definition(schema_definition, f)
        }
    }

    pub fn to_string<T: SchemaDefinition>(schema_definition: &T) -> String {
        let mut s = String::new();
        Self::fmt(schema_definition, &mut s).expect("fmt returned an error unexpectedly");
        s
    }

    fn is_implicit<T: SchemaDefinition>(schema_definition: &T) -> bool {
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
                .schema_directives()
                .map(AsIter::is_empty)
                .unwrap_or(true)
    }

    fn fmt_explicit_schema_definition<T: SchemaDefinition, W: Write>(
        schema_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = schema_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "schema")?;

        if let Some(directives) = schema_definition.schema_directives() {
            if !directives.is_empty() {
                write!(f, " ")?;
                DisplayDirectives::fmt(directives, f)?;
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

#[cfg(test)]
mod tests {
    use super::DisplaySchemaDefinition;
    use bluejay_parser::ast::definition::{DefinitionDocument, SchemaDefinition};

    #[test]
    fn test_schema_dump() {
        insta::glob!("test_data/schema_definition/*.graphql", |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let document = DefinitionDocument::parse(input.as_str()).unwrap();
            let schema_definition = SchemaDefinition::try_from(&document).unwrap();
            similar_asserts::assert_eq!(
                input,
                DisplaySchemaDefinition::to_string(&schema_definition)
            );
        });
    }
}
