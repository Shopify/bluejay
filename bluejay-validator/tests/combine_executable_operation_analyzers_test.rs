use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition},
    executable::ExecutableDocument,
    Parse,
};
use bluejay_validator::executable::{
    operation::{
        analyzers::{ComplexityCost, QueryDepth},
        Orchestrator,
    },
    Cache,
};
use serde_json::Map;

type CustomAnalyzer<'a, E, S, V, U> =
    Orchestrator<'a, E, S, V, U, (ComplexityCost<'a, E, S, V>, QueryDepth)>;

#[test]
fn test_combine_executable_rules() {
    let definition_document: DefinitionDocument =
        DefinitionDocument::parse("type Query { foo: String! }").expect("Schema had parse errors");
    let schema_definition: SchemaDefinition =
        SchemaDefinition::try_from(&definition_document).expect("Schema had errors");
    let executable_document_str = "{ foo }";
    let executable_document =
        ExecutableDocument::parse(executable_document_str).expect("Document had parse errors");
    let cache = Cache::new(&executable_document, &schema_definition);
    let (complexity_cost, query_depth) = CustomAnalyzer::analyze(
        &executable_document,
        &schema_definition,
        None,
        &Map::new(),
        &cache,
        (),
    )
    .unwrap();
    assert_eq!(complexity_cost, 1);
    assert_eq!(query_depth, 1);
}
