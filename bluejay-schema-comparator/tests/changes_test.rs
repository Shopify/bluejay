use bluejay_core::definition::SchemaDefinition as CoreSchemaDefinition;
use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition},
    Parse,
};
use bluejay_schema_comparator::{compare, Change};

#[test]
fn test_safe_field_changes() {
    let old_schema = r#"
        schema {
            query: Query
        }

        type Query {
            a: String!
            b: String
            c: [String!]!
            d: [String]!
            e: [String!]
            f: [String]
            g: [String]
            h: String!
        }
    "#;

    let new_schema = r#"
        schema {
            query: Query
        }

        type Query {
            a: String
            b: String!
            c: [String!]
            d: [String]
            e: [String!]!
            f: [String!]
            g: [String!]!
            h: [String!]!
        }
    "#;

    let document_a: DefinitionDocument = DefinitionDocument::parse(old_schema).result.unwrap();
    let document_b: DefinitionDocument = DefinitionDocument::parse(new_schema).result.unwrap();

    let schema_definition_a = SchemaDefinition::try_from(&document_a).unwrap();
    let schema_definition_b = SchemaDefinition::try_from(&document_b).unwrap();

    let result = compare(&schema_definition_a, &schema_definition_b);

    let changes = result.changes;

    assert_eq!(8, changes.len());

    let change_a = change_for_path(&changes, "Query.a").unwrap();
    assert!(change_a.breaking());

    let change_b = change_for_path(&changes, "Query.b").unwrap();
    assert!(change_b.non_breaking());

    let change_c = change_for_path(&changes, "Query.c").unwrap();
    assert!(change_c.breaking());

    let change_d = change_for_path(&changes, "Query.d").unwrap();
    assert!(change_d.breaking());

    let change_e = change_for_path(&changes, "Query.e").unwrap();
    assert!(change_e.non_breaking());

    let change_f = change_for_path(&changes, "Query.f").unwrap();
    assert!(change_f.non_breaking());

    let change_g = change_for_path(&changes, "Query.g").unwrap();
    assert!(change_g.non_breaking());

    let change_h = change_for_path(&changes, "Query.h").unwrap();
    assert!(change_h.breaking());
}

#[test]
fn test_safe_input_field_changes() {
    let old_schema = r#"
        schema {
            query: Query
        }

        type Query {
            foo(
                a: String!
                b: String
                c: [String!]!
                d: [String]!
                e: [String!]
                f: [String]
                g: [String]
                h: [String!]!
            ): String
        }
    "#;

    let new_schema = r#"
        schema {
            query: Query
        }

        type Query {
            foo(
                a: String
                b: String!
                c: [String!]
                d: [String]
                e: [String!]!
                f: [String!]
                g: [String!]!
                h: String!
            ): String
        }
    "#;

    let document_a: DefinitionDocument = DefinitionDocument::parse(old_schema).result.unwrap();
    let document_b: DefinitionDocument = DefinitionDocument::parse(new_schema).result.unwrap();

    let schema_definition_a = SchemaDefinition::try_from(&document_a).unwrap();
    let schema_definition_b = SchemaDefinition::try_from(&document_b).unwrap();

    let result = compare(&schema_definition_a, &schema_definition_b);
    let changes = result.changes;

    assert_eq!(8, changes.len());

    let change_a = change_for_path(&changes, "Query.foo.a").unwrap();
    assert!(change_a.non_breaking());

    let change_b = change_for_path(&changes, "Query.foo.b").unwrap();
    assert!(change_b.breaking());

    let change_c = change_for_path(&changes, "Query.foo.c").unwrap();
    assert!(change_c.non_breaking());

    let change_d = change_for_path(&changes, "Query.foo.d").unwrap();
    assert!(change_d.non_breaking());

    let change_e = change_for_path(&changes, "Query.foo.e").unwrap();
    assert!(change_e.breaking());

    let change_f = change_for_path(&changes, "Query.foo.f").unwrap();
    assert!(change_f.breaking());

    let change_g = change_for_path(&changes, "Query.foo.g").unwrap();
    assert!(change_g.breaking());

    let change_h = change_for_path(&changes, "Query.foo.h").unwrap();
    assert!(change_h.breaking());
}

fn change_for_path<'a, S: CoreSchemaDefinition>(
    changes: &'a [Change<'a, S>],
    path: &str,
) -> Option<&'a Change<'a, S>> {
    changes.iter().find(|change| change.path() == path)
}
