use bluejay_core::executable::{ExecutableDocument, VariableDefinition};

use crate::directives::EmptyDirectives;

pub struct MergedVariableDefinition<'a, E: ExecutableDocument> {
    name: &'a str,
    r#type: &'a E::VariableType,
    default_value: Option<&'a E::Value<true>>,
}

impl<'a, E: ExecutableDocument> VariableDefinition for MergedVariableDefinition<'a, E> {
    type Value = E::Value<true>;
    type VariableType = E::VariableType;
    type Directives = EmptyDirectives<true, E>;

    fn variable(&self) -> &str {
        self.name
    }

    fn r#type(&self) -> &Self::VariableType {
        self.r#type
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.default_value
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }
}

impl<'a, E: ExecutableDocument> MergedVariableDefinition<'a, E> {
    pub(crate) fn new(
        name: &'a str,
        r#type: &'a E::VariableType,
        default_value: Option<&'a E::Value<true>>,
    ) -> Self {
        Self {
            name,
            r#type,
            default_value,
        }
    }

    /// This method is added in addition to the `bluejay_core::executable::VariableDefinition` method
    /// of the same name to allow getting a reference with lifetime `'a`.
    pub(crate) fn default_value(&self) -> Option<&'a E::Value<true>> {
        self.default_value
    }
}
