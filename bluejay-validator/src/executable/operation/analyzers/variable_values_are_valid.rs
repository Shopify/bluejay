use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{
    executable::{
        operation::{Analyzer, VariableValues, Visitor},
        Cache,
    },
    value::input_coercion::{CoerceInput, Error as CoerceInputError},
};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, VariableDefinition};

pub struct VariableValuesAreValid<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    VV: VariableValues,
> {
    executable_document: PhantomData<E>,
    schema_definition: &'a S,
    indexed_variable_values: HashMap<&'a str, (&'a VV::Key, &'a VV::Value)>,
    cache: &'a Cache<'a, E, S>,
    errors: Vec<VariableValueError<'a, E, VV>>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues, U: Copy>
    Visitor<'a, E, S, VV, U> for VariableValuesAreValid<'a, E, S, VV>
{
    fn new(
        _: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a VV,
        cache: &'a Cache<'a, E, S>,
        _: U,
    ) -> Self {
        Self {
            executable_document: PhantomData,
            schema_definition,
            indexed_variable_values: variable_values
                .iter()
                .map(|(key, value)| (key.as_ref(), (key, value)))
                .collect(),
            cache,
            errors: Vec::new(),
        }
    }

    fn visit_variable_definition(
        &mut self,
        variable_definition: &'a <E as ExecutableDocument>::VariableDefinition,
    ) {
        let key_and_value = self
            .indexed_variable_values
            .remove(variable_definition.variable());
        let Some(variable_definition_input_type) = self
            .cache
            .variable_definition_input_type(variable_definition.r#type())
        else {
            return;
        };
        match key_and_value {
            Some((_, value)) => {
                if let Err(errors) = self.schema_definition.coerce_const_value(
                    variable_definition_input_type,
                    value,
                    Default::default(),
                ) {
                    self.errors.push(VariableValueError::InvalidValue {
                        variable_definition,
                        value,
                        errors,
                    });
                }
            }
            None => {
                if variable_definition.is_required() {
                    self.errors.push(VariableValueError::MissingValue {
                        variable_definition,
                    });
                }
            }
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues, U: Copy>
    Analyzer<'a, E, S, VV, U> for VariableValuesAreValid<'a, E, S, VV>
{
    type Output = Vec<VariableValueError<'a, E, VV>>;

    fn into_output(mut self) -> Self::Output {
        self.errors.extend(
            self.indexed_variable_values
                .into_values()
                .map(|(key, value)| VariableValueError::UnusedValue { key, value }),
        );
        self.errors
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum VariableValueError<'a, E: ExecutableDocument, VV: VariableValues> {
    MissingValue {
        variable_definition: &'a E::VariableDefinition,
    },
    InvalidValue {
        variable_definition: &'a E::VariableDefinition,
        value: &'a VV::Value,
        errors: Vec<CoerceInputError<'a, true, <VV as VariableValues>::Value>>,
    },
    UnusedValue {
        key: &'a VV::Key,
        value: &'a VV::Value,
    },
}

impl<'a, E: ExecutableDocument, VV: VariableValues> VariableValueError<'a, E, VV> {
    pub fn message(&self) -> String {
        match self {
            Self::MissingValue {
                variable_definition,
            } => format!(
                "Missing value for required variable ${}",
                variable_definition.variable()
            ),
            Self::InvalidValue {
                variable_definition,
                errors,
                ..
            } => format!(
                "Invalid value for variable ${}:\n- {}",
                variable_definition.variable(),
                errors
                    .iter()
                    .map(|error| error.message())
                    .collect::<Vec<_>>()
                    .join("\n- ")
            ),
            Self::UnusedValue { key, .. } => {
                format!("No variable definition for provided key `{}`", key.as_ref())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::executable::{operation::Orchestrator, Cache};
    use bluejay_parser::ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        executable::ExecutableDocument,
        Parse,
    };
    use once_cell::sync::Lazy;

    use super::VariableValuesAreValid;

    const TEST_SCHEMA_SDL: &str = r#"
        type Query {
            noArgs: String!
            optionalArg(arg: String): String!
            requiredArg(arg: String!): String!
        }
    "#;

    static TEST_DEFINITION_DOCUMENT: Lazy<DefinitionDocument<'static>> =
        Lazy::new(|| DefinitionDocument::parse(TEST_SCHEMA_SDL).unwrap());

    static TEST_SCHEMA_DEFINITION: Lazy<SchemaDefinition<'static>> =
        Lazy::new(|| SchemaDefinition::try_from(&*TEST_DEFINITION_DOCUMENT).unwrap());

    fn validate_variable_values(
        source: &str,
        operation_name: Option<&str>,
        variable_values: &serde_json::Value,
        f: fn(Vec<String>),
    ) {
        let executable_document = ExecutableDocument::parse(source).unwrap();
        let cache = Cache::new(&executable_document, &*TEST_SCHEMA_DEFINITION);
        f(
            Orchestrator::<_, _, _, (), VariableValuesAreValid<_, _, _>>::analyze(
                &executable_document,
                &*TEST_SCHEMA_DEFINITION,
                operation_name,
                variable_values
                    .as_object()
                    .expect("Variables must be an object"),
                &cache,
                (),
            )
            .unwrap()
            .into_iter()
            .map(|err| err.message())
            .collect(),
        );
    }

    #[test]
    fn test_no_variables() {
        validate_variable_values(
            r#"
                query {
                    noArgs
                }
            "#,
            None,
            &serde_json::json!({}),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query {
                    noArgs
                }
            "#,
            None,
            &serde_json::json!({ "foo": "bar" }),
            |errors| {
                assert_eq!(
                    errors,
                    vec!["No variable definition for provided key `foo`"],
                )
            },
        );
    }

    #[test]
    fn test_optional_variables() {
        validate_variable_values(
            r#"
                query($arg: String) {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({}),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String) {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": "value" }),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String) {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": null }),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String) {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": 1 }),
            |errors| {
                assert_eq!(
                    errors,
                    vec!["Invalid value for variable $arg:\n- No implicit conversion of integer to String"],
                )
            },
        );
    }

    #[test]
    fn test_required_variables() {
        validate_variable_values(
            r#"
                query($arg: String!) {
                    requiredArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": "value" }),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String!) {
                    requiredArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": null }),
            |errors| {
                assert_eq!(
                    errors,
                    vec!["Invalid value for variable $arg:\n- Got null when non-null value of type String! was expected"],
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String!) {
                    requiredArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({}),
            |errors| assert_eq!(errors, vec!["Missing value for required variable $arg"],),
        );
    }

    #[test]
    fn test_variables_with_defaults() {
        validate_variable_values(
            r#"
                query($arg: String = "default") {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({}),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String! = "default") {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({}),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String = "default") {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": "value" }),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String = "default") {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": null }),
            |errors| {
                assert!(
                    errors.is_empty(),
                    "Expected errors to be empty: {:?}",
                    errors
                )
            },
        );
        validate_variable_values(
            r#"
                query($arg: String = "default") {
                    optionalArg(arg: $arg)
                }
            "#,
            None,
            &serde_json::json!({ "arg": 1 }),
            |errors| {
                assert_eq!(
                    errors,
                    vec!["Invalid value for variable $arg:\n- No implicit conversion of integer to String"],
                )
            },
        );
    }
}
