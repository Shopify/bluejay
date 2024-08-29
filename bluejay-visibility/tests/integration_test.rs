use bluejay_core::{
    definition::{prelude::*, SchemaDefinition as CoreSchemaDefinition, TypeDefinitionReference},
    AsIter, Directive as _,
};
use bluejay_parser::{
    ast::{
        definition::{
            DefaultContext, DefinitionDocument, Directives,
            SchemaDefinition as ParserSchemaDefinition,
        },
        Parse,
    },
    Error,
};
use bluejay_printer::definition::SchemaDefinitionPrinter;
use bluejay_visibility::{Cache, SchemaDefinition, Warden};
use std::marker::PhantomData;

#[derive(Default)]
struct DirectiveWarden<'a>(PhantomData<ParserSchemaDefinition<'a>>);

impl<'a> DirectiveWarden<'a> {
    fn has_visible_directive(directives: Option<&Directives<'a, DefaultContext>>) -> bool {
        directives.map_or(false, |directives| {
            directives
                .iter()
                .any(|directive| directive.name() == "visible")
        })
    }
}

impl<'a> Warden for DirectiveWarden<'a> {
    type SchemaDefinition = ParserSchemaDefinition<'a>;
    type TypeDefinitionsForName<'b> = std::option::IntoIter<
        TypeDefinitionReference<
            'b,
            <Self::SchemaDefinition as CoreSchemaDefinition>::TypeDefinition,
        >,
    > where Self: 'b;

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

    fn is_directive_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::DirectiveDefinition,
    ) -> bool {
        true
    }

    fn is_custom_scalar_type_definition_visible(
        &self,
        custom_scalar_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::CustomScalarTypeDefinition,
    ) -> bool {
        Self::has_visible_directive(custom_scalar_type_definition.directives())
    }

    fn is_enum_type_definition_visible(
        &self,
        enum_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::EnumTypeDefinition,
    ) -> bool {
        Self::has_visible_directive(enum_type_definition.directives())
    }

    fn is_input_object_type_definition_visible(
        &self,
        input_object_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InputObjectTypeDefinition,
    ) -> bool {
        Self::has_visible_directive(input_object_type_definition.directives())
    }

    fn is_interface_type_definition_visible(
        &self,
        interface_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InterfaceTypeDefinition,
    ) -> bool {
        Self::has_visible_directive(interface_type_definition.directives())
    }

    fn is_object_type_definition_visible(
        &self,
        object_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::ObjectTypeDefinition,
    ) -> bool {
        Self::has_visible_directive(object_type_definition.directives())
    }

    fn is_union_type_definition_visible(
        &self,
        union_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::UnionTypeDefinition,
    ) -> bool {
        Self::has_visible_directive(union_type_definition.directives())
    }

    fn type_definitions_for_name<'b>(
        &self,
        schema_definition: &'b Self::SchemaDefinition,
        type_name: &str,
    ) -> Self::TypeDefinitionsForName<'b> {
        schema_definition.get_type_definition(type_name).into_iter()
    }
}

#[test]
fn test_visibility() {
    insta::glob!("test_data/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document: DefinitionDocument = DefinitionDocument::parse(&input)
            .unwrap_or_else(|errors| {
                panic!(
                    "Schema `{}` had parse errors:\n{}",
                    path.display(),
                    Error::format_errors(&input, path.file_name().and_then(|f| f.to_str()), errors)
                )
            });
        let schema_definition = ParserSchemaDefinition::try_from(&definition_document)
            .unwrap_or_else(|errors| {
                panic!(
                    "Schema `{}` had coercion errors:\n:{}",
                    path.display(),
                    Error::format_errors(&input, path.file_name().and_then(|f| f.to_str()), errors)
                )
            });

        let cache = Cache::new(DirectiveWarden::default(), &schema_definition);
        let visibility_scoped_schema_definition = SchemaDefinition::new(&cache).unwrap();

        let printed_schema_definition =
            SchemaDefinitionPrinter::to_string(&visibility_scoped_schema_definition);

        insta::assert_snapshot!(printed_schema_definition);
    });
}

#[test]
fn test_fields_definition_get() {
    let schema = "
        directive @visible on FIELD_DEFINITION | ENUM_VALUE | INPUT_FIELD_DEFINITION | ARGUMENT_DEFINITION | SCALAR | OBJECT | INTERFACE | UNION | ENUM

        type Query @visible {
            otherField: String @visible
            field: String
            field: String @visible
        }
    ";

    let definition_document: DefinitionDocument =
        DefinitionDocument::parse(schema).unwrap_or_else(|errors| {
            panic!(
                "Schema had parse errors:\n{}",
                Error::format_errors(schema, None, errors)
            )
        });
    let schema_definition =
        ParserSchemaDefinition::try_from(&definition_document).unwrap_or_else(|errors| {
            panic!(
                "Schema had coercion errors:\n:{}",
                Error::format_errors(schema, None, errors)
            )
        });

    let cache = Cache::new(DirectiveWarden::default(), &schema_definition);
    let visibility_scoped_schema_definition = SchemaDefinition::new(&cache).unwrap();

    assert_eq!(
        visibility_scoped_schema_definition
            .query()
            .fields_definition()
            .get("field")
            .map(|fd| fd.name()),
        Some("field"),
        "Expected field to be visible",
    );
}
