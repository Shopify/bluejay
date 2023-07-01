use bluejay_core::{definition::prelude::*, AsIter};
use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition as ParserSchemaDefinition},
    Directives,
};
use bluejay_printer::definition::DisplaySchemaDefinition;
use bluejay_visibility::{Cache, SchemaDefinition, Warden};
use std::marker::PhantomData;

#[derive(Default)]
struct DirectiveWarden<'a>(PhantomData<ParserSchemaDefinition<'a>>);

impl<'a> DirectiveWarden<'a> {
    fn has_visible_directive(directives: Option<&Directives<'a, true>>) -> bool {
        directives.map_or(false, |directives| {
            directives
                .iter()
                .any(|directive| directive.name() == "visible")
        })
    }
}

impl<'a> Warden for DirectiveWarden<'a> {
    type SchemaDefinition = ParserSchemaDefinition<'a>;

    fn is_enum_value_definition_visible(
        &self,
        enum_value_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::EnumValueDefinition,
    ) -> bool {
        Self::has_visible_directive(enum_value_definition.directives())
    }

    fn is_field_definition_visible(
        &self,
        field_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::FieldDefinition,
    ) -> bool {
        Self::has_visible_directive(field_definition.directives())
    }

    fn is_input_value_definition_visible(
        &self,
        input_value_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InputValueDefinition,
    ) -> bool {
        Self::has_visible_directive(input_value_definition.directives())
    }

    fn is_interface_implementation_visible(
        &self,
        _: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InterfaceImplementation,
    ) -> bool {
        true
    }

    fn is_union_member_type_visible(
        &self,
        _: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::UnionMemberType,
    ) -> bool {
        true
    }
}

#[test]
fn test_visibility() {
    let path = std::path::Path::new("tests/test_data/schema.graphql");
    let input = std::fs::read_to_string(path).unwrap();
    let definition_document: DefinitionDocument = DefinitionDocument::parse(&input)
        .unwrap_or_else(|_| panic!("Schema `{}` had parse errors", path.display()));
    let schema_definition = ParserSchemaDefinition::try_from(&definition_document)
        .unwrap_or_else(|_| panic!("Schema `{}` had coercion errors", path.display()));

    let cache = Cache::new(DirectiveWarden::default());
    let visibility_scoped_schema_definition = SchemaDefinition::new(&schema_definition, &cache);

    let printed_schema_definition =
        DisplaySchemaDefinition::to_string(&visibility_scoped_schema_definition);

    insta::assert_snapshot!(printed_schema_definition);
}
