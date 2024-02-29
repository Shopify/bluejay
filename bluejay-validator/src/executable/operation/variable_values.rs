use bluejay_core::executable::{
    OperationDefinition, VariableDefinition, VariableType, VariableTypeReference,
};
use bluejay_core::{AsIter, BuiltinScalarDefinition, Value, Variable};

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

    fn evaluate_int<V: Variable, VV: VariableValues>(
        &self,
        variable: &V,
        variable_values: &VV,
    ) -> Option<i32>;
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

    fn evaluate_int<V: Variable, VV: VariableValues>(
        &self,
        variable: &V,
        variable_values: &VV,
    ) -> Option<i32> {
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

#[cfg(feature = "serde_json")]
impl VariableValues for serde_json::Map<String, serde_json::Value> {
    type Key = String;
    type Value = serde_json::Value;
    type Iterator<'a> = serde_json::map::Iter<'a>;

    fn iter(&self) -> Self::Iterator<'_> {
        self.iter()
    }
}
