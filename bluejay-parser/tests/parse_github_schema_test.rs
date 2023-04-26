use bluejay_parser::ast::definition::{DefinitionDocument, SchemaDefinition};

#[test]
fn test_parser() {
    let s = std::fs::read_to_string("../data/schema.docs.graphql").unwrap();
    let document = DefinitionDocument::parse(s.as_str()).unwrap();
    assert_eq!(1246, document.definition_count());

    let schema_definition: Result<SchemaDefinition, _> = (&document).try_into();
    assert!(schema_definition.is_ok())
}
