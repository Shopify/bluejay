use crate::{
    definition::{
        field_definition::DisplayFieldsDefinition,
        interface_implementations::DisplayInterfaceImplementations,
    },
    directive::DisplayDirectives,
    string_value::DisplayStringValue,
};
use bluejay_core::{definition::ObjectTypeDefinition, AsIter};
use std::fmt::{Error, Write};

pub(crate) struct DisplayObjectTypeDefinition;

impl DisplayObjectTypeDefinition {
    pub(crate) fn fmt<T: ObjectTypeDefinition, W: Write>(
        object_type_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = object_type_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "type {} ", object_type_definition.name())?;

        if let Some(interface_implementations) = object_type_definition.interface_implementations()
        {
            DisplayInterfaceImplementations::fmt(interface_implementations, f)?;
        }

        if let Some(directives) = object_type_definition.directives() {
            if !directives.is_empty() {
                DisplayDirectives::fmt(directives, f)?;
                write!(f, " ")?;
            }
        }

        DisplayFieldsDefinition::fmt(object_type_definition.fields_definition(), f, 0)?;

        writeln!(f)
    }
}
