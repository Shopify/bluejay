use crate::definition::InputFieldsDefinition;

pub trait InputObjectTypeDefinition {
    type InputFieldsDefinition: InputFieldsDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn input_field_definitions(&self) -> &Self::InputFieldsDefinition;
}
