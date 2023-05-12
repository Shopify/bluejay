use crate::executable::VariableType;
use crate::{AsIter, ConstDirectives, ConstValue};

pub trait VariableDefinition {
    type VariableType: VariableType;
    type Directives: ConstDirectives;
    type Value: ConstValue;

    fn variable(&self) -> &str;
    fn r#type(&self) -> &Self::VariableType;
    fn directives(&self) -> &Self::Directives;
    fn default_value(&self) -> Option<&Self::Value>;
}

pub trait VariableDefinitions: AsIter<Item = Self::VariableDefinition> {
    type VariableDefinition: VariableDefinition;
}
