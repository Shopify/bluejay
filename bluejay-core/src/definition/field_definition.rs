use crate::definition::{AbstractOutputTypeReference, ArgumentsDefinition};
use crate::ConstDirectives;

pub trait FieldDefinition {
    type ArgumentsDefinition: ArgumentsDefinition;
    type OutputTypeReference: AbstractOutputTypeReference;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition>;
    fn r#type(&self) -> &Self::OutputTypeReference;
    fn directives(&self) -> Option<&Self::Directives>;
    fn is_builtin(&self) -> bool;
}
