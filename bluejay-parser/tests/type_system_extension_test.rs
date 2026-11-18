use bluejay_core::definition::{
    DirectiveDefinition as _, FieldDefinition as _, ObjectTypeDefinition as _, TypeDefinition as _,
};
use bluejay_core::AsIter;
use bluejay_parser::ast::definition::{
    DefaultContext, DefinitionDocument, SchemaDefinition, TypeDefinition, TypeSystemExtension,
};
use bluejay_parser::ast::Parse;
use bluejay_parser::{Error, HasSpan};

fn parse(s: &str) -> DefinitionDocument<'_, DefaultContext> {
    DefinitionDocument::parse(s)
        .result
        .unwrap_or_else(|errors| panic!("`{s}` had parse errors: {errors:?}"))
}

fn assert_parse_error(s: &str) {
    assert!(
        DefinitionDocument::<DefaultContext>::parse(s)
            .result
            .is_err(),
        "Expected `{s}` to be rejected",
    );
}

fn text<'a>(s: &'a str, span: &impl HasSpan) -> &'a str {
    &s[span.span().byte_range()]
}

#[test]
fn parses_every_kind_of_extension_in_source_order() {
    let s = r#"
        extend schema @schemaDirective {
            mutation: Mutation
        }

        extend scalar DateTime @specifiedBy(url: "https://example.com/date-time")

        extend type Query implements Node & Entity @objectDirective {
            product: Product
        }

        extend interface Node implements Entity @interfaceDirective {
            legacyId: ID
        }

        extend union SearchResult @unionDirective = Product | Collection

        extend enum ProductStatus @enumDirective {
            UNLISTED
        }

        extend input ProductInput @inputDirective {
            title: String
        }
    "#;

    let document = parse(s);
    let extensions = document.type_system_extensions();
    assert_eq!(7, extensions.len());
    assert_eq!(7, document.definition_count());

    assert_eq!(
        vec![
            None,
            Some("DateTime"),
            Some("Query"),
            Some("Node"),
            Some("SearchResult"),
            Some("ProductStatus"),
            Some("ProductInput"),
        ],
        extensions
            .iter()
            .map(TypeSystemExtension::name)
            .collect::<Vec<_>>(),
    );
    assert!(extensions
        .iter()
        .all(|extension| extension.directives().is_some_and(|d| d.len() == 1)));

    let TypeSystemExtension::Schema(schema) = &extensions[0] else {
        panic!("Expected a schema extension");
    };
    let root_operation_type_definitions = schema
        .root_operation_type_definitions()
        .expect("Expected root operation type definitions");
    assert_eq!(1, root_operation_type_definitions.len());
    assert_eq!("Mutation", root_operation_type_definitions[0].name());

    let TypeSystemExtension::Object(object) = &extensions[2] else {
        panic!("Expected an object type extension");
    };
    assert_eq!(
        vec!["Node", "Entity"],
        object
            .interface_implementations()
            .expect("Expected interface implementations")
            .iter()
            .map(|implementation| implementation.name_token().as_str())
            .collect::<Vec<_>>(),
    );
    // An extension's fields are only the fields it declares: no builtin `__typename`.
    assert_eq!(
        vec!["product"],
        object
            .fields_definition()
            .expect("Expected fields")
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
    );

    let TypeSystemExtension::Interface(interface) = &extensions[3] else {
        panic!("Expected an interface type extension");
    };
    assert_eq!(
        vec!["legacyId"],
        interface
            .fields_definition()
            .expect("Expected fields")
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
    );

    let TypeSystemExtension::Union(union) = &extensions[4] else {
        panic!("Expected a union type extension");
    };
    assert_eq!(
        vec!["Product", "Collection"],
        union
            .union_member_types()
            .expect("Expected member types")
            .iter()
            .map(|member| member.name_token().as_str())
            .collect::<Vec<_>>(),
    );

    let TypeSystemExtension::Enum(enum_extension) = &extensions[5] else {
        panic!("Expected an enum type extension");
    };
    assert_eq!(
        1,
        enum_extension
            .enum_value_definitions()
            .expect("Expected values")
            .len()
    );

    let TypeSystemExtension::InputObject(input) = &extensions[6] else {
        panic!("Expected an input object type extension");
    };
    assert_eq!(
        1,
        input
            .input_field_definitions()
            .expect("Expected input fields")
            .len()
    );
}

#[test]
fn parses_each_optional_part_of_an_extension_on_its_own() {
    for s in [
        "extend schema @schemaDirective",
        "extend schema { query: Query }",
        "extend scalar DateTime @specifiedBy(url: \"https://example.com/date-time\")",
        "extend type Query implements Node",
        "extend type Query @objectDirective",
        "extend type Query { product: Product }",
        "extend interface Node implements LegacyNode",
        "extend interface Node @interfaceDirective",
        "extend interface Node { legacyId: ID }",
        "extend union SearchResult @unionDirective",
        "extend union SearchResult = Product | Collection",
        "extend union SearchResult = | Product",
        "extend enum ProductStatus @enumDirective",
        "extend enum ProductStatus { UNLISTED }",
        "extend input ProductInput @inputDirective",
        "extend input ProductInput { title: String }",
    ] {
        assert_eq!(1, parse(s).type_system_extensions().len(), "`{s}`");
    }
}

#[test]
fn rejects_extensions_that_add_nothing() {
    for s in [
        "extend schema",
        "extend scalar DateTime",
        "extend type Query",
        "extend interface Node",
        "extend union SearchResult",
        "extend enum ProductStatus",
        "extend input ProductInput",
        "extend type Query {}",
        "extend enum ProductStatus {}",
        "extend input ProductInput {}",
        "extend union SearchResult =",
        "extend schema {}",
    ] {
        assert_parse_error(s);
    }
}

#[test]
fn continues_parsing_after_an_extension_that_adds_nothing() {
    // The token after the empty extension starts the next definition. It is not consumed, so
    // that definition parses and the error in the last extension is also reported.
    let s = "extend type Query\ntype Other { b: Int }\nextend scalar Foo\n";
    let errors = DefinitionDocument::<DefaultContext>::parse(s)
        .result
        .expect_err("Expected parse errors");
    assert_eq!(2, errors.len());
}

#[test]
fn empty_extension_errors_point_to_the_extension_header() {
    for (header, expected) in [
        ("extend schema", "@, {"),
        ("extend scalar DateTime", "@"),
        ("extend type Query", "implements, @, {"),
        ("extend interface Node", "implements, @, {"),
        ("extend union SearchResult", "@, ="),
        ("extend enum ProductStatus", "@, {"),
        ("extend input ProductInput", "@, {"),
    ] {
        for suffix in ["", "\ntype Other { b: Int }\n"] {
            let s = format!("type Query {{ a: Int }}\n  {header}{suffix}");
            let errors = DefinitionDocument::<DefaultContext>::parse(&s)
                .result
                .expect_err("Expected an empty extension error");
            let errors = Error::into_graphql_errors(&s, errors);
            assert_eq!(1, errors.len(), "`{s}`");
            assert_eq!(
                format!("A type system extension must add at least one of: {expected}"),
                errors[0].message,
                "`{s}`",
            );
            assert_eq!(1, errors[0].locations.len());
            assert_eq!(2, errors[0].locations[0].line, "`{s}`");
            assert_eq!(3, errors[0].locations[0].col, "`{s}`");
        }
    }
}

#[test]
fn rejects_unknown_extensions_and_descriptions() {
    for s in [
        // Directive extensions are in the draft spec only.
        "extend directive @foo @bar",
        "extend { query: Query }",
        "extend",
        "extend Query { a: Int }",
        "\"Description\" extend type Query { a: Int }",
        "extend type Query @foo(bar: $variable)",
    ] {
        assert_parse_error(s);
    }
}

#[test]
fn extension_spans_cover_the_whole_extension() {
    let s = r#"# comment
extend schema @a { mutation: Mutation }
extend scalar DateTime @a @b(c: 1)
extend type Query implements Node
extend type Query @a
extend type Query implements Node @a {
  "Description"
  product(id: ID!): Product @b
}
extend interface Node { id: ID! }
extend union SearchResult @a
extend union SearchResult = A | B
extend enum Status @a
extend enum Status { A B }
extend input ProductInput @a
extend input ProductInput { title: String = "x" @a }
type Next { a: Int }
"#;
    let document = parse(s);
    assert_eq!(
        vec![
            "extend schema @a { mutation: Mutation }",
            "extend scalar DateTime @a @b(c: 1)",
            "extend type Query implements Node",
            "extend type Query @a",
            "extend type Query implements Node @a {\n  \"Description\"\n  product(id: ID!): Product @b\n}",
            "extend interface Node { id: ID! }",
            "extend union SearchResult @a",
            "extend union SearchResult = A | B",
            "extend enum Status @a",
            "extend enum Status { A B }",
            "extend input ProductInput @a",
            "extend input ProductInput { title: String = \"x\" @a }",
        ],
        document
            .type_system_extensions()
            .iter()
            .map(|extension| text(s, extension))
            .collect::<Vec<_>>(),
    );
}

#[test]
fn definition_spans_cover_the_whole_definition() {
    let s = r#"
"""
Block description
"""
schema @a { query: Query }
"Description" scalar DateTime @a
scalar Plain
type Query implements Node @a { id: ID! }
type Bodyless implements Node
interface Node @a
union SearchResult @a = A | B
union Empty @a
enum Status { A B }
input ProductInput @a
"Description" directive @a(b: Int) repeatable on FIELD_DEFINITION | OBJECT
"#;
    let document = parse(s);
    assert_eq!(
        vec!["\"\"\"\nBlock description\n\"\"\"\nschema @a { query: Query }"],
        document
            .schema_definitions()
            .iter()
            .map(|definition| text(s, definition))
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        vec![
            "\"Description\" scalar DateTime @a",
            "scalar Plain",
            "type Query implements Node @a { id: ID! }",
            "type Bodyless implements Node",
            "interface Node @a",
            "union SearchResult @a = A | B",
            "union Empty @a",
            "enum Status { A B }",
            "input ProductInput @a",
        ],
        document
            .type_definitions()
            .iter()
            .filter(|definition| !definition.as_ref().is_builtin())
            .map(|definition| {
                &s[definition
                    .span()
                    .expect("Expected a span for a parsed definition")
                    .byte_range()]
            })
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        vec!["\"Description\" directive @a(b: Int) repeatable on FIELD_DEFINITION | OBJECT"],
        document
            .directive_definitions()
            .iter()
            .filter(|definition| !definition.is_builtin())
            .map(|definition| text(s, definition))
            .collect::<Vec<_>>(),
    );
}

#[test]
fn parses_definitions_without_a_body() {
    let s = "type Query\ninterface Node\nunion SearchResult\nenum Status\ninput ProductInput\n";
    let document = parse(s);
    let definitions: Vec<&TypeDefinition<DefaultContext>> = document
        .type_definitions()
        .iter()
        .filter(|definition| !definition.as_ref().is_builtin())
        .collect();
    assert_eq!(5, definitions.len());

    // A bodyless object type still has the builtin `__typename` field, and nothing else.
    let TypeDefinition::Object(query) = definitions[0] else {
        panic!("Expected an object type definition");
    };
    assert!(query
        .fields_definition()
        .iter()
        .all(|field| field.is_builtin()));

    // The body is not optional when it is present: `{}` is not a fields definition.
    for s in [
        "type Query {}",
        "enum Status {}",
        "input ProductInput {}",
        "union U =",
    ] {
        assert_parse_error(s);
    }
}

#[test]
fn building_a_schema_rejects_extensions_and_empty_types() {
    let s = "type Query { a: Int }\nextend type Query { b: Int }\nextend schema @a\n";
    let document = parse(s);
    let errors = SchemaDefinition::try_from(&document).expect_err("Expected errors");
    assert_eq!(2, errors.len());

    let s = "type Query { a: Int }\ntype Empty\nunion U\nenum E\ninput I\ninterface N\n";
    let document = parse(s);
    let errors = SchemaDefinition::try_from(&document).expect_err("Expected errors");
    assert_eq!(5, errors.len());
}

#[test]
fn extension_rejection_graphql_errors_are_descriptive() {
    let s = "type Query { a: Int }\n\
        extend schema @a\n\
        extend scalar DateTime @a\n\
        extend type Query { b: Int }\n\
        extend interface Node { id: ID! }\n\
        extend union SearchResult = Query\n\
        extend enum ProductStatus { UNLISTED }\n\
        extend input ProductInput { title: String }\n";
    let document = parse(s);
    let errors = SchemaDefinition::try_from(&document).expect_err("Expected errors");
    let errors = Error::into_graphql_errors(s, errors);
    assert_eq!(
        vec![
            "Schema extensions are not supported when building a schema",
            "Extension of `DateTime` is not supported when building a schema",
            "Extension of `Query` is not supported when building a schema",
            "Extension of `Node` is not supported when building a schema",
            "Extension of `SearchResult` is not supported when building a schema",
            "Extension of `ProductStatus` is not supported when building a schema",
            "Extension of `ProductInput` is not supported when building a schema",
        ],
        errors
            .iter()
            .map(|error| error.message.as_ref())
            .collect::<Vec<_>>(),
    );
    for (i, error) in errors.iter().enumerate() {
        assert_eq!(1, error.locations.len());
        assert_eq!(i + 2, error.locations[0].line);
        assert_eq!(1, error.locations[0].col);
    }
}

#[test]
fn extensions_are_not_applied_to_a_bodyless_root() {
    let s = "schema { query: QueryRoot }\n\
        type QueryRoot\n\
        extend type QueryRoot { a: Int }\n\
        extend type QueryRoot { b: Int }\n";
    let document = parse(s);
    let root = document
        .type_definitions()
        .iter()
        .find_map(|definition| match definition {
            TypeDefinition::Object(root) if root.name_token().as_str() == "QueryRoot" => Some(root),
            _ => None,
        })
        .expect("Expected the bodyless root definition");
    assert_eq!("type QueryRoot", text(s, root));
    assert!(root
        .fields_definition()
        .iter()
        .all(|field| field.is_builtin()));
    assert_eq!(
        vec![
            "extend type QueryRoot { a: Int }",
            "extend type QueryRoot { b: Int }",
        ],
        document
            .type_system_extensions()
            .iter()
            .map(|extension| text(s, extension))
            .collect::<Vec<_>>(),
    );

    let errors = SchemaDefinition::try_from(&document).expect_err("Expected errors");
    assert_eq!(
        vec![
            "Type system extensions are not supported when building a schema",
            "Type system extensions are not supported when building a schema",
            "Object type `QueryRoot` must define one or more fields",
        ],
        errors
            .into_iter()
            .map(Error::from)
            .map(|error| error.message().to_owned())
            .collect::<Vec<_>>(),
    );
}

#[test]
fn documents_without_extensions_build_as_before() {
    let s = "\"Root\" type Query implements Node { id: ID! }\ninterface Node { id: ID! }\n";
    let document = parse(s);
    assert!(document.type_system_extensions().is_empty());
    let schema_definition = SchemaDefinition::try_from(&document).expect("Expected a schema");
    assert_eq!(
        vec!["__typename", "id", "__schema", "__type"],
        bluejay_core::definition::SchemaDefinition::query(&schema_definition)
            .fields_definition()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
    );
}
