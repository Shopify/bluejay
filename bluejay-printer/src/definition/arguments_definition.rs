use crate::{
    definition::input_value_definition::InputValueDefinitionPrinter, write_indent, INDENTATION_SIZE,
};
use bluejay_core::definition::ArgumentsDefinition;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct ArgumentsDefinitionPrinter<'a, T: ArgumentsDefinition> {
    arguments_definition: &'a T,
    indentation: usize,
}

impl<'a, T: ArgumentsDefinition> ArgumentsDefinitionPrinter<'a, T> {
    pub(crate) fn new(arguments_definition: &'a T, indentation: usize) -> Self {
        Self {
            arguments_definition,
            indentation,
        }
    }
}

impl<'a, T: ArgumentsDefinition> Display for ArgumentsDefinitionPrinter<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            arguments_definition,
            indentation,
        } = *self;
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
                write!(
                    f,
                    "{}",
                    InputValueDefinitionPrinter::new(ivd, indentation + INDENTATION_SIZE)
                )
            })?;

        write_indent(f, indentation)?;
        write!(f, ")")
    }
}
