use crate::executable::{
    operation::{Analyzer, VariableValues, Visitor},
    Cache,
};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::ExecutableDocument;
use std::cmp::max;

pub struct QueryDepth {
    current_depth: usize,
    max_depth: usize,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Visitor<'a, E, S, VV>
    for QueryDepth
{
    type ExtraInfo = ();

    fn new(
        _: &'a E::OperationDefinition,
        _s: &'a S,
        _: &'a VV,
        _: &'a Cache<'a, E, S>,
        _: Self::ExtraInfo,
    ) -> Self {
        Self {
            current_depth: 0,
            max_depth: 0,
        }
    }

    fn visit_field(
        &mut self,
        _field: &'a <E as ExecutableDocument>::Field,
        _field_definition: &'a S::FieldDefinition,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
        if included {
            self.current_depth += 1;
            self.max_depth = max(self.max_depth, self.current_depth);
        }
    }

    fn leave_field(
        &mut self,
        _field: &'a <E as ExecutableDocument>::Field,
        _field_definition: &'a S::FieldDefinition,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
        if included {
            self.current_depth -= 1;
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Analyzer<'a, E, S, VV>
    for QueryDepth
{
    type Output = usize;

    fn into_output(self) -> Self::Output {
        self.max_depth
    }
}

#[cfg(test)]
mod tests {
    use super::QueryDepth;
    use crate::executable::{operation::Orchestrator, Cache};
    use bluejay_parser::ast::{
        definition::{
            DefaultContext, DefinitionDocument, SchemaDefinition as ParserSchemaDefinition,
        },
        executable::ExecutableDocument as ParserExecutableDocument,
        Parse,
    };
    use serde_json::{Map as JsonMap, Value as JsonValue};

    type DepthAnalyzer<'a, E, S> = Orchestrator<'a, E, S, JsonMap<String, JsonValue>, QueryDepth>;

    const TEST_SCHEMA: &str = r#"
        type Query {
          node: Node
          thing: Thing!
          ping: String!
        }
        interface Node {
          id: ID!
        }
        type Product implements Node {
            id: ID!
            title: String!
            things: [Thing]!
        }
        type Thing implements Node {
          id: ID!
          title: String!
          parent: Thing!
        }
        schema {
          query: Query
        }
    "#;

    fn check_depth(
        source: &str,
        operation_name: Option<&str>,
        variables: JsonValue,
        expected_depth: usize,
    ) {
        let definition_document: DefinitionDocument<'_, DefaultContext> =
            DefinitionDocument::parse(TEST_SCHEMA).expect("Schema had parse errors");
        let schema_definition =
            ParserSchemaDefinition::try_from(&definition_document).expect("Schema had errors");
        let executable_document = ParserExecutableDocument::parse(source)
            .unwrap_or_else(|_| panic!("Document had parse errors"));
        let cache = Cache::new(&executable_document, &schema_definition);
        let variables = variables.as_object().expect("Variables must be an object");
        let depth = DepthAnalyzer::analyze(
            &executable_document,
            &schema_definition,
            operation_name,
            variables,
            &cache,
            (),
        )
        .unwrap();

        assert_eq!(depth, expected_depth);
    }

    #[test]
    fn basic_depth_metrics() {
        check_depth(r#"{ ping }"#, None, serde_json::json!({}), 1);
        check_depth(r#"{ thing { id } }"#, None, serde_json::json!({}), 2);
        check_depth(
            r#"{ thing { parent { id } } }"#,
            None,
            serde_json::json!({}),
            3,
        );
        check_depth(
            r#"{
                thing { parent { parent { id } } }
                thing { parent { id } }
            }"#,
            None,
            serde_json::json!({}),
            4,
        );
    }

    #[test]
    fn depth_with_operation_context() {
        check_depth(
            r#"
            query D2{
                thing { title }
            }
            query D4 {
                d4: thing { parent { parent { id } } }
            }"#,
            Some("D2"),
            serde_json::json!({}),
            2,
        );
    }

    #[test]
    fn depth_with_inline_fragments() {
        check_depth(
            r#"{
                node {
                    ...on Product { title }
                    ...on Thing { parent { title } }
                }
            }"#,
            None,
            serde_json::json!({}),
            3,
        );
    }

    #[test]
    fn depth_with_fragment_spreads() {
        check_depth(
            r#"{
                node {
                    ...ProductAttrs
                    ...ThingAttrs
                }
            }
            fragment ProductAttrs on Product {
                title
            }
            fragment ThingAttrs on Thing {
                parent { title }
            }
            "#,
            None,
            serde_json::json!({}),
            3,
        );
    }

    #[test]
    fn depth_with_const_skip_include_fields() {
        check_depth(
            r#"{
                d1: ping
                d2: thing @skip(if: false) { title }
                d4: thing @skip(if: true) { parent { parent { id } } }
            }"#,
            None,
            serde_json::json!({}),
            2,
        );

        check_depth(
            r#"{
                d1: ping
                d2: thing @include(if: true) { title }
                d4: thing @include(if: false) { parent { parent { id } } }
            }"#,
            None,
            serde_json::json!({}),
            2,
        );
    }

    #[test]
    fn depth_with_variable_skip_include_fields() {
        check_depth(
            r#"query($no: Boolean, $yes: Boolean) {
                d1: ping
                d2: thing @skip(if: $no) { title }
                d4: thing @skip(if: $yes) { parent { parent { id } } }
            }"#,
            None,
            serde_json::json!({ "no": false, "yes": true }),
            2,
        );

        check_depth(
            r#"query($no: Boolean, $yes: Boolean) {
                d1: ping
                d2: thing @include(if: $yes) { title }
                d4: thing @include(if: $no) { parent { parent { id } } }
            }"#,
            None,
            serde_json::json!({ "no": false, "yes": true }),
            2,
        );
    }

    #[test]
    fn depth_with_default_variable_skip_include_fields() {
        check_depth(
            r#"query($no: Boolean = false, $yes: Boolean = true) {
                d1: ping
                d2: thing @skip(if: $no) { title }
                d4: thing @skip(if: $yes) { parent { parent { id } } }
            }"#,
            None,
            serde_json::json!({}),
            2,
        );
    }

    #[test]
    fn depth_with_skip_include_fragments() {
        check_depth(
            r#"query {
                d1: ping
                ...@skip(if: false) { d2: thing { title } }
                ...@skip(if: true) { d4: thing { parent { parent { id } } } }
            }"#,
            None,
            serde_json::json!({}),
            2,
        );

        check_depth(
            r#"query {
                d1: ping
                ...D2 @skip(if: false)
                ...D4 @skip(if: true)
            }
            fragment D2 on Query {
                d2: thing { title }
            }
            fragment D4 on Query {
                d4: thing { parent { parent { id } } }
            }"#,
            None,
            serde_json::json!({}),
            2,
        );
    }
}
