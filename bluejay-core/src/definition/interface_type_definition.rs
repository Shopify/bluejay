use crate::definition::{FieldsDefinition, InterfaceImplementations};

pub trait InterfaceTypeDefinition {
    type FieldsDefinition: FieldsDefinition;
    type InterfaceImplementations: InterfaceImplementations;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn fields_definition(&self) -> &Self::FieldsDefinition;
    fn interface_impelementations(&self) -> &Self::InterfaceImplementations;
}
