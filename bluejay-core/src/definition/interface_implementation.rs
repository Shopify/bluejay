use crate::definition::InterfaceTypeDefinition;

pub trait InterfaceImplementation {
    type InterfaceTypeDefinition: InterfaceTypeDefinition;

    fn interface(&self) -> &Self::InterfaceTypeDefinition;
}
