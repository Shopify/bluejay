use crate::executable::document::VariableDefinitionInputType;
use bluejay_core::definition::{
    BaseInputTypeReference, InputType, InputTypeReference, SchemaDefinition,
};
use std::fmt::{Display, Formatter};

pub(crate) enum InputTypeViewReference<'a, S: SchemaDefinition, I> {
    Base(BaseInputTypeReference<'a, S>, bool),
    List(I, bool),
}

impl<S: SchemaDefinition, I> InputTypeViewReference<'_, S, I> {
    pub(crate) fn is_required(&self) -> bool {
        match self {
            Self::Base(_, required) | Self::List(_, required) => *required,
        }
    }
}

pub(crate) trait InputTypeView<'a>: Copy {
    type SchemaDefinition: SchemaDefinition;

    fn as_ref(
        self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> InputTypeViewReference<'a, Self::SchemaDefinition, Self>;

    fn display_name(self) -> String;

    fn base(
        self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> BaseInputTypeReference<'a, Self::SchemaDefinition> {
        match self.as_ref(schema_definition) {
            InputTypeViewReference::Base(base, _) => base,
            InputTypeViewReference::List(inner, _) => inner.base(schema_definition),
        }
    }
}

pub(crate) struct SchemaInputTypeView<'a, S: SchemaDefinition> {
    input_type: &'a S::InputType,
}

impl<'a, S: SchemaDefinition> SchemaInputTypeView<'a, S> {
    pub(crate) fn new(input_type: &'a S::InputType) -> Self {
        Self { input_type }
    }
}

impl<S: SchemaDefinition> Clone for SchemaInputTypeView<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: SchemaDefinition> Copy for SchemaInputTypeView<'_, S> {}

impl<'a, S: SchemaDefinition> InputTypeView<'a> for SchemaInputTypeView<'a, S> {
    type SchemaDefinition = S;

    fn as_ref(self, schema_definition: &'a S) -> InputTypeViewReference<'a, S, Self> {
        match self.input_type.as_ref(schema_definition) {
            InputTypeReference::Base(base, required) => {
                InputTypeViewReference::Base(base, required)
            }
            InputTypeReference::List(inner, required) => {
                InputTypeViewReference::List(Self::new(inner), required)
            }
        }
    }

    fn display_name(self) -> String {
        self.input_type.display_name()
    }
}

pub(crate) struct VariableDefinitionInputTypeView<'a, S: SchemaDefinition> {
    input_type: &'a VariableDefinitionInputType<'a, S>,
}

impl<'a, S: SchemaDefinition> VariableDefinitionInputTypeView<'a, S> {
    pub(crate) fn new(input_type: &'a VariableDefinitionInputType<'a, S>) -> Self {
        Self { input_type }
    }
}

impl<S: SchemaDefinition> Clone for VariableDefinitionInputTypeView<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: SchemaDefinition> Copy for VariableDefinitionInputTypeView<'_, S> {}

impl<'a, S: SchemaDefinition> InputTypeView<'a> for VariableDefinitionInputTypeView<'a, S> {
    type SchemaDefinition = S;

    fn as_ref(self, _: &'a S) -> InputTypeViewReference<'a, S, Self> {
        match self.input_type {
            VariableDefinitionInputType::Base(base, required) => {
                InputTypeViewReference::Base(*base, *required)
            }
            VariableDefinitionInputType::List(inner, required) => {
                InputTypeViewReference::List(Self::new(inner.as_ref()), *required)
            }
        }
    }

    fn display_name(self) -> String {
        VariableDefinitionInputTypeDisplay(self.input_type).to_string()
    }
}

struct VariableDefinitionInputTypeDisplay<'a, 'schema, S: SchemaDefinition>(
    &'a VariableDefinitionInputType<'schema, S>,
);

impl<S: SchemaDefinition> Display for VariableDefinitionInputTypeDisplay<'_, '_, S> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            VariableDefinitionInputType::Base(base, required) => {
                write!(
                    formatter,
                    "{}{}",
                    base.name(),
                    if *required { "!" } else { "" }
                )
            }
            VariableDefinitionInputType::List(inner, required) => {
                write!(
                    formatter,
                    "[{}]{}",
                    Self(inner.as_ref()),
                    if *required { "!" } else { "" }
                )
            }
        }
    }
}
