use crate::definition::{HasDirectives, InputType};
use crate::ConstValue;

pub trait InputValueDefinition: HasDirectives {
    type InputType: InputType;
    type Value: ConstValue;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn r#type(&self) -> &Self::InputType;
    fn default_value(&self) -> Option<&Self::Value>;

    fn is_required(&self) -> bool {
        self.default_value().is_none() && self.r#type().is_required()
    }
}
