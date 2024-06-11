use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition},
    executable::ExecutableDocument,
    Parse,
};
use bluejay_printer::{definition::SchemaDefinitionPrinter, executable::ExecutableDocumentPrinter};
use similar_asserts::assert_eq;

#[test]
fn test_definition_printer() {
    let s = std::fs::read_to_string("../data/schema.docs.graphql").unwrap();
    let original_document: DefinitionDocument = DefinitionDocument::parse(s.as_str()).unwrap();
    let original_schema_definition = SchemaDefinition::try_from(&original_document).unwrap();

    let printed = SchemaDefinitionPrinter::to_string(&original_schema_definition);
    insta::assert_snapshot!(printed);

    let printed_document: DefinitionDocument = DefinitionDocument::parse(printed.as_str()).unwrap();
    let printed_schema_definition = SchemaDefinition::try_from(&printed_document).unwrap();
    let reprinted = SchemaDefinitionPrinter::to_string(&printed_schema_definition);

    assert_eq!(
        original_document.definition_count(),
        printed_document.definition_count()
    );
    similar_asserts::assert_eq!(printed, reprinted);
}

#[test]
fn test_executable_printer() {
    insta::glob!("test_data/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let executable_document = ExecutableDocument::parse(input.as_str())
            .unwrap_or_else(|_| panic!("Document `{}` had parse errors", path.display()));
        let printed = ExecutableDocumentPrinter::to_string(&executable_document);
        assert_eq!(input, printed);
    });
}
