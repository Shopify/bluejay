use bluejay_core::{
    definition::{
        FieldDefinition, FieldsDefinition, InputValueDefinition, ObjectTypeDefinition,
        SchemaDefinition as _,
    },
    AsIter, Value, ValueReference,
};
use bluejay_parser::{
    ast::{
        definition::{Context, CustomScalarTypeDefinition, DefinitionDocument, SchemaDefinition},
        executable::ExecutableDocument,
        Parse,
    },
    error::GraphQLError,
    Error,
};
use bluejay_validator::{
    executable::{
        document::BuiltinRulesValidator,
        operation::{analyzers::VariableValuesAreValid, Orchestrator},
        Cache,
    },
    value::input_coercion::CoerceInput,
};
use std::borrow::Cow;

#[derive(Debug)]
struct NumericContext;

impl Context for NumericContext {
    fn coerce_custom_scalar_input<const CONST: bool>(
        _: &CustomScalarTypeDefinition<Self>,
        value: &impl Value<CONST>,
    ) -> Result<(), Cow<'static, str>> {
        match value.as_ref() {
            ValueReference::Integer(_) => Ok(()),
            _ => Err("BigInt expects an integer".into()),
        }
    }
}

const SCHEMA: &str = "
    scalar BigInt
    type Query {
        f(i: Int, f: Float, id: ID, custom: BigInt, ints: [Int], floats: [Float], input: Numbers): Boolean
    }
    input Numbers { i: Int, f: Float }
";
const INT_ERROR: &str = "Int values must be between -2147483648 and 2147483647";
const FLOAT_ERROR: &str = "Float values must be finite";

fn validate(source: &str) -> Vec<GraphQLError> {
    let definition = DefinitionDocument::<NumericContext>::parse(SCHEMA)
        .result
        .unwrap();
    let schema = SchemaDefinition::try_from(&definition).unwrap();
    let document = ExecutableDocument::parse(source)
        .result
        .expect("numeric syntax must parse before scalar validation");
    let cache = Cache::new(&document, &schema);
    Error::into_graphql_errors(
        source,
        BuiltinRulesValidator::validate(&document, &schema, &cache),
    )
}

#[test]
fn int_range_is_validated() {
    for literal in ["-2147483648", "2147483647", "-0", "0"] {
        assert!(validate(&format!("{{ f(i: {literal}) }}")).is_empty());
    }
    for literal in ["2147483648", "-2147483649", "18446744073709551616"] {
        let errors = validate(&format!("{{ f(i: {literal}) }}"));
        assert_eq!(1, errors.len());
        assert_eq!(INT_ERROR, errors[0].message);
        assert_eq!(1, errors[0].locations[0].line);
        assert_eq!(8, errors[0].locations[0].col);
    }
}

#[test]
fn id_and_custom_scalars_accept_lossless_large_integers() {
    for literal in [
        "9007199254740993".to_owned(),
        "18446744073709551616".to_owned(),
        "9".repeat(1000),
        format!("-{}", "9".repeat(1000)),
    ] {
        let source = format!("{{ f(id: {literal}, custom: {literal}) }}");
        assert!(validate(&source).is_empty());
    }
}

#[test]
fn float_inputs_must_be_finite() {
    for literal in [
        "0",
        "-0",
        "0.0",
        "-0.0",
        "1e-9999",
        "1e308",
        "-1e308",
        "1.7976931348623157e308",
        "9007199254740993",
        "18446744073709551616",
    ] {
        assert!(
            validate(&format!("{{ f(f: {literal}) }}")).is_empty(),
            "{literal}"
        );
    }
    for literal in [
        "1e9999".to_owned(),
        "-1.0e9999".to_owned(),
        "9".repeat(1000),
        format!("-{}", "9".repeat(1000)),
    ] {
        let errors = validate(&format!("{{ f(f: {literal}) }}"));
        assert_eq!(1, errors.len());
        assert_eq!(FLOAT_ERROR, errors[0].message);
        assert_eq!(8, errors[0].locations[0].col);
    }
}

#[test]
fn ranges_are_checked_in_nested_values_and_variable_defaults() {
    for source in [
        "{ f(ints: [2147483648], floats: [1e9999]) }",
        "{ f(input: { i: 2147483648, f: 1e9999 }) }",
        "query Q($i: Int = 2147483648, $f: Float = 1e9999) { f(i: $i, f: $f) }",
    ] {
        let errors = validate(source);
        assert_eq!(2, errors.len(), "{source}: {errors:?}");
        assert!(errors.iter().any(|error| error.message == INT_ERROR));
        assert!(errors.iter().any(|error| error.message == FLOAT_ERROR));
    }
    let source = "query Q($id: ID = 18446744073709551616) { f(id: $id) }";
    assert!(validate(source).is_empty());
}

#[test]
fn schema_defaults_use_the_same_scalar_coercion_checks() {
    let source = "type Query { f(i: Int = 2147483648, f: Float = 1e9999, id: ID = 18446744073709551616): Boolean }";
    let definition: DefinitionDocument = DefinitionDocument::parse(source).result.unwrap();
    let schema = SchemaDefinition::try_from(&definition).unwrap();
    // The definition validator does not currently validate defaults. Consumers
    // that coerce schema defaults use the same checked API as executable values.
    let errors = schema
        .query()
        .fields_definition()
        .get("f")
        .unwrap()
        .arguments_definition()
        .unwrap()
        .iter()
        .flat_map(|argument| {
            schema
                .coerce_const_value(
                    argument.r#type(),
                    argument.default_value().unwrap(),
                    Default::default(),
                )
                .err()
                .into_iter()
                .flatten()
        });
    let errors = Error::into_graphql_errors(source, errors);
    assert_eq!(2, errors.len(), "{errors:?}");
    assert!(errors.iter().any(|error| error.message == INT_ERROR));
    assert!(errors.iter().any(|error| error.message == FLOAT_ERROR));
}

#[test]
fn json_integer_variables_are_checked_without_losing_their_kind() {
    let definition = DefinitionDocument::<NumericContext>::parse(SCHEMA)
        .result
        .unwrap();
    let schema = SchemaDefinition::try_from(&definition).unwrap();
    for (argument, input_type) in [
        ("i", "Int"),
        ("id", "ID"),
        ("f", "Float"),
        ("custom", "BigInt"),
    ] {
        let source = format!("query Q($value: {input_type}) {{ f({argument}: $value) }}");
        let document = ExecutableDocument::parse(&source).result.unwrap();
        let cache = Cache::new(&document, &schema);
        for number in [
            serde_json::json!(2147483648i64),
            serde_json::json!(i64::MIN),
            serde_json::json!(u64::MAX),
        ] {
            let variables = serde_json::json!({ "value": number });
            let errors = Orchestrator::<_, _, _, VariableValuesAreValid<_, _, _>>::analyze(
                &document,
                &schema,
                None,
                variables.as_object().unwrap(),
                &cache,
                (),
            )
            .unwrap();
            assert_eq!(
                input_type != "Int",
                errors.is_empty(),
                "{input_type}: {errors:?}"
            );
            if input_type == "Int" {
                assert!(errors[0].message().contains(INT_ERROR));
            }
        }
    }
}

#[test]
fn integer_value_equality_is_not_limited_to_native_ranges() {
    for source in [
        "{ f(id: 18446744073709551616) f(id: 18446744073709551616) }",
        "{ f(i: 0) f(i: -0) }",
    ] {
        assert!(validate(source).is_empty());
    }
    let source = "{ f(id: 18446744073709551616) f(id: 18446744073709551617) }";
    assert!(!validate(source).is_empty());
}
