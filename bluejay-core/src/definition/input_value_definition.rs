use crate::definition::InputType;
use crate::{ConstDirectives, ConstValue};

pub trait InputValueDefinition {
    type InputType: InputType;
    type Value: ConstValue;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn r#type(&self) -> &Self::InputType;
    fn default_value(&self) -> Option<&Self::Value>;
    fn directives(&self) -> Option<&Self::Directives>;
}
