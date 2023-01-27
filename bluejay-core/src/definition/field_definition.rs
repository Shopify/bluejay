use crate::definition::{ArgumentsDefinition, AbstractOutputTypeReference};

pub trait FieldDefinition {
    type ArgumentsDefinition: ArgumentsDefinition;
    type OutputTypeReference: AbstractOutputTypeReference;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments_definition(&self) -> &Self::ArgumentsDefinition;
    fn r#type(&self) -> &Self::OutputTypeReference;
    fn is_builtin(&self) -> bool;
}
