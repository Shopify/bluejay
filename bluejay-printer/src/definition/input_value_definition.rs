use crate::{
    directive::DirectivesPrinter, string_value::BlockStringValuePrinter, value::ValuePrinter,
    write_indent,
};
use bluejay_core::{
    definition::{InputType, InputValueDefinition},
    AsIter,
};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct InputValueDefinitionPrinter<'a, T: InputValueDefinition> {
    input_value_definition: &'a T,
    indentation: usize,
}

impl<'a, T: InputValueDefinition> InputValueDefinitionPrinter<'a, T> {
    pub(crate) fn new(input_value_definition: &'a T, indentation: usize) -> Self {
        Self {
            input_value_definition,
            indentation,
        }
    }
}

impl<'a, T: InputValueDefinition> Display for InputValueDefinitionPrinter<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            input_value_definition,
            indentation,
        } = *self;
        if let Some(description) = input_value_definition.description() {
            write!(
                f,
                "{}",
                BlockStringValuePrinter::new(description, indentation)
            )?;
        }

        write_indent(f, indentation)?;
        write!(
            f,
            "{}: {}",
            input_value_definition.name(),
            input_value_definition.r#type().as_ref().display_name(),
        )?;

        if let Some(default_value) = input_value_definition.default_value() {
            write!(f, " = {}", ValuePrinter::new(default_value))?;
        }

        if let Some(directives) = input_value_definition.directives() {
            if !directives.is_empty() {
                write!(f, " {}", DirectivesPrinter::new(directives))?;
            }
        }

        writeln!(f)
    }
}
