#![feature(assert_matches)]

use bluejay_parser::ast::definition::{DefinitionDocument, SchemaDefinition};
use std::assert_matches::assert_matches;

#[test]
fn test_parser() {
    let s = std::fs::read_to_string("data/schema.docs.graphql").unwrap();
    let (document, errors) = DefinitionDocument::parse(s.as_str());
    assert_eq!(0, errors.len());
    assert_eq!(1246, document.definition_count());

    let schema_definition: Result<SchemaDefinition, _> = (&document).try_into();
    assert_matches!(schema_definition, Ok(_))
}
