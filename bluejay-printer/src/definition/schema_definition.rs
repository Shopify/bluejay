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
    pub fn fmt<'a, T: SchemaDefinition<'a>, W: Write>(
        schema_definition: &'a T,
        f: &mut W,
    ) -> Result<(), Error> {
        schema_definition
            .directive_definitions()
            .filter(|dd| !dd.is_builtin())
            .try_for_each(|dd| DisplayDirectiveDefinition::fmt(dd, f))?;

        schema_definition
            .type_definitions()
            .try_for_each(|tdr| match tdr {
                TypeDefinitionReference::BuiltinScalarType(_) => Ok(()),
                TypeDefinitionReference::CustomScalarType(cstd, _) => {
                    DisplayScalarTypeDefinition::fmt(cstd.as_ref(), f)
                }
                TypeDefinitionReference::EnumType(etd, _) => {
                    DisplayEnumTypeDefinition::fmt(etd.as_ref(), f)
                }
                TypeDefinitionReference::InputObjectType(iotd, _) => {
                    DisplayInputObjectTypeDefinition::fmt(iotd.as_ref(), f)
                }
                TypeDefinitionReference::InterfaceType(itd, _) => {
                    DisplayInterfaceTypeDefinition::fmt(itd.as_ref(), f)
                }
                TypeDefinitionReference::ObjectType(otd, _) => {
                    DisplayObjectTypeDefinition::fmt(otd.as_ref(), f)
                }
                TypeDefinitionReference::UnionType(utd, _) => {
                    DisplayUnionTypeDefinition::fmt(utd.as_ref(), f)
                }
            })?;

        if Self::is_implicit(schema_definition) {
            Ok(())
        } else {
            Self::fmt_explicit_schema_definition(schema_definition, f)
        }
    }

    pub fn to_string<'a, T: SchemaDefinition<'a>>(schema_definition: &'a T) -> String {
        let mut s = String::new();
        Self::fmt(schema_definition, &mut s).expect("fmt returned an error unexpectedly");
        s
    }

    fn is_implicit<'a, T: SchemaDefinition<'a>>(schema_definition: &T) -> bool {
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

    fn fmt_explicit_schema_definition<'a, T: SchemaDefinition<'a>, W: Write>(
        schema_definition: &'a T,
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

    macro_rules! assert_prints {
        ($val:ident) => {
            let document = DefinitionDocument::parse($val).unwrap();
            let schema_definition = SchemaDefinition::try_from(&document).unwrap();
            assert_eq!($val, DisplaySchemaDefinition::to_string(&schema_definition));
        };
    }

    #[test]
    fn test_schema_dump() {
        let s = r#""""
This is an input object
"""
input MyInput {
  intField: Int!

  booleanField: Boolean!
}

"""
This is an object
"""
type Query {
  """
  This is a field
  """
  foo(
    """
    This is an argument
    """
    bar: MyInput!
  ): Int!

  """
  This is another field
  """
  bar: [String!]!
}

"#;

        assert_prints!(s);
    }
}
