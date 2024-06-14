use crate::{EmptyDirectives, MergedValue};
use bluejay_core::executable::{ExecutableDocument, VariableDefinition};
use std::borrow::Cow;

pub struct MergedVariableDefinition<'a, E: ExecutableDocument> {
    name: Cow<'a, str>,
    r#type: &'a E::VariableType,
    default_value: Option<MergedValue<'a, true>>,
}

impl<'a, E: ExecutableDocument> VariableDefinition for MergedVariableDefinition<'a, E> {
    type Value = MergedValue<'a, true>;
    type VariableType = E::VariableType;
    type Directives = EmptyDirectives<'a>;

    fn variable(&self) -> &str {
        self.name.as_ref()
    }

    fn r#type(&self) -> &Self::VariableType {
        self.r#type
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.default_value.as_ref()
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }
}

impl<'a, E: ExecutableDocument> MergedVariableDefinition<'a, E> {
    pub(crate) fn new(
        name: Cow<'a, str>,
        r#type: &'a E::VariableType,
        default_value: Option<MergedValue<'a, true>>,
    ) -> Self {
        Self {
            name,
            r#type,
            default_value,
        }
    }

    /// This method is added in addition to the `bluejay_core::executable::VariableDefinition` method
    /// of the same name to allow getting a reference with lifetime `'a`.
    pub(crate) fn default_value(&self) -> Option<&MergedValue<'a, true>> {
        self.default_value.as_ref()
    }
}
