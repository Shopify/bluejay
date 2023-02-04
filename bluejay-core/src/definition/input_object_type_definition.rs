use crate::definition::InputFieldsDefinition;
use crate::ConstDirectives;

pub trait InputObjectTypeDefinition {
    type InputFieldsDefinition: InputFieldsDefinition;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;
    fn input_field_definitions(&self) -> &Self::InputFieldsDefinition;
}
