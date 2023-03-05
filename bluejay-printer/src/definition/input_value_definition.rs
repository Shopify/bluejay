use crate::{
    directive::DisplayDirectives, string_value::DisplayStringValue, value::DisplayValue,
    write_indent,
};
use bluejay_core::{definition::InputValueDefinition, AsIter};
use std::fmt::{Error, Write};

pub(crate) struct DisplayInputValueDefinition;

impl DisplayInputValueDefinition {
    pub(crate) fn fmt<T: InputValueDefinition, W: Write>(
        input_value_definition: &T,
        f: &mut W,
        indentation: usize,
    ) -> Result<(), Error> {
        if let Some(description) = input_value_definition.description() {
            DisplayStringValue::fmt_block(description, f, indentation)?;
        }

        write_indent(f, indentation)?;
        write!(
            f,
            "{}: {}",
            input_value_definition.name(),
            input_value_definition.r#type().as_ref().display_name(),
        )?;

        if let Some(default_value) = input_value_definition.default_value() {
            write!(f, " = ")?;
            DisplayValue::fmt(default_value.as_ref(), f)?;
        }

        if let Some(directives) = input_value_definition.directives() {
            if !directives.is_empty() {
                write!(f, " ")?;
                DisplayDirectives::fmt(directives, f)?;
            }
        }

        writeln!(f)
    }
}
