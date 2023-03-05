use crate::{
    definition::input_value_definition::DisplayInputValueDefinition, write_indent, INDENTATION_SIZE,
};
use bluejay_core::definition::ArgumentsDefinition;
use std::fmt::{Error, Write};

pub(crate) struct DisplayArgumentsDefinition;

impl DisplayArgumentsDefinition {
    pub(crate) fn fmt<T: ArgumentsDefinition, W: Write>(
        arguments_definition: &T,
        f: &mut W,
        indentation: usize,
    ) -> Result<(), Error> {
        if arguments_definition.is_empty() {
            return Ok(());
        }

        writeln!(f, "(")?;

        arguments_definition
            .iter()
            .enumerate()
            .try_for_each(|(idx, ivd)| {
                if idx != 0 {
                    writeln!(f)?;
                }
                DisplayInputValueDefinition::fmt(ivd, f, indentation + INDENTATION_SIZE)
            })?;

        write_indent(f, indentation)?;
        write!(f, ")")
    }
}
