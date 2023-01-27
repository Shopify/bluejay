use crate::definition::AbstractInputTypeReference;
use crate::AbstractConstValue;

pub trait InputValueDefinition {
    type InputTypeReference: AbstractInputTypeReference;
    type Value: AbstractConstValue;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn r#type(&self) -> &Self::InputTypeReference;
    fn default_value(&self) -> Option<&Self::Value>;
}
