use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{
    executable::{
        operation::{Analyzer, VariableValues, Visitor},
        Cache,
    },
    value::input_coercion::{CoerceInput, Error as CoerceInputError},
};
use bluejay_core::definition::{prelude::*, SchemaDefinition};
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

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Visitor<'a, E, S, VV>
    for VariableValuesAreValid<'a, E, S, VV>
{
    fn new(
        _: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a VV,
        cache: &'a Cache<'a, E, S>,
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
                if variable_definition_input_type.is_required() {
                    self.errors.push(VariableValueError::MissingValue {
                        variable_definition,
                    });
                }
            }
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Analyzer<'a, E, S, VV>
    for VariableValuesAreValid<'a, E, S, VV>
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

#[cfg(test)]
mod tests {
    use crate::executable::{operation::Orchestrator, Cache};
    use bluejay_parser::ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        executable::ExecutableDocument,
    };
    use once_cell::sync::Lazy;

    use super::{CoerceInputError, VariableValueError, VariableValuesAreValid};

    type JsonValues = serde_json::Map<String, serde_json::Value>;

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
        f: fn(Vec<VariableValueError<ExecutableDocument, JsonValues>>),
    ) {
        let executable_document = ExecutableDocument::parse(source).unwrap();
        let cache = Cache::new(&executable_document, &*TEST_SCHEMA_DEFINITION);
        f(
            Orchestrator::<_, _, _, VariableValuesAreValid<_, _, _>>::analyze(
                &executable_document,
                &*TEST_SCHEMA_DEFINITION,
                operation_name,
                variable_values
                    .as_object()
                    .expect("Variables must be an object"),
                &cache,
            )
            .unwrap(),
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
                assert!(
                    matches!(errors.as_slice(), [VariableValueError::UnusedValue { key, value }] if *key == "foo" && *value == "bar"),
                    "Expected errors to be empty: {:?}",
                    errors,
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
                assert!(
                    matches!(
                    errors.as_slice(),
                    [VariableValueError::InvalidValue { errors, .. }]
                        if matches!(
                            errors.as_slice(),
                            [CoerceInputError::NoImplicitConversion { value, input_type_name, .. }]
                                if value.as_i64() == Some(1) && input_type_name == "String")
                        ),
                    "Expected no implicit conversion error, got: {:?}",
                    errors
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
                assert!(
                    matches!(
                    errors.as_slice(),
                    [VariableValueError::InvalidValue { errors, .. }]
                        if matches!(
                            errors.as_slice(),
                            [CoerceInputError::NullValueForRequiredType { value, input_type_name, .. }]
                                if value.is_null() && input_type_name == "String!")
                        ),
                    "Expected null value for required type error, got: {:?}",
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
            &serde_json::json!({}),
            |errors| {
                assert!(
                    matches!(
                    errors.as_slice(),
                    [VariableValueError::MissingValue { variable_definition }]
                        if variable_definition.variable().name() == "arg"),
                    "Expected missing value error, got: {:?}",
                    errors
                )
            },
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
                assert!(
                    matches!(
                    errors.as_slice(),
                    [VariableValueError::InvalidValue { errors, .. }]
                        if matches!(
                            errors.as_slice(),
                            [CoerceInputError::NoImplicitConversion { value, input_type_name, .. }]
                                if value.as_i64() == Some(1) && input_type_name == "String")
                        ),
                    "Expected no implicit conversion error, got: {:?}",
                    errors
                )
            },
        );
    }
}
