use crate::definition::{ArgumentsDefinition, HasDirectives, OutputType};

pub trait FieldDefinition: HasDirectives {
    type ArgumentsDefinition: ArgumentsDefinition;
    type OutputType: OutputType;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition>;
    fn r#type(&self) -> &Self::OutputType;
    fn is_builtin(&self) -> bool;
}
