use bluejay_core::executable::{
    OperationDefinition, VariableDefinition, VariableType, VariableTypeReference,
};
use bluejay_core::{AsIter, BuiltinScalarDefinition, IntegerValue, Value, Variable};

pub trait VariableValues {
    type Key: AsRef<str>;
    type Value: Value<true>;
    type Iterator<'a>: Iterator<Item = (&'a Self::Key, &'a Self::Value)>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_>;

    fn get(&self, key: &str) -> Option<&Self::Value> {
        self.iter().find(|(k, _)| k.as_ref() == key).map(|(_, v)| v)
    }
}

pub trait OperationDefinitionValueEvaluationExt: OperationDefinition {
    fn evaluate_bool<V: Variable, VV: VariableValues>(
        &self,
        variable: &V,
        variable_values: &VV,
    ) -> Option<bool>;

    /// Resolve an `Int` variable or its default without enforcing scalar range limits.
    /// Input validation is separate; callers can use `IntegerValue`'s checked conversions.
    fn evaluate_int<'a, V: Variable, VV: VariableValues>(
        &'a self,
        variable: &V,
        variable_values: &'a VV,
    ) -> Option<IntegerValue<'a>>;
}

impl<T: OperationDefinition> OperationDefinitionValueEvaluationExt for T {
    fn evaluate_bool<V: Variable, VV: VariableValues>(
        &self,
        variable: &V,
        variable_values: &VV,
    ) -> Option<bool> {
        let variable_definitions = self.as_ref().variable_definitions()?;
        let variable_definition = variable_definitions.iter().find(|variable_definition| {
            variable_definition.variable() == variable.name()
                && matches!(
                    variable_definition.r#type().as_ref(),
                    VariableTypeReference::Named(type_name, _) if type_name == BuiltinScalarDefinition::Boolean.as_ref()
                )
        })?;

        let value = variable_values
            .iter()
            .find(|(key, _)| key.as_ref() == variable.name())
            .map(|(_, value)| value);

        if let Some(value) = value {
            value.as_ref().as_boolean().copied()
        } else {
            variable_definition
                .default_value()
                .and_then(|value| value.as_ref().as_boolean().copied())
        }
    }

    fn evaluate_int<'a, V: Variable, VV: VariableValues>(
        &'a self,
        variable: &V,
        variable_values: &'a VV,
    ) -> Option<IntegerValue<'a>> {
        let variable_definitions = self.as_ref().variable_definitions()?;
        let variable_definition = variable_definitions.iter().find(|variable_definition| {
            variable_definition.variable() == variable.name()
                && matches!(
                    variable_definition.r#type().as_ref(),
                    VariableTypeReference::Named(type_name, _) if type_name == BuiltinScalarDefinition::Int.as_ref()
                )
        })?;

        let value = variable_values
            .iter()
            .find(|(key, _)| key.as_ref() == variable.name())
            .map(|(_, value)| value);

        if let Some(value) = value {
            value.as_ref().as_integer().copied()
        } else {
            variable_definition
                .default_value()
                .and_then(|value| value.as_ref().as_integer().copied())
        }
    }
}

#[cfg(all(test, feature = "serde_json"))]
mod tests {
    use super::OperationDefinitionValueEvaluationExt;
    use bluejay_parser::ast::{executable::ExecutableDocument, Parse};
    use serde_json::json;

    #[test]
    fn evaluate_int_preserves_variables_and_defaults_losslessly() {
        for (default, variables, expected) in [
            ("2147483647", json!({}), Some("2147483647")),
            ("-2147483648", json!({}), Some("-2147483648")),
            ("2147483648", json!({}), Some("2147483648")),
            (
                "18446744073709551616",
                json!({}),
                Some("18446744073709551616"),
            ),
            (
                "-18446744073709551616",
                json!({}),
                Some("-18446744073709551616"),
            ),
            ("-0", json!({}), Some("-0")),
            ("42", json!({"n": 2147483648i64}), Some("2147483648")),
            ("42", json!({"n": i64::MIN}), Some("-9223372036854775808")),
            ("42", json!({"n": i64::MAX}), Some("9223372036854775807")),
            ("42", json!({"n": u64::MAX}), Some("18446744073709551615")),
            ("42", json!({"n": -1}), Some("-1")),
            ("42", json!({"n": null}), None),
            ("42", json!({"n": 1.0}), None),
            ("42", json!({"n": "2147483648"}), None),
        ] {
            let source = format!("query Q($n: Int = {default}) {{ f(n: $n) }}");
            let document = ExecutableDocument::parse(&source).result.unwrap();
            assert_eq!(
                expected.map(str::to_owned),
                document.operation_definitions()[0]
                    .evaluate_int(&"n", variables.as_object().unwrap())
                    .map(|value| value.to_string()),
                "{default}, {variables}"
            );
        }
    }

    #[test]
    fn evaluate_int_requires_an_int_variable_and_a_value() {
        for source in [
            "query Q { f }",
            "query Q($other: Int = 42) { f(n: $other) }",
            "query Q($n: Float) { f(n: $n) }",
            "query Q($n: [Int]) { f(n: $n) }",
        ] {
            let document = ExecutableDocument::parse(source).result.unwrap();
            assert_eq!(
                None,
                document.operation_definitions()[0]
                    .evaluate_int(&"n", json!({"n": 2147483648i64}).as_object().unwrap()),
                "{source}"
            );
        }
        let document = ExecutableDocument::parse("query Q($n: Int) { f(n: $n) }")
            .result
            .unwrap();
        assert_eq!(
            None,
            document.operation_definitions()[0].evaluate_int(&"n", json!({}).as_object().unwrap())
        );
    }
}

#[cfg(feature = "serde_json")]
impl VariableValues for serde_json::Map<String, serde_json::Value> {
    type Key = String;
    type Value = serde_json::Value;
    type Iterator<'a> = serde_json::map::Iter<'a>;

    fn iter(&self) -> Self::Iterator<'_> {
        self.iter()
    }
}
