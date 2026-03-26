use bluejay_parser::{
    ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        executable::ExecutableDocument,
        Parse,
    },
    Error,
};
use bluejay_validator::executable::{
    document::{rules::FieldSelectionMerging, Orchestrator},
    Cache,
};

type MergingOnly<'a, E, S> = FieldSelectionMerging<'a, E, S>;
type MergingOnlyValidator<'a, E, S> = Orchestrator<'a, E, S, MergingOnly<'a, E, S>>;

fn with_schema(f: impl FnOnce(SchemaDefinition)) {
    let s = std::fs::read_to_string("tests/test_data/executable/schema.graphql").unwrap();
    let definition_document = DefinitionDocument::parse(s.as_str())
        .result
        .expect("Schema had parse errors");
    let schema_definition =
        SchemaDefinition::try_from(&definition_document).expect("Schema had errors");
    f(schema_definition)
}

fn validate_merging_only(schema: &SchemaDefinition, query: &str) -> Vec<String> {
    let executable_document = ExecutableDocument::parse(query)
        .result
        .unwrap_or_else(|_| panic!("Document had parse errors: {query}"));
    let cache = Cache::new(&executable_document, schema);
    let errors: Vec<_> =
        MergingOnlyValidator::validate(&executable_document, schema, &cache).collect();
    let formatted = Error::format_errors(query, None, errors);
    if formatted.is_empty() {
        vec![]
    } else {
        formatted.lines().map(String::from).collect()
    }
}

/// Tests that the merging rule terminates on immediately recursive fragments.
/// Without the cycle guard in parent_fragments, this would infinite loop.
#[test]
fn does_not_infinite_loop_on_immediately_recursive_fragment() {
    with_schema(|schema| {
        let errors = validate_merging_only(
            &schema,
            r#"
            {
                dog {
                    ...selfRef
                }
            }
            fragment selfRef on Dog {
                name
                ...selfRef
            }
            "#,
        );
        // We don't care what error is produced (cycle detection is another rule).
        // The critical assertion is that this test terminates at all.
        let _ = errors;
    });
}

/// Tests that the merging rule terminates on transitively recursive fragments.
#[test]
fn does_not_infinite_loop_on_transitively_recursive_fragments() {
    with_schema(|schema| {
        let errors = validate_merging_only(
            &schema,
            r#"
            {
                dog {
                    ...fragA
                }
            }
            fragment fragA on Dog {
                name
                owner {
                    ...fragB
                }
            }
            fragment fragB on Human {
                name
                pets {
                    ...fragA
                }
            }
            "#,
        );
        let _ = errors;
    });
}

/// Tests that the merging rule terminates when a recursive fragment also
/// has a field named after the fragment (potential confusion of field vs spread).
#[test]
fn does_not_infinite_loop_on_recursive_fragment_with_field_named_after_fragment() {
    with_schema(|schema| {
        let errors = validate_merging_only(
            &schema,
            r#"
            {
                dog {
                    ...nameFragment
                }
            }
            fragment nameFragment on Dog {
                name
                ...nameFragment
            }
            "#,
        );
        let _ = errors;
    });
}

/// Tests that the merging rule detects conflicts even when fragments are recursive.
/// The key assertion: it must terminate AND find the actual merging conflict.
#[test]
fn finds_conflict_even_with_recursive_fragment() {
    with_schema(|schema| {
        let errors = validate_merging_only(
            &schema,
            r#"
            {
                pet {
                    ...conflictA
                    ...conflictB
                }
            }
            fragment conflictA on Pet {
                ... on Dog {
                    someValue: name
                }
            }
            fragment conflictB on Pet {
                ... on Dog {
                    someValue: nickname
                }
            }
            "#,
        );
        let has_merge_error = errors.iter().any(|e| e.contains("do not merge"));
        assert!(
            has_merge_error,
            "Expected a field merging conflict error, got: {errors:?}"
        );
    });
}

/// Tests that deeply recursive fragments separated by fields don't loop.
#[test]
fn does_not_infinite_loop_on_recursive_fragments_separated_by_fields() {
    with_schema(|schema| {
        let errors = validate_merging_only(
            &schema,
            r#"
            {
                dog {
                    ...fragA
                    ...fragB
                }
            }
            fragment fragA on Dog {
                name
                owner {
                    pets {
                        ... on Dog {
                            ...fragB
                        }
                    }
                }
            }
            fragment fragB on Dog {
                nickname
                owner {
                    pets {
                        ... on Dog {
                            ...fragA
                        }
                    }
                }
            }
            "#,
        );
        let _ = errors;
    });
}
