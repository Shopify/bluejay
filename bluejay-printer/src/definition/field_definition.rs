use crate::{
    definition::arguments_definition::DisplayArgumentsDefinition, directive::DisplayDirectives,
    string_value::DisplayStringValue, write_indent, INDENTATION_SIZE,
};
use bluejay_core::{
    definition::{FieldDefinition, FieldsDefinition},
    AsIter,
};
use std::fmt::{Error, Write};

pub(crate) struct DisplayFieldDefinition;

impl DisplayFieldDefinition {
    pub(crate) fn fmt<T: FieldDefinition, W: Write>(
        field_definition: &T,
        f: &mut W,
        indentation: usize,
    ) -> Result<(), Error> {
        if let Some(description) = field_definition.description() {
            DisplayStringValue::fmt_block(description, f, indentation)?;
        }

        write_indent(f, indentation)?;
        write!(f, "{}", field_definition.name(),)?;

        if let Some(arguments_definition) = field_definition.arguments_definition() {
            DisplayArgumentsDefinition::fmt(arguments_definition, f, indentation)?;
        }

        write!(f, ": {}", field_definition.r#type().as_ref().display_name())?;

        if let Some(directives) = field_definition.directives() {
            if !directives.is_empty() {
                write!(f, " ")?;
                DisplayDirectives::fmt(directives, f)?;
            }
        }

        writeln!(f)
    }
}

pub(crate) struct DisplayFieldsDefinition;

impl DisplayFieldsDefinition {
    pub(crate) fn fmt<T: FieldsDefinition, W: Write>(
        fields_definition: &T,
        f: &mut W,
        indentation: usize,
    ) -> Result<(), Error> {
        writeln!(f, "{{")?;

        fields_definition
            .iter()
            .filter(|fd| !fd.is_builtin())
            .enumerate()
            .try_for_each(|(idx, fd)| {
                if idx != 0 {
                    writeln!(f)?;
                }
                DisplayFieldDefinition::fmt(fd, f, indentation + INDENTATION_SIZE)
            })?;

        write_indent(f, indentation)?;
        writeln!(f, "}}")
    }
}
