use crate::definition::{ArgumentsDefinition, OutputType};
use crate::ConstDirectives;

pub trait FieldDefinition {
    type ArgumentsDefinition: ArgumentsDefinition;
    type OutputType: OutputType;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition>;
    fn r#type(&self) -> &Self::OutputType;
    fn directives(&self) -> Option<&Self::Directives>;
    fn is_builtin(&self) -> bool;
}
