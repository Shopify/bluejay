use crate::definition::AbstractInputTypeReference;
use crate::{ConstDirectives, ConstValue};

pub trait InputValueDefinition {
    type InputTypeReference: AbstractInputTypeReference;
    type Value: ConstValue;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn r#type(&self) -> &Self::InputTypeReference;
    fn default_value(&self) -> Option<&Self::Value>;
    fn directives(&self) -> Option<&Self::Directives>;
}
