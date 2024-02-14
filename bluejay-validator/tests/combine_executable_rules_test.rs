use bluejay_parser::{
    ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        executable::ExecutableDocument,
    },
    Error,
};
use bluejay_validator::{
    combine_executable_rules,
    executable::{
        document::{
            rules::{AllVariableUsagesAllowed, AllVariableUsesDefined},
            Orchestrator,
        },
        Cache,
    },
};

combine_executable_rules!(
    CustomRules,
    bluejay_validator::executable::document::Error,
    [AllVariableUsagesAllowed, AllVariableUsesDefined],
);

type CustomRulesValidator<'a, E, S> = Orchestrator<'a, E, S, CustomRules<'a, E, S>>;

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
    let errors: Vec<_> =
        CustomRulesValidator::validate(&executable_document, &schema_definition, &cache).collect();
    assert!(
        errors.is_empty(),
        "Document had validation errors:\n{}",
        Error::format_errors(executable_document_str, errors),
    )
}
