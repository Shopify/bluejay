use crate::ConstDirectives;
use crate::{AbstractConstValue, AbstractTypeReference, Variable};

pub trait VariableDefinition {
    type Variable: Variable;
    type TypeReference: AbstractTypeReference;
    type Directives: ConstDirectives;
    type Value: AbstractConstValue;

    fn variable(&self) -> &Self::Variable;
    fn r#type(&self) -> &Self::TypeReference;
    fn directives(&self) -> &Self::Directives;
    fn default_value(&self) -> Option<&Self::Value>;
}

pub trait VariableDefinitions: AsRef<[Self::VariableDefinition]> {
    type VariableDefinition: VariableDefinition;
}
