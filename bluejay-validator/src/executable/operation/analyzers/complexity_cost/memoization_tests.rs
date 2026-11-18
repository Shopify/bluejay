use super::{ComplexityCost, RelayCostComputer};
use crate::executable::{
    operation::{Analyzer, Orchestrator, VariableValues, Visitor},
    Cache,
};
use bluejay_core::{
    definition::{SchemaDefinition as CoreSchemaDefinition, TypeDefinitionReference},
    executable::ExecutableDocument as CoreExecutableDocument,
};
use bluejay_parser::ast::{
    definition::{DefaultContext, DefinitionDocument, SchemaDefinition},
    executable::ExecutableDocument,
    Parse,
};
use serde_json::{json, Value};

type ComplexityAnalyzer<'a, E, S, V> =
    Orchestrator<'a, E, S, V, ComplexityCost<'a, E, S, V, RelayCostComputer<'a, E, S, V>>>;

const SCHEMA: &str = r#"
    directive @cost(weight: String!, kind: String) on FIELD_DEFINITION

    interface Item {
        id: ID!
        next: Item
        choice: Choice
        free: Item @cost(weight: "0.0")
        value: Int @cost(weight: "2.0")
    }

    type A implements Item {
        id: ID!
        next: Item
        choice: Choice
        free: Item @cost(weight: "0.0")
        value: Int @cost(weight: "5.0")
        extra: Int @cost(weight: "11.0")
    }

    type B implements Item {
        id: ID!
        next: Item
        choice: Choice
        free: Item @cost(weight: "0.0")
        value: Int @cost(weight: "7.0")
        extra: Int @cost(weight: "13.0")
    }

    type C implements Item {
        id: ID!
        next: Item
        choice: Choice
        free: Item @cost(weight: "0.0")
        value: Int @cost(weight: "2.0")
    }

    union Choice = A | B | C

    type Connection {
        nodes: [Item!]!
    }

    type Query {
        item: Item
        choice: Choice
        connection(first: Int): Connection @cost(weight: "1.0", kind: "connection")
    }
"#;

fn check_complexity(source: &str, variables: Value, expected: usize) {
    let definition_document: DefinitionDocument<'_, DefaultContext> =
        DefinitionDocument::parse(SCHEMA)
            .result
            .expect("Schema had parse errors");
    let schema_definition =
        SchemaDefinition::try_from(&definition_document).expect("Schema had errors");
    let executable_document = ExecutableDocument::parse(source)
        .result
        .expect("Document had parse errors");
    let cache = Cache::new(&executable_document, &schema_definition);
    let actual = ComplexityAnalyzer::analyze(
        &executable_document,
        &schema_definition,
        None,
        variables.as_object().expect("Variables must be an object"),
        &cache,
        (),
    )
    .unwrap();
    assert_eq!(actual, expected, "{source}");
}

#[test]
fn recursive_interfaces_and_unions_preserve_cost_at_each_depth() {
    for depth in [0, 1, 4, 7] {
        let mut interface_selection = "value".to_owned();
        let mut union_selection = "... on Item { value }".to_owned();
        for _ in 0..depth {
            interface_selection = format!("next {{ {interface_selection} }}");
            union_selection = format!("... on Item {{ choice {{ {union_selection} }} }}");
        }

        // One root, one object per level, and the interface field's weight of two.
        check_complexity(
            &format!("{{ item {{ {interface_selection} }} }}"),
            json!({}),
            depth + 3,
        );
        check_complexity(
            &format!("{{ choice {{ {union_selection} }} }}"),
            json!({}),
            depth + 3,
        );
    }
}

#[test]
fn overlapping_abstract_and_concrete_scopes_preserve_distinct_merged_costs() {
    for root in ["item", "choice"] {
        check_complexity(
            &format!(
                r#"{{
                    {root} {{ ... on Item {{ next {{ value }} }} }}
                    {root} {{
                        ... on A {{ next {{ ... on A {{ value extra }} }} }}
                        ... on B {{ next {{ ... on B {{ value extra }} }} }}
                    }}
                }}"#
            ),
            json!({}),
            // Merge the common next.value with each concrete branch independently:
            // root + next + max(max(2, 5) + 11, max(2, 7) + 13, 2).
            22,
        );
    }
}

#[test]
fn reused_fragments_under_aliases_keep_their_connection_multipliers() {
    check_complexity(
        r#"
            {
                small: connection(first: 3) { ...Fields }
                large: connection(first: 20) { ...Fields }
            }
            fragment Fields on Connection {
                entries: nodes { next { value } }
            }
        "#,
        json!({}),
        // (1 + (1 + 1 + 2) * 2) + (1 + (1 + 1 + 2) * 5).
        30,
    );
}

#[test]
fn directives_are_evaluated_for_each_analysis() {
    let source = r#"
        query($include: Boolean!, $skip: Boolean!) {
            item {
                next {
                    value @include(if: $include)
                    ... on A @skip(if: $skip) { extra }
                }
            }
        }
    "#;
    for (include, skip, expected) in [
        (false, true, 2),
        (true, true, 4),
        (false, false, 13),
        (true, false, 15),
    ] {
        check_complexity(
            source,
            json!({ "include": include, "skip": skip }),
            expected,
        );
    }
}

#[test]
fn recursive_zero_cost_selections_stay_zero() {
    let mut selection = "id".to_owned();
    for _ in 0..7 {
        selection = format!("free {{ {selection} }}");
    }
    check_complexity(&format!("{{ item {{ {selection} }} }}"), json!({}), 1);
}

#[test]
fn recursive_costs_still_saturate_when_multiplied_and_added() {
    check_complexity(
        r#"{
            connection(first: 18446744073709551616) {
                nodes { next { next { value } } }
            }
            item { value }
        }"#,
        json!({}),
        usize::MAX,
    );
}

struct MeasuredComplexity<'a, E: CoreExecutableDocument, S: CoreSchemaDefinition, V: VariableValues>(
    ComplexityCost<'a, E, S, V>,
);

impl<'a, E: CoreExecutableDocument, S: CoreSchemaDefinition, V: VariableValues> Visitor<'a, E, S, V>
    for MeasuredComplexity<'a, E, S, V>
{
    type ExtraInfo = ();

    fn new(
        operation_definition: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a V,
        cache: &'a Cache<'a, E, S>,
        _: (),
    ) -> Self {
        Self(ComplexityCost::new(
            operation_definition,
            schema_definition,
            variable_values,
            cache,
            (),
        ))
    }

    fn visit_field(
        &mut self,
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        scoped_type: TypeDefinitionReference<'a, S>,
        included: bool,
    ) {
        self.0
            .visit_field(field, field_definition, scoped_type, included);
    }

    fn leave_field(
        &mut self,
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        scoped_type: TypeDefinitionReference<'a, S>,
        included: bool,
    ) {
        self.0
            .leave_field(field, field_definition, scoped_type, included);
    }
}

impl<'a, E: CoreExecutableDocument, S: CoreSchemaDefinition, V: VariableValues>
    Analyzer<'a, E, S, V> for MeasuredComplexity<'a, E, S, V>
{
    type Output = (usize, usize);

    fn into_output(self) -> Self::Output {
        let complexity = self.0.result();
        (complexity, self.0.scope_evaluations.get())
    }
}

#[test]
fn recursive_abstract_types_do_not_repeat_child_cost_evaluation() {
    // Count result-phase work: cost-computer callbacks only run while building
    // the tree and would not detect exponential expansion in the final walk.
    // Run the small case first so a regression fails before reaching the depths
    // that would require an infeasible amount of work without memoization.
    for (implementors, depth) in [(3, 4), (3, 16), (46, 16)] {
        for abstract_type in ["Animal", "AnimalUnion"] {
            let type_names = (0..implementors)
                .map(|index| format!("Animal{index}"))
                .collect::<Vec<_>>();
            let types = type_names
                .iter()
                .map(|name| {
                    format!(
                        "type {name} implements Animal {{ owner: {abstract_type} name: String }}"
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let schema = format!(
                "interface Animal {{ owner: {abstract_type} name: String }}
                 {types}
                 union AnimalUnion = {}
                 type Query {{ animal: {abstract_type} }}",
                type_names.join(" | ")
            );
            let definition_document: DefinitionDocument =
                DefinitionDocument::parse(&schema).result.unwrap();
            let schema_definition = SchemaDefinition::try_from(&definition_document).unwrap();

            let mut selection = "... on Animal { name }".to_owned();
            for _ in 0..depth {
                selection = format!("... on Animal {{ owner {{ {selection} }} }}");
            }
            let source = format!("{{ animal {{ {selection} }} }}");
            let executable_document = ExecutableDocument::parse(&source).result.unwrap();
            let cache = Cache::new(&executable_document, &schema_definition);
            let (complexity, evaluations) =
                Orchestrator::<_, _, _, MeasuredComplexity<'_, _, _, _>>::analyze(
                    &executable_document,
                    &schema_definition,
                    None,
                    &serde_json::Map::new(),
                    &cache,
                    (),
                )
                .unwrap();

            assert_eq!(complexity, depth + 2);
            assert!(
                evaluations <= 2 * (depth + 2),
                "{abstract_type} with {implementors} possible types at depth {depth} evaluated {evaluations} scope sets"
            );
        }
    }
}
