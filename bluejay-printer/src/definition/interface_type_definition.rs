use crate::{
    definition::{
        field_definition::FieldsDefinitionPrinter,
        interface_implementations::InterfaceImplementationsPrinter,
    },
    directive::DirectivesPrinter,
    string_value::BlockStringValuePrinter,
};
use bluejay_core::definition::InterfaceTypeDefinition;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct InterfaceTypeDefinitionPrinter<'a, I: InterfaceTypeDefinition>(&'a I);

impl<'a, I: InterfaceTypeDefinition> InterfaceTypeDefinitionPrinter<'a, I> {
    pub(crate) fn new(interface_type_definition: &'a I) -> Self {
        Self(interface_type_definition)
    }
}

impl<I: InterfaceTypeDefinition> Display for InterfaceTypeDefinitionPrinter<'_, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(interface_type_definition) = *self;
        if let Some(description) = interface_type_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "interface {}", interface_type_definition.name())?;

        if let Some(interface_implementations) =
            interface_type_definition.interface_implementations()
        {
            write!(
                f,
                "{}",
                InterfaceImplementationsPrinter::new(interface_implementations)
            )?;
        }

        if let Some(directives) = interface_type_definition.directives() {
            write!(f, "{}", DirectivesPrinter::new(directives))?;
        }

        write!(
            f,
            " {}",
            FieldsDefinitionPrinter::new(interface_type_definition.fields_definition(), 0)
        )
    }
}
