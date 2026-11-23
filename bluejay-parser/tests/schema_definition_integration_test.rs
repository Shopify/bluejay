use bluejay_core::definition::{
    DirectiveDefinition as _, FieldDefinition as _, ObjectTypeDefinition as _, TypeDefinition as _,
};
use bluejay_core::AsIter;
use bluejay_parser::{
    ast::{
        definition::{DefaultContext, DefinitionDocument, SchemaDefinition, TypeDefinition},
        Parse,
    },
    Error, HasSpan,
};

#[test]
fn test_error() {
    insta::glob!("test_data/schema_definition/error/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document = DefinitionDocument::<DefaultContext>::parse(input.as_str());
        let errors = match definition_document.result {
            Ok(definition_document) => match SchemaDefinition::try_from(&definition_document) {
                Ok(_) => panic!("Document did not have any errors"),
                Err(errors) => errors.into_iter().map(Error::from).collect(),
            },
            Err(errors) => errors,
        };
        let formatted_errors = Error::format_errors(
            input.as_str(),
            path.file_name().and_then(|f| f.to_str()),
            errors,
        );
        insta::assert_snapshot!(formatted_errors);
    });
}

#[test]
fn test_valid() {
    insta::glob!("test_data/schema_definition/valid/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document = DefinitionDocument::<DefaultContext>::parse(input.as_str());
        assert!(definition_document.result.is_ok(), "Document had errors");
        let definition_document = definition_document.result.unwrap();
        assert!(definition_document.type_system_extensions().is_empty());
        let schema_definition = SchemaDefinition::try_from(&definition_document);
        assert!(schema_definition.is_ok(), "Document had errors");
    });
}

fn parse(s: &str) -> DefinitionDocument<'_, DefaultContext> {
    DefinitionDocument::parse(s)
        .result
        .unwrap_or_else(|errors| panic!("`{s}` had parse errors: {errors:?}"))
}

fn text<'a>(s: &'a str, span: &impl HasSpan) -> &'a str {
    &s[span.span().byte_range()]
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
    for (s, expected) in [
        ("type Query {}", "Expected a name"),
        ("enum Status {}", "Expected a name"),
        ("input ProductInput {}", "Expected a name"),
        ("union U =", "Unexpected EOF"),
    ] {
        let errors = DefinitionDocument::<DefaultContext>::parse(s)
            .result
            .expect_err("Expected parse error");
        let errors = Error::into_graphql_errors(s, errors);
        assert_eq!(1, errors.len(), "`{s}`");
        assert_eq!(expected, errors[0].message, "`{s}`");
    }
}

#[test]
fn building_a_schema_rejects_empty_types() {
    let s = "type Query { a: Int }\ntype Empty\nunion U\nenum E\ninput I\ninterface N\n";
    let document = parse(s);
    let errors = SchemaDefinition::try_from(&document).expect_err("Expected errors");
    let mut messages: Vec<String> = errors
        .into_iter()
        .map(Error::from)
        .map(|error| error.message().to_owned())
        .collect();
    messages.sort();
    assert_eq!(
        vec![
            "Enum type `E` must define one or more values",
            "Input object type `I` must define one or more input fields",
            "Interface type `N` must define one or more fields",
            "Object type `Empty` must define one or more fields",
            "Union type `U` must include one or more member types",
        ],
        messages,
    );
}
