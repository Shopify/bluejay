use bluejay_parser::ast::{executable::ExecutableDocument, Parse};

/// Verifies that the ExecutableDefinitionsRule from the GraphQL spec
/// (sec 5.1.1) is enforced at the parser level: type and schema definitions
/// are rejected when parsing as an ExecutableDocument.
#[test]
fn rejects_type_definition_in_executable_document() {
    let doc = r#"
        type Foo {
            bar: String
        }
    "#;
    let result = ExecutableDocument::parse(doc).result;
    assert!(
        result.is_err(),
        "Expected parse error for type definition in executable document"
    );
}

#[test]
fn rejects_schema_definition_in_executable_document() {
    let doc = r#"
        schema {
            query: Query
        }
    "#;
    let result = ExecutableDocument::parse(doc).result;
    assert!(
        result.is_err(),
        "Expected parse error for schema definition in executable document"
    );
}

#[test]
fn rejects_mixed_executable_and_type_definitions() {
    let doc = r#"
        query { foo }
        type Bar { baz: Int }
    "#;
    let result = ExecutableDocument::parse(doc).result;
    assert!(
        result.is_err(),
        "Expected parse error when mixing executable and type definitions"
    );
}

#[test]
fn accepts_operations_and_fragments_only() {
    let doc = r#"
        query GetDog {
            dog { ...DogFields }
        }
        fragment DogFields on Dog {
            name
        }
    "#;
    let result = ExecutableDocument::parse(doc).result;
    assert!(
        result.is_ok(),
        "Expected success for document with only operations and fragments"
    );
}
