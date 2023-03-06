use crate::{
    definition::{
        field_definition::DisplayFieldsDefinition,
        interface_implementations::DisplayInterfaceImplementations,
    },
    directive::DisplayDirectives,
    string_value::DisplayStringValue,
};
use bluejay_core::{definition::InterfaceTypeDefinition, AsIter};
use std::fmt::{Error, Write};

pub(crate) struct DisplayInterfaceTypeDefinition;

impl DisplayInterfaceTypeDefinition {
    pub(crate) fn fmt<T: InterfaceTypeDefinition, W: Write>(
        interface_type_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = interface_type_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "interface {} ", interface_type_definition.name())?;

        if let Some(interface_implementations) =
            interface_type_definition.interface_implementations()
        {
            DisplayInterfaceImplementations::fmt(interface_implementations, f)?;
        }

        if let Some(directives) = interface_type_definition.directives() {
            if !directives.is_empty() {
                DisplayDirectives::fmt(directives, f)?;
                write!(f, " ")?;
            }
        }

        DisplayFieldsDefinition::fmt(interface_type_definition.fields_definition(), f, 0)
    }
}
