use crate::{
    definition::{
        field_definition::FieldsDefinitionPrinter,
        interface_implementations::InterfaceImplementationsPrinter,
    },
    directive::DirectivesPrinter,
    string_value::BlockStringValuePrinter,
};
use bluejay_core::definition::ObjectTypeDefinition;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct ObjectTypeDefinitionPrinter<'a, O: ObjectTypeDefinition>(&'a O);

impl<'a, O: ObjectTypeDefinition> ObjectTypeDefinitionPrinter<'a, O> {
    pub(crate) fn new(object_type_definition: &'a O) -> Self {
        Self(object_type_definition)
    }
}

impl<O: ObjectTypeDefinition> Display for ObjectTypeDefinitionPrinter<'_, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(object_type_definition) = *self;
        if let Some(description) = object_type_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "type {}", object_type_definition.name())?;

        if let Some(interface_implementations) = object_type_definition.interface_implementations()
        {
            write!(
                f,
                "{}",
                InterfaceImplementationsPrinter::new(interface_implementations)
            )?;
        }

        if let Some(directives) = object_type_definition.directives() {
            write!(f, "{}", DirectivesPrinter::new(directives))?;
        }

        write!(
            f,
            " {}",
            FieldsDefinitionPrinter::new(object_type_definition.fields_definition(), 0)
        )
    }
}
