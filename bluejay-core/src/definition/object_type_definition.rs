use crate::definition::{FieldsDefinition, HasDirectives, InterfaceImplementations};

pub trait ObjectTypeDefinition: HasDirectives {
    type FieldsDefinition: FieldsDefinition;
    type InterfaceImplementations: InterfaceImplementations;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn interface_implementations(&self) -> Option<&Self::InterfaceImplementations>;
    fn fields_definition(&self) -> &Self::FieldsDefinition;
    fn is_builtin(&self) -> bool;
}
