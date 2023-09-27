use crate::definition::{HasDirectives, InputFieldsDefinition};

pub trait InputObjectTypeDefinition: HasDirectives {
    type InputFieldsDefinition: InputFieldsDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn input_field_definitions(&self) -> &Self::InputFieldsDefinition;
}
